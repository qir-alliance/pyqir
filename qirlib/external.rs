/*
Copyright (c) 2015 Peter Marheine

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

// In order to make sure we perform LLVM linking the same way as llvm-sys,
// the following code is from its build.rs:
// https://github.com/tari/llvm-sys.rs/blob/master/build.rs
// Slight changes have been made to satisfy clippy and remove features
// that we don't currently support.
pub mod llvm_sys {
    use std::{
        env,
        ffi::OsStr,
        io,
        path::{Path, PathBuf},
        process::Command,
    };

    lazy_static! {
        /// Filesystem path to an llvm-config binary for the correct version.
        pub static ref LLVM_CONFIG_PATH: Option<PathBuf> = crate::locate_llvm_config();
    }

    /// Get the output from running `llvm-config` with the given argument.
    ///
    /// Lazily searches for or compiles LLVM as configured by the environment
    /// variables.
    pub fn llvm_config(arg: &str) -> String {
        llvm_config_ex(&*LLVM_CONFIG_PATH.clone().unwrap(), arg)
            .expect("Surprising failure from llvm-config")
    }

    /// Invoke the specified binary as llvm-config.
    ///
    /// Explicit version of the `llvm_config` function that bubbles errors
    /// up.
    pub fn llvm_config_ex<S: AsRef<OsStr>>(binary: S, arg: &str) -> io::Result<String> {
        Command::new(binary)
            .arg(arg)
            .arg("--link-static") // Don't use dylib for >= 3.9
            .output()
            .map(|output| {
                String::from_utf8(output.stdout)
                    .expect("Output from llvm-config was not valid UTF-8")
            })
    }

    pub fn get_llvm_cflags() -> String {
        let output = llvm_config("--cflags");
        if target_env_is("msvc") {
            // MSVC doesn't accept -W... options, so don't try to strip them and
            // possibly strip something that should be retained. Also do nothing if
            // the user requests it.
            return output;
        }

        llvm_config("--cflags")
            .split(&[' ', '\n'][..])
            .filter(|word| !word.starts_with("-W"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn get_llvm_cxxflags() -> String {
        let output = llvm_config("--cxxflags");
        if target_env_is("msvc") {
            // MSVC doesn't accept -W... options, so don't try to strip them and
            // possibly strip something that should be retained. Also do nothing if
            // the user requests it.
            return output;
        }

        llvm_config("--cxxflags")
            .split(&[' ', '\n'][..])
            .filter(|word| !word.starts_with("-W"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn target_env_is(name: &str) -> bool {
        match env::var_os("CARGO_CFG_TARGET_ENV") {
            Some(s) => s == name,
            None => false,
        }
    }

    pub fn target_os_is(name: &str) -> bool {
        match env::var_os("CARGO_CFG_TARGET_OS") {
            Some(s) => s == name,
            None => false,
        }
    }

    /// Get the names of the dylibs required by LLVM, including the C++ standard
    /// library.
    pub fn get_system_libraries() -> Vec<String> {
        llvm_config("--system-libs")
            .split(&[' ', '\n'] as &[char])
            .filter(|s| !s.is_empty())
            .map(|flag| {
                if cfg!(target_env = "msvc") {
                    // Same as --libnames, foo.lib
                    assert!(flag.ends_with(".lib"));
                    &flag[..flag.len() - 4]
                } else if cfg!(target_os = "macos") {
                    // Linker flags style, -lfoo
                    assert!(flag.starts_with("-l"));
                    if flag.ends_with(".tbd") && flag.starts_with("-llib") {
                        &flag[5..flag.len() - 4]
                    } else {
                        &flag[2..]
                    }
                } else {
                    if flag.starts_with("-l") {
                        // Linker flags style, -lfoo
                        return flag
                            .strip_prefix("-l")
                            .expect("could not strip -l prefix")
                            .to_owned();
                    }

                    let maybe_lib = Path::new(&flag);
                    if maybe_lib.is_file() {
                        // Library on disk, likely an absolute path to a .so
                        if let Some(p) = maybe_lib.parent() {
                            println!("cargo:rustc-link-search={}", p.display())
                        }
                        &maybe_lib.file_stem().unwrap().to_str().unwrap()[3..]
                    } else {
                        panic!("Unable to parse result of llvm-config --system-libs")
                    }
                }
                .to_owned()
            })
            .chain(get_system_libcpp().map(str::to_owned))
            .collect::<Vec<String>>()
    }

    /// Get the library that must be linked for C++, if any.
    pub fn get_system_libcpp() -> Option<&'static str> {
        if cfg!(target_env = "msvc") {
            // MSVC doesn't need an explicit one.
            None
        } else if cfg!(target_os = "macos") || cfg!(target_os = "freebsd") {
            // On OS X 10.9 and later, LLVM's libc++ is the default. On earlier
            // releases GCC's libstdc++ is default. Unfortunately we can't
            // reasonably detect which one we need (on older ones libc++ is
            // available and can be selected with -stdlib=lib++), so assume the
            // latest, at the cost of breaking the build on older OS releases
            // when LLVM was built against libstdc++.
            Some("c++")
        } else {
            // Otherwise assume GCC's libstdc++.
            // This assumption is probably wrong on some platforms, but would need
            // testing on them.
            Some("stdc++")
        }
    }

    /// Get the names of libraries to link against.
    pub fn get_link_libraries() -> Vec<String> {
        // Using --libnames in conjunction with --libdir is particularly important
        // for MSVC when LLVM is in a path with spaces, but it is generally less of
        // a hack than parsing linker flags output from --libs and --ldflags.
        llvm_config("--libnames")
            .split(&[' ', '\n'] as &[char])
            .filter(|s| !s.is_empty())
            .map(|name| {
                // --libnames gives library filenames. Extract only the name that
                // we need to pass to the linker.
                if cfg!(target_env = "msvc") {
                    // LLVMfoo.lib
                    assert!(name.ends_with(".lib"));
                    &name[..name.len() - 4]
                } else {
                    // libLLVMfoo.a
                    assert!(name.starts_with("lib") && name.ends_with(".a"));
                    &name[3..name.len() - 2]
                }
            })
            .map(str::to_owned)
            .collect::<Vec<String>>()
    }
}
