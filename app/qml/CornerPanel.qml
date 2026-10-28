/*
 * SPDX-FileCopyrightText: 2026 KDE Connectify contributors
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

Kirigami.AbstractCard {
    id: cornerPanel

    property bool expanded: true
    property var transferList: applicationWindow().transfers || []
    visible: transferList.length > 0

    anchors.right: parent ? parent.right : undefined
    anchors.bottom: parent ? parent.bottom : undefined
    anchors.margins: Kirigami.Units.largeSpacing

    width: Kirigami.Units.gridUnit * 18
    implicitHeight: expanded ? Kirigami.Units.gridUnit * 14 : Kirigami.Units.gridUnit * 3

    Behavior on implicitHeight {
        NumberAnimation { duration: Kirigami.Units.longDuration; easing.type: Easing.InOutQuad }
    }

    contentItem: ColumnLayout {
        spacing: Kirigami.Units.smallSpacing

        // Header
        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            Kirigami.Icon {
                source: "document-send"
                Layout.preferredWidth: Kirigami.Units.iconSizes.small
                Layout.preferredHeight: Kirigami.Units.iconSizes.small
                color: Kirigami.Theme.highlightColor
            }

            QQC2.Label {
                Layout.fillWidth: true
                text: i18np("1 Transfer", "%1 Transfers", cornerPanel.transferList.length)
                font.bold: true
                font.pixelSize: Kirigami.Theme.smallFont.pixelSize
            }

            QQC2.ToolButton {
                icon.name: cornerPanel.expanded ? "arrow-down" : "arrow-up"
                onClicked: cornerPanel.expanded = !cornerPanel.expanded
            }

            QQC2.ToolButton {
                icon.name: "dialog-close"
                onClicked: applicationWindow().clearCompletedTransfers()
            }
        }

        // Expanded transfer list
        QQC2.ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: cornerPanel.expanded
            clip: true

            ListView {
                model: cornerPanel.transferList
                spacing: Kirigami.Units.smallSpacing

                delegate: ColumnLayout {
                    width: ListView.view ? ListView.view.width : 200
                    spacing: 2

                    RowLayout {
                        Layout.fillWidth: true
                        QQC2.Label {
                            Layout.fillWidth: true
                            text: modelData.fileName || i18n("File")
                            font.bold: true
                            font.pixelSize: Kirigami.Theme.smallFont.pixelSize
                            elide: Text.ElideMiddle
                        }
                        QQC2.Label {
                            text: modelData.peerName || ""
                            opacity: 0.7
                            font.pixelSize: Kirigami.Theme.smallFont.pixelSize * 0.9
                            elide: Text.ElideRight
                        }
                    }

                    QQC2.ProgressBar {
                        Layout.fillWidth: true
                        value: modelData.progress || 0
                    }
                }
            }
        }
    }
}
