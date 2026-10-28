/*
 * SPDX-FileCopyrightText: 2026 KDE Connectify contributors
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Layouts
import org.kde.kirigami as Kirigami

QQC2.AbstractButton {
    id: tile

    property string iconName: ""
    property string title: ""
    property string subtitle: ""

    width: Kirigami.Units.gridUnit * 12.5
    height: Kirigami.Units.gridUnit * 4.2
    padding: Kirigami.Units.largeSpacing
    hoverEnabled: true

    background: Rectangle {
        radius: Kirigami.Units.cornerRadius
        color: tile.down ? Qt.darker(Kirigami.Theme.alternateBackgroundColor, 1.08)
             : tile.hovered ? Kirigami.Theme.alternateBackgroundColor
             : Kirigami.Theme.backgroundColor
        border.color: tile.visualFocus ? Kirigami.Theme.focusColor : Qt.rgba(Kirigami.Theme.textColor.r, Kirigami.Theme.textColor.g, Kirigami.Theme.textColor.b, 0.15)
        border.width: 1

        Behavior on color {
            ColorAnimation { duration: Kirigami.Units.shortDuration }
        }
    }

    contentItem: RowLayout {
        spacing: Kirigami.Units.largeSpacing

        Kirigami.Icon {
            source: tile.iconName
            Layout.preferredWidth: Kirigami.Units.iconSizes.medium
            Layout.preferredHeight: Kirigami.Units.iconSizes.medium
            color: Kirigami.Theme.highlightColor
        }

        ColumnLayout {
            spacing: 2
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter

            QQC2.Label {
                text: tile.title
                font.bold: true
                Layout.fillWidth: true
                elide: Text.ElideRight
                color: Kirigami.Theme.textColor
            }

            QQC2.Label {
                text: tile.subtitle
                opacity: 0.7
                font: Kirigami.Theme.smallFont
                wrapMode: Text.Wrap
                maximumLineCount: 2
                elide: Text.ElideRight
                Layout.fillWidth: true
                color: Kirigami.Theme.textColor
            }
        }
    }
}
