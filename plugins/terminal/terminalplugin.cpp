/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#include "terminalplugin.h"
#include <device.h>
#include <kdeconnect_debug.h>

#include <KPluginFactory>
#include <QDebug>
#include <QJsonDocument>
#include <QJsonObject>
#include <QRandomGenerator>

K_PLUGIN_CLASS_WITH_JSON(TerminalPlugin, "kdeconnect_terminal.json")

TerminalPlugin::~TerminalPlugin()
{
    stopTerminal();
}

QString TerminalPlugin::dbusPath() const
{
    return QStringLiteral("/modules/kdeconnect/devices/") + device()->id() + QStringLiteral("/terminal");
}

QString TerminalPlugin::generatePin()
{
    quint32 val = QRandomGenerator::global()->bounded(100000, 999999);
    m_pinCode = QString::number(val);
    Q_EMIT pinCodeChanged(m_pinCode);
    return m_pinCode;
}

bool TerminalPlugin::startTerminal(const QString &enteredPin)
{
    if (m_active) {
        return true;
    }

    if (m_pinCode.isEmpty()) {
        generatePin();
    }

    if (enteredPin != m_pinCode) {
        qWarning() << "Invalid PIN entered for terminal session";
        return false;
    }

    if (!m_shellProcess) {
        m_shellProcess = new QProcess(this);
        connect(m_shellProcess, &QProcess::readyReadStandardOutput, this, &TerminalPlugin::onProcessReadyRead);
        connect(m_shellProcess, &QProcess::readyReadStandardError, this, &TerminalPlugin::onProcessReadyRead);
        connect(m_shellProcess, QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished), this, [this](int code, QProcess::ExitStatus) {
            onProcessFinished(code);
        });

        QString shell = qEnvironmentVariable("SHELL");
        if (shell.isEmpty()) {
            shell = QStringLiteral("/bin/bash");
        }

        m_shellProcess->start(shell, QStringList() << QStringLiteral("-i"));
        if (!m_shellProcess->waitForStarted(2000)) {
            qWarning() << "Failed to start shell process:" << shell;
            delete m_shellProcess;
            m_shellProcess = nullptr;
            return false;
        }
    }

    m_active = true;
    Q_EMIT activeChanged(true);
    qDebug() << "Terminal session active on device:" << device()->id();
    return true;
}

void TerminalPlugin::stopTerminal()
{
    if (!m_active) {
        return;
    }

    if (m_shellProcess) {
        m_shellProcess->terminate();
        if (!m_shellProcess->waitForFinished(1000)) {
            m_shellProcess->kill();
        }
        delete m_shellProcess;
        m_shellProcess = nullptr;
    }

    m_active = false;
    Q_EMIT activeChanged(false);
    qDebug() << "Terminal session stopped";
}

void TerminalPlugin::writeInput(const QString &input)
{
    if (m_shellProcess && m_shellProcess->isOpen()) {
        m_shellProcess->write(input.toUtf8());
    }
}

void TerminalPlugin::resizeTerminal(int cols, int rows)
{
    Q_UNUSED(cols);
    Q_UNUSED(rows);
    // Pty window size resize via ioctl(TIOCSWINSZ) when running with PTY fd
}

void TerminalPlugin::receivePacket(const NetworkPacket &np)
{
    if (np.type() == QLatin1String("kdeconnect.linksy.terminal")) {
        const QString action = np.get<QString>(QStringLiteral("action"));
        if (action == QLatin1String("auth")) {
            const QString pin = np.get<QString>(QStringLiteral("pin"));
            startTerminal(pin);
        } else if (action == QLatin1String("input")) {
            const QString data = np.get<QString>(QStringLiteral("data"));
            writeInput(data);
        } else if (action == QLatin1String("resize")) {
            int cols = np.get<int>(QStringLiteral("cols"), 80);
            int rows = np.get<int>(QStringLiteral("rows"), 24);
            resizeTerminal(cols, rows);
        } else if (action == QLatin1String("close")) {
            stopTerminal();
        }
    }
}

void TerminalPlugin::onProcessReadyRead()
{
    if (!m_shellProcess)
        return;

    const QByteArray output = m_shellProcess->readAll();
    if (output.isEmpty())
        return;

    NetworkPacket np(QStringLiteral("kdeconnect.linksy.terminal.data"));
    np.set(QStringLiteral("data"), QString::fromUtf8(output));
    sendPacket(np);

    Q_EMIT outputReceived(QString::fromUtf8(output));
}

void TerminalPlugin::onProcessFinished(int exitCode)
{
    Q_UNUSED(exitCode);
    stopTerminal();
}

#include "terminalplugin.moc"
