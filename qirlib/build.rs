// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::boxed::Box;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, io};

extern crate cc;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate semver;

lazy_static! {
    /// Filesystem path to an llvm-config binary for the correct version.
    static ref LLVM_CONFIG_PATH: Option<PathBuf> = locate_llvm_config();
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=config.cmake");
    println!("cargo:rerun-if-changed=CMakeLists.txt");
    println!("cargo:rerun-if-changed=target.c");
    println!("cargo:rerun-if-env-changed=PYQIR_DOWNLOAD_LLVM");
    println!("cargo:rerun-if-env-changed=PYQIR_LLVM_BUILDS_URL");
    println!("cargo:rerun-if-env-changed=PYQIR_LLVM_PKG_NAME");
    println!("cargo:rerun-if-env-changed=PYQIR_PKG_DEST");
    println!("cargo:rerun-if-env-changed=PYQIR_CACHE_DIR");

    if cfg!(feature = "download-llvm") || cfg!(feature = "install-llvm") {
        println!("Downloading llvm");
        download_llvm()?;
    } else if cfg!(feature = "build-llvm") || cfg!(feature = "package-llvm") {
        println!("Building llvm");
        compile_llvm()?;
    }
    if cfg!(feature = "download-llvm") || cfg!(feature = "build-llvm") {
        println!("Linking llvm");
        link_llvm();
        let build_dir = get_build_dir()?;
        compile_target_wrappers(&build_dir);
    } else {
        println!("LLVM_SYS_{{}}_PREFIX will provide the LLVM linkage");
    }

    Ok(())
}

fn download_llvm() -> Result<(), Box<dyn Error>> {
    let url = env::var("PYQIR_LLVM_BUILDS_URL")
        .unwrap_or("https://msquantumpublic.blob.core.windows.net/llvm-builds".to_owned());

    let enable_download = env::var("PYQIR_DOWNLOAD_LLVM").unwrap_or("true".to_owned());

    let build_dir = get_build_dir()?;

    let mut config = cmake::Config::new(&build_dir);
    config
        .generator("Ninja")
        .no_build_target(true)
        .env("PYQIR_LLVM_PKG_NAME", get_package_file_name()?)
        .env("PYQIR_LLVM_BUILDS_URL", url)
        .env("PYQIR_DOWNLOAD_LLVM", enable_download)
        .define("CPACK_PACKAGE_FILE_NAME", get_package_name()?)
        .define("CMAKE_INSTALL_PREFIX", get_llvm_install_dir())
        .very_verbose(true);
    let _ = config.build();

    Ok(())
}

fn get_llvm_compile_target() -> String {
    if cfg!(feature = "package-llvm") {
        "llvm-prefix/src/llvm-stamp/llvm-package".to_owned()
    } else {
        "llvm-prefix/src/llvm-stamp/llvm-install".to_owned()
    }
}

fn compile_llvm() -> Result<(), Box<dyn Error>> {
    let build_dir = get_build_dir()?;
    let mut config = cmake::Config::new(&build_dir);

    config
        .generator("Ninja")
        .build_target(get_llvm_compile_target().as_str())
        .define("CPACK_PACKAGE_FILE_NAME", get_package_name()?)
        .define("CMAKE_INSTALL_PREFIX", get_llvm_install_dir());
    let _ = config.build();

    if cfg!(feature = "package-llvm") {
        package_llvm()?;
    }
    Ok(())
}

fn package_llvm() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR").expect("Could not get OUT_DIR environment variable");
    let output = PathBuf::from(out_dir)
        .join("build")
        .join("llvm-prefix")
        .join("src")
        .join("llvm-build")
        .join(get_package_file_name()?);

    let dest_dir = env::var("PYQIR_PKG_DEST").expect("Could not read PYQIR_PKG_DEST");
    let dest = PathBuf::from(dest_dir).join(get_package_file_name()?);
    fs::rename(output, dest)?;

    Ok(())
}

fn get_build_dir() -> Result<PathBuf, Box<dyn Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let build_dir = PathBuf::from(manifest_dir.as_str());
    let normalized_build_dir = fs::canonicalize(&build_dir)?;
    println!(
        "llvm build files dir: {}",
        normalized_build_dir.to_str().unwrap()
    );
    Ok(normalized_build_dir)
}

fn link_llvm() {
    let libdir = llvm_config("--libdir");

    // Export information to other crates
    println!(
        "cargo:config_path={}",
        LLVM_CONFIG_PATH.clone().unwrap().display()
    ); // will be DEP_QIRLIB_CONFIG_PATH
    println!("cargo:libdir={}", libdir); // DEP_QIRLIB_LIBDIR

    // Link LLVM libraries
    println!("cargo:rustc-link-search=native={}", libdir);
    for name in get_link_libraries() {
        println!("cargo:rustc-link-lib=static={}", name);
    }

    // Link system libraries
    for name in get_system_libraries() {
        println!("cargo:rustc-link-lib=dylib={}", name);
    }
}

fn compile_target_wrappers(external_buf: &Path) {
    let target_c = external_buf.join("target.c");
    std::env::set_var("CFLAGS", get_llvm_cflags());
    cc::Build::new().file(target_c).compile("targetwrappers");
}

fn get_package_file_name() -> Result<String, Box<dyn Error>> {
    let mut base_name = get_package_name()?;

    if target_os_is("windows") {
        base_name.push_str(".zip");
    } else {
        base_name.push_str(".tar.gz");
    }

    Ok(base_name)
}

fn get_package_name() -> Result<String, Box<dyn Error>> {
    // TODO: this should be a build/env var
    // passed into the  build for cmake.
    let sha = "1fdec59bf";
    let triple = get_target_triple()?;
    // TODO: replace aq with qirlib/pyqir
    let package_name = format!("aq-llvm-{}-{}", triple, sha);
    Ok(package_name)
}

fn get_target_triple() -> Result<String, Box<dyn Error>> {
    let target = if target_os_is("windows") {
        // TODO: remove static linking and just return the TARGET
        "x86_64-pc-windows-msvc-static".to_owned()
    } else {
        env::var("TARGET")?
    };
    Ok(target)
}

fn get_llvm_install_dir() -> PathBuf {
    if let Ok(path) = env::var("PYQIR_CACHE_DIR") {
        PathBuf::from(path)
    } else {
        // if we install to OUT_DIR the llvm install task during the extraction
        // of the archive will empty the target directory breaking the build.
        // To avoid that, we put llvm binaries into the OUT_DIR/llvm folder.
        let out_dir = env::var("OUT_DIR").expect("Could not get OUT_DIR environment variable");
        PathBuf::from(out_dir).join("llvm")
    }
}

fn locate_llvm_config() -> Option<PathBuf> {
    let dir = get_llvm_install_dir();
    let prefix = dir.join("bin");
    let binary_name = llvm_config_name();
    let binary_path = prefix.join(binary_name);
    if binary_path.as_path().exists() {
        Some(binary_path)
    } else {
        None
    }
}

fn llvm_config_name() -> String {
    let mut base_name = "llvm-config".to_owned();

    // On Windows, also search for llvm-config.exe
    if target_os_is("windows") {
        base_name.push_str(".exe");
    }

    base_name
}

/// Get the output from running `llvm-config` with the given argument.
///
/// Lazily searches for or compiles LLVM as configured by the environment
/// variables.
fn llvm_config(arg: &str) -> String {
    llvm_config_ex(&*LLVM_CONFIG_PATH.clone().unwrap(), arg)
        .expect("Surprising failure from llvm-config")
}

/// Invoke the specified binary as llvm-config.
///
/// Explicit version of the `llvm_config` function that bubbles errors
/// up.
fn llvm_config_ex<S: AsRef<OsStr>>(binary: S, arg: &str) -> io::Result<String> {
    Command::new(binary)
        .arg(arg)
        .arg("--link-static") // Don't use dylib for >= 3.9
        .output()
        .map(|output| {
            String::from_utf8(output.stdout).expect("Output from llvm-config was not valid UTF-8")
        })
}

fn get_llvm_cflags() -> String {
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

fn target_env_is(name: &str) -> bool {
    match env::var_os("CARGO_CFG_TARGET_ENV") {
        Some(s) => s == name,
        None => false,
    }
}

fn target_os_is(name: &str) -> bool {
    match env::var_os("CARGO_CFG_TARGET_OS") {
        Some(s) => s == name,
        None => false,
    }
}

/// Get the names of the dylibs required by LLVM, including the C++ standard
/// library.
fn get_system_libraries() -> Vec<String> {
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
fn get_system_libcpp() -> Option<&'static str> {
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
fn get_link_libraries() -> Vec<String> {
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
