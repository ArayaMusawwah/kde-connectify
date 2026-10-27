/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#ifndef WEBCAMPLUGIN_H
#define WEBCAMPLUGIN_H

#include <QObject>
#include <QString>
#include <core/kdeconnectplugin.h>

class WebcamPlugin : public KdeConnectPlugin
{
    Q_OBJECT
    Q_CLASSINFO("D-Bus Interface", "org.kde.kdeconnect.device.webcam")
    Q_PROPERTY(bool active READ active NOTIFY activeChanged)
    Q_PROPERTY(bool isBackCamera READ isBackCamera NOTIFY cameraChanged)
    Q_PROPERTY(int fps READ fps NOTIFY fpsChanged)
    Q_PROPERTY(QString resolution READ resolution NOTIFY resolutionChanged)

public:
    using KdeConnectPlugin::KdeConnectPlugin;
    ~WebcamPlugin() override;

    bool active() const
    {
        return m_active;
    }
    bool isBackCamera() const
    {
        return m_isBackCamera;
    }
    int fps() const
    {
        return m_fps;
    }
    QString resolution() const
    {
        return m_resolution;
    }

    Q_SCRIPTABLE bool startWebcam();
    Q_SCRIPTABLE void stopWebcam();
    Q_SCRIPTABLE void switchCamera(bool back);
    Q_SCRIPTABLE void setFps(int fps);
    Q_SCRIPTABLE void setResolution(int width, int height);
    Q_SCRIPTABLE void setZoom(double zoom);

    void receivePacket(const NetworkPacket &np) override;
    QString dbusPath() const override;

Q_SIGNALS:
    void activeChanged(bool active);
    void cameraChanged(bool isBack);
    void fpsChanged(int fps);
    void resolutionChanged(const QString &resolution);

private Q_SLOTS:
    void onMediaDataReceived(const QString &sessionId, const QByteArray &data);
    void onSessionEnded(const QString &sessionId);

private:
    void sendControl(const QJsonObject &obj);

    bool m_active = false;
    bool m_isBackCamera = true;
    int m_fps = 30;
    QString m_resolution = QStringLiteral("1280x720");
    QString m_sessionId;
};

#endif // WEBCAMPLUGIN_H
