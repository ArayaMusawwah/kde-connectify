/*
 * SPDX-FileCopyrightText: 2026 KDE Connectify contributors
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

import QtQuick
import QtCore
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import QtQuick.Dialogs
import org.kde.kirigami as Kirigami
import org.kde.kirigamiaddons.components as Components
import org.kde.kdeconnect

Kirigami.ScrollablePage {
    id: root

    property QtObject currentDevice
    title: currentDevice ? currentDevice.name : ""

    readonly property bool isPaired: currentDevice && currentDevice.isPaired
    readonly property bool isReachable: currentDevice && currentDevice.isReachable

    BatteryInfo {
        id: batteryInfo
        device: root.currentDevice
    }

    ConnectivityInfo {
        id: connectivityInfo
        device: root.currentDevice
    }

    function openSettings() {
        for (let index = 0; index < pageStack.depth; ++index) {
            const page = pageStack.get(index);
            if (page.isPluginSettingsPage && page.device === currentDevice.id()) {
                pageStack.currentIndex = index;
                return page;
            }
        }
        return pageStack.push(Qt.resolvedUrl("PluginSettings.qml"), { device: currentDevice.id() });
    }

    actions: [
        Kirigami.Action {
            icon.name: "network-disconnect"
            text: i18n("Unpair")
            visible: root.isPaired
            onTriggered: {
                root.currentDevice.unpair();
                pageStack.pop(0);
            }
        },
        Kirigami.Action {
            icon.name: "settings-configure"
            text: i18n("Plugin Settings")
            visible: root.isPaired && root.isReachable
            onTriggered: root.openSettings()
        }
    ]

    DropArea {
        anchors.fill: parent
        enabled: root.isPaired && root.isReachable
        onDropped: drop => {
            if (drop.hasUrls) {
                applicationWindow().sendFiles(root.currentDevice.id(), drop.urls);
            }
        }
    }

    ColumnLayout {
        spacing: Kirigami.Units.largeSpacing * 1.5
        width: parent.width

        // 1. Device Header
        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.largeSpacing * 1.5

            Components.Avatar {
                Layout.preferredWidth: Kirigami.Units.gridUnit * 3.5
                Layout.preferredHeight: Kirigami.Units.gridUnit * 3.5
                name: root.title
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                Layout.alignment: Qt.AlignVCenter

                Kirigami.Heading {
                    level: 2
                    text: root.title
                    elide: Text.ElideRight
                }

                QQC2.Label {
                    Layout.fillWidth: true
                    wrapMode: Text.Wrap
                    opacity: 0.75
                    text: {
                        let parts = [];
                        if (root.currentDevice) {
                            parts.push(root.currentDevice.type);
                            parts.push(root.isReachable ? i18n("Connected") : i18n("Offline"));
                            if (batteryInfo.hasBattery) parts.push(batteryInfo.displayString);
                            if (connectivityInfo.available) parts.push(connectivityInfo.displayString);
                        }
                        return parts.join(" • ");
                    }
                }
            }
        }

        // 2. Pairing Banner (if reachable but not paired)
        Kirigami.InlineMessage {
            Layout.fillWidth: true
            visible: root.isReachable && !root.isPaired
            type: Kirigami.MessageType.Information
            text: i18n("Pair with this device to access files, media controls, and hardware features.")
            actions: Kirigami.Action {
                text: i18n("Pair")
                icon.name: "network-connect"
                onTriggered: root.currentDevice.requestPairing()
            }
        }

        // 3. Quick Action Tiles Grid
        Flow {
            Layout.fillWidth: true
            spacing: Kirigami.Units.largeSpacing
            visible: root.isPaired && root.isReachable

            ActionTile {
                iconName: "system-file-manager"
                title: i18n("Browse Files")
                subtitle: i18n("Open in Dolphin")
                onClicked: {
                    let sftp = SftpDbusInterfaceFactory.create(root.currentDevice.id());
                    if (sftp) sftp.startBrowsing();
                    else Qt.openUrlExternally("kdeconnect://" + root.currentDevice.id() + "/");
                }
            }

            ActionTile {
                iconName: "document-send"
                title: i18n("Send Files")
                subtitle: i18n("Choose files to send")
                onClicked: deviceFileDialog.open()
            }

            ActionTile {
                iconName: "edit-paste"
                title: i18n("Send Clipboard")
                subtitle: i18n("Push clipboard to phone")
                onClicked: {
                    let clip = ClipboardDbusInterfaceFactory.create(root.currentDevice.id());
                    clip.sendClipboard();
                    applicationWindow().showPassiveNotification(i18n("Clipboard sent to %1", root.title));
                }
            }

            ActionTile {
                iconName: "audio-volume-high"
                title: i18n("Find Phone")
                subtitle: i18n("Ring device to locate it")
                onClicked: {
                    let finder = FindMyPhoneDbusInterfaceFactory.create(root.currentDevice.id());
                    finder.ring();
                }
            }

            ActionTile {
                iconName: "preferences-desktop-notification-bell"
                title: i18n("Send Ping")
                subtitle: i18n("Test notification")
                onClicked: root.currentDevice.pluginCall("ping", "sendPing")
            }

            ActionTile {
                iconName: "video-display"
                title: i18n("Screen Mirror")
                subtitle: i18n("View mobile screen")
                onClicked: root.currentDevice.pluginCall("screenmirror", "startMirror")
            }
        }

        // 4. Remote Media Player Card
        MprisCard {
            Layout.fillWidth: true
            device: root.currentDevice
            visible: root.isPaired && root.isReachable
        }

        // 5. Remote Storage & SFTP Card
        StorageCard {
            Layout.fillWidth: true
            device: root.currentDevice
            visible: root.isPaired && root.isReachable
        }

        // 6. Linksy Hardware Hub
        HardwareHub {
            Layout.fillWidth: true
            device: root.currentDevice
            visible: root.isPaired && root.isReachable
        }

        // 7. Remote Tools Bar
        Kirigami.Heading {
            level: 3
            text: i18n("Tools & Input")
            visible: root.isPaired && root.isReachable
        }

        Flow {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing
            visible: root.isPaired && root.isReachable

            QQC2.Button {
                icon.name: "input-mouse"
                text: i18n("Virtual Trackpad")
                onClicked: pageStack.push(Qt.resolvedUrl("mousepad.qml"), {
                    pluginInterface: RemoteControlDbusInterfaceFactory.create(root.currentDevice.id()),
                    device: root.currentDevice
                })
            }

            QQC2.Button {
                icon.name: "preferences-desktop-notification-bell"
                text: i18n("Notification Mirror")
                onClicked: pageStack.push(Qt.resolvedUrl("NotificationPanel.qml"), {
                    pluginInterface: DeviceNotificationsDbusInterfaceFactory.create(root.currentDevice.id()),
                    device: root.currentDevice
                })
            }

            QQC2.Button {
                icon.name: "message-new"
                text: i18n("SMS Messages")
                onClicked: {
                    let sms = SmsDbusInterfaceFactory.create(root.currentDevice.id());
                    sms.launchApp();
                }
            }

            QQC2.Button {
                icon.name: "system-run"
                text: i18n("Run Commands")
                onClicked: pageStack.push(Qt.resolvedUrl("runcommand.qml"), {
                    pluginInterface: RemoteCommandsDbusInterfaceFactory.create(root.currentDevice.id()),
                    device: root.currentDevice
                })
            }
        }
    }

    FileDialog {
        id: deviceFileDialog
        title: i18n("Choose Files to Send")
        fileMode: FileDialog.OpenFiles
        onAccepted: applicationWindow().sendFiles(root.currentDevice.id(), selectedFiles)
    }
}
