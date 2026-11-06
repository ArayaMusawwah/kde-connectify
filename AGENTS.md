# AGENTS.md - KDE Connectify

**Project**: KDE Connect + Linksy integration (QUIC, PipeWire, GStreamer, Wayland)
**Version**: 26.11.70
**Contact**: Via KDE Connect mailing list or GitHub issues

---

## Build & Run

### Prerequisites
```bash
# Required
- C++20 compiler (GCC 13+ or Clang 16+)
- CMake 3.25+
- Rust toolchain (stable, for linksy-ffi FFI bridge)
- Qt 6.7+ (Core, DBus, Gui, Network, Quick, Multimedia, WaylandClient)
- KDE Frameworks 6.3+ (CoreAddons, I18n, KIO, Kirigami, Notifications, StatusNotifierItem)
- PipeWire dev headers (libpipewire-0.3)
```

### Build Commands
```bash
git clone --recursive https://github.com/ArayaMusawwah/kde-connectify.git
cd kde-connectify

# Configure (Debug recommended for development)
cmake -B build -S . \
  -DCMAKE_BUILD_TYPE=Debug \
  -DBUILD_TESTING=OFF \
  -DWITH_PULSEAUDIO=ON \
  -DWITH_X11=ON

# Build daemon, core library, plugins
cmake --build build -j$(nproc)

# Run tests (if enabled)
cargo test --manifest-path core/linksy-ffi/Cargo.toml

# Install (optional)
sudo cmake --install build
```

### Build Outputs
- **Daemon**: `build/bin/kdeconnectd`
- **App**: `build/bin/kdeconnect-app`
- **CLI**: `build/bin/kdeconnect-cli`
- **Core**: `build/bin/libkdeconnectcore.so`
- **Plugins**: `build/bin/kdeconnect/*.so`

---

## Architecture

### Directory Structure
```
kde-connectify/
├── core/                    # Core libraries + Rust FFI bridge
│   ├── linksy-ffi/         # Rust QUIC bridge to core
│   └── backends/
│       ├── linksy/         # Linksy transport backend
│       └── lan/            # Traditional LAN backend
├── daemon/                  # Desktop daemon (C++ Qt)
├── app/                     # QML desktop app
├── cli/                     # Command-line tool
├── plugins/                 # C++ KDE Connect plugins
│   ├── runcommand/         # Saved commands (desktop + phone)
│   ├── remotecontrol/      # Mouse control
│   ├── mpriscontrol/       # Desktop MPRIS2 (local players)
│   ├── mprisremote/        # Remote MPRIS2 control
│   ├── webcam/             # Phone camera → PipeWire
│   ├── microphone/         # Phone mic → PipeWire
│   ├── sidecar/            # Second display
│   ├── screenmirror/       # Screen mirroring
│   └── terminal/           # PTY shell
├── declarativeplugin/      # QML plugin exports
├── models/                  # Qt models (QAbstractListModel)
├── dbusinterfaces/          # Generated D-Bus interfaces
└── data/                    # Icons, schemas, translations
```

### Module Boundaries
- **Desktop daemon**: Manages connections, plugins, D-Bus services
- **App (QML)**: Kirigami-based UI, uses declarativeplugin exports
- **Plugins**: Plugin interface + config UI (QML per-plugin)
- **FFI Bridge**: Rust (`linksy-ffi`) → C ABI (Corrosion) → C++ Core

### Key Design Patterns
1. **Plugin System**: Each feature = plugin with `*.json` metadata and `*_config.qml` UI
2. **Model-View**: Qt `QAbstractListModel` + Kirigami `ListView` for lists
3. **DBus Interface Generation**: `dbusxml2cpp` generates `OrgKdeKdeconnectDevice*Interface`
4. **Declarative Plugin Export**: C++ classes exposed to QML via `QML_NAMED_ELEMENT`/`QML_FOREIGN`

---

## Plugin Development

### Creating a New Plugin
```bash
# Template: create plugin directory + files
mkdir -p plugins/myplugin
cat > plugins/myplugin/kdeconnect_myplugin.json <<'EOF'
{
    "$schema": "../kdeconnect.schema.json",
    "KPlugin": {
        "Name": "My Plugin",
        "Description": "Plugin description",
        "Icon": "system-run",
        "EnabledByDefault": true
    },
    "X-KdeConnect-OutgoingPacketType": ["kdeconnect.myplugin"],
    "X-KdeConnect-SupportedPacketType": []
}
EOF
cat > plugins/myplugin/mypluginplugin.h <<'EOF'
#pragma once
#include <core/kdeconnectplugin.h>
class MyPlugin : public KdeConnectPlugin {
    Q_OBJECT
public:
    using KdeConnectPlugin::KdeConnectPlugin;
protected:
    void receivePacket(const NetworkPacket &np) override;
};
EOF
cat > plugins/myplugin/mypluginplugin.cpp <<'EOF'
#include "mypluginplugin.h"
#include "plugin_myplugin_debug.h"
K_PLUGIN_CLASS_WITH_JSON(MyPlugin, "kdeconnect_myplugin.json")
void MyPlugin::receivePacket(const NetworkPacket &np) {
    // Handle incoming packets
}
EOF
cat > plugins/myplugin/CMakeLists.txt <<'EOF'
kdeconnect_add_plugin(myplugin SOURCES mypluginplugin.cpp)
target_link_libraries(myplugin kdeconnectcore Qt::DBus)
install(FILES kdeconnect_myplugin_config.qml DESTINATION ${KDE_INSTALL_DATADIR}/kdeconnect)
EOF
```

### Plugin Config UI (QML)
```qml
// plugins/myplugin/kdeconnect_myplugin_config.qml
import QtQuick
import org.kde.kirigami as Kirigami

Kirigami.ScrollablePage {
    title: i18nd("kdeconnect-plugins", "My Plugin")
    
    Kirigami.FormLayout {
        Kirigami.FormDelegateSeparator {}
        // Config fields here
    }
}
```

### Plugin Model Exposure
- **Desktop config**: `PluginModel` reads `kdeconnect/*_config.qml` from data dir
- **Available**: Set `configSource != ""` in `PluginModel`
- **Loading**: `pageStack.push(pluginDelegate.model.configSource, { device: ... })`

---

## Models & QML Integration

### CommandsModel (Desktop Saved Commands)
```qml
// models/commandsmodel.cpp/h - Local saved commands on PC
CommandsModel {
    deviceId: "device-id"
}
// Methods: addCommand(), changeCommand(), removeCommand()
```

### RemoteCommandsModel (Phone Commands)
```qml
// models/remotecommandsmodel.cpp/h - Commands from phone
RemoteCommandsModel {
    deviceId: "device-id"
    // Read-only: shows commands from phone
}
```

### Mpris Models
```qml
// Desktop shows local players
MprisDbusInterfaceFactory.create(deviceId)

// Phone sends local players to desktop
MprisRemotePlugin (plugins/mprisremote/) - Sends MPRIS events from phone
```

### Exposing to QML
```cpp
// declarativeplugin/kdeconnectdeclarativeplugin.h
class CommandsModelForeign {
    Q_GADGET
    QML_NAMED_ELEMENT(CommandsModel)
    QML_FOREIGN(CommandsModel)
};
```

---

## Debugging

### Enable Logging
```bash
# Set KDED_DEBUG env var
export KDED_DEBUG=1

# Build with debug symbols
cmake -DCMAKE_BUILD_TYPE=Debug -B build

# Check plugin logs
journalctl -f | grep -i kdeconnect
```

### Common Issues
1. **Plugin not loading**: Check `kdeconnect.plugin.<name>.json` exists in data dir
2. **QML errors**: Check `journalctl -f` for Qt Quick warnings
3. **DBus not responding**: Verify `org.kde.kdeconnect.daemon` is running
4. **FFI not working**: Check `core/linksy-ffi/target/release/` has `liblinksy_ffi.a`

### Test Commands
```bash
# Start daemon in foreground
kdeconnectd

# Connect via CLI
kdeconnect-cli

# List devices
kdeconnect-cli --list-devices
```

---

## Testing Strategy

### Plugin Tests
```bash
# Tests are in tests/ directory
cmake -DBUILD_TESTING=ON -B build
cmake --build build
ctest --output-on-failure
```

### Manual Testing
1. Ensure daemon running: `systemctl --user status kdeconnectd`
2. Open app: `kdeconnect-app`
3. Pair device: Click "Pair" in device list
4. Check logs: `journalctl -f -u kdeconnectd`

---

## Important Constraints

### Unicode Characters
**Do NOT use**: `×`, `…` → Use: `-`, `->`, `x`, `...`

### Code Style
- Based on WebKit coding style (clang-format)
- 4-space indent, 160-char line limit
- Use `kdeconnect_add_plugin()` macro for consistency
- Prefer existing infrastructure over new abstractions

### AI Usage Rules
- **Do NOT write MR descriptions or review responses**
- **Do NOT push without explicit human approval**
- **Do NOT implement features contributor doesn't understand**
- **Do NOT generate changes too extensive for contributor to review**

---

## Known Limitations

1. **Media Control**: Desktop shows local players via MPRIS2, but phone media appears via MPRISRemote plugin (needs testing)
2. **Remote Commands**: Local commands (CommandsModel) work on desktop, remote commands (RemoteCommandsModel) are read-only from desktop
3. **FFI Build**: Rust component builds via CMake custom command (`core/linksy-ffi/CMakeLists.txt`)
4. **Testing Disabled**: Currently BUILD_TESTING=OFF, enable for CI
5. **Wayland Only**: Pointer locker only implemented for Wayland (Linux/FreeBSD)

---

## Contributing Workflow

1. Read [CONTRIBUTING.md](./CONTRIBUTING.md)
2. Create feature branch: `git checkout -b feature/your-feature`
3. Implement in plugin or core module
4. Follow code style (`.clang-format`)
5. Test manually before committing
6. **STOP** - confirm with human before committing

---

## External Resources

- [KDE Connect Desktop](https://invent.kde.org/network/kdeconnect-kde)
- [Linksy Desktop](https://github.com/LinkSy-mg/linksy-desktop)
- [Qt Quick Controls 2](https://doc.qt.io/qt-6/qtquickcontrols-index.html)
- [Kirigami](https://doc.qt.io/qt-6/kirigami-index.html)
