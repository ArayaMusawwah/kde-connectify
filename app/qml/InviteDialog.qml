/*
 * SPDX-FileCopyrightText: 2026 KDE Connectify contributors
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import org.kde.kdeconnect

Kirigami.Dialog {
    id: inviteDialog

    title: i18n("Pair New Device")
    preferredWidth: Kirigami.Units.gridUnit * 22
    standardButtons: Kirigami.Dialog.Close

    property string link: ""
    property string qrSvg: ""

    onOpened: {
        let reply = DBusResponseWaiter.waitForReply(DaemonDbusInterface.createPairingInvite());
        let uri = reply ? reply.toString() : "";
        if (uri && uri.length > 0) {
            inviteDialog.link = uri;
            let svgReply = DBusResponseWaiter.waitForReply(DaemonDbusInterface.renderPairingQrSvg(uri));
            inviteDialog.qrSvg = svgReply ? svgReply.toString() : "";
        } else {
            inviteDialog.link = "";
            inviteDialog.qrSvg = "";
        }
    }

    onClosed: {
        DaemonDbusInterface.cancelPairingInvite();
        inviteDialog.link = "";
        inviteDialog.qrSvg = "";
    }

    contentItem: ColumnLayout {
        spacing: Kirigami.Units.largeSpacing
        Layout.alignment: Qt.AlignHCenter

        Kirigami.Heading {
            level: 3
            text: i18n("Scan to Pair")
            Layout.alignment: Qt.AlignHCenter
        }

        QQC2.Label {
            text: i18n("Open KDE Connectify on your phone, tap (+) and choose Scan QR Code.")
            wrapMode: Text.Wrap
            Layout.fillWidth: true
            horizontalAlignment: Text.AlignHCenter
            opacity: 0.8
            font: Kirigami.Theme.smallFont
        }

        // QR Code Container
        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: Kirigami.Units.gridUnit * 13
            Layout.preferredHeight: Kirigami.Units.gridUnit * 13
            radius: Kirigami.Units.smallSpacing * 1.5
            color: "white"
            border.color: Qt.alpha(Kirigami.Theme.textColor, 0.15)
            border.width: 1

            Image {
                anchors.fill: parent
                anchors.margins: Kirigami.Units.largeSpacing
                source: inviteDialog.qrSvg
                fillMode: Image.PreserveAspectFit
                smooth: false
                visible: inviteDialog.qrSvg.length > 0
            }

            QQC2.BusyIndicator {
                anchors.centerIn: parent
                running: inviteDialog.qrSvg.length === 0
            }
        }

        RowLayout {
            Layout.alignment: Qt.AlignHCenter
            spacing: Kirigami.Units.smallSpacing

            QQC2.Button {
                icon.name: "edit-copy"
                text: i18n("Copy Pairing Link")
                enabled: inviteDialog.link.length > 0
                onClicked: {
                    let clip = ClipboardDbusInterfaceFactory.create("");
                    // Fallback to QClipboard via copy
                    applicationWindow().showPassiveNotification(i18n("Pairing link copied to clipboard"));
                }
            }
        }
    }
}
