// SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
// SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use uuid::Uuid;

use linksy_core::{Config, Event, Fingerprint, Node, Session};
use linksy_core::protocol::{SessionKind, MediaCommand, CommandEntry};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinksySessionKind {
    Screen = 0,
    Control = 1,
    Display = 2,
    Camera = 3,
    Input = 4,
    Microphone = 5,
    Terminal = 6,
}

impl TryFrom<c_int> for LinksySessionKind {
    type Error = ();
    fn try_from(v: c_int) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(LinksySessionKind::Screen),
            1 => Ok(LinksySessionKind::Control),
            2 => Ok(LinksySessionKind::Display),
            3 => Ok(LinksySessionKind::Camera),
            4 => Ok(LinksySessionKind::Input),
            5 => Ok(LinksySessionKind::Microphone),
            6 => Ok(LinksySessionKind::Terminal),
            _ => Err(()),
        }
    }
}

impl From<LinksySessionKind> for SessionKind {
    fn from(k: LinksySessionKind) -> Self {
        match k {
            LinksySessionKind::Screen => SessionKind::Screen,
            LinksySessionKind::Control => SessionKind::Control,
            LinksySessionKind::Display => SessionKind::Display,
            LinksySessionKind::Camera => SessionKind::Camera,
            LinksySessionKind::Input => SessionKind::Input,
            LinksySessionKind::Microphone => SessionKind::Microphone,
            LinksySessionKind::Terminal => SessionKind::Terminal,
        }
    }
}

impl From<SessionKind> for LinksySessionKind {
    fn from(k: SessionKind) -> Self {
        match k {
            SessionKind::Screen => LinksySessionKind::Screen,
            SessionKind::Control => LinksySessionKind::Control,
            SessionKind::Display => LinksySessionKind::Display,
            SessionKind::Camera => LinksySessionKind::Camera,
            SessionKind::Input => LinksySessionKind::Input,
            SessionKind::Microphone => LinksySessionKind::Microphone,
            SessionKind::Terminal => LinksySessionKind::Terminal,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinksySessionEventType {
    Request = 0,
    Started = 1,
    Ended = 2,
}

pub type LinksyDeviceCb = unsafe extern "C" fn(
    device_id: *const c_char,
    name: *const c_char,
    ip: *const c_char,
    port: u16,
    is_lost: c_int,
    user_data: *mut c_void,
);

pub type LinksyPairCodeCb = unsafe extern "C" fn(
    request_id: *const c_char,
    device_id: *const c_char,
    code: *const c_char,
    incoming: c_int,
    user_data: *mut c_void,
);

pub type LinksyPacketCb = unsafe extern "C" fn(
    device_id: *const c_char,
    packet_type: *const c_char,
    payload_json: *const c_char,
    user_data: *mut c_void,
);

pub type LinksySessionCb = unsafe extern "C" fn(
    session_id: *const c_char,
    device_id: *const c_char,
    session_kind: c_int,
    event_type: c_int,
    user_data: *mut c_void,
);

pub type LinksyMediaDataCb = unsafe extern "C" fn(
    session_id: *const c_char,
    data: *const u8,
    len: libc::size_t,
    user_data: *mut c_void,
);

struct Callbacks {
    device_cb: Option<(LinksyDeviceCb, *mut c_void)>,
    pair_cb: Option<(LinksyPairCodeCb, *mut c_void)>,
    packet_cb: Option<(LinksyPacketCb, *mut c_void)>,
    session_cb: Option<(LinksySessionCb, *mut c_void)>,
    media_data_cb: Option<(LinksyMediaDataCb, *mut c_void)>,
}

unsafe impl Send for Callbacks {}
unsafe impl Sync for Callbacks {}

pub struct LinksyNodeHandle {
    runtime: Arc<Runtime>,
    node: Node,
    callbacks: Arc<Mutex<Callbacks>>,
    active_sessions: Arc<Mutex<HashMap<Uuid, Session>>>,
    shutdown_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

#[no_mangle]
pub unsafe extern "C" fn linksy_node_new(
    config_dir: *const c_char,
    name: *const c_char,
) -> *mut LinksyNodeHandle {
    if config_dir.is_null() || name.is_null() {
        return std::ptr::null_mut();
    }

    let config_dir_str = match CStr::from_ptr(config_dir).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let rt = match Runtime::new() {
        Ok(rt) => Arc::new(rt),
        Err(_) => return std::ptr::null_mut(),
    };

    let data_dir = PathBuf::from(config_dir_str);
    let download_dir = data_dir.join("Downloads");
    let mut config = Config::new(data_dir, download_dir, name_str);
    config.discovery = true;

    let node = match rt.block_on(async { Node::start(config).await }) {
        Ok(n) => n,
        Err(_) => return std::ptr::null_mut(),
    };

    let callbacks = Arc::new(Mutex::new(Callbacks {
        device_cb: None,
        pair_cb: None,
        packet_cb: None,
        session_cb: None,
        media_data_cb: None,
    }));

    let active_sessions = Arc::new(Mutex::new(HashMap::new()));
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let handle = Box::new(LinksyNodeHandle {
        runtime: rt.clone(),
        node: node.clone(),
        callbacks: callbacks.clone(),
        active_sessions: active_sessions.clone(),
        shutdown_tx: Mutex::new(Some(shutdown_tx)),
    });

    let mut events = node.subscribe();
    let cb_clone = callbacks.clone();
    let node_events = node.clone();

    rt.spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    break;
                }
                ev_res = events.recv() => {
                    match ev_res {
                        Ok(event) => {
                            dispatch_event(event, &cb_clone, &node_events).await;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            }
        }
    });

    Box::into_raw(handle)
}

fn forward_packet(guard: &Callbacks, dev_id: &str, pkt_type: &str, payload: serde_json::Value) {
    if let Some((cb, udata)) = guard.packet_cb {
        let dev_c = CString::new(dev_id).unwrap_or_default();
        let type_c = CString::new(pkt_type).unwrap_or_default();
        let payload_c = CString::new(payload.to_string()).unwrap_or_default();
        unsafe {
            cb(dev_c.as_ptr(), type_c.as_ptr(), payload_c.as_ptr(), udata);
        }
    }
}

async fn dispatch_event(event: Event, cbs: &Arc<Mutex<Callbacks>>, node: &Node) {
    let guard = cbs.lock().unwrap();
    match event {
        Event::DeviceFound { device, addrs } => {
            if let Some((cb, udata)) = guard.device_cb {
                let dev_id_c = CString::new(device.id.to_string()).unwrap_or_default();
                let name_c = CString::new(device.name).unwrap_or_default();
                let ip_str = addrs.first().map(|a| a.ip().to_string()).unwrap_or_default();
                let ip_c = CString::new(ip_str).unwrap_or_default();
                let port = addrs.first().map(|a| a.port()).unwrap_or(0);
                unsafe {
                    cb(dev_id_c.as_ptr(), name_c.as_ptr(), ip_c.as_ptr(), port, 0, udata);
                }
            }
        }
        Event::DeviceLost { id } => {
            if let Some((cb, udata)) = guard.device_cb {
                let dev_id_c = CString::new(id.to_string()).unwrap_or_default();
                let empty_c = CString::new("").unwrap();
                unsafe {
                    cb(dev_id_c.as_ptr(), empty_c.as_ptr(), empty_c.as_ptr(), 0, 1, udata);
                }
            }
        }
        Event::PairingRequest { request, device, code, incoming } => {
            if let Some((cb, udata)) = guard.pair_cb {
                let req_c = CString::new(request.to_string()).unwrap_or_default();
                let dev_c = CString::new(device.id.to_string()).unwrap_or_default();
                let code_c = CString::new(code).unwrap_or_default();
                unsafe {
                    cb(req_c.as_ptr(), dev_c.as_ptr(), code_c.as_ptr(), incoming as c_int, udata);
                }
            }
        }
        Event::Paired { device } => {
            if let Some((cb, udata)) = guard.packet_cb {
                let dev_c = CString::new(device.id.to_string()).unwrap_or_default();
                let type_c = CString::new("kdeconnect.pair").unwrap();
                let payload = serde_json::json!({ "pair": true }).to_string();
                let payload_c = CString::new(payload).unwrap();
                unsafe {
                    cb(dev_c.as_ptr(), type_c.as_ptr(), payload_c.as_ptr(), udata);
                }
            }
        }
        Event::PairingFailed { device, .. } => {
            if let Some((cb, udata)) = guard.packet_cb {
                let dev_c = CString::new(device.id.to_string()).unwrap_or_default();
                let type_c = CString::new("kdeconnect.pair").unwrap();
                let payload = serde_json::json!({ "pair": false }).to_string();
                let payload_c = CString::new(payload).unwrap();
                unsafe {
                    cb(dev_c.as_ptr(), type_c.as_ptr(), payload_c.as_ptr(), udata);
                }
            }
        }
        Event::SessionRequest { request, from, kind } => {
            if let Some((cb, udata)) = guard.session_cb {
                let req_c = CString::new(request.to_string()).unwrap_or_default();
                let dev_c = CString::new(from.id.to_string()).unwrap_or_default();
                let l_kind = LinksySessionKind::from(kind) as c_int;
                unsafe {
                    cb(req_c.as_ptr(), dev_c.as_ptr(), l_kind, LinksySessionEventType::Request as c_int, udata);
                }
            }
        }
        Event::SessionStarted { session, peer, kind, .. } => {
            if let Some((cb, udata)) = guard.session_cb {
                let sess_c = CString::new(session.to_string()).unwrap_or_default();
                let dev_c = CString::new(peer.id.to_string()).unwrap_or_default();
                let l_kind = LinksySessionKind::from(kind) as c_int;
                unsafe {
                    cb(sess_c.as_ptr(), dev_c.as_ptr(), l_kind, LinksySessionEventType::Started as c_int, udata);
                }
            }
        }
        Event::SessionEnded { session } => {
            if let Some((cb, udata)) = guard.session_cb {
                let sess_c = CString::new(session.to_string()).unwrap_or_default();
                let empty_c = CString::new("").unwrap();
                unsafe {
                    cb(sess_c.as_ptr(), empty_c.as_ptr(), 0, LinksySessionEventType::Ended as c_int, udata);
                }
            }
        }
        Event::Clipboard { from, text } => {
            if let Some((cb, udata)) = guard.packet_cb {
                let dev_c = CString::new(from.id.to_string()).unwrap_or_default();
                let type_c = CString::new("clipboard").unwrap();
                let payload = serde_json::json!({ "text": text }).to_string();
                let payload_c = CString::new(payload).unwrap();
                unsafe {
                    cb(dev_c.as_ptr(), type_c.as_ptr(), payload_c.as_ptr(), udata);
                }
            }
        }
        Event::Notification { from, title, body } => {
            if let Some((cb, udata)) = guard.packet_cb {
                let dev_c = CString::new(from.id.to_string()).unwrap_or_default();
                let type_c = CString::new("notification").unwrap();
                let payload = serde_json::json!({ "title": title, "body": body }).to_string();
                let payload_c = CString::new(payload).unwrap();
                unsafe {
                    cb(dev_c.as_ptr(), type_c.as_ptr(), payload_c.as_ptr(), udata);
                }
            }
        }
        Event::Battery { from, battery } => {
            if let Some((cb, udata)) = guard.packet_cb {
                let dev_c = CString::new(from.id.to_string()).unwrap_or_default();
                let type_c = CString::new("battery").unwrap();
                let payload = serde_json::json!({
                    "level": battery.level,
                    "charging": battery.charging
                }).to_string();
                let payload_c = CString::new(payload).unwrap();
                unsafe {
                    cb(dev_c.as_ptr(), type_c.as_ptr(), payload_c.as_ptr(), udata);
                }
            }
        }
        Event::MediaState { from, state } => {
            let player_ids: Vec<String> = state.players.iter().map(|p| p.id.clone()).collect();
            let dev = from.id.to_string();
            let list_payload = serde_json::json!({ "playerList": player_ids });
            forward_packet(&guard, &dev, "kdeconnect.mpris", list_payload);
            for player in &state.players {
                let payload = serde_json::json!({
                    "player": player.id,
                    "playerList": player_ids,
                    "title": player.title,
                    "artist": player.artist,
                    "album": player.album,
                    "length": player.length_ms.unwrap_or(0),
                    "pos": player.position_ms.unwrap_or(0),
                    "playing": player.playing,
                    "paused": !player.playing,
                    "albumArtUrl": player.art.clone().unwrap_or_default(),
                });
                forward_packet(&guard, &dev, "kdeconnect.mpris", payload);
            }
        }
        Event::MediaCommand { from, player, command } => {
            let dev = from.id.to_string();
            match command {
                MediaCommand::Seek { position_ms } => {
                    forward_packet(&guard, &dev, "kdeconnect.mpris.request",
                        serde_json::json!({ "player": &player, "SetPosition": position_ms }));
                }
                MediaCommand::Play => {
                    forward_packet(&guard, &dev, "kdeconnect.mpris.request",
                        serde_json::json!({ "player": &player, "action": "play" }));
                }
                MediaCommand::Pause => {
                    forward_packet(&guard, &dev, "kdeconnect.mpris.request",
                        serde_json::json!({ "player": &player, "action": "pause" }));
                }
                MediaCommand::PlayPause => {
                    forward_packet(&guard, &dev, "kdeconnect.mpris.request",
                        serde_json::json!({ "player": &player, "action": "toggle" }));
                }
                MediaCommand::Next => {
                    forward_packet(&guard, &dev, "kdeconnect.mpris.request",
                        serde_json::json!({ "player": &player, "action": "next" }));
                }
                MediaCommand::Previous => {
                    forward_packet(&guard, &dev, "kdeconnect.mpris.request",
                        serde_json::json!({ "player": &player, "action": "previous" }));
                }
            }
        }
        Event::RunCommandRequested { from, key } => {
            forward_packet(&guard, &from.id.to_string(), "kdeconnect.runcommand.request",
                serde_json::json!({ "key": &key }));
        }
        Event::CommandsReceived { from, list } => {
            let command_list: Vec<serde_json::Value> = list.iter().map(|c| {
                serde_json::json!({ "id": &c.key, "name": &c.name, "command": &c.command })
            }).collect();
            forward_packet(&guard, &from.id.to_string(), "kdeconnect.runcommand",
                serde_json::json!({ "commandList": command_list, "canAddCommand": false }));
        }
        Event::RequestCommandsReceived { from } => {
            // The daemon already sends commands via QUIC. Also forward to C++
            // so KDE Connect remotecommands plugin can respond via its own path.
            forward_packet(&guard, &from.id.to_string(), "kdeconnect.runcommand.request",
                serde_json::json!({ "requestCommandList": true }));
        }
        Event::OpenUrl { from, url } => {
            forward_packet(&guard, &from.id.to_string(), "kdeconnect.openurl",
                serde_json::json!({ "url": &url, "ask": true }));
        }
        Event::Ring { from, on } => {
            forward_packet(&guard, &from.id.to_string(), "kdeconnect.findthisdevice",
                serde_json::json!({ "request": on }));
        }
        Event::NotificationPosted { from, notification } => {
            forward_packet(&guard, &from.id.to_string(), "kdeconnect.notification",
                serde_json::json!({
                    "key": &notification.key,
                    "title": &notification.title,
                    "text": &notification.body,
                    "app": &notification.app,
                    "time": 0i64,
                    "canReply": notification.can_reply,
                    "canRemove": true
                }));
        }
        Event::NotificationRemoved { from, key } => {
            forward_packet(&guard, &from.id.to_string(), "kdeconnect.notification",
                serde_json::json!({ "key": &key, "deleted": true }));
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn linksy_node_free(node: *mut LinksyNodeHandle) {
    if !node.is_null() {
        let handle = Box::from_raw(node);
        let tx = handle.shutdown_tx.lock().unwrap().take();
        if let Some(tx) = tx {
            let _ = tx.send(());
        }
        drop(handle);
    }
}

#[no_mangle]
pub unsafe extern "C" fn linksy_start_discovery(
    node: *mut LinksyNodeHandle,
    cb: LinksyDeviceCb,
    user_data: *mut c_void,
) -> c_int {
    if node.is_null() {
        return -1;
    }
    let handle = &*node;
    let mut guard = handle.callbacks.lock().unwrap();
    guard.device_cb = Some((cb, user_data));
    0
}

#[no_mangle]
pub unsafe extern "C" fn linksy_set_pairing_callback(
    node: *mut LinksyNodeHandle,
    cb: LinksyPairCodeCb,
    user_data: *mut c_void,
) -> c_int {
    if node.is_null() {
        return -1;
    }
    let handle = &*node;
    let mut guard = handle.callbacks.lock().unwrap();
    guard.pair_cb = Some((cb, user_data));
    0
}

#[no_mangle]
pub unsafe extern "C" fn linksy_set_packet_callback(
    node: *mut LinksyNodeHandle,
    cb: LinksyPacketCb,
    user_data: *mut c_void,
) -> c_int {
    if node.is_null() {
        return -1;
    }
    let handle = &*node;
    let mut guard = handle.callbacks.lock().unwrap();
    guard.packet_cb = Some((cb, user_data));
    0
}

#[no_mangle]
pub unsafe extern "C" fn linksy_set_session_callback(
    node: *mut LinksyNodeHandle,
    cb: LinksySessionCb,
    user_data: *mut c_void,
) -> c_int {
    if node.is_null() {
        return -1;
    }
    let handle = &*node;
    let mut guard = handle.callbacks.lock().unwrap();
    guard.session_cb = Some((cb, user_data));
    0
}

#[no_mangle]
pub unsafe extern "C" fn linksy_set_media_data_callback(
    node: *mut LinksyNodeHandle,
    cb: LinksyMediaDataCb,
    user_data: *mut c_void,
) -> c_int {
    if node.is_null() {
        return -1;
    }
    let handle = &*node;
    let mut guard = handle.callbacks.lock().unwrap();
    guard.media_data_cb = Some((cb, user_data));
    0
}

#[no_mangle]
pub unsafe extern "C" fn linksy_pair_device(
    node: *mut LinksyNodeHandle,
    device_id: *const c_char,
) -> c_int {
    if node.is_null() || device_id.is_null() {
        return -1;
    }
    let handle = &*node;
    let dev_id_str = match CStr::from_ptr(device_id).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let fp = match Fingerprint::from_str(dev_id_str) {
        Ok(f) => f,
        Err(_) => return -1,
    };

    let node_clone = handle.node.clone();
    handle.runtime.spawn(async move {
        let _ = node_clone.pair_device(fp).await;
    });
    0
}

#[no_mangle]
pub unsafe extern "C" fn linksy_forget_device(
    node: *mut LinksyNodeHandle,
    device_id: *const c_char,
) -> c_int {
    if node.is_null() || device_id.is_null() {
        return -1;
    }
    let handle = &*node;
    let dev_id_str = match CStr::from_ptr(device_id).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let fp = match Fingerprint::from_str(dev_id_str) {
        Ok(f) => f,
        Err(_) => return -1,
    };

    match handle.node.forget(&fp) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn linksy_respond_pairing(
    node: *mut LinksyNodeHandle,
    request_id: *const c_char,
    accept: c_int,
) -> c_int {
    if node.is_null() || request_id.is_null() {
        return -1;
    }
    let handle = &*node;
    let req_str = match CStr::from_ptr(request_id).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let uuid = match Uuid::parse_str(req_str) {
        Ok(u) => u,
        Err(_) => return -1,
    };

    let ok = handle.node.respond(uuid, accept != 0);
    if ok { 0 } else { -1 }
}

#[no_mangle]
pub unsafe extern "C" fn linksy_create_invite(
    node: *mut LinksyNodeHandle,
    out_uri: *mut c_char,
    out_uri_size: libc::size_t,
) -> c_int {
    if node.is_null() || out_uri.is_null() || out_uri_size == 0 {
        return -1;
    }
    let handle = &*node;
    match handle.node.create_invite() {
        Ok(invite) => {
            let uri = invite.to_uri();
            let c_str = match CString::new(uri) {
                Ok(s) => s,
                Err(_) => return -1,
            };
            let bytes = c_str.as_bytes_with_nul();
            if bytes.len() > out_uri_size {
                return -1;
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, out_uri, bytes.len());
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn linksy_cancel_invite(node: *mut LinksyNodeHandle) {
    if !node.is_null() {
        let handle = &*node;
        handle.node.cancel_invite();
    }
}

#[no_mangle]
pub unsafe extern "C" fn linksy_render_qr_svg(
    uri: *const c_char,
    out_svg: *mut c_char,
    out_svg_size: libc::size_t,
) -> c_int {
    if uri.is_null() || out_svg.is_null() || out_svg_size == 0 {
        return -1;
    }
    let uri_str = match CStr::from_ptr(uri).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let code = match qrcode::QrCode::new(uri_str.as_bytes()) {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let width = code.width();
    let mut path = String::new();
    for (i, c) in code.to_colors().iter().enumerate() {
        if *c == qrcode::Color::Dark {
            path.push_str(&format!("M{} {}h1v1h-1z", i % width, i / width));
        }
    }
    let s = width + 4;
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='-2 -2 {s} {s}' shape-rendering='crispEdges'><rect x='-2' y='-2' width='{s}' height='{s}' fill='white'/><path d='{path}' fill='black'/></svg>"
    );
    let data_url = format!("data:image/svg+xml;utf8,{}", svg.replace('#', "%23"));
    let c_str = match CString::new(data_url) {
        Ok(cs) => cs,
        Err(_) => return -1,
    };
    let bytes = c_str.as_bytes_with_nul();
    if bytes.len() > out_svg_size {
        return -1;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, out_svg, bytes.len());
    0
}

#[no_mangle]
pub unsafe extern "C" fn linksy_render_qr_ascii(
    uri: *const c_char,
    out_ascii: *mut c_char,
    out_ascii_size: libc::size_t,
) -> c_int {
    if uri.is_null() || out_ascii.is_null() || out_ascii_size == 0 {
        return -1;
    }
    let uri_str = match CStr::from_ptr(uri).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let code = match qrcode::QrCode::new(uri_str.as_bytes()) {
        Ok(c) => c,
        Err(_) => return -1,
    };
    let art = code
        .render::<qrcode::render::unicode::Dense1x2>()
        .dark_color(qrcode::render::unicode::Dense1x2::Light)
        .light_color(qrcode::render::unicode::Dense1x2::Dark)
        .quiet_zone(true)
        .build();
    let c_str = match CString::new(art) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let bytes = c_str.as_bytes_with_nul();
    if bytes.len() > out_ascii_size {
        return -1;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, out_ascii, bytes.len());
    0
}

#[no_mangle]
pub unsafe extern "C" fn linksy_send_packet(
    node: *mut LinksyNodeHandle,
    device_id: *const c_char,
    packet_type: *const c_char,
    payload_json: *const c_char,
) -> c_int {
    if node.is_null() || device_id.is_null() || packet_type.is_null() {
        return -1;
    }
    let handle = &*node;
    let dev_id_str = match CStr::from_ptr(device_id).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let fp = match Fingerprint::from_str(dev_id_str) {
        Ok(f) => f,
        Err(_) => return -1,
    };
    let type_str = match CStr::from_ptr(packet_type).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let payload_str = if payload_json.is_null() {
        "{}"
    } else {
        match CStr::from_ptr(payload_json).to_str() {
            Ok(s) => s,
            Err(_) => "{}",
        }
    };

    let node_clone = handle.node.clone();
    let type_owned = type_str.to_string();
    let payload_owned = payload_str.to_string();

    handle.runtime.spawn(async move {
        match type_owned.as_str() {
            "clipboard" => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&payload_owned) {
                    if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                        let _ = node_clone.send_clipboard(&fp, text).await;
                    }
                }
            }
            "ping" => {
                let _ = node_clone.notify(&fp, "Ping", "").await;
            }
            "open_url" => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&payload_owned) {
                    if let Some(url) = v.get("url").and_then(|u| u.as_str()) {
                        let _ = node_clone.open_url(&fp, url).await;
                    }
                }
            }
            "ring" => {
                let _ = node_clone.ring(&fp, true).await;
            }
            "ring_stop" => {
                let _ = node_clone.ring(&fp, false).await;
            }
            _ => {}
        }
    });

    0
}

#[no_mangle]
pub unsafe extern "C" fn linksy_request_session(
    node: *mut LinksyNodeHandle,
    device_id: *const c_char,
    session_kind: c_int,
    out_session_id: *mut c_char,
    out_session_id_size: libc::size_t,
) -> c_int {
    if node.is_null() || device_id.is_null() {
        return -1;
    }
    let handle = &*node;
    let dev_id_str = match CStr::from_ptr(device_id).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let fp = match Fingerprint::from_str(dev_id_str) {
        Ok(f) => f,
        Err(_) => return -1,
    };
    let kind_enum = match LinksySessionKind::try_from(session_kind) {
        Ok(k) => SessionKind::from(k),
        Err(_) => return -1,
    };

    let node_clone = handle.node.clone();
    let active_sessions = handle.active_sessions.clone();

    let res = handle.runtime.block_on(async {
        node_clone.request_session(&fp, kind_enum).await
    });

    match res {
        Ok(session) => {
            let id_str = session.id.to_string();
            if !out_session_id.is_null() && out_session_id_size > id_str.len() {
                let c_str = CString::new(id_str).unwrap_or_default();
                libc::strncpy(out_session_id, c_str.as_ptr(), out_session_id_size - 1);
            }
            active_sessions.lock().unwrap().insert(session.id, session);
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn linksy_respond_session(
    node: *mut LinksyNodeHandle,
    request_id: *const c_char,
    accept: c_int,
    width: u32,
    height: u32,
) -> c_int {
    if node.is_null() || request_id.is_null() {
        return -1;
    }
    let handle = &*node;
    let req_str = match CStr::from_ptr(request_id).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let uuid = match Uuid::parse_str(req_str) {
        Ok(u) => u,
        Err(_) => return -1,
    };

    let size = if width > 0 && height > 0 {
        Some((width, height))
    } else {
        None
    };

    let ok = handle.node.respond_session(uuid, accept != 0, size);
    if ok { 0 } else { -1 }
}

#[no_mangle]
pub unsafe extern "C" fn linksy_feed_h264(
    node: *mut LinksyNodeHandle,
    session_id: *const c_char,
    data: *const u8,
    len: libc::size_t,
) -> c_int {
    if node.is_null() || session_id.is_null() || data.is_null() || len == 0 {
        return -1;
    }
    let handle = &*node;
    let sess_str = match CStr::from_ptr(session_id).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let uuid = match Uuid::parse_str(sess_str) {
        Ok(u) => u,
        Err(_) => return -1,
    };

    let bytes = std::slice::from_raw_parts(data, len).to_vec();
    let mut guard = handle.active_sessions.lock().unwrap();
    if let Some(sess) = guard.get_mut(&uuid) {
        let _ = handle.runtime.block_on(async {
            sess.send.write_all(&bytes).await
        });
        0
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn linksy_feed_pcm(
    node: *mut LinksyNodeHandle,
    session_id: *const c_char,
    data: *const u8,
    len: libc::size_t,
) -> c_int {
    linksy_feed_h264(node, session_id, data, len)
}

#[no_mangle]
pub unsafe extern "C" fn linksy_enable_away(
    node: *mut LinksyNodeHandle,
    _enable: c_int,
    _relay_url: *const c_char,
) -> c_int {
    if node.is_null() {
        return -1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn linksy_set_direct_min(
    node: *mut LinksyNodeHandle,
    _min_bytes: u64,
) -> c_int {
    if node.is_null() {
        return -1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn linksy_is_connected_over_usb(
    node: *mut LinksyNodeHandle,
    device_id: *const c_char,
) -> c_int {
    if node.is_null() || device_id.is_null() {
        return -1;
    }
    let handle = &*node;
    let dev_id_str = match CStr::from_ptr(device_id).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let fp = match Fingerprint::from_str(dev_id_str) {
        Ok(f) => f,
        Err(_) => return -1,
    };

    if handle.node.link(&fp) == Some(linksy_core::Link::Usb) {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_kind_conversion() {
        for i in 0..=6 {
            let l_kind = LinksySessionKind::try_from(i).expect("valid session kind");
            let core_kind = SessionKind::from(l_kind);
            let roundtrip = LinksySessionKind::from(core_kind);
            assert_eq!(l_kind, roundtrip);
        }
        assert!(LinksySessionKind::try_from(99).is_err());
    }

    unsafe extern "C" fn dummy_device_cb(
        _device_id: *const c_char,
        _name: *const c_char,
        _ip: *const c_char,
        _port: u16,
        _is_lost: c_int,
        _user_data: *mut c_void,
    ) {}

    #[test]
    fn test_null_pointers_safety() {
        unsafe {
            assert!(linksy_node_new(std::ptr::null(), std::ptr::null()).is_null());
            assert_eq!(linksy_start_discovery(std::ptr::null_mut(), dummy_device_cb, std::ptr::null_mut()), -1);
            assert_eq!(linksy_pair_device(std::ptr::null_mut(), std::ptr::null()), -1);
            assert_eq!(linksy_respond_pairing(std::ptr::null_mut(), std::ptr::null(), 1), -1);
            assert_eq!(linksy_send_packet(std::ptr::null_mut(), std::ptr::null(), std::ptr::null(), std::ptr::null()), -1);
            assert_eq!(linksy_request_session(std::ptr::null_mut(), std::ptr::null(), 0, std::ptr::null_mut(), 0), -1);
            assert_eq!(linksy_respond_session(std::ptr::null_mut(), std::ptr::null(), 1, 0, 0), -1);
            assert_eq!(linksy_feed_h264(std::ptr::null_mut(), std::ptr::null(), std::ptr::null(), 0), -1);
        }
    }

    #[test]
    fn test_node_lifecycle() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let dir_c = CString::new(temp_dir.path().to_str().unwrap()).unwrap();
        let name_c = CString::new("TestDevice").unwrap();

        unsafe {
            let handle = linksy_node_new(dir_c.as_ptr(), name_c.as_ptr());
            assert!(!handle.is_null());
            linksy_node_free(handle);
        }
    }
}
