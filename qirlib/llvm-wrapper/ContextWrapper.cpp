// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#include "LLVMWrapper.h"

#include "llvm/IR/LLVMContext.h"

#ifdef _WIN32
#define QIR_SHARED_API __declspec(dllexport)
#else
#define QIR_SHARED_API
#endif

#ifdef __cplusplus
extern "C"
{
#endif

QIR_SHARED_API LLVMContextRef LLVMRustContextCreate(LLVMBool OpaquePointers) {
  auto C = new llvm::LLVMContext();
  // In LLVM 15+ we uncomment this next line
  //C->setOpaquePointers(OpaquePointers);
  return llvm::wrap(C);
}

#ifdef __cplusplus
} // extern "C"
#endif
