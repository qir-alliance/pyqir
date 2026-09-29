# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

cmake_minimum_required(VERSION 3.10.0)

# CMake 4.4 gives command-line entries descriptive HELPSTRINGs. Forward
# supported build inputs by name instead, before project() initializes defaults.
# Discard the old snapshot so reconfiguration uses the current cache values.
unset(QIRLIB_CMAKE_CL_ARGS CACHE)
set(_qirlib_forward_variables
  BUILD_SHARED_LIBS
  CMAKE_BUILD_TYPE
  CMAKE_INSTALL_PREFIX
  CMAKE_STAGING_PREFIX
  CMAKE_TOOLCHAIN_FILE
  CMAKE_SYSTEM_NAME
  CMAKE_SYSTEM_PROCESSOR
  CMAKE_SYSTEM_VERSION
  CMAKE_OSX_ARCHITECTURES
  CMAKE_OSX_SYSROOT
  CMAKE_OSX_DEPLOYMENT_TARGET
  CMAKE_SYSROOT
  CMAKE_SYSROOT_COMPILE
  CMAKE_SYSROOT_LINK
  CMAKE_PREFIX_PATH
  CMAKE_FIND_ROOT_PATH
  CMAKE_FIND_ROOT_PATH_MODE_PROGRAM
  CMAKE_FIND_ROOT_PATH_MODE_LIBRARY
  CMAKE_FIND_ROOT_PATH_MODE_INCLUDE
  CMAKE_FIND_ROOT_PATH_MODE_PACKAGE
  CMAKE_MSVC_RUNTIME_LIBRARY
  CMAKE_POSITION_INDEPENDENT_CODE
  CMAKE_VERBOSE_MAKEFILE
  CMAKE_MAKE_PROGRAM
  CMAKE_AR
  CMAKE_RANLIB
  CMAKE_LINKER
  CMAKE_NM
  CMAKE_STRIP
  CMAKE_OBJCOPY
  CMAKE_OBJDUMP
)
set(_qirlib_cache_content "")
get_cmake_property(_qirlib_cache_variables CACHE_VARIABLES)
foreach(_qirlib_var IN LISTS _qirlib_cache_variables)
  if(NOT _qirlib_var IN_LIST _qirlib_forward_variables
      AND NOT _qirlib_var MATCHES "^(LLVM_|CPACK_)"
      AND NOT _qirlib_var MATCHES "^CMAKE_(C|CXX|ASM|RC)_(COMPILER(_ARG1|_TARGET|_EXTERNAL_TOOLCHAIN|_LAUNCHER|_AR|_RANLIB)?|FLAGS(_[A-Za-z0-9_]+)?)$"
      AND NOT _qirlib_var MATCHES "^CMAKE_(EXE|SHARED|MODULE|STATIC)_LINKER_FLAGS(_[A-Za-z0-9_]+)?$")
    continue()
  endif()

  get_property(_qirlib_type CACHE "${_qirlib_var}" PROPERTY TYPE)
  if(_qirlib_type STREQUAL "INTERNAL" OR _qirlib_type STREQUAL "STATIC")
    continue()
  endif()
  get_property(_qirlib_value CACHE "${_qirlib_var}" PROPERTY VALUE)
  string(REPLACE "\\" "\\\\" _qirlib_value "${_qirlib_value}")
  string(REPLACE "\"" "\\\"" _qirlib_value "${_qirlib_value}")
  string(REPLACE "$" "\\$" _qirlib_value "${_qirlib_value}")
  # A preload script preserves lists and quoting without encoding separators.
  # Restore TYPE separately because set(CACHE) does not accept UNINITIALIZED.
  string(APPEND _qirlib_cache_content
    "set(${_qirlib_var} \"${_qirlib_value}\" CACHE STRING \"Forwarded from qirlib\" FORCE)\n"
    "set_property(CACHE ${_qirlib_var} PROPERTY TYPE ${_qirlib_type})\n")
endforeach()

set(QIRLIB_LLVM_CACHE_FILE "${CMAKE_CURRENT_BINARY_DIR}/qirlib-llvm-cache.cmake")
set(_qirlib_previous_cache_content "")
if(EXISTS "${QIRLIB_LLVM_CACHE_FILE}")
  file(READ "${QIRLIB_LLVM_CACHE_FILE}" _qirlib_previous_cache_content)
endif()
if(NOT EXISTS "${QIRLIB_LLVM_CACHE_FILE}"
    OR NOT _qirlib_cache_content STREQUAL _qirlib_previous_cache_content)
  file(WRITE "${QIRLIB_LLVM_CACHE_FILE}" "${_qirlib_cache_content}")
endif()

project(qirlib_external)
include(ExternalProject)

# Environment variables
# Download vars
# - QIRLIB_DOWNLOAD_LLVM => If set, download pre-built LLVM binaries.
# - QIRLIB_LLVM_BUILDS_URL => URL root where downloads are hosted.
# - QIRLIB_LLVM_PKG_NAME => name of package to be downloaded/created
# - CPACK_PACKAGE_FILE_NAME
# Build vars
# - QIRLIB_LLVM_TAG
# Both
# - CMAKE_INSTALL_PREFIX

if ($ENV{QIRLIB_DOWNLOAD_LLVM} MATCHES "true")
  message (STATUS "Install LLVM to ${CMAKE_INSTALL_PREFIX}")

  set(SHA256_NAME $ENV{QIRLIB_LLVM_PKG_NAME}.sha256)
  message (STATUS "Download file: $ENV{QIRLIB_LLVM_BUILDS_URL}/$ENV{QIRLIB_LLVM_PKG_NAME}")
  message (STATUS "Download sha:$ENV{QIRLIB_LLVM_BUILDS_URL}/${SHA256_NAME}")

  # download the expected archive hash
  file(DOWNLOAD "$ENV{QIRLIB_LLVM_BUILDS_URL}/${SHA256_NAME}" ${CMAKE_BINARY_DIR}/${SHA256_NAME} STATUS SHA256_DOWNLOAD_STATUS)
  list(GET SHA256_DOWNLOAD_STATUS 0 SHA256_DOWNLOAD_STATUS)
  if (NOT (${SHA256_DOWNLOAD_STATUS} EQUAL 0))
    list(GET SHA256_DOWNLOAD_STATUS 1 ERROR_MESSAGE)
    message(FATAL_ERROR "Failed to download sha256 hash: ${ERROR_MESSAGE}")
  endif ()

  # load the expected archive hash into a variable for cmake to check
  file(STRINGS ${CMAKE_BINARY_DIR}/${SHA256_NAME} LLVM_SHA256_SUM REGEX [0-9a-fA-F]+)
  string(STRIP ${LLVM_SHA256_SUM} LLVM_SHA256_SUM)

  ExternalProject_Add(llvm
    URL $ENV{QIRLIB_LLVM_BUILDS_URL}/$ENV{QIRLIB_LLVM_PKG_NAME}
    URL_HASH SHA256=${LLVM_SHA256_SUM}
    CONFIGURE_COMMAND ""
    SOURCE_DIR ${CMAKE_INSTALL_PREFIX}
    BUILD_COMMAND ""
    INSTALL_COMMAND ""
    TEST_COMMAND ""
    USES_TERMINAL_DOWNLOAD  TRUE
  )
else()

  ExternalProject_Add(llvm
    GIT_REPOSITORY    https://github.com/llvm/llvm-project.git
    GIT_TAG           $ENV{QIRLIB_LLVM_TAG}
    GIT_SHALLOW       TRUE
    GIT_PROGRESS      TRUE
    SOURCE_SUBDIR     llvm
    CMAKE_ARGS        -C "${CMAKE_CURRENT_LIST_DIR}/config.cmake"
                      -C "${QIRLIB_LLVM_CACHE_FILE}"
    BUILD_COMMAND     ninja
    USES_TERMINAL_DOWNLOAD  TRUE
    USES_TERMINAL_CONFIGURE TRUE
    USES_TERMINAL_BUILD     TRUE
    USES_TERMINAL_INSTALL   TRUE
  )

  ExternalProject_Add_StepDependencies(
    llvm
    configure
    "${CMAKE_CURRENT_LIST_DIR}/config.cmake"
    "${QIRLIB_LLVM_CACHE_FILE}"
  )

  ExternalProject_Add_Step(llvm package
    COMMAND           ninja package
    WORKING_DIRECTORY <BINARY_DIR>
    COMMENT           "package llvm"
    DEPENDEES         build configure
    ALWAYS            TRUE
    EXCLUDE_FROM_MAIN TRUE
    USES_TERMINAL     TRUE
  )
  ExternalProject_Add_StepTargets(llvm package)
endif()