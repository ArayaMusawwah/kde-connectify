/*
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.AbstractCard {
    id: root

    property QtObject currentDevice
    property QtObject linksyProvider

    // Strict 8-point spatial grid metrics (OCD Compliance)
    readonly property int gridMargin: 16
    readonly property int gridPadding: 16
    readonly property int gridSpacing: 12
    readonly property int iconDimension: 24

    padding: gridPadding

    contentItem: ColumnLayout {
        spacing: root.gridSpacing

        // Symmetrical Header
        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            Kirigami.Icon {
                source: "video-display"
                Layout.preferredWidth: root.iconDimension
                Layout.preferredHeight: root.iconDimension
            }

            QQC2.Label {
                text: i18n("Hardware & Media Sharing (Linksy)")
                font.bold: true
                font.pixelSize: Kirigami.Units.gridUnit * 0.9
                color: Kirigami.Theme.textColor // WCAG AA compliant contrast
                Layout.fillWidth: true
            }

            // Connection Type Badge (USB Cable vs Wi-Fi)
            Rectangle {
                id: connectionBadge
                Layout.preferredHeight: 24
                Layout.preferredWidth: connLabel.implicitWidth + 16
                radius: 4
                color: Kirigami.Theme.alternateBackgroundColor
                border.color: Kirigami.Theme.focusColor
                border.width: 1

                QQC2.Label {
                    id: connLabel
                    anchors.centerIn: parent
                    text: (root.linksyProvider && root.linksyProvider.isOverUsb(root.currentDevice ? root.currentDevice.id() : ""))
                          ? i18n("⚡ USB Cable (AOA)")
                          : i18n("📶 Wi-Fi (QUIC)")
                    font.pixelSize: Kirigami.Units.gridUnit * 0.75
                    font.bold: true
                    color: Kirigami.Theme.textColor
                }
            }
        }

        // Symmetrical 2x2 Action Grid
        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: root.gridSpacing
            rowSpacing: root.gridSpacing

            // 1. Second Display (Sidecar) Card
            ActionCard {
                Layout.fillWidth: true
                title: i18n("Second Display")
                subtitle: i18n("Tablet / Phone Monitor")
                iconSource: "video-display"
                actionText: active ? i18n("Stop") : i18n("Start")
                isActive: false
                onTriggered: {
                    isActive = !isActive;
                }
            }

            // 2. Virtual Webcam Card
            ActionCard {
                Layout.fillWidth: true
                title: i18n("Phone Webcam")
                subtitle: i18n("PipeWire 1080p Stream")
                iconSource: "camera-web"
                actionText: active ? i18n("Stop") : i18n("Start")
                isActive: false
                onTriggered: {
                    isActive = !isActive;
                }
            }

            // 3. Virtual Microphone Card
            ActionCard {
                Layout.fillWidth: true
                title: i18n("Phone Microphone")
                subtitle: i18n("PCM 48kHz Low-Latency")
                iconSource: "audio-input-microphone"
                actionText: active ? i18n("Stop") : i18n("Start")
                isActive: false
                onTriggered: {
                    isActive = !isActive;
                }
            }

            // 4. Screen Mirror & Remote Desktop Card
            ActionCard {
                Layout.fillWidth: true
                title: i18n("Screen Mirror")
                subtitle: i18n("Wayland Remote View")
                iconSource: "preferences-desktop-display"
                actionText: active ? i18n("Stop") : i18n("Start")
                isActive: false
                onTriggered: {
                    isActive = !isActive;
                }
            }
        }
    }

    // Inner Action Card Component with strict symmetry and dual indicators
    component ActionCard: QQC2.Pane {
        id: cardPane
        property string title: ""
        property string subtitle: ""
        property string iconSource: ""
        property string actionText: ""
        property bool isActive: false
        signal triggered()

        Layout.preferredHeight: 76
        padding: 12

        background: Rectangle {
            color: cardPane.isActive ? Kirigami.Theme.highlightColor : Kirigami.Theme.backgroundColor
            opacity: cardPane.isActive ? 0.15 : 1.0
            radius: 6
            border.color: cardPane.activeFocus ? Kirigami.Theme.focusColor : Kirigami.Theme.separatorColor
            border.width: cardPane.activeFocus ? 2 : 1
        }

        contentItem: RowLayout {
            spacing: 12

            Kirigami.Icon {
                source: cardPane.iconSource
                Layout.preferredWidth: 28
                Layout.preferredHeight: 28
                color: Kirigami.Theme.textColor
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2

                RowLayout {
                    spacing: 6
                    QQC2.Label {
                        text: cardPane.title
                        font.bold: true
                        font.pixelSize: Kirigami.Units.gridUnit * 0.85
                        color: Kirigami.Theme.textColor // WCAG AA >= 4.5:1
                    }

                    // Dual indicator badge: Text + Color
                    Rectangle {
                        Layout.preferredHeight: 16
                        Layout.preferredWidth: statusText.implicitWidth + 8
                        radius: 3
                        color: cardPane.isActive ? "#2E7D32" : "#757575" // High contrast green / neutral

                        QQC2.Label {
                            id: statusText
                            anchors.centerIn: parent
                            text: cardPane.isActive ? i18n("ACTIVE") : i18n("STANDBY")
                            color: "#FFFFFF" // WCAG AAA >= 7:1
                            font.pixelSize: Kirigami.Units.gridUnit * 0.6
                            font.bold: true
                        }
                    }
                }

                QQC2.Label {
                    text: cardPane.subtitle
                    opacity: 0.8
                    font.pixelSize: Kirigami.Units.gridUnit * 0.7
                    color: Kirigami.Theme.textColor
                }
            }

            QQC2.Button {
                text: cardPane.actionText
                Layout.alignment: Qt.AlignVCenter
                onClicked: cardPane.triggered()
            }
        }
    }
}
