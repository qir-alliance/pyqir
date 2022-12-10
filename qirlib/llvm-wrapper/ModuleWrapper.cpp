// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#include "LLVMWrapper.h"

#include "llvm/IR/Module.h"

#ifdef _WIN32
#define QIR_SHARED_API __declspec(dllexport)
#else
#define QIR_SHARED_API
#endif

#ifdef __cplusplus
extern "C"
{
#endif

enum LLVMRustModFlagBehavior {
    Error = 1,
    Warning = 2,
    Require = 3,
    Override = 4,
    Append = 5,
    AppendUnique = 6,
    Max = 7,
#if LLVM_VERSION_GE(14, 0)
    Min = 8,
#endif

    // Markers:
    ModFlagBehaviorFirstVal = Error,
#if LLVM_VERSION_GE(14, 0)
    ModFlagBehaviorLastVal = Min
#else
    ModFlagBehaviorLastVal = Max
#endif
  };


static llvm::Module::ModFlagBehavior
map_to_llvmRustModFlagBehavior(LLVMRustModFlagBehavior Behavior) {
  switch (Behavior) {
  case LLVMRustModFlagBehavior::Error:
    return llvm::Module::ModFlagBehavior::Error;
  case LLVMRustModFlagBehavior::Warning:
    return llvm::Module::ModFlagBehavior::Warning;
  case LLVMRustModFlagBehavior::Require:
    return llvm::Module::ModFlagBehavior::Require;
  case LLVMRustModFlagBehavior::Override:
    return llvm::Module::ModFlagBehavior::Override;
  case LLVMRustModFlagBehavior::Append:
    return llvm::Module::ModFlagBehavior::Append;
  case LLVMRustModFlagBehavior::AppendUnique:
    return llvm::Module::ModFlagBehavior::AppendUnique;
  case LLVMRustModFlagBehavior::Max:
    return llvm::Module::ModFlagBehavior::Max;
#if LLVM_VERSION_GE(14, 0)
  case LLVMRustModFlagBehavior::Min:
    return llvm::Module::ModFlagBehavior::Min;
#endif
  }
  llvm_unreachable("Unknown LLVMRustModFlagBehavior");
}

QIR_SHARED_API void LLVMRustAddModuleFlag(LLVMModuleRef M, LLVMRustModFlagBehavior Behavior,
                       const char *Key, size_t KeyLen,
                       LLVMMetadataRef Val) {

  llvm::unwrap(M)->addModuleFlag(map_to_llvmRustModFlagBehavior(Behavior), {Key, KeyLen}, llvm::unwrap(Val));
}

#ifdef __cplusplus
} // extern "C"
#endif
