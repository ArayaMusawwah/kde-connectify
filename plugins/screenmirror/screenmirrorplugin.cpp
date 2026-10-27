/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#include "screenmirrorplugin.h"
#include <backends/linksy/linksylinkprovider.h>
#include <device.h>
#include <linksy_ffi.h>

#include <KPluginFactory>
#include <QDebug>
#include <QJsonDocument>
#include <QJsonObject>

K_PLUGIN_CLASS_WITH_JSON(ScreenMirrorPlugin, "kdeconnect_screenmirror.json")

ScreenMirrorPlugin::~ScreenMirrorPlugin()
{
    stopMirror();
}

QString ScreenMirrorPlugin::dbusPath() const
{
    return QStringLiteral("/modules/kdeconnect/devices/") + device()->id() + QStringLiteral("/screenmirror");
}

bool ScreenMirrorPlugin::startMirror(bool enableControl)
{
    if (m_active) {
        return true;
    }

    LinksyLinkProvider *linksyProvider = LinksyLinkProvider::instance();
    if (!linksyProvider) {
        qWarning() << "LinksyLinkProvider not available for ScreenMirror session";
        return false;
    }

    m_controlEnabled = enableControl;
    int kind = enableControl ? LINKSY_SESSION_CONTROL : LINKSY_SESSION_SCREEN;

    QString sessId;
    int ret = linksyProvider->requestSession(device()->id(), kind, sessId);
    if (ret != 0) {
        qWarning() << "Failed to request ScreenMirror session on" << device()->id();
        return false;
    }

    m_sessionId = sessId;
    m_active = true;

    connect(linksyProvider, &LinksyLinkProvider::sessionEnded, this, &ScreenMirrorPlugin::onSessionEnded, Qt::UniqueConnection);

    Q_EMIT activeChanged(true);
    Q_EMIT controlChanged(m_controlEnabled);
    qDebug() << "ScreenMirror session started:" << m_sessionId << "Control:" << m_controlEnabled;
    return true;
}

void ScreenMirrorPlugin::stopMirror()
{
    if (!m_active) {
        return;
    }

    m_active = false;
    m_controlEnabled = false;
    m_sessionId.clear();
    Q_EMIT activeChanged(false);
    Q_EMIT controlChanged(false);
    qDebug() << "ScreenMirror session stopped";
}

void ScreenMirrorPlugin::receivePacket(const NetworkPacket &np)
{
    if (np.type() == QLatin1String("kdeconnect.linksy.screen.control")) {
        const QJsonObject control = np.get<QJsonObject>(QStringLiteral("control"));
        Q_UNUSED(control);
    }
}

void ScreenMirrorPlugin::onSessionEnded(const QString &sessionId)
{
    if (sessionId == m_sessionId) {
        stopMirror();
    }
}

#include "screenmirrorplugin.moc"
