/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#include "microphoneplugin.h"
#include <KPluginFactory>
#include <QDebug>
#include <QJsonDocument>
#include <QJsonObject>
#include <backends/linksy/linksylinkprovider.h>
#include <daemon.h>
#include <device.h>
#include <linksy_ffi.h>

K_PLUGIN_CLASS_WITH_JSON(MicrophonePlugin, "kdeconnect_microphone.json")

MicrophonePlugin::~MicrophonePlugin()
{
    stopMicrophone();
}

QString MicrophonePlugin::dbusPath() const
{
    return QStringLiteral("/modules/kdeconnect/devices/") + device()->id() + QStringLiteral("/microphone");
}

bool MicrophonePlugin::startMicrophone()
{
    if (m_active) {
        return true;
    }

    LinksyLinkProvider *linksyProvider = LinksyLinkProvider::instance();
    if (!linksyProvider) {
        qWarning() << "LinksyLinkProvider not available for Microphone session";
        return false;
    }

    QString sessId;
    int ret = linksyProvider->requestSession(device()->id(), LINKSY_SESSION_MICROPHONE, sessId);
    if (ret != 0) {
        qWarning() << "Failed to request Microphone session on" << device()->id();
        return false;
    }

    m_sessionId = sessId;
    m_active = true;

    connect(linksyProvider, &LinksyLinkProvider::mediaDataReceived, this, &MicrophonePlugin::onMediaDataReceived, Qt::UniqueConnection);
    connect(linksyProvider, &LinksyLinkProvider::sessionEnded, this, &MicrophonePlugin::onSessionEnded, Qt::UniqueConnection);

    Q_EMIT activeChanged(true);
    qDebug() << "Microphone session started:" << m_sessionId;
    return true;
}

void MicrophonePlugin::stopMicrophone()
{
    if (!m_active) {
        return;
    }

    m_active = false;
    m_sessionId.clear();
    Q_EMIT activeChanged(false);
    qDebug() << "Microphone session stopped";
}

void MicrophonePlugin::setMuted(bool mute)
{
    m_muted = mute;
    QJsonObject obj;
    obj[QStringLiteral("muted")] = mute;
    NetworkPacket np(QStringLiteral("kdeconnect.linksy.mic.control"));
    np.set(QStringLiteral("control"), obj);
    sendPacket(np);
    Q_EMIT mutedChanged(mute);
}

void MicrophonePlugin::setVolume(int volume)
{
    m_volume = qBound(0, volume, 100);
    QJsonObject obj;
    obj[QStringLiteral("volume")] = m_volume;
    NetworkPacket np(QStringLiteral("kdeconnect.linksy.mic.control"));
    np.set(QStringLiteral("control"), obj);
    sendPacket(np);
    Q_EMIT volumeChanged(m_volume);
}

void MicrophonePlugin::receivePacket(const NetworkPacket &np)
{
    if (np.type() == QLatin1String("kdeconnect.linksy.mic.control")) {
        const QJsonObject control = np.get<QJsonObject>(QStringLiteral("control"));
        if (control.contains(QStringLiteral("muted"))) {
            m_muted = control.value(QStringLiteral("muted")).toBool();
            Q_EMIT mutedChanged(m_muted);
        }
    }
}

void MicrophonePlugin::onMediaDataReceived(const QString &sessionId, const QByteArray &data)
{
    if (sessionId != m_sessionId || !m_active || m_muted) {
        return;
    }
    // Feed 16-bit 48kHz mono PCM to PipeWire audio source node
    Q_UNUSED(data);
}

void MicrophonePlugin::onSessionEnded(const QString &sessionId)
{
    if (sessionId == m_sessionId) {
        stopMicrophone();
    }
}

#include "microphoneplugin.moc"
