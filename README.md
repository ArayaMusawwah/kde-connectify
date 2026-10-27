# KDE Connectify 🚀

> **KDE Connectify** is an enhanced, next-generation build of **KDE Connect** (`kdeconnect-kde`) integrated with the high-performance media, hardware, and transport capabilities of **Linksy** (`linksy-desktop`).

It combines KDE Connect's mature desktop integration (Plasma, GNOME, notifications, SMS, clipboard, MPRIS) with Linksy's low-latency streaming pipeline (QUIC, PipeWire, GStreamer, Wayland portals, Android Open Accessory USB).

---

## 🌟 Key Features

In addition to all standard KDE Connect features, KDE Connectify brings 8 advanced features:

| Feature | Description | Protocol / Tech |
|---|---|---|
| 🖥️ **Sidecar (Second Display)** | Use your Android tablet or phone as a virtual secondary monitor for your desktop with touch and keyboard input. | Wayland KWin / DRM + GStreamer H.264 (QUIC `display`) |
| 📷 **Phone as Virtual Webcam** | Stream phone camera directly into Linux apps (OBS, Zoom, Meet, Teams) as a native PipeWire camera device ("Linksy Camera"). Supports front/back switch, zoom, and 60 FPS. | QUIC `camera` + PipeWire `pw_stream` / v4l2loopback |
| 🎙️ **Phone as Microphone** | Low-latency audio input directly into PipeWire ("Linksy Microphone"). No Bluetooth compression or lag. | Raw PCM 16-bit 48kHz mono (QUIC `microphone`) |
| 🪞 **Screen Mirror & Remote Desktop** | High-framerate real-time desktop screen mirroring to phone/tablet or vice versa, with remote mouse and keyboard input. | Wayland Portal `ScreenCast` & `RemoteDesktop` + H.264 |
| ⚡ **USB Cable Mode (AOA)** | Connect phone directly via USB cable using Android Open Accessory. Zero Wi-Fi needed, no ADB/developer mode required, lowest latency (< 5ms). | Linux USB bulk transfer + loopback QUIC bridge |
| 🌐 **Away from Home (WAN P2P)** | Connect to paired devices securely over the Internet without port forwarding or public IP. | `iroh` P2P NAT traversal + cryptographic lease |
| ⚡ **Wi-Fi Direct Transfer** | Automatic ad-hoc 5GHz direct connection for transfers $\ge 256$ MB achieving speeds up to 80 MB/s. | Wi-Fi P2P Group + NetworkManager DBus |
| 💻 **Secure Remote Terminal** | Interactive shell session on your PC from your mobile device, protected by a desktop-generated 6-digit PIN. | Unix PTY (`openpty`) + JSON framing |

---

## 🏗️ Architecture

KDE Connectify uses a **zero-overhead Rust FFI Bridge via Corrosion** and **Modular C++ Plugins**:

```
[ Android: Linksy Mobile / KDE Connect Mobile ]
                        │
                        ▼ (QUIC / TLS 1.3 / mDNS / AOA / Iroh)
┌─────────────────────────────────────────────────────────────┐
│ KDE Connect Desktop Core (`kdeconnectcore`)                 │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Rust FFI Core Layer (`core/linksy-ffi`)              │   │
│  │  - Quinn QUIC & Rustls TLS 1.3 mutual auth           │   │
│  │  - mDNS & HMAC Private Discovery                     │   │
│  │  - Iroh WAN P2P Endpoint                             │   │
│  │  - AOA USB Device Loopback                           │   │
│  └──────────────────────────┬───────────────────────────┘   │
│                             │ C-ABI (Corrosion CMake)       │
│  ┌──────────────────────────▼───────────────────────────┐   │
│  │ C++ Daemon Core (`kdeconnectd`)                      │   │
│  │  - LinksyLinkProvider : public LinkProvider          │   │
│  │  - LinksyDeviceLink   : public DeviceLink            │   │
│  └──────────────────────────┬───────────────────────────┘   │
│                             │ Qt Signals / Slots            │
│  ┌──────────────────────────▼───────────────────────────┐   │
│  │ Modular C++ Plugins:                                 │   │
│  │  - `kdeconnect_webcam`                               │   │
│  │  - `kdeconnect_microphone`                           │   │
│  │  - `kdeconnect_sidecar`                              │   │
│  │  - `kdeconnect_screenmirror`                         │   │
│  │  - `kdeconnect_terminal`                             │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎨 UI/UX Design Standards

- **Strict 8-Point Spatial Grid**: Consistent 16px margins, 12px card spacing, and pixel-perfect symmetry for visual harmony.
- **Accessibility (WCAG AA Compliance)**: All text and action elements maintain contrast ratios $\ge 4.5:1$ on both light and dark themes.
- **Dual Status Indicators**: States are indicated with both clear badge text (`[ ACTIVE ]` / `[ STANDBY ]`) and color coding.
- **Plasma System Tray Integration**: Direct quick-actions in the Plasmoid menu for one-click Webcam or Second Display activation.

---

## 🔧 Building from Source

### Prerequisites

Ensure you have the following installed:
- C++ compiler with C++20 support (GCC 13+ or Clang 16+)
- CMake 3.25+
- Rust toolchain (stable `rustc` & `cargo`)
- Qt 6 (Core, DBus, Gui, Network, Quick, Multimedia, WaylandClient)
- KDE Frameworks 6 (CoreAddons, I18n, KIO, Kirigami, Notifications, StatusNotifierItem)
- PipeWire development headers (`libpipewire-0.3`)

### Build Steps

```bash
# 1. Clone repository with submodules
git clone --recursive https://github.com/ArayaMusawwah/kde-connectify.git
cd kde-connectify

# 2. Configure build
cmake -B build -S . \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_TESTING=OFF

# 3. Build daemon, libraries, and plugins
cmake --build build -j$(nproc)

# 4. Run tests
cargo test --manifest-path core/linksy-ffi/Cargo.toml
```

Artifacts will be located in:
- Daemon & CLI: `build/bin/kdeconnectd`, `build/bin/kdeconnect-cli`
- Core Library: `build/bin/libkdeconnectcore.so`
- New Plugins: `build/bin/kdeconnect/kdeconnect_{webcam,microphone,sidecar,screenmirror,terminal}.so`

---

## 📱 Mobile Compatibility

Desktop communicates directly with:
1. **Linksy Android App** (Flutter + `linksy-core` compiled via NDK) for full 8-feature support.
2. Standard **KDE Connect Android App** for legacy notifications, clipboard, SMS, and media controls.

---

## 📄 License

KDE Connectify is licensed under the [GNU General Public License v2.0 or later](LICENSES/GPL-2.0-only.txt) and [GNU General Public License v3.0 or later](LICENSES/GPL-3.0-only.txt), consistent with KDE Connect and Linksy.
