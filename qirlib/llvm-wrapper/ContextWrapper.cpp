// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#include "LLVMWrapper.h"

#include "llvm/IR/LLVMContext.h"

#ifdef _WIN32
#define QIR_SHARED_API __declspec(dllexport)
#else
#define QIR_SHARED_API
#endif

using namespace llvm;

extern "C"
{
    QIR_SHARED_API LLVMContextRef LLVMRustContextCreate(LLVMBool OpaquePointers)
    {
        auto C = new LLVMContext();
        // In LLVM 15+ we uncomment this next line
        // C->setOpaquePointers(OpaquePointers);
        return wrap(C);
    }
} // extern "C"
