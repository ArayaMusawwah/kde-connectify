/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#include "linksydevicelink.h"
#include "kdeconnect_debug.h"
#include "linksy_ffi.h"
#include "linksylinkprovider.h"

#include <QJsonDocument>
#include <QJsonObject>

LinksyDeviceLink::LinksyDeviceLink(const DeviceInfo &deviceInfo, const QString &address, LinksyLinkProvider *parent, LinksyNodeHandle *node)
    : DeviceLink(deviceInfo.id, parent)
    , m_deviceInfo(deviceInfo)
    , m_address(address)
    , m_node(node)
{
}

LinksyDeviceLink::~LinksyDeviceLink() = default;

bool LinksyDeviceLink::sendPacket(NetworkPacket &np)
{
    if (!m_node) {
        return false;
    }

    const QByteArray devIdBytes = m_deviceInfo.id.toUtf8();
    const QString type = np.type();

    if (type == QLatin1String("kdeconnect.clipboard") || type == QLatin1String("kdeconnect.clipboard.connect")) {
        const QString text = np.get<QString>(QStringLiteral("content"), QString());
        QJsonObject root;
        root[QStringLiteral("text")] = text;
        const QByteArray payload = QJsonDocument(root).toJson(QJsonDocument::Compact);
        return linksy_send_packet(m_node, devIdBytes.constData(), "clipboard", payload.constData()) == 0;
    }

    if (type == QLatin1String("kdeconnect.ping")) {
        return linksy_send_packet(m_node, devIdBytes.constData(), "ping", "{}") == 0;
    }

    if (type == QLatin1String("kdeconnect.findmyphone.request")) {
        return linksy_send_packet(m_node, devIdBytes.constData(), "ring", "{}") == 0;
    }

    // Generic packet serialization
    const QByteArray jsonPayload = np.serialize();
    return linksy_send_packet(m_node, devIdBytes.constData(), type.toUtf8().constData(), jsonPayload.constData()) == 0;
}

void LinksyDeviceLink::injectPacket(const NetworkPacket &np)
{
    Q_EMIT receivedPacket(np);
}
