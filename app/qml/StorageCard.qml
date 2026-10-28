/*
 * SPDX-FileCopyrightText: 2026 KDE Connectify contributors
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import org.kde.kdeconnect

Kirigami.Card {
    id: storageCard

    property QtObject device
    property var sftp: null

    Component.onCompleted: {
        if (device) {
            try {
                sftp = SftpDbusInterfaceFactory.create(device.id());
            } catch (e) {
                sftp = null;
            }
        }
    }

    banner.title: i18n("Device Storage")
    banner.titleIcon: "drive-harddisk"

    contentItem: ColumnLayout {
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.largeSpacing

            Rectangle {
                Layout.preferredWidth: Kirigami.Units.gridUnit * 2.8
                Layout.preferredHeight: Kirigami.Units.gridUnit * 2.8
                radius: Kirigami.Units.smallSpacing
                color: Qt.alpha(Kirigami.Theme.highlightColor, 0.12)

                Kirigami.Icon {
                    anchors.centerIn: parent
                    source: "folder-remote"
                    width: Kirigami.Units.iconSizes.medium
                    height: Kirigami.Units.iconSizes.medium
                    color: Kirigami.Theme.highlightColor
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                Layout.alignment: Qt.AlignVCenter

                QQC2.Label {
                    Layout.fillWidth: true
                    text: i18n("Remote Storage Filesystem")
                    font.bold: true
                    font.pixelSize: Kirigami.Theme.defaultFont.pixelSize
                    elide: Text.ElideRight
                }

                QQC2.Label {
                    Layout.fillWidth: true
                    text: i18n("Access phone files in Dolphin via SFTP")
                    opacity: 0.7
                    font.pixelSize: Kirigami.Theme.smallFont.pixelSize
                    elide: Text.ElideRight
                }
            }

            QQC2.Button {
                icon.name: "system-file-manager"
                text: i18n("Open in Dolphin")
                highlighted: true
                onClicked: {
                    if (storageCard.sftp) {
                        storageCard.sftp.startBrowsing();
                    } else if (storageCard.device) {
                        Qt.openUrlExternally("kdeconnect://" + storageCard.device.id() + "/");
                    }
                }
            }
        }
    }
}
