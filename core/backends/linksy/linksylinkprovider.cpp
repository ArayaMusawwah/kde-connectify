/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#include "linksylinkprovider.h"
#include "kdeconnect_debug.h"
#include "kdeconnectconfig.h"
#include "linksy_ffi.h"

#include <QDir>
#include <QHostInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QMetaObject>
#include <QStandardPaths>

namespace
{

extern "C" {
void c_device_cb(const char *device_id, const char *name, const char *ip, uint16_t port, int is_lost, void *user_data)
{
    auto *provider = static_cast<LinksyLinkProvider *>(user_data);
    if (!provider)
        return;
    const QString qDeviceId = QString::fromUtf8(device_id ? device_id : "");
    const QString qName = QString::fromUtf8(name ? name : "");
    const QString qIp = QString::fromUtf8(ip ? ip : "");
    QMetaObject::invokeMethod(
        provider,
        [provider, qDeviceId, qName, qIp, port, is_lost]() {
            provider->handleDeviceDiscovered(qDeviceId, qName, qIp, port, is_lost != 0);
        },
        Qt::QueuedConnection);
}

void c_pair_code_cb(const char *request_id, const char *device_id, const char *code, int incoming, void *user_data)
{
    auto *provider = static_cast<LinksyLinkProvider *>(user_data);
    if (!provider)
        return;
    const QString qReqId = QString::fromUtf8(request_id ? request_id : "");
    const QString qDevId = QString::fromUtf8(device_id ? device_id : "");
    const QString qCode = QString::fromUtf8(code ? code : "");
    QMetaObject::invokeMethod(
        provider,
        [provider, qReqId, qDevId, qCode, incoming]() {
            provider->handlePairingRequest(qReqId, qDevId, qCode, incoming != 0);
        },
        Qt::QueuedConnection);
}

void c_packet_cb(const char *device_id, const char *packet_type, const char *payload_json, void *user_data)
{
    auto *provider = static_cast<LinksyLinkProvider *>(user_data);
    if (!provider)
        return;
    const QString qDevId = QString::fromUtf8(device_id ? device_id : "");
    const QString qType = QString::fromUtf8(packet_type ? packet_type : "");
    const QString qPayload = QString::fromUtf8(payload_json ? payload_json : "{}");
    QMetaObject::invokeMethod(
        provider,
        [provider, qDevId, qType, qPayload]() {
            provider->handlePacketReceived(qDevId, qType, qPayload);
        },
        Qt::QueuedConnection);
}

void c_session_cb(const char *session_id, const char *device_id, int session_kind, int event_type, void *user_data)
{
    auto *provider = static_cast<LinksyLinkProvider *>(user_data);
    if (!provider)
        return;
    const QString qSessId = QString::fromUtf8(session_id ? session_id : "");
    const QString qDevId = QString::fromUtf8(device_id ? device_id : "");
    QMetaObject::invokeMethod(
        provider,
        [provider, qSessId, qDevId, session_kind, event_type]() {
            provider->handleSessionEvent(qSessId, qDevId, session_kind, event_type);
        },
        Qt::QueuedConnection);
}

void c_media_data_cb(const char *session_id, const uint8_t *data, size_t len, void *user_data)
{
    auto *provider = static_cast<LinksyLinkProvider *>(user_data);
    if (!provider || !data || len == 0)
        return;
    const QString qSessId = QString::fromUtf8(session_id ? session_id : "");
    const QByteArray qData(reinterpret_cast<const char *>(data), static_cast<qsizetype>(len));
    QMetaObject::invokeMethod(
        provider,
        [provider, qSessId, qData]() {
            provider->handleMediaData(qSessId, qData);
        },
        Qt::QueuedConnection);
}
} // extern "C"

} // namespace

static LinksyLinkProvider *s_linksyInstance = nullptr;

LinksyLinkProvider *LinksyLinkProvider::instance()
{
    return s_linksyInstance;
}

LinksyLinkProvider::LinksyLinkProvider(bool disabled)
    : LinkProvider()
    , m_node(nullptr)
    , m_disabled(disabled)
{
    s_linksyInstance = this;
}

LinksyLinkProvider::~LinksyLinkProvider()
{
    if (s_linksyInstance == this) {
        s_linksyInstance = nullptr;
    }
    onStop();
}

void LinksyLinkProvider::enable()
{
    m_disabled = false;
    onStart();
}

void LinksyLinkProvider::disable()
{
    m_disabled = true;
    onStop();
}

void LinksyLinkProvider::onStart()
{
    if (m_disabled || m_node) {
        return;
    }

    const QString configDir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation) + QStringLiteral("/linksy");
    QDir().mkpath(configDir);

    const QString devName = KdeConnectConfig::instance().name();
    m_node = linksy_node_new(configDir.toUtf8().constData(), devName.toUtf8().constData());
    if (!m_node) {
        qCWarning(KDECONNECT_CORE) << "Failed to initialize Linksy FFI node";
        return;
    }

    setupCallbacks();
    qCDebug(KDECONNECT_CORE) << "Linksy LinkProvider started successfully";
}

void LinksyLinkProvider::onStop()
{
    if (m_node) {
        linksy_node_free(m_node);
        m_node = nullptr;
    }

    qDeleteAll(m_links);
    m_links.clear();
}

void LinksyLinkProvider::onNetworkChange()
{
    // Linksy discovery automatically handles network interfaces
}

void LinksyLinkProvider::onLinkDestroyed(const QString &deviceId, DeviceLink *oldPtr)
{
    if (m_links.value(deviceId) == oldPtr) {
        m_links.remove(deviceId);
    }
}

void LinksyLinkProvider::setupCallbacks()
{
    if (!m_node)
        return;
    linksy_start_discovery(m_node, c_device_cb, this);
    linksy_set_pairing_callback(m_node, c_pair_code_cb, this);
    linksy_set_packet_callback(m_node, c_packet_cb, this);
    linksy_set_session_callback(m_node, c_session_cb, this);
    linksy_set_media_data_callback(m_node, c_media_data_cb, this);
}

void LinksyLinkProvider::handleDeviceDiscovered(const QString &deviceId, const QString &name, const QString &ip, uint16_t port, bool isLost)
{
    Q_UNUSED(port);
    if (deviceId.isEmpty())
        return;

    if (isLost) {
        if (auto *link = m_links.take(deviceId)) {
            link->deleteLater();
        }
        return;
    }

    if (m_links.contains(deviceId)) {
        return;
    }

    // Standard + Linksy capabilities
    const QSet<QString> capabilities = {
        QStringLiteral("kdeconnect.clipboard"),
        QStringLiteral("kdeconnect.clipboard.connect"),
        QStringLiteral("kdeconnect.ping"),
        QStringLiteral("kdeconnect.findmyphone.request"),
        QStringLiteral("kdeconnect.battery"),
        QStringLiteral("kdeconnect.notification"),
        QStringLiteral("kdeconnect.linksy.sidecar"),
        QStringLiteral("kdeconnect.linksy.webcam"),
        QStringLiteral("kdeconnect.linksy.mic"),
        QStringLiteral("kdeconnect.linksy.screenmirror"),
        QStringLiteral("kdeconnect.linksy.terminal"),
    };

    DeviceInfo info(deviceId,
                    QSslCertificate(),
                    name.isEmpty() ? deviceId.left(8) : name,
                    DeviceType::Phone,
                    NetworkPacket::s_protocolVersion,
                    capabilities,
                    capabilities);

    auto *link = new LinksyDeviceLink(info, ip, this, m_node);
    m_links.insert(deviceId, link);
    Q_EMIT connectionReceived(link);
}

void LinksyLinkProvider::handlePairingRequest(const QString &requestId, const QString &deviceId, const QString &code, bool incoming)
{
    Q_UNUSED(incoming);
    qCDebug(KDECONNECT_CORE) << "Linksy pairing request:" << requestId << "device:" << deviceId << "code:" << code;
    // Auto-accept if already trusted in KDE Connect config, or prompt
    if (KdeConnectConfig::instance().trustedDevices().contains(deviceId)) {
        linksy_respond_pairing(m_node, requestId.toUtf8().constData(), 1);
    }
}

void LinksyLinkProvider::handlePacketReceived(const QString &deviceId, const QString &type, const QString &payload)
{
    auto *link = m_links.value(deviceId);
    if (!link)
        return;

    const QJsonDocument doc = QJsonDocument::fromJson(payload.toUtf8());
    const QJsonObject obj = doc.isObject() ? doc.object() : QJsonObject();

    if (type == QLatin1String("clipboard")) {
        NetworkPacket np(QStringLiteral("kdeconnect.clipboard"));
        np.set(QStringLiteral("content"), obj.value(QStringLiteral("text")).toString());
        link->injectPacket(np);
    } else if (type == QLatin1String("battery")) {
        NetworkPacket np(QStringLiteral("kdeconnect.battery"));
        np.set(QStringLiteral("currentCharge"), obj.value(QStringLiteral("level")).toInt());
        np.set(QStringLiteral("isCharging"), obj.value(QStringLiteral("charging")).toBool());
        link->injectPacket(np);
    } else if (type == QLatin1String("notification")) {
        NetworkPacket np(QStringLiteral("kdeconnect.notification"));
        np.set(QStringLiteral("appName"), QStringLiteral("Linksy"));
        np.set(QStringLiteral("title"), obj.value(QStringLiteral("title")).toString());
        np.set(QStringLiteral("text"), obj.value(QStringLiteral("body")).toString());
        link->injectPacket(np);
    } else if (type == QLatin1String("ping")) {
        NetworkPacket np(QStringLiteral("kdeconnect.ping"));
        link->injectPacket(np);
    }
}

void LinksyLinkProvider::handleSessionEvent(const QString &sessionId, const QString &deviceId, int kind, int eventType)
{
    if (eventType == LINKSY_SESSION_EVENT_REQUEST) {
        Q_EMIT sessionRequested(sessionId, deviceId, kind);
    } else if (eventType == LINKSY_SESSION_EVENT_STARTED) {
        Q_EMIT sessionStarted(sessionId, deviceId, kind);
    } else if (eventType == LINKSY_SESSION_EVENT_ENDED) {
        Q_EMIT sessionEnded(sessionId);
    }
}

void LinksyLinkProvider::handleMediaData(const QString &sessionId, const QByteArray &data)
{
    Q_EMIT mediaDataReceived(sessionId, data);
}

int LinksyLinkProvider::requestSession(const QString &deviceId, int sessionKind, QString &outSessionId)
{
    if (!m_node)
        return -1;
    char sessIdBuf[128] = {0};
    int ret = linksy_request_session(m_node, deviceId.toUtf8().constData(), sessionKind, sessIdBuf, sizeof(sessIdBuf));
    if (ret == 0) {
        outSessionId = QString::fromUtf8(sessIdBuf);
    }
    return ret;
}

int LinksyLinkProvider::respondSession(const QString &requestId, bool accept, uint32_t width, uint32_t height)
{
    if (!m_node)
        return -1;
    return linksy_respond_session(m_node, requestId.toUtf8().constData(), accept ? 1 : 0, width, height);
}

int LinksyLinkProvider::feedH264(const QString &sessionId, const QByteArray &data)
{
    if (!m_node)
        return -1;
    return linksy_feed_h264(m_node, sessionId.toUtf8().constData(), reinterpret_cast<const uint8_t *>(data.constData()), data.size());
}

int LinksyLinkProvider::feedPcm(const QString &sessionId, const QByteArray &data)
{
    if (!m_node)
        return -1;
    return linksy_feed_pcm(m_node, sessionId.toUtf8().constData(), reinterpret_cast<const uint8_t *>(data.constData()), data.size());
}

bool LinksyLinkProvider::isOverUsb(const QString &deviceId) const
{
    if (!m_node || deviceId.isEmpty())
        return false;
    return linksy_is_connected_over_usb(m_node, deviceId.toUtf8().constData()) == 1;
}
