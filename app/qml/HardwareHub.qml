/*
 * SPDX-FileCopyrightText: 2026 KDE Connectify contributors
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import org.kde.kdeconnect

ColumnLayout {
    id: hub

    property QtObject device
    spacing: Kirigami.Units.largeSpacing

    Kirigami.Heading {
        level: 3
        text: i18n("Hardware Hub")
    }

    Flow {
        Layout.fillWidth: true
        spacing: Kirigami.Units.largeSpacing

        HardwareCard {
            title: i18n("Virtual Webcam")
            subtitle: i18n("Phone camera as desktop webcam")
            iconName: "camera-web"
            pluginName: "webcam"
            startMethod: "startWebcam"
            stopMethod: "stopWebcam"
            device: hub.device
        }

        HardwareCard {
            title: i18n("Virtual Microphone")
            subtitle: i18n("Phone audio as desktop microphone")
            iconName: "audio-input-microphone"
            pluginName: "microphone"
            startMethod: "startMicrophone"
            stopMethod: "stopMicrophone"
            device: hub.device
        }

        HardwareCard {
            title: i18n("Second Display")
            subtitle: i18n("Use phone/tablet as Sidecar monitor")
            iconName: "video-display"
            pluginName: "sidecar"
            startMethod: "startSidecar"
            stopMethod: "stopSidecar"
            device: hub.device
        }

        HardwareCard {
            title: i18n("Remote Terminal Shell")
            subtitle: i18n("Interactive PTY shell over QUIC")
            iconName: "utilities-terminal"
            pluginName: "terminal"
            startMethod: "startTerminal"
            stopMethod: "stopTerminal"
            device: hub.device
        }
    }

    component HardwareCard: Kirigami.Card {
        id: card

        property string title
        property string subtitle
        property string iconName
        property string pluginName
        property string startMethod
        property string stopMethod
        property QtObject device
        property bool active: false

        width: Kirigami.Units.gridUnit * 13
        banner.title: card.title
        banner.titleIcon: card.iconName

        contentItem: QQC2.Label {
            text: card.active ? i18n("Active now") : card.subtitle
            wrapMode: Text.Wrap
            opacity: card.active ? 1.0 : 0.7
            font: Kirigami.Theme.smallFont
            color: card.active ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.textColor
        }

        actions: [
            Kirigami.Action {
                text: card.active ? i18n("Stop") : i18n("Start")
                icon.name: card.active ? "media-playback-stop" : "media-playback-start"
                onTriggered: {
                    if (card.device) {
                        card.device.pluginCall(card.pluginName, card.active ? card.stopMethod : card.startMethod);
                        card.active = !card.active;
                    }
                }
            }
        ]
    }
}
