/*
 * SPDX-FileCopyrightText: 2026 KDE Connectify contributors
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import org.kde.kdeconnect

Rectangle {
    id: island

    readonly property int activeFiles: applicationWindow().activeTransfersCount || 0
    readonly property real progressFraction: applicationWindow().transfersFraction || 0

    Kirigami.Theme.colorSet: Kirigami.Theme.View
    Kirigami.Theme.inherit: false

    implicitHeight: Kirigami.Units.gridUnit * 2.8
    radius: height / 2
    color: Kirigami.Theme.backgroundColor
    border.color: Qt.alpha(Kirigami.Theme.textColor, 0.12)
    border.width: 1

    RowLayout {
        anchors.fill: parent
        anchors.margins: 4
        spacing: 4

        // 1. Transfer progress circle / icon
        QQC2.AbstractButton {
            id: transfersBtn
            Layout.preferredWidth: island.height - 8
            Layout.preferredHeight: island.height - 8
            hoverEnabled: true
            QQC2.ToolTip.text: island.activeFiles > 0 ? i18np("1 file transferring", "%1 files transferring", island.activeFiles) : i18n("No active transfers")
            QQC2.ToolTip.visible: hovered
            onClicked: {
                if (island.activeFiles > 0) {
                    applicationWindow().toggleCornerPanel();
                }
            }

            background: Rectangle {
                radius: width / 2
                color: transfersBtn.hovered ? Qt.alpha(Kirigami.Theme.highlightColor, 0.15) : "transparent"
            }

            contentItem: Item {
                Canvas {
                    id: progressRing
                    anchors.fill: parent
                    anchors.margins: 2
                    visible: island.activeFiles > 0
                    readonly property real val: island.progressFraction
                    onValChanged: requestPaint()
                    onVisibleChanged: requestPaint()
                    onPaint: {
                        let ctx = getContext("2d");
                        ctx.reset();
                        let r = width / 2 - 2;
                        ctx.lineWidth = 3;
                        ctx.strokeStyle = Qt.alpha(Kirigami.Theme.highlightColor, 0.2);
                        ctx.beginPath();
                        ctx.arc(width / 2, height / 2, r, 0, Math.PI * 2);
                        ctx.stroke();

                        ctx.lineCap = "round";
                        ctx.strokeStyle = Kirigami.Theme.highlightColor;
                        ctx.beginPath();
                        ctx.arc(width / 2, height / 2, r, -Math.PI / 2, -Math.PI / 2 + val * Math.PI * 2);
                        ctx.stroke();
                    }
                }

                Kirigami.Icon {
                    anchors.centerIn: parent
                    width: Kirigami.Units.iconSizes.small
                    height: Kirigami.Units.iconSizes.small
                    source: island.activeFiles > 0 ? "document-send" : "network-connect"
                    color: Kirigami.Theme.highlightColor
                }
            }
        }

        // 2. Action Button: Pair New Device
        QQC2.Button {
            Layout.fillWidth: true
            Layout.fillHeight: true
            icon.name: "list-add"
            text: i18n("Add Device (QR)")
            highlighted: true
            onClicked: applicationWindow().openInviteDialog()
        }
    }
}
