//! Serves this computer's `$HOME` to paired devices (the reverse of the phone's
//! FileServer): the same `FsRequest` protocol, answered with `std::fs`. A device
//! asking is served whatever it can name under the root; paths that climb out of
//! it are refused. This mirrors the phone test helper in phone_files.rs but against
//! the real filesystem, with no artificial delay or blocklist.

use std::io::{ErrorKind, Read, Seek, Write};
use std::path::{Component, Path, PathBuf};

use linksy_core::fs::{FsCall, MAX_CHUNK};
use linksy_core::{FsEntry, FsRequest, Node};

/// Where this computer lets other devices browse. `$HOME`, or `/` if unset.
pub fn root() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"))
}

enum Reply {
    Entries(Vec<FsEntry>),
    Entry(FsEntry),
    Data(Vec<u8>),
    Ok,
}

fn entry(p: &Path) -> Option<FsEntry> {
    let m = std::fs::metadata(p).ok()?;
    Some(FsEntry {
        name: p.file_name()?.to_string_lossy().into_owned(),
        dir: m.is_dir(),
        size: m.len(),
        modified_ms: m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_millis() as u64).unwrap_or(0),
    })
}

/// The path a request names, resolved inside `root`; `Err` if it would escape it.
fn resolve(root: &Path, p: &str) -> std::io::Result<PathBuf> {
    let rel = p.trim_start_matches('/');
    if rel.is_empty() {
        return Ok(root.to_path_buf());
    }
    let mut out = root.to_path_buf();
    for c in Path::new(rel).components() {
        match c {
            Component::Normal(part) => out.push(part),
            // `..` climbs out of the root: refused, never resolved
            Component::ParentDir => return Err(std::io::Error::new(ErrorKind::PermissionDenied, "outside the shared folder")),
            // `foo/.` is fine (and `/` in requests means root, handled above)
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => {}
        }
    }
    Ok(out)
}

/// Answer one FS call from a phone: read/write run on a blocking thread so big
/// files don't stall the async runtime.
async fn answer(mut call: FsCall, root: PathBuf) {
    let path = match &call.request {
        FsRequest::List { path } | FsRequest::Stat { path } | FsRequest::Read { path, .. } | FsRequest::Write { path, .. } => path.clone(),
        FsRequest::Mkdir { path } | FsRequest::Delete { path } | FsRequest::Rename { path, .. } | FsRequest::Open { path } => path.clone(),
        FsRequest::Info => String::new(),
    };
    if matches!(call.request, FsRequest::Info) {
        let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "Home".into());
        let _ = call.info(&name).await;
        return;
    }
    let done = match resolve(&root, &path) {
        Err(e) => Err(e),
        Ok(at) if path.is_empty() => {
            let mut entries: Vec<FsEntry> = match std::fs::read_dir(&at) {
                Ok(dir) => dir.flatten().filter_map(|e| entry(&e.path())).collect(),
                Err(e) => { let _ = call.error(&e.to_string(), e.kind() == ErrorKind::NotFound).await; return; }
            };
            entries.sort_by(|a, b| b.dir.cmp(&a.dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
            Ok(Reply::Entries(entries))
        }
        Ok(at) => {
            let request = call.request.clone();
            run_ops(&request, &at, &root, &mut call).await
        }
    };
    match done {
        Ok(Reply::Entries(e)) => { let _ = call.entries(e).await; }
        Ok(Reply::Entry(e)) => { let _ = call.entry(e).await; }
        Ok(Reply::Data(d)) => { let _ = call.data(&d).await; }
        Ok(Reply::Ok) => { let _ = call.ok().await; }
        Err(e) => { let _ = call.error(&e.to_string(), e.kind() == ErrorKind::NotFound).await; }
    }
}

async fn run_ops(request: &FsRequest, at: &Path, root: &Path, call: &mut FsCall) -> std::io::Result<Reply> {
    Ok(match request {
        FsRequest::List { .. } => {
            let at = at.to_path_buf();
            let entries = tokio::task::spawn_blocking(move || {
                let mut entries = std::fs::read_dir(&at)?.flatten().filter_map(|e| entry(&e.path())).collect::<Vec<_>>();
                entries.sort_by(|a, b| b.dir.cmp(&a.dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
                Ok::<Vec<FsEntry>, std::io::Error>(entries)
            })
            .await
            .map_err(io_from_join)??;
            Reply::Entries(entries)
        }
        FsRequest::Stat { .. } => Reply::Entry(entry(at).ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, "No such file"))?),
        FsRequest::Read { offset, length, .. } => {
            let (at, offset, length) = (at.to_path_buf(), *offset, *length);
            let data = tokio::task::spawn_blocking(move || {
                let mut f = std::fs::File::open(at)?;
                f.seek(std::io::SeekFrom::Start(offset))?;
                let mut data = Vec::new();
                f.take(length).read_to_end(&mut data)?;
                Ok::<Vec<u8>, std::io::Error>(data)
            })
            .await
            .map_err(io_from_join)??;
            Reply::Data(data)
        }
        FsRequest::Write { offset, truncate, .. } => {
            let body = call.body().await.map_err(|e| std::io::Error::new(ErrorKind::Other, e.to_string()))?;
            if body.len() as u64 > MAX_CHUNK {
                return Err(std::io::Error::new(ErrorKind::PermissionDenied, "at most 4 MiB at a time"));
            }
            let (at, offset, truncate) = (at.to_path_buf(), *offset, *truncate);
            tokio::task::spawn_blocking(move || {
                let mut f = std::fs::OpenOptions::new().create(true).write(true).truncate(truncate).open(at)?;
                f.seek(std::io::SeekFrom::Start(offset))?;
                f.write_all(&body)?;
                Ok::<(), std::io::Error>(())
            })
            .await
            .map_err(io_from_join)??;
            Reply::Ok
        }
        FsRequest::Mkdir { .. } => std::fs::create_dir(at).map(|_| Reply::Ok)?,
        FsRequest::Delete { .. } => {
            if at.is_dir() { std::fs::remove_dir_all(at) } else { std::fs::remove_file(at) }?;
            Reply::Ok
        }
        FsRequest::Rename { to, .. } => std::fs::rename(at, resolve(root, to)?).map(|_| Reply::Ok)?,
        FsRequest::Open { .. } => {
            // Best-effort: open on this desktop. We don't block waiting for xdg-open;
            // spawn it detached so a slow viewer doesn't tie up the file stream.
            let open = std::process::Command::new("xdg-open").arg(at).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn();
            if let Err(e) = open { return Err(e); }
            Reply::Ok
        }
        FsRequest::Info => Reply::Ok,
    })
}

fn io_from_join(e: tokio::task::JoinError) -> std::io::Error {
    std::io::Error::new(ErrorKind::Other, e.to_string())
}

/// Serve this computer's files to connected devices. Run it once per daemon.
pub async fn run(node: Node) {
    let Some(mut calls) = node.incoming_fs() else { return };
    let root = root();
    while let Some(call) = calls.recv().await {
        let root = root.clone();
        tokio::spawn(async move { answer(call, root).await });
    }
}
