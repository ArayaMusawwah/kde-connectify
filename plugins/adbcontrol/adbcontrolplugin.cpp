/**
 * SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
 *
 * SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only
 */

#include "adbcontrolplugin.h"

#include <KLocalizedString>
#include <KPluginFactory>

#include <QDebug>
#include <QProcess>
#include <QStandardPaths>

#include "plugin_adbcontrol_debug.h"
#include <core/device.h>

K_PLUGIN_CLASS_WITH_JSON(AdbControlPlugin, "kdeconnect_adbcontrol.json")

QString AdbControlPlugin::dbusPath() const
{
    return QStringLiteral("/modules/kdeconnect/devices/") + device()->id() + QStringLiteral("/adbcontrol");
}

QString AdbControlPlugin::adb() const
{
    // Prefer the user's adb if it's on PATH; fall back to the Android SDK's.
    if (QStandardPaths::findExecutable(QStringLiteral("adb")).isEmpty()) {
        const QByteArray home = qgetenv("ANDROID_HOME");
        return QStringLiteral("%1/adb").arg(home.isEmpty() ? QStringLiteral("/home/root_iqbal/Android/Sdk/platform-tools")
                                                           : QString::fromUtf8(home) + QStringLiteral("/platform-tools"));
    }
    return QStringLiteral("adb");
}

void AdbControlPlugin::runAdb(const QStringList &args, bool wait)
{
    QStringList full = {adb()};
    if (!m_serial.isEmpty()) {
        full << QStringLiteral("-s") << m_serial;
    }
    full << args;

    QProcess *proc = new QProcess(this);
    proc->start(full.first(), full.mid(1));
    if (wait) {
        proc->waitForFinished(5000);
        Q_EMIT outputLine(QString::fromUtf8(proc->readAllStandardOutput()).trimmed());
    } else {
        connect(proc, &QProcess::finished, this, [this, proc](int) {
            Q_EMIT outputLine(QString::fromUtf8(proc->readAllStandardOutput()).trimmed());
        });
    }
    qCDebug(KDECONNECT_PLUGIN_ADBCONTROL) << "adb" << args;
}

bool AdbControlPlugin::connectDevice(const QString &address)
{
    if (address.isEmpty()) {
        // Use the single attached device.
        m_serial.clear();
        return true;
    }
    // `adb connect ip:port` (TCP/UDP wireless) or serial as-is.
    QProcess p;
    p.start(adb(), {QStringLiteral("connect"), address});
    p.waitForFinished(5000);
    qCDebug(KDECONNECT_PLUGIN_ADBCONTROL) << p.readAllStandardOutput();
    // Find the serial that matched the connection.
    QProcess l;
    l.start(adb(), {QStringLiteral("devices")});
    l.waitForFinished(5000);
    const QString out = QString::fromUtf8(l.readAllStandardOutput());
    for (const QString &line : out.split(QLatin1Char('\n'))) {
        if (line.contains(address.split(QLatin1Char(':')).first())) {
            const int tab = line.indexOf(QLatin1Char('\t'));
            if (tab > 0) {
                m_serial = line.left(tab);
                return true;
            }
        }
    }
    m_serial = address; // serial itself
    return true;
}

void AdbControlPlugin::disconnectDevice()
{
    m_serial.clear();
}

void AdbControlPlugin::tap(const QPoint &p, int holdMs)
{
    // `input tap x y` is a single tap; hold a pointer by swiping in place.
    if (holdMs > 0) {
        runAdb({QStringLiteral("shell"), QStringLiteral("input swipe %1 %2 %1 %2 %3").arg(p.x()).arg(p.y()).arg(holdMs)});
    } else {
        runAdb({QStringLiteral("shell"), QStringLiteral("input tap %1 %2").arg(p.x()).arg(p.y())});
    }
}

void AdbControlPlugin::swipe(const QPoint &from, const QPoint &to, int ms)
{
    runAdb({QStringLiteral("shell"), QStringLiteral("input swipe %1 %2 %3 %4 %5").arg(from.x()).arg(from.y()).arg(to.x()).arg(to.y()).arg(ms)});
}

void AdbControlPlugin::keyEvent(int keycode)
{
    runAdb({QStringLiteral("shell"), QStringLiteral("input keyevent %1").arg(keycode)});
}

void AdbControlPlugin::typeText(const QString &text)
{
    // Shell-escape: single quotes around, inner ' become '\''
    QString esc = text;
    esc.replace(QLatin1Char('\''), QStringLiteral("'\\''"));
    runAdb({QStringLiteral("shell"), QStringLiteral("input text '%1'").arg(esc)});
}

bool AdbControlPlugin::startScrcpy(int maxSize, int bitrate, int fps)
{
    QString scrcpy = QStandardPaths::findExecutable(QStringLiteral("scrcpy"));
    if (scrcpy.isEmpty()) {
        qWarning() << "scrcpy binary not found; cannot start mirroring";
        return false;
    }
    QStringList args = {QStringLiteral("--window-title=KDE-Connectify-View"),
                        QStringLiteral("--max-size"),
                        QString::number(maxSize),
                        QStringLiteral("--bit-rate"),
                        QString::number(bitrate),
                        QStringLiteral("--video-fps"),
                        QString::number(fps)};
    if (!m_serial.isEmpty()) {
        args.prepend(QStringLiteral("--serial"));
        args.prepend(m_serial);
    }
    QProcess *proc = new QProcess(this);
    proc->start(scrcpy, args);
    qCDebug(KDECONNECT_PLUGIN_ADBCONTROL) << "started" << scrcpy << args;
    return proc->waitForStarted(3000);
}

void AdbControlPlugin::stopScrcpy()
{
    // Kill any scrcpy we started.
    QProcess::startDetached(QStringLiteral("pkill"), {QStringLiteral("-f"), QStringLiteral("scrcpy.*KDE-Connectify-View")});
}

#include "adbcontrolplugin.moc"
#include "moc_adbcontrolplugin.cpp"
