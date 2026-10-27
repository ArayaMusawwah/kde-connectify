# SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
# SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only

kdeconnect_add_plugin(kdeconnect_sidecar SOURCES sidecarplugin.cpp)
target_link_libraries(kdeconnect_sidecar
    kdeconnectcore
    Qt::Core
    Qt::DBus
    KF6::I18n
)
