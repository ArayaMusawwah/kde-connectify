/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#include "sidecarplugin.h"
#include <backends/linksy/linksylinkprovider.h>
#include <device.h>
#include <linksy_ffi.h>

#include <KPluginFactory>
#include <QDebug>
#include <QJsonDocument>
#include <QJsonObject>

K_PLUGIN_CLASS_WITH_JSON(SidecarPlugin, "kdeconnect_sidecar.json")

SidecarPlugin::~SidecarPlugin()
{
    stopSidecar();
}

QString SidecarPlugin::dbusPath() const
{
    return QStringLiteral("/modules/kdeconnect/devices/") + device()->id() + QStringLiteral("/sidecar");
}

bool SidecarPlugin::startSidecar(int width, int height)
{
    if (m_active) {
        return true;
    }

    LinksyLinkProvider *linksyProvider = LinksyLinkProvider::instance();
    if (!linksyProvider) {
        qWarning() << "LinksyLinkProvider not available for Sidecar session";
        return false;
    }

    m_width = width > 0 ? width : 1920;
    m_height = height > 0 ? height : 1080;

    QString sessId;
    int ret = linksyProvider->requestSession(device()->id(), LINKSY_SESSION_DISPLAY, sessId);
    if (ret != 0) {
        qWarning() << "Failed to request Sidecar session on" << device()->id();
        return false;
    }

    m_sessionId = sessId;
    m_active = true;

    connect(linksyProvider, &LinksyLinkProvider::sessionEnded, this, &SidecarPlugin::onSessionEnded, Qt::UniqueConnection);

    Q_EMIT activeChanged(true);
    Q_EMIT resolutionChanged(m_width, m_height);
    qDebug() << "Sidecar virtual display started:" << m_sessionId << "Resolution:" << m_width << "x" << m_height;
    return true;
}

void SidecarPlugin::stopSidecar()
{
    if (!m_active) {
        return;
    }

    m_active = false;
    m_sessionId.clear();
    Q_EMIT activeChanged(false);
    qDebug() << "Sidecar session stopped";
}

void SidecarPlugin::receivePacket(const NetworkPacket &np)
{
    if (np.type() == QLatin1String("kdeconnect.linksy.sidecar.control")) {
        const QJsonObject control = np.get<QJsonObject>(QStringLiteral("control"));
        // Handle input events from the tablet (pointer, button, scroll, key)
        const QString type = control.value(QStringLiteral("type")).toString();
        if (type == QLatin1String("pointer")) {
            double x = control.value(QStringLiteral("x")).toDouble();
            double y = control.value(QStringLiteral("y")).toDouble();
            // Inject touch coordinates to KWin VirtualInput
            Q_UNUSED(x);
            Q_UNUSED(y);
        }
    }
}

void SidecarPlugin::onSessionEnded(const QString &sessionId)
{
    if (sessionId == m_sessionId) {
        stopSidecar();
    }
}

#include "sidecarplugin.moc"
