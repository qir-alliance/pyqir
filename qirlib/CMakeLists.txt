# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

cmake_minimum_required(VERSION 3.10.0)

project(qirlib_external)
include(ExternalProject)

# Environment variables
# PYQIR_DOWNLOAD_LLVM => If set, download pre-built LLVM binaries.
# PYQIR_LLVM_BUILDS_URL => URL root where downloads are hosted.
# PYQIR_LLVM_PKG_NAME => name of package to be downloaded/created
# PYQIR_PKG_DEST

if ($ENV{PYQIR_DOWNLOAD_LLVM})
  message (STATUS "Install LLVM to ${CMAKE_INSTALL_PREFIX}")

  set(SHA256_NAME $ENV{PYQIR_LLVM_PKG_NAME}.sha256)
  message (STATUS "Download file: $ENV{PYQIR_LLVM_BUILDS_URL}/$ENV{PYQIR_LLVM_PKG_NAME}")
  message (STATUS "Download sha:$ENV{PYQIR_LLVM_BUILDS_URL}/${SHA256_NAME}")

  # download the expected archive hash
  file(DOWNLOAD "$ENV{PYQIR_LLVM_BUILDS_URL}/${SHA256_NAME}" ${CMAKE_BINARY_DIR}/${SHA256_NAME} STATUS SHA256_DOWNLOAD_STATUS)
  list(GET SHA256_DOWNLOAD_STATUS 0 SHA256_DOWNLOAD_STATUS)
  if (NOT (${SHA256_DOWNLOAD_STATUS} EQUAL 0))
    list(GET SHA256_DOWNLOAD_STATUS 1 ERROR_MESSAGE)
    message(FATAL_ERROR "Failed to download sha256 hash: ${ERROR_MESSAGE}")
  endif ()

  # load the expected archive hash into a variable for cmake to check
  file(STRINGS ${CMAKE_BINARY_DIR}/${SHA256_NAME} LLVM_SHA256_SUM REGEX [0-9a-f]+)
  string(STRIP ${LLVM_SHA256_SUM} LLVM_SHA256_SUM)

  ExternalProject_Add(llvm
    URL $ENV{PYQIR_LLVM_BUILDS_URL}/$ENV{PYQIR_LLVM_PKG_NAME}
    URL_HASH SHA256=${LLVM_SHA256_SUM}
    CONFIGURE_COMMAND ""
    SOURCE_DIR ${CMAKE_INSTALL_PREFIX}
    BUILD_COMMAND ""
    INSTALL_COMMAND ""
    TEST_COMMAND ""
    USES_TERMINAL_DOWNLOAD  TRUE
  )
else()
  # The outer cmake call from rust will configure CMAKE_INSTALL_PREFIX
  # and CPACK_PACKAGE_FILE_NAME. These are the only variables we care about
  # when passing to the LLVM build. Unless the user overrides the install path
  # rust will configure it to be the crate OUT_DIR
  set (EXTERNAL_EXTRA_CMAKE_ARGS)
  list (APPEND EXTERNAL_EXTRA_CMAKE_ARGS 
    -DCMAKE_INSTALL_PREFIX=${CMAKE_INSTALL_PREFIX}
    -DCPACK_PACKAGE_FILE_NAME=${CPACK_PACKAGE_FILE_NAME}
  )

  ExternalProject_Add(llvm
    GIT_REPOSITORY    https://github.com/llvm/llvm-project.git
    GIT_TAG           llvmorg-11.1.0
    GIT_SHALLOW       TRUE
    GIT_PROGRESS      TRUE
    SOURCE_SUBDIR     llvm
    CONFIGURE_COMMAND ${CMAKE_COMMAND} -G Ninja -C ${CMAKE_CURRENT_LIST_DIR}/config.cmake ${EXTERNAL_EXTRA_CMAKE_ARGS} <SOURCE_DIR><SOURCE_SUBDIR>
    BUILD_COMMAND     ninja
    USES_TERMINAL_DOWNLOAD  TRUE
    USES_TERMINAL_CONFIGURE TRUE
    USES_TERMINAL_BUILD     TRUE
    USES_TERMINAL_INSTALL   TRUE
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