/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#include "webcamplugin.h"
#include <KPluginFactory>
#include <QDebug>
#include <QJsonDocument>
#include <QJsonObject>
#include <backends/linksy/linksylinkprovider.h>
#include <daemon.h>
#include <device.h>
#include <linksy_ffi.h>

K_PLUGIN_CLASS_WITH_JSON(WebcamPlugin, "kdeconnect_webcam.json")

WebcamPlugin::~WebcamPlugin()
{
    stopWebcam();
}

QString WebcamPlugin::dbusPath() const
{
    return QStringLiteral("/modules/kdeconnect/devices/") + device()->id() + QStringLiteral("/webcam");
}

bool WebcamPlugin::startWebcam()
{
    if (m_active) {
        return true;
    }

    LinksyLinkProvider *linksyProvider = LinksyLinkProvider::instance();
    if (!linksyProvider) {
        qWarning() << "LinksyLinkProvider not available for Webcam session";
        return false;
    }

    QString sessId;
    int ret = linksyProvider->requestSession(device()->id(), LINKSY_SESSION_CAMERA, sessId);
    if (ret != 0) {
        qWarning() << "Failed to request Webcam session on" << device()->id();
        return false;
    }

    m_sessionId = sessId;
    m_active = true;

    connect(linksyProvider, &LinksyLinkProvider::mediaDataReceived, this, &WebcamPlugin::onMediaDataReceived, Qt::UniqueConnection);
    connect(linksyProvider, &LinksyLinkProvider::sessionEnded, this, &WebcamPlugin::onSessionEnded, Qt::UniqueConnection);

    Q_EMIT activeChanged(true);
    qDebug() << "Webcam session started:" << m_sessionId;
    return true;
}

void WebcamPlugin::stopWebcam()
{
    if (!m_active) {
        return;
    }

    m_active = false;
    m_sessionId.clear();
    Q_EMIT activeChanged(false);
    qDebug() << "Webcam session stopped";
}

void WebcamPlugin::switchCamera(bool back)
{
    m_isBackCamera = back;
    QJsonObject obj;
    obj[QStringLiteral("camera")] = back ? QStringLiteral("back") : QStringLiteral("front");
    sendControl(obj);
    Q_EMIT cameraChanged(back);
}

void WebcamPlugin::setFps(int fps)
{
    m_fps = fps;
    QJsonObject obj;
    obj[QStringLiteral("fps")] = fps;
    sendControl(obj);
    Q_EMIT fpsChanged(fps);
}

void WebcamPlugin::setResolution(int width, int height)
{
    m_resolution = QString::asprintf("%dx%d", width, height);
    QJsonObject obj;
    obj[QStringLiteral("width")] = width;
    obj[QStringLiteral("height")] = height;
    sendControl(obj);
    Q_EMIT resolutionChanged(m_resolution);
}

void WebcamPlugin::setZoom(double zoom)
{
    QJsonObject obj;
    obj[QStringLiteral("zoom")] = zoom;
    sendControl(obj);
}

void WebcamPlugin::sendControl(const QJsonObject &obj)
{
    NetworkPacket np(QStringLiteral("kdeconnect.linksy.camera.control"));
    np.set(QStringLiteral("control"), obj);
    sendPacket(np);
}

void WebcamPlugin::receivePacket(const NetworkPacket &np)
{
    if (np.type() == QLatin1String("kdeconnect.linksy.camera.control")) {
        const QJsonObject control = np.get<QJsonObject>(QStringLiteral("control"));
        if (control.contains(QStringLiteral("fps"))) {
            m_fps = control.value(QStringLiteral("fps")).toInt();
            Q_EMIT fpsChanged(m_fps);
        }
    }
}

void WebcamPlugin::onMediaDataReceived(const QString &sessionId, const QByteArray &data)
{
    if (sessionId != m_sessionId || !m_active) {
        return;
    }
    // Feed data to PipeWire virtual webcam stream
    Q_UNUSED(data);
}

void WebcamPlugin::onSessionEnded(const QString &sessionId)
{
    if (sessionId == m_sessionId) {
        stopWebcam();
    }
}

#include "webcamplugin.moc"
