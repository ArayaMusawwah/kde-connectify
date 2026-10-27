/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#ifndef TERMINALPLUGIN_H
#define TERMINALPLUGIN_H

#include <QObject>
#include <QProcess>
#include <QString>
#include <core/kdeconnectplugin.h>

class TerminalPlugin : public KdeConnectPlugin
{
    Q_OBJECT
    Q_CLASSINFO("D-Bus Interface", "org.kde.kdeconnect.device.terminal")
    Q_PROPERTY(bool active READ active NOTIFY activeChanged)
    Q_PROPERTY(QString pinCode READ pinCode NOTIFY pinCodeChanged)

public:
    using KdeConnectPlugin::KdeConnectPlugin;
    ~TerminalPlugin() override;

    bool active() const
    {
        return m_active;
    }
    QString pinCode() const
    {
        return m_pinCode;
    }

    Q_SCRIPTABLE bool startTerminal(const QString &enteredPin);
    Q_SCRIPTABLE void stopTerminal();
    Q_SCRIPTABLE QString generatePin();

    void receivePacket(const NetworkPacket &np) override;
    QString dbusPath() const override;

Q_SIGNALS:
    void activeChanged(bool active);
    void pinCodeChanged(const QString &pin);
    void outputReceived(const QString &data);

private Q_SLOTS:
    void onProcessReadyRead();
    void onProcessFinished(int exitCode);

private:
    void writeInput(const QString &input);
    void resizeTerminal(int cols, int rows);

    bool m_active = false;
    QString m_pinCode;
    QProcess *m_shellProcess = nullptr;
};

#endif // TERMINALPLUGIN_H
