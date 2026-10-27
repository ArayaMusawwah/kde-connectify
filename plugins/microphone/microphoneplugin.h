/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#ifndef MICROPHONEPLUGIN_H
#define MICROPHONEPLUGIN_H

#include <QObject>
#include <QString>
#include <core/kdeconnectplugin.h>

class MicrophonePlugin : public KdeConnectPlugin
{
    Q_OBJECT
    Q_CLASSINFO("D-Bus Interface", "org.kde.kdeconnect.device.microphone")
    Q_PROPERTY(bool active READ active NOTIFY activeChanged)
    Q_PROPERTY(bool muted READ muted NOTIFY mutedChanged)
    Q_PROPERTY(int volume READ volume NOTIFY volumeChanged)

public:
    using KdeConnectPlugin::KdeConnectPlugin;
    ~MicrophonePlugin() override;

    bool active() const
    {
        return m_active;
    }
    bool muted() const
    {
        return m_muted;
    }
    int volume() const
    {
        return m_volume;
    }

    Q_SCRIPTABLE bool startMicrophone();
    Q_SCRIPTABLE void stopMicrophone();
    Q_SCRIPTABLE void setMuted(bool mute);
    Q_SCRIPTABLE void setVolume(int volume);

    void receivePacket(const NetworkPacket &np) override;
    QString dbusPath() const override;

Q_SIGNALS:
    void activeChanged(bool active);
    void mutedChanged(bool muted);
    void volumeChanged(int volume);

private Q_SLOTS:
    void onMediaDataReceived(const QString &sessionId, const QByteArray &data);
    void onSessionEnded(const QString &sessionId);

private:
    bool m_active = false;
    bool m_muted = false;
    int m_volume = 100;
    QString m_sessionId;
};

#endif // MICROPHONEPLUGIN_H
