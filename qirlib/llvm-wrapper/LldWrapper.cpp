// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#include "lld/Common/Driver.h"
#include "llvm/Support/CrashRecoveryContext.h"

#ifdef _WIN32
#define QIR_SHARED_API __declspec(dllexport)
#else
#define QIR_SHARED_API
#endif

using namespace lld;
using namespace llvm;
using namespace llvm::sys;
using namespace llvm::wasm;

/// wasm linker main()
static int lldMain(int argc, const char **argv, llvm::raw_ostream &stdoutOS,
  llvm::raw_ostream &stderrOS, bool exitEarly = true) {
std::vector<const char *> args(argv, argv + argc);
// Run the driver. If an error occurs, false will be returned.
bool r = lld::wasm::link(args, stdoutOS, stderrOS, exitEarly, /*inTestOutputDisabled=*/false);

// Call exit() if we can to avoid calling destructors.
// when we are in a crc it just exits the crc, no the process
if (exitEarly)
exitLld(!r ? 1 : 0);

// Delete the global context and clear the global context pointer, so that it
// cannot be accessed anymore.
CommonLinkerContext::destroy();

return !r ? 1 : 0;
}

// Similar to lldMain except that exceptions are caught.
SafeReturn safe_lldMain(int argc, const char **argv,
  llvm::raw_ostream &stdoutOS,
  llvm::raw_ostream &stderrOS) {
int r = 0;
{
// The crash recovery is here only to be able to recover from arbitrary
// control flow when fatal() is called (through setjmp/longjmp or
// __try/__except).
llvm::CrashRecoveryContext crc;
if (!crc.RunSafely([&]() {
r = lldMain(argc, argv, stdoutOS, stderrOS, /*exitEarly=*/false);
}))
return {crc.RetCode, /*canRunAgain=*/false};
}

// Cleanup memory and reset everything back in pristine condition. This path
// is only taken when LLD is in test, or when it is used as a library.
llvm::CrashRecoveryContext crc;
if (!crc.RunSafely([&]() { CommonLinkerContext::destroy(); })) {
// The memory is corrupted beyond any possible recovery.
return {r, /*canRunAgain=*/false};
}
return {r, /*canRunAgain=*/true};
}

extern "C"
{
  QIR_SHARED_API SafeReturn safeLldMainWrapper(int argc, const char **argv,
    char **stdoutBuffer, char **stderrBuffer)
  {
    auto stdback = std::string();
    auto stderrback = std::string();
    llvm::raw_string_ostream stdoutOS(stdback);
    llvm::raw_string_ostream stderrOS(stderrback);
  
    auto res = safe_lldMain(argc, argv, stdoutOS, stderrOS);

    *stdoutBuffer = strdup(stdback.c_str());
    *stderrBuffer = strdup(stderrback.c_str());
    return res;
  }

} // extern "C"
