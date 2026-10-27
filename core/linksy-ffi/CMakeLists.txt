# SPDX-FileCopyrightText: 2026 Super KDE Connect contributors
# SPDX-License-Identifier: GPL-2.0-only OR GPL-3.0-only

set(LINKSY_FFI_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/target/release")
set(LINKSY_FFI_LIB "${LINKSY_FFI_TARGET_DIR}/liblinksy_ffi.a")

add_custom_command(
    OUTPUT ${LINKSY_FFI_LIB}
    COMMAND cargo build --release --manifest-path "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml"
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    COMMENT "Building Linksy Rust FFI core library..."
    VERBATIM
)

add_custom_target(linksy_ffi_target DEPENDS ${LINKSY_FFI_LIB})

add_library(linksy_ffi STATIC IMPORTED GLOBAL)
add_dependencies(linksy_ffi linksy_ffi_target)
set_target_properties(linksy_ffi PROPERTIES
    IMPORTED_LOCATION "${LINKSY_FFI_LIB}"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/include"
)
