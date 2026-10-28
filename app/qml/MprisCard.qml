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
    id: mprisCard

    property QtObject device
    property var mpris: null

    Component.onCompleted: {
        if (device) {
            try {
                mpris = MprisDbusInterfaceFactory.create(device.id());
            } catch (e) {
                mpris = null;
            }
        }
    }

    visible: mpris !== null && (mpris.isPlaying || (mpris.title && mpris.title.length > 0))

    banner.title: i18n("Now Playing")
    banner.titleIcon: "media-playback-start"

    contentItem: ColumnLayout {
        spacing: Kirigami.Units.largeSpacing

        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.largeSpacing

            // Album Art
            Rectangle {
                Layout.preferredWidth: Kirigami.Units.gridUnit * 4
                Layout.preferredHeight: Kirigami.Units.gridUnit * 4
                radius: Kirigami.Units.smallSpacing
                color: Qt.alpha(Kirigami.Theme.highlightColor, 0.1)
                clip: true

                Image {
                    anchors.fill: parent
                    source: (mprisCard.mpris && mprisCard.mpris.localAlbumArtUrl) ? mprisCard.mpris.localAlbumArtUrl : ""
                    fillMode: Image.PreserveAspectCrop
                    visible: status === Image.Ready
                }

                Kirigami.Icon {
                    anchors.centerIn: parent
                    width: Kirigami.Units.iconSizes.large
                    height: Kirigami.Units.iconSizes.large
                    source: "audio-x-generic"
                    color: Kirigami.Theme.highlightColor
                    visible: !mprisCard.mpris || !mprisCard.mpris.localAlbumArtUrl
                }
            }

            // Track details
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                Layout.alignment: Qt.AlignVCenter

                QQC2.Label {
                    Layout.fillWidth: true
                    text: (mprisCard.mpris && mprisCard.mpris.title) ? mprisCard.mpris.title : i18n("Media Player")
                    font.bold: true
                    font.pixelSize: Kirigami.Theme.defaultFont.pixelSize * 1.15
                    elide: Text.ElideRight
                }

                QQC2.Label {
                    Layout.fillWidth: true
                    text: (mprisCard.mpris && mprisCard.mpris.artist) ? mprisCard.mpris.artist : ""
                    opacity: 0.8
                    font.pixelSize: Kirigami.Theme.defaultFont.pixelSize
                    elide: Text.ElideRight
                    visible: text.length > 0
                }

                QQC2.Label {
                    Layout.fillWidth: true
                    text: (mprisCard.mpris && mprisCard.mpris.album) ? mprisCard.mpris.album : ""
                    opacity: 0.6
                    font.pixelSize: Kirigami.Theme.smallFont.pixelSize
                    elide: Text.ElideRight
                    visible: text.length > 0
                }
            }
        }

        // Timeline Scrub Slider
        MprisSlider {
            Layout.fillWidth: true
            plugin: mprisCard.mpris
            visible: mprisCard.mpris && mprisCard.mpris.canSeek && mprisCard.mpris.length > 0
        }

        // Playback & Volume controls
        RowLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            QQC2.ToolButton {
                icon.name: "media-skip-backward"
                Accessible.name: i18n("Previous")
                onClicked: if (mprisCard.mpris) mprisCard.mpris.sendAction("Previous")
            }

            QQC2.ToolButton {
                icon.name: (mprisCard.mpris && mprisCard.mpris.isPlaying) ? "media-playback-pause" : "media-playback-start"
                Accessible.name: (mprisCard.mpris && mprisCard.mpris.isPlaying) ? i18n("Pause") : i18n("Play")
                onClicked: if (mprisCard.mpris) mprisCard.mpris.sendAction("PlayPause")
            }

            QQC2.ToolButton {
                icon.name: "media-skip-forward"
                Accessible.name: i18n("Next")
                onClicked: if (mprisCard.mpris) mprisCard.mpris.sendAction("Next")
            }

            Item { Layout.fillWidth: true }

            Kirigami.Icon {
                source: (mprisCard.mpris && mprisCard.mpris.volume === 0) ? "audio-volume-muted" : "audio-volume-medium"
                Layout.preferredWidth: Kirigami.Units.iconSizes.small
                Layout.preferredHeight: Kirigami.Units.iconSizes.small
            }

            QQC2.Slider {
                Layout.preferredWidth: Kirigami.Units.gridUnit * 7
                from: 0
                to: 100
                value: (mprisCard.mpris && mprisCard.mpris.volume) ? mprisCard.mpris.volume : 0
                onMoved: if (mprisCard.mpris) mprisCard.mpris.volume = value
            }
        }
    }
}
