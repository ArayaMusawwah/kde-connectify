/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#ifndef SCREENMIRRORPLUGIN_H
#define SCREENMIRRORPLUGIN_H

#include <QObject>
#include <QString>
#include <core/kdeconnectplugin.h>

class ScreenMirrorPlugin : public KdeConnectPlugin
{
    Q_OBJECT
    Q_CLASSINFO("D-Bus Interface", "org.kde.kdeconnect.device.screenmirror")
    Q_PROPERTY(bool active READ active NOTIFY activeChanged)
    Q_PROPERTY(bool controlEnabled READ controlEnabled NOTIFY controlChanged)

public:
    using KdeConnectPlugin::KdeConnectPlugin;
    ~ScreenMirrorPlugin() override;

    bool active() const
    {
        return m_active;
    }
    bool controlEnabled() const
    {
        return m_controlEnabled;
    }

    Q_SCRIPTABLE bool startMirror(bool enableControl = false);
    Q_SCRIPTABLE void stopMirror();

    void receivePacket(const NetworkPacket &np) override;
    QString dbusPath() const override;

Q_SIGNALS:
    void activeChanged(bool active);
    void controlChanged(bool enabled);

private Q_SLOTS:
    void onSessionEnded(const QString &sessionId);

private:
    bool m_active = false;
    bool m_controlEnabled = false;
    QString m_sessionId;
};

#endif // SCREENMIRRORPLUGIN_H
