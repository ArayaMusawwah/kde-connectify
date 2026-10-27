/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#ifndef SIDECARPLUGIN_H
#define SIDECARPLUGIN_H

#include <QObject>
#include <QString>
#include <core/kdeconnectplugin.h>

class SidecarPlugin : public KdeConnectPlugin
{
    Q_OBJECT
    Q_CLASSINFO("D-Bus Interface", "org.kde.kdeconnect.device.sidecar")
    Q_PROPERTY(bool active READ active NOTIFY activeChanged)
    Q_PROPERTY(int width READ width NOTIFY resolutionChanged)
    Q_PROPERTY(int height READ height NOTIFY resolutionChanged)
    Q_PROPERTY(int fps READ fps NOTIFY fpsChanged)

public:
    using KdeConnectPlugin::KdeConnectPlugin;
    ~SidecarPlugin() override;

    bool active() const
    {
        return m_active;
    }
    int width() const
    {
        return m_width;
    }
    int height() const
    {
        return m_height;
    }
    int fps() const
    {
        return m_fps;
    }

    Q_SCRIPTABLE bool startSidecar(int width = 1920, int height = 1080);
    Q_SCRIPTABLE void stopSidecar();

    void receivePacket(const NetworkPacket &np) override;
    QString dbusPath() const override;

Q_SIGNALS:
    void activeChanged(bool active);
    void resolutionChanged(int width, int height);
    void fpsChanged(int fps);

private Q_SLOTS:
    void onSessionEnded(const QString &sessionId);

private:
    bool m_active = false;
    int m_width = 1920;
    int m_height = 1080;
    int m_fps = 60;
    QString m_sessionId;
};

#endif // SIDECARPLUGIN_H
