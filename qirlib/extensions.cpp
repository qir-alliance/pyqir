// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#include "llvm-c/Core.h"
#include "llvm/IR/Module.h"
#include "llvm/Support/CBindingWrapping.h"

#ifdef _WIN32
#define QIR_SHARED_API __declspec(dllexport)
#else
#define QIR_SHARED_API
#endif

#ifdef __cplusplus
extern "C"
{
#endif

typedef enum LLVMModFlagBehavior {
    Error = 1,
    Warning = 2,
    Require = 3,
    Override = 4,
    Append = 5,
    AppendUnique = 6,
    Max = 7,
    Min = 8,

    // Markers:
    ModFlagBehaviorFirstVal = Error,
    ModFlagBehaviorLastVal = Min
  };


static llvm::Module::ModFlagBehavior
map_to_llvmModFlagBehavior(LLVMModFlagBehavior Behavior) {
  switch (Behavior) {
  case LLVMModFlagBehavior::Error:
    return llvm::Module::ModFlagBehavior::Error;
  case LLVMModFlagBehavior::Warning:
    return llvm::Module::ModFlagBehavior::Warning;
  case LLVMModFlagBehavior::Require:
    return llvm::Module::ModFlagBehavior::Require;
  case LLVMModFlagBehavior::Override:
    return llvm::Module::ModFlagBehavior::Override;
  case LLVMModFlagBehavior::Append:
    return llvm::Module::ModFlagBehavior::Append;
  case LLVMModFlagBehavior::AppendUnique:
    return llvm::Module::ModFlagBehavior::AppendUnique;
  case LLVMModFlagBehavior::Max:
    return llvm::Module::ModFlagBehavior::Max;
  // Leaving this commented out for the moment. It requires
  // LLVM 14 and I don't have a conditional definition to hook on.
  // case LLVMModFlagBehavior::Min:
  //   return llvm::Module::ModFlagBehavior::Min;
  }
  llvm_unreachable("Unknown LLVMModFlagBehavior");
}

QIR_SHARED_API void fixed_LLVMAddModuleFlag(LLVMModuleRef M, LLVMModFlagBehavior Behavior,
                       const char *Key, size_t KeyLen,
                       LLVMMetadataRef Val) {

  llvm::unwrap(M)->addModuleFlag(map_to_llvmModFlagBehavior(Behavior), {Key, KeyLen}, llvm::unwrap(Val));
}

#ifdef __cplusplus
} // extern "C"
#endif