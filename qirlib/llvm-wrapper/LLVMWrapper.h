// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#include "llvm-c/Core.h"
#include "llvm/Support/CBindingWrapping.h"

#define LLVM_VERSION_GE(major, minor)                                          \
  (LLVM_VERSION_MAJOR > (major) ||                                             \
   LLVM_VERSION_MAJOR == (major) && LLVM_VERSION_MINOR >= (minor))
