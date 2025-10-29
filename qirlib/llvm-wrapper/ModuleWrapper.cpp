// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#include "llvm-c/Core.h"
#include "llvm/Support/CBindingWrapping.h"
#include "llvm/IR/Module.h"

#ifdef _WIN32
#define QIR_SHARED_API __declspec(dllexport)
#else
#define QIR_SHARED_API
#endif

using namespace llvm;

extern "C"
{
  enum LLVMRustModFlagBehavior
  {
    Error = 1,
    Warning = 2,
    Require = 3,
    Override = 4,
    Append = 5,
    AppendUnique = 6,
    Max = 7,
    Min = 8,

    // Markers:
    ModFlagBehaviorFirstVal = LLVMRustModFlagBehavior::Error,
    ModFlagBehaviorLastVal = Min
  };

  static llvm::Module::ModFlagBehavior
  map_to_llvmRustModFlagBehavior(LLVMRustModFlagBehavior Behavior)
  {
    switch (Behavior)
    {
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
    case LLVMRustModFlagBehavior::Min:
      return llvm::Module::ModFlagBehavior::Min;
    }
    llvm_unreachable("Unknown LLVMRustModFlagBehavior");
  }

  QIR_SHARED_API void LLVMRustAddModuleFlag(LLVMModuleRef M, LLVMRustModFlagBehavior Behavior,
                                            const char *Key, size_t KeyLen,
                                            LLVMMetadataRef Val)
  {

    llvm::unwrap(M)->addModuleFlag(map_to_llvmRustModFlagBehavior(Behavior), {Key, KeyLen}, llvm::unwrap(Val));
  }

} // extern "C"
