/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#pragma once

#include <QObject>
#include <QPoint>
#include <QString>

#include <core/kdeconnectplugin.h>

/// ADB-driven remote control of an Android phone, like scrcpy does from a
/// computer: the PC runs `adb shell input tap/swipe/text/keyevent` (and
/// scrcpy's screen mirroring) instead of having an on-phone app inject
/// anything. No AccessibilityService or overlay needed on the phone.
class AdbControlPlugin : public KdeConnectPlugin
{
    Q_OBJECT
    Q_CLASSINFO("D-Bus Interface", "org.kde.kdeconnect.device.adbcontrol")

public:
    using KdeConnectPlugin::KdeConnectPlugin;

    QString dbusPath() const override;

    /// Connect to [address] over USB/UDP (`adb connect` / `-s`), so later
    /// commands target that device. Empty [address] = the only attached
    /// device.
    Q_SCRIPTABLE bool connectDevice(const QString &address = {});
    Q_SCRIPTABLE void disconnectDevice();

    /// Touch: absolute tap at [p] (device pixels), hold [ms] ms.
    Q_SCRIPTABLE void tap(const QPoint &p, int holdMs = 40);
    /// Touch: swipe from [from] to [to] over [ms] ms (1 = default).
    Q_SCRIPTABLE void swipe(const QPoint &from, const QPoint &to, int ms = 200);
    /// Key: [keycode] is an Android `keyevent` number (e.g. 4 = BACK).
    Q_SCRIPTABLE void keyEvent(int keycode);
    /// Type [text] into the focused field.
    Q_SCRIPTABLE void typeText(const QString &text);

    /// Screen: show scrcpy for this device (needs the `scrcpy` binary).
    Q_SCRIPTABLE bool startScrcpy(int maxSize = 1080, int bitrate = 8000000, int fps = 30);
    Q_SCRIPTABLE void stopScrcpy();

Q_SIGNALS:
    void outputLine(const QString &line);

private:
    QString m_serial;
    QString adb() const;
    void runAdb(const QStringList &args, bool wait = false);
};
