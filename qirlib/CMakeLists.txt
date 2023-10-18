# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

cmake_minimum_required(VERSION 3.10.0)

# Copy the command line arguments to a cache variable so that they can be used
# with the inner project. This is needed because the inner project is a
# needs to use the same configuration and the command line arguments are not
# passed to the inner project.
# This also manifests itself in apple silicon systems where the inner project
# defaults to x86_64 even if the outer project is arm64.
# This needs to be done before the project command is called.
if (NOT QIRLIB_CMAKE_CL_ARGS)
  # get all cache variables
  get_cmake_property(vars CACHE_VARIABLES)
  foreach(var ${vars})
    get_property(currentHelpString CACHE "${var}" PROPERTY HELPSTRING)
    # only add the cache variable to the list if it is set on the command line
    if("${currentHelpString}" MATCHES "No help, variable specified on the command line." OR "${currentHelpString}" STREQUAL "")
      message ("${var} = [${${var}}]  --  ${currentHelpString}")
      list(APPEND QIRLIB_CMAKE_CL_ARGS "-D${var}=${${var}}")
    endif()
  endforeach()
  # cache the command line arguments
  set(QIRLIB_CMAKE_CL_ARGS ${QIRLIB_CMAKE_CL_ARGS} CACHE STRING "comment")
endif ()

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
    CMAKE_ARGS        -C ${CMAKE_CURRENT_LIST_DIR}/config.cmake ${QIRLIB_CMAKE_CL_ARGS}
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