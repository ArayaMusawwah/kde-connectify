/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#ifndef LINKSYLINKPROVIDER_H
#define LINKSYLINKPROVIDER_H

#include <QMap>
#include <QObject>
#include <QString>

#include "backends/linkprovider.h"
#include "kdeconnectcore_export.h"
#include "linksydevicelink.h"

struct LinksyNodeHandle;

class KDECONNECTCORE_EXPORT LinksyLinkProvider : public LinkProvider
{
    Q_OBJECT

public:
    explicit LinksyLinkProvider(bool disabled = false);
    ~LinksyLinkProvider() override;

    static LinksyLinkProvider *instance();

    QString id() const override
    {
        return QStringLiteral("linksy");
    }

    QString displayName() const override
    {
        return i18nc("@info", "Linksy (QUIC)");
    }

    int priority() override
    {
        return 30;
    }

    void enable() override;
    void disable() override;

    LinksyNodeHandle *nodeHandle() const
    {
        return m_node;
    }

    // Media & Hardware Plugin API
    int requestSession(const QString &deviceId, int sessionKind, QString &outSessionId);
    int respondSession(const QString &requestId, bool accept, uint32_t width = 0, uint32_t height = 0);
    int feedH264(const QString &sessionId, const QByteArray &data);
    int feedPcm(const QString &sessionId, const QByteArray &data);
    bool isOverUsb(const QString &deviceId) const;

public Q_SLOTS:
    void onStart() override;
    void onStop() override;
    void onNetworkChange() override;
    void onLinkDestroyed(const QString &deviceId, DeviceLink *oldPtr) override;

    void handleDeviceDiscovered(const QString &deviceId, const QString &name, const QString &ip, uint16_t port, bool isLost);
    void handlePairingRequest(const QString &requestId, const QString &deviceId, const QString &code, bool incoming);
    void handlePacketReceived(const QString &deviceId, const QString &type, const QString &payload);
    void handleSessionEvent(const QString &sessionId, const QString &deviceId, int kind, int eventType);
    void handleMediaData(const QString &sessionId, const QByteArray &data);

Q_SIGNALS:
    void sessionRequested(const QString &requestId, const QString &deviceId, int kind);
    void sessionStarted(const QString &sessionId, const QString &deviceId, int kind);
    void sessionEnded(const QString &sessionId);
    void mediaDataReceived(const QString &sessionId, const QByteArray &data);

private:
    void setupCallbacks();

    LinksyNodeHandle *m_node;
    bool m_disabled;
    QMap<QString, LinksyDeviceLink *> m_links;
};

#endif // LINKSYLINKPROVIDER_H
