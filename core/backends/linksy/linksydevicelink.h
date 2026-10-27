/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#ifndef LINKSYDEVICELINK_H
#define LINKSYDEVICELINK_H

#include <QObject>
#include <QString>

#include "backends/devicelink.h"
#include "deviceinfo.h"
#include "kdeconnectcore_export.h"
#include "networkpacket.h"

struct LinksyNodeHandle;
class LinksyLinkProvider;

class KDECONNECTCORE_EXPORT LinksyDeviceLink : public DeviceLink
{
    Q_OBJECT

public:
    LinksyDeviceLink(const DeviceInfo &deviceInfo, const QString &address, LinksyLinkProvider *parent, LinksyNodeHandle *node);
    ~LinksyDeviceLink() override;

    bool sendPacket(NetworkPacket &np) override;

    DeviceInfo deviceInfo() const override
    {
        return m_deviceInfo;
    }

    QString address() const override
    {
        return m_address;
    }

    void injectPacket(const NetworkPacket &np);

private:
    DeviceInfo m_deviceInfo;
    QString m_address;
    LinksyNodeHandle *m_node;
};

#endif // LINKSYDEVICELINK_H
