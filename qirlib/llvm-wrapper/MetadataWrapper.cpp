// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#include "LLVMWrapper.h"
#include "llvm/IR/Constants.h"
#include "llvm/IR/Metadata.h"

#ifdef _WIN32
#define QIR_SHARED_API __declspec(dllexport)
#else
#define QIR_SHARED_API
#endif

using namespace llvm;

extern "C"
{
    QIR_SHARED_API LLVMMetadataRef LLVMValueAsMetadata(LLVMValueRef Val)
    {
        auto *V = unwrap(Val);
        if (auto *C = dyn_cast<Constant>(V))
            return wrap(ConstantAsMetadata::get(C));
        if (auto *MAV = dyn_cast<MetadataAsValue>(V))
            return wrap(MAV->getMetadata());
        return wrap(ValueAsMetadata::get(V));
    }

    QIR_SHARED_API LLVMValueRef LLVMRustIsAMDConstant(LLVMValueRef Val)
    {
        if (auto *MD = dyn_cast_or_null<MetadataAsValue>(unwrap(Val)))
            if (isa<ConstantAsMetadata>(MD->getMetadata()))
                return Val;
        return nullptr;
    }

    QIR_SHARED_API LLVMValueRef LLVMRustExtractMDConstant(LLVMValueRef Val)
    {
        if (auto *MD = dyn_cast_or_null<MetadataAsValue>(unwrap(Val)))
            if (isa<ConstantAsMetadata>(MD->getMetadata()))
                if (auto *CMD = dyn_cast_or_null<ConstantAsMetadata>(MD->getMetadata()))
                    return wrap(CMD->getValue());
        return nullptr;
    }

} // extern "C"
