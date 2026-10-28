# KDE Connectify Native Plasma Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transition KDE Connectify Desktop to Linksy's native Rust/Tokio/Qt6/Kirigami engine (`linksyd` + `linksy-qt` + `linksy-cli`) in `3rdparty/linksy-desktop/`, rebranded as KDE Connectify, with deep, smooth KDE Plasma 6 integration (MPRIS2, Dolphin WebDAV/Places/Service Menu, SNI tray, notifications).

**Architecture:**
- **Daemon (`crates/linksyd`)**: Async Tokio runtime, QUIC (port 47200), TLS 1.3, mDNS, StatusNotifierItem (`ksni`), MPRIS2 media controller proxy (`org.mpris.MediaPlayer2.connectify_*`), WebDAV bridge with Dolphin Places bookmarking.
- **Frontend GUI (`crates/linksy-qt`)**: Qt 6, CXX-Qt, Kirigami desktop styling, 16 modern screens, Dynamic Island, transfer cockpit, camera/screen mirroring, notification reply dialog.
- **CLI (`crates/linksy-cli`)**: Rich terminal control tool `kdeconnectify-cli`.
- **KDE Plasma Integration**: D-Bus MPRIS2 dotless service names, magic byte album art detection, `Seeked` signal emission, Dolphin KIO WebDAV URL + Places XBEL bookmark, Dolphin right-click service menu `kdeconnectify_send.desktop`, inline notification reply support.

**Tech Stack:** Rust 2024, Tokio, zbus 5, ksni, Qt 6.8+, CXX-Qt 0.10, Kirigami, KIO, WebDAV.

## Global Constraints

- Avoid Unicode symbols (`×`, `…`) in C++/Rust code; use ASCII equivalents (`x`, `...`).
- Code style: clean, well-indented, readable. Follow OCD visual guidelines: pixel-perfect symmetry and WCAG AA contrast.
- Binary names: `kdeconnectify` (GUI), `kdeconnectifyd` (daemon), `kdeconnectify-cli` (CLI). Maintain backwards-compatible aliases (`linksy-qt`, `linksyd`, `linksy`).
- Desktop ID: `org.kde.kdeconnectify`.
- All paths are relative to `/home/root_iqbal/Codes/Project KDE Connectify/3rdparty/linksy-desktop/` unless starting with absolute path.

---

### Task 1: Rebrand Executable Binaries and Cargo Workspace Aliases

**Files:**
- Modify: `crates/linksyd/Cargo.toml:1-10`
- Modify: `crates/linksy-cli/Cargo.toml:1-12`
- Modify: `crates/linksy-qt/Cargo.toml:7-12`
- Modify: `crates/linksy-qt/include/linksy-qt/app.h:50-58`

**Interfaces:**
- Produces: Executables `kdeconnectify`, `kdeconnectifyd`, `kdeconnectify-cli` alongside `linksy-qt`, `linksyd`, `linksy`.
- Consumes: Cargo workspace configuration.

- [ ] **Step 1: Update Cargo.toml files with dual binary definitions**

In `crates/linksyd/Cargo.toml`:
```toml
[[bin]]
name = "kdeconnectifyd"
path = "src/main.rs"

[[bin]]
name = "linksyd"
path = "src/main.rs"
```

In `crates/linksy-cli/Cargo.toml`:
```toml
[[bin]]
name = "kdeconnectify-cli"
path = "src/main.rs"

[[bin]]
name = "linksy"
path = "src/main.rs"
```

In `crates/linksy-qt/Cargo.toml`:
```toml
[[bin]]
name = "kdeconnectify"
path = "src/main.rs"

[[bin]]
name = "linksy-qt"
path = "src/main.rs"
```

- [ ] **Step 2: Update application identity in `crates/linksy-qt/include/linksy-qt/app.h`**

Update `run_app()` lines 50-58:
```cpp
    QApplication app(argc, argv);
    QApplication::setApplicationName(QStringLiteral("KDE Connectify"));
    QApplication::setOrganizationDomain(QStringLiteral("kde.org"));
    QApplication::setDesktopFileName(QStringLiteral("org.kde.kdeconnectify"));
    QApplication::setWindowIcon(QIcon::fromTheme(QStringLiteral("kdeconnectify"), QIcon::fromTheme(QStringLiteral("mg.diary.linksy.Linksy"))));
```

- [ ] **Step 3: Test cargo build of the rebranded binaries**

Run: `cargo check -p linksyd -p linksy -p linksy-qt`
Expected: PASS with binaries `kdeconnectifyd`, `kdeconnectify-cli`, `kdeconnectify` recognized.

- [ ] **Step 4: Commit changes**

```bash
git add crates/linksyd/Cargo.toml crates/linksy-cli/Cargo.toml crates/linksy-qt/Cargo.toml crates/linksy-qt/include/linksy-qt/app.h
git commit -m "feat: rebrand binary targets to kdeconnectify, kdeconnectifyd, kdeconnectify-cli"
```

---

### Task 2: MPRIS2 Media Player KDE Plasma Integration (`crates/linksyd/src/media.rs`)

**Files:**
- Modify: `crates/linksyd/src/media.rs:25-30,320-355,360-375,424-455,520-543`
- Test: `crates/linksyd/src/media.rs:523-550`

**Interfaces:**
- Produces: D-Bus service `org.mpris.MediaPlayer2.connectify_d<id>p<n>`, image MIME detection (`.png`, `.jpg`, `.webp`), and MPRIS `Seeked` signal.
- Consumes: `zbus`, `linksy_core::MediaState`, `linksy_core::MediaPlayer`.

- [ ] **Step 1: Write failing unit tests for MPRIS bus name and album art format detection**

Add to `crates/linksyd/src/media.rs` test module:
```rust
#[test]
fn detects_album_art_extensions() {
    assert_eq!(detect_image_extension(b"\x89PNG\r\n\x1a\n123"), "png");
    assert_eq!(detect_image_extension(b"\xFF\xD8\xFF\xE0\x00\x10JFIF"), "jpg");
    assert_eq!(detect_image_extension(b"RIFF\x20\x00\x00\x00WEBPVP8 "), "webp");
    assert_eq!(detect_image_extension(b"unknown bytes"), "jpg");
}

#[test]
fn plasma_compatible_mpris_bus_name() {
    let bus_prefix = OURS;
    assert_eq!(bus_prefix, "org.mpris.MediaPlayer2.connectify_");
    let name = format!("{bus_prefix}d12345678p0");
    assert!(!name.strip_prefix("org.mpris.MediaPlayer2.").unwrap().contains('.'));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p linksyd detects_album_art_extensions`
Expected: FAIL (`detect_image_extension` not found).

- [ ] **Step 3: Implement MPRIS changes in `crates/linksyd/src/media.rs`**

1. Update `OURS` constant at line 26:
```rust
/// The names published for phone players; no dots after prefix for KDE Plasma compatibility.
const OURS: &str = "org.mpris.MediaPlayer2.connectify_";
```

2. Add `detect_image_extension` helper:
```rust
pub(crate) fn detect_image_extension(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"\x89PNG") {
        "png"
    } else if bytes.starts_with(b"\xFF\xD8\xFF") {
        "jpg"
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "webp"
    } else {
        "jpg"
    }
}
```

3. Update `save_art` in `Phones`:
```rust
    fn save_art(&self, from: &DeviceInfo, player: &str, b64: &str) -> Option<String> {
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
        let ext = detect_image_extension(&bytes);
        let safe: String = format!("{}-{player}", from.id.short()).chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
        let path = self.art_dir.join(format!("{safe}.{ext}"));
        if std::fs::read(&path).ok().as_deref() != Some(&bytes[..]) {
            std::fs::create_dir_all(&self.art_dir).ok()?;
            std::fs::write(&path, &bytes).ok()?;
        }
        Some(format!("file://{}", path.display()))
    }
```

4. Add `Seeked` signal to `PhonePlayer` interface:
```rust
#[zbus::interface(name = "org.mpris.MediaPlayer2.Player")]
impl PhonePlayer {
    #[zbus(signal)]
    async fn seeked(emitter: &zbus::object_server::SignalEmitter<'_>, position: i64) -> zbus::Result<()>;
...
```

5. In `refresh`, emit `seeked` signal when position changes significantly:
```rust
async fn refresh(bus: &Connection, fresh: PhonePlayer) -> zbus::Result<()> {
    let iface = bus.object_server().interface::<_, PhonePlayer>(PATH).await?;
    let mut p = iface.get_mut().await;
    let old_pos = p.position();
    let new_pos = fresh.position();
    *p = fresh;
    let emitter = iface.signal_emitter();
    p.playback_status_changed(emitter).await?;
    p.metadata_changed(emitter).await?;
    p.can_go_next_changed(emitter).await?;
    p.can_go_previous_changed(emitter).await?;
    p.can_seek_changed(emitter).await?;
    if (old_pos - new_pos).abs() > 1_000_000 {
        let _ = PhonePlayer::seeked(emitter, new_pos).await;
    }
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p linksyd detects_album_art_extensions plasma_compatible_mpris_bus_name`
Expected: PASS.

- [ ] **Step 5: Commit changes**

```bash
git add crates/linksyd/src/media.rs
git commit -m "fix(media): plasma mpris bus naming, magic byte album art, and seeked signal"
```

---

### Task 3: Dolphin Remote Storage Browsing & KIO WebDAV Integration (`crates/linksyd/src/dav.rs`)

**Files:**
- Modify: `crates/linksyd/src/dav.rs:70-170`
- Test: `crates/linksyd/src/dav.rs:550-610`

**Interfaces:**
- Consumes: WebDAV local port and token.
- Produces: `webdav://127.0.0.1:<port>/<token>/`, GVfs mount resolution, Places bookmark in `~/.local/share/user-places.xbel`.

- [ ] **Step 1: Write test for user-places.xbel bookmark insertion**

Add to `crates/linksyd/src/dav.rs` tests:
```rust
#[test]
fn injects_bookmark_into_xbel() {
    let initial_xbel = r#"<?xml version="1.0" encoding="UTF-8"?>
<xbel xmlns:bookmark="http://www.freedesktop.org/standards/desktop-bookmarks">
 <bookmark href="file:///home/user"><title>Home</title></bookmark>
</xbel>"#;
    let updated = insert_xbel_bookmark(initial_xbel, "webdav://127.0.0.1:4000/token/", "Pixel 9 (KDE Connectify)");
    assert!(updated.contains("href=\"webdav://127.0.0.1:4000/token/\""));
    assert!(updated.contains("<title>Pixel 9 (KDE Connectify)</title>"));
    assert!(updated.ends_with("</xbel>"));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p linksyd injects_bookmark_into_xbel`
Expected: FAIL (`insert_xbel_bookmark` undefined).

- [ ] **Step 3: Implement KIO WebDAV, GVfs path resolution, and XBEL bookmarking**

1. Implement `insert_xbel_bookmark` and `bookmark_kde_places`:
```rust
pub(crate) fn insert_xbel_bookmark(xml: &str, uri: &str, title: &str) -> String {
    if xml.contains(&format!("href=\"{uri}\"")) {
        return xml.to_string();
    }
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let entry = format!(
        " <bookmark href=\"{uri}\">\n  <title>{title}</title>\n  <info>\n   <metadata owner=\"http://freedesktop.org\">\n    <bookmark:icon name=\"kdeconnectify\"/>\n   </metadata>\n   <metadata owner=\"http://www.kde.org\">\n    <ID>{now}/0</ID>\n    <isSystemItem>false</isSystemItem>\n   </metadata>\n  </info>\n </bookmark>\n"
    );
    if let Some(pos) = xml.rfind("</xbel>") {
        let mut out = String::with_capacity(xml.len() + entry.len());
        out.push_str(&xml[..pos]);
        out.push_str(&entry);
        out.push_str(&xml[pos..]);
        out
    } else {
        format!("{xml}\n{entry}")
    }
}

pub(crate) fn bookmark_kde_places(uri: &str, title: &str) {
    if std::env::var_os("FLATPAK_ID").is_some() {
        return;
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let data = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".local/share"));
    let xbel_path = data.join("user-places.xbel");
    if !xbel_path.exists() {
        return;
    }
    if let Ok(xml) = std::fs::read_to_string(&xbel_path) {
        let updated = insert_xbel_bookmark(&xml, uri, title);
        let _ = std::fs::write(&xbel_path, updated);
    }
}
```

2. Update `Bridges::browse` to use `webdav://` on KDE Plasma and bookmark both GTK and KDE Places:
```rust
        let uri = self.serve(phone).await.map_err(|e| e.to_string())?;
        let kde_uri = uri.replace("dav://", "webdav://");
        let mounted = tokio::process::Command::new("gio").args(["mount", &uri]).output().await;
        if let Ok(out) = &mounted
            && !out.status.success()
            && !String::from_utf8_lossy(&out.stderr).contains("already mounted")
        {
            debug!("gio mount: {}", String::from_utf8_lossy(&out.stderr).trim());
        }
        bookmark(&uri, &label);
        bookmark_kde_places(&kde_uri, &label);
        if std::env::var_os("LINKSY_FILES_NO_OPEN").is_none() {
            let is_kde = std::env::var("XDG_CURRENT_DESKTOP").map(|d| d.contains("KDE")).unwrap_or(false);
            if is_kde {
                show_folder(&kde_uri).await;
            } else {
                show_folder(&uri).await;
            }
        }
        Ok(uri)
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p linksyd injects_bookmark_into_xbel`
Expected: PASS.

- [ ] **Step 5: Commit changes**

```bash
git add crates/linksyd/src/dav.rs
git commit -m "feat(dav): dolphin places bookmarking and kio webdav support"
```

---

### Task 4: StatusNotifierItem Tray KDE Connectify Branding & Quick Actions (`crates/linksyd/src/tray.rs`)

**Files:**
- Modify: `crates/linksyd/src/tray.rs:240-265,330-340`
- Test: `cargo check -p linksyd`

**Interfaces:**
- Produces: SNI tray item with ID `kdeconnectify`, title `KDE Connectify`, tray icons `kdeconnectify-tray` and `kdeconnectify-tray-idle`.

- [ ] **Step 1: Update Tray SNI implementation in `crates/linksyd/src/tray.rs`**

Update lines 240-265:
```rust
impl ksni::Tray for Tray {
    fn id(&self) -> String {
        "kdeconnectify".into()
    }

    fn title(&self) -> String {
        "KDE Connectify".into()
    }

    fn category(&self) -> ksni::Category {
        ksni::Category::Communications
    }

    // Filled in while a device is connected, hollow and dim otherwise
    fn icon_name(&self) -> String {
        if self.connected() > 0 { "kdeconnectify-tray" } else { "kdeconnectify-tray-idle" }.into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip { title: "KDE Connectify".into(), description: self.status(), ..Default::default() }
    }

    // A click opens the window; the menu is on the other button
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.actions.send(Action::Open);
    }
```

Update line 336:
```rust
        menu.push(item(t!("Show KDE Connectify").to_string(), "kdeconnectify", true, || Action::Open));
        menu.push(item(t!("Quit").to_string(), "application-exit-symbolic", true, || Action::Quit));
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p linksyd`
Expected: PASS.

- [ ] **Step 3: Commit changes**

```bash
git add crates/linksyd/src/tray.rs
git commit -m "feat(tray): rebrand SNI tray to kdeconnectify with breeze icon names"
```

---

### Task 5: Notification Reply Routing & CLI Launch Script Fixes

**Files:**
- Modify: `data/linksy-open:1-25`
- Modify: `crates/linksyd/src/mirror.rs:170-182`
- Modify: `crates/linksyd/src/notify.rs:160-190`
- Modify: `crates/linksy-qt/src/main.rs:1-15`
- Modify: `crates/linksy-qt/include/linksy-qt/app.h:40-75`

**Interfaces:**
- Produces: CLI argument forwarding (`"$@"`) to Qt app, notification `--reply <device> <key>` routing into `kdeconnectify`, and KDE Plasma inline-reply hint.

- [ ] **Step 1: Fix argument passing in `data/linksy-open`**

Rewrite `data/linksy-open`:
```bash
#!/bin/sh
# Opens KDE Connectify window: on KDE Plasma the Qt app (kdeconnectify or linksy-qt),
# elsewhere the GTK app (linksy-app), forwarding all arguments.
here=$(dirname "$(readlink -f "$0")")
has() { [ -x "$here/$1" ] || command -v "$1" >/dev/null 2>&1; }
run() { app=$1; shift; if [ -x "$here/$app" ]; then exec "$here/$app" "$@"; else exec "$app" "$@"; fi; }

case "${LINKSY_UI:-}" in
  qt)
    has kdeconnectify && run kdeconnectify "$@"
    has linksy-qt && run linksy-qt "$@"
    ;;
  gtk)
    has linksy-app && run linksy-app "$@"
    ;;
esac

case ":${XDG_CURRENT_DESKTOP:-}:" in
  *:KDE:*)
    has kdeconnectify && run kdeconnectify "$@"
    has linksy-qt && run linksy-qt "$@"
    ;;
esac

has kdeconnectify && run kdeconnectify "$@"
has linksy-qt && run linksy-qt "$@"
has linksy-app && run linksy-app "$@"

echo "kdeconnectify: no UI binary found" >&2
exit 1
```

- [ ] **Step 2: Update `crates/linksyd/src/mirror.rs` to route reply through `window_app()`**

In `crates/linksyd/src/mirror.rs` lines 171-180:
```rust
                    NotificationResponse::Action(a) if a == "reply" => {
                        let opened = std::process::Command::new(crate::sessions::window_app())
                            .args(["--reply", &from.to_string(), key])
                            .env("LINKSY_SOCKET", &mirror.inner.socket)
                            .stdin(std::process::Stdio::null())
                            .spawn();
                        if let Err(e) = opened {
                            warn!("could not open the reply window: {e}");
                        }
                    }
```

- [ ] **Step 3: Add KDE Plasma inline-reply hint in `crates/linksyd/src/notify.rs`**

In `crates/linksyd/src/notify.rs`:
Add hint `Hint::Custom("x-kde-reply-placeholder-text".into(), "Reply...".into())` when `n.can_reply` is true.

- [ ] **Step 4: Update `crates/linksy-qt` to accept CLI arguments in `src/main.rs` & `app.h`**

In `crates/linksy-qt/src/main.rs`:
```rust
fn main() {
    linksy_logs::catch_crashes("kdeconnectify", env!("CARGO_PKG_VERSION"));
    std::process::exit(app::ffi::run_app());
}
```

In `crates/linksy-qt/include/linksy-qt/app.h`:
Forward `QCoreApplication::arguments()` to QML context properties `cmdArgs` so `Main.qml` can open the Reply dialog or process files sent from CLI / Dolphin Service Menu.

- [ ] **Step 5: Verify build**

Run: `cargo check -p linksyd -p linksy-qt`
Expected: PASS.

- [ ] **Step 6: Commit changes**

```bash
git add data/linksy-open crates/linksyd/src/mirror.rs crates/linksyd/src/notify.rs crates/linksy-qt/src/main.rs crates/linksy-qt/include/linksy-qt/app.h
git commit -m "fix(notify): route reply through window_app with cli args and kde inline reply"
```

---

### Task 6: Dolphin Service Menu, Desktop Launcher, and Autostart Integration

**Files:**
- Create: `data/org.kde.kdeconnectify.desktop`
- Create: `data/org.kde.kdeconnectify.daemon.desktop`
- Create: `data/kdeconnectifyd.service`
- Create: `data/kdeconnectify_send.desktop`
- Create: `packaging/linux/install-plasma.sh`

**Interfaces:**
- Produces: Native desktop application launcher, systemd autostart unit, and Dolphin right-click context menu.

- [ ] **Step 1: Create `data/org.kde.kdeconnectify.desktop`**

```ini
[Desktop Entry]
Type=Application
Name=KDE Connectify
GenericName=Device Sharing
Comment=Connect phones and computers smoothly on KDE Plasma
Exec=kdeconnectify %U
Icon=kdeconnectify
Terminal=false
Categories=Qt;KDE;Network;FileTransfer;
Keywords=connect;kde;plasma;phone;share;send;transfer;mpris;
MimeType=application/octet-stream;inode/directory;
StartupNotify=true
DBusActivatable=false
X-KDE-Protocols=webdav
```

- [ ] **Step 2: Create `data/org.kde.kdeconnectify.daemon.desktop`**

```ini
[Desktop Entry]
Type=Application
Name=KDE Connectify Daemon
Comment=Keeps KDE Connectify background service active
Exec=kdeconnectifyd
Icon=kdeconnectify
NoDisplay=true
Terminal=false
X-systemd-skip=true
X-KDE-autostart-phase=2
```

- [ ] **Step 3: Create `data/kdeconnectifyd.service`**

```ini
[Unit]
Description=KDE Connectify Daemon
PartOf=graphical-session.target
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/bin/kdeconnectifyd
Restart=on-failure
Slice=app.slice

[Install]
WantedBy=graphical-session.target
```

- [ ] **Step 4: Create Dolphin Service Menu `data/kdeconnectify_send.desktop`**

```ini
[Desktop Entry]
Type=Service
MimeType=all/allfiles;
Actions=SendWithKdeConnectify;
X-KDE-Submenu=Send to Device with KDE Connectify

[Desktop Action SendWithKdeConnectify]
Name=Send to Device with KDE Connectify
Icon=kdeconnectify
Exec=kdeconnectify %F
```

- [ ] **Step 5: Create `packaging/linux/install-plasma.sh`**

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

BIN="$HOME/.local/bin"
SHARE="${XDG_DATA_HOME:-$HOME/.local/share}"
UNITS="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
AUTOSTART="${XDG_CONFIG_HOME:-$HOME/.config}/autostart"
SERVICES="$SHARE/kio/servicemenus"

echo "Building KDE Connectify targets..."
cargo build --release -p linksyd -p linksy -p linksy-qt

mkdir -p "$BIN" "$SHARE/applications" "$SHARE/icons/hicolor/scalable/apps" "$SERVICES" "$UNITS" "$AUTOSTART"

# Install binaries
install -Dm755 target/release/kdeconnectify "$BIN/kdeconnectify"
install -Dm755 target/release/kdeconnectifyd "$BIN/kdeconnectifyd"
install -Dm755 target/release/kdeconnectify-cli "$BIN/kdeconnectify-cli"
install -Dm755 data/linksy-open "$BIN/kdeconnectify-open"

# Install desktop files
install -Dm644 data/org.kde.kdeconnectify.desktop "$SHARE/applications/org.kde.kdeconnectify.desktop"
install -Dm644 data/org.kde.kdeconnectify.daemon.desktop "$AUTOSTART/org.kde.kdeconnectify.daemon.desktop"
install -Dm644 data/kdeconnectifyd.service "$UNITS/kdeconnectifyd.service"
install -Dm644 data/kdeconnectify_send.desktop "$SERVICES/kdeconnectify_send.desktop"

# Install icons
install -Dm644 data/icons/mg.diary.linksy.Linksy.svg "$SHARE/icons/hicolor/scalable/apps/kdeconnectify.svg"
install -Dm644 data/icons/mg.diary.linksy.Linksy-tray-symbolic.svg "$SHARE/icons/hicolor/scalable/apps/kdeconnectify-tray.svg"
install -Dm644 data/icons/mg.diary.linksy.Linksy-tray-idle-symbolic.svg "$SHARE/icons/hicolor/scalable/apps/kdeconnectify-tray-idle.svg"

command -v update-desktop-database >/dev/null && update-desktop-database -q "$SHARE/applications" || true
command -v kbuildsycoca6 >/dev/null && kbuildsycoca6 2>/dev/null || true

echo "KDE Connectify installed successfully."
```
Make executable: `chmod +x packaging/linux/install-plasma.sh`

- [ ] **Step 6: Commit integration files**

```bash
git add data/org.kde.kdeconnectify.desktop data/org.kde.kdeconnectify.daemon.desktop data/kdeconnectifyd.service data/kdeconnectify_send.desktop packaging/linux/install-plasma.sh
git commit -m "feat: add plasma desktop launcher, dolphin service menu, and install script"
```

---

### Task 7: End-to-End Build and Verification

**Files:**
- Test all crates and verify integration.

- [ ] **Step 1: Run complete workspace cargo test**

Run: `cargo test -p linksyd -p linksy`
Expected: ALL PASS.

- [ ] **Step 2: Build release binaries**

Run: `cargo build --release -p linksyd -p linksy -p linksy-qt`
Expected: Binaries `target/release/kdeconnectify`, `target/release/kdeconnectifyd`, and `target/release/kdeconnectify-cli` exist.

- [ ] **Step 3: Verify CLI commands and help output**

Run: `target/release/kdeconnectify-cli --version`
Expected: Output showing version string.

- [ ] **Step 4: Verify Dolphin service menu syntax**

Run: `desktop-file-validate data/org.kde.kdeconnectify.desktop`
Expected: Validation success (0 warnings/errors).

- [ ] **Step 5: Verify MPRIS D-Bus prefix registration in dry-run**

Inspect `crates/linksyd/src/media.rs` ensuring `OURS` matches `"org.mpris.MediaPlayer2.connectify_"`.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-03-31-kde-connectify-linksy-native-plasma-engine.md`. Two execution options:

1. **Subagent-Driven (recommended)** - Fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach?
