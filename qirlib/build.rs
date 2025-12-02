// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::boxed::Box;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use cc::Build;
use cmake::Config;

mod external;
use external::llvm_sys;

extern crate cc;

// Make sure one version of llvm features is used
#[cfg(all(
    not(any(feature = "llvm18-1")),
    not(any(feature = "llvm19-1")),
    not(any(feature = "llvm20-1"))
))]
compile_error!(
    "One of the features `qirlib/llvm18-1`, `qirlib/llvm19-1`, and `qirlib/llvm20-1` must be used exclusive."
);

// Make sure only one llvm option is used.
#[cfg(any(
    all(feature = "llvm18-1", feature = "llvm19-1"),
    all(feature = "llvm19-1", feature = "llvm18-1"),
    all(feature = "llvm18-1", feature = "llvm20-1"),
    all(feature = "llvm19-1", feature = "llvm20-1"),
    all(feature = "llvm20-1", feature = "llvm18-1"),
    all(feature = "llvm20-1", feature = "llvm19-1"),
))]
compile_error!(
    "Features `qirlib/llvm18-1`, `qirlib/llvm19-1`, and `qirlib/llvm20-1` must be used exclusive."
);

// Make sure one of the linking features is used
#[cfg(all(
    not(any(feature = "qirlib-llvm-linking")),
    not(any(feature = "external-llvm-linking")),
    not(any(feature = "no-llvm-linking")),
))]
compile_error!("One of the features `qirlib/qirlib-llvm-linking`, `qirlib/external-llvm-linking`, and `qirlib/no-llvm-linking` must be used exclusive.");

// Make sure only one linking option is used.
#[cfg(any(
    all(
        feature = "qirlib-llvm-linking",
        any(feature = "external-llvm-linking", feature = "no-llvm-linking")
    ),
    all(
        feature = "external-llvm-linking",
        any(feature = "qirlib-llvm-linking", feature = "no-llvm-linking")
    ),
    all(
        feature = "no-llvm-linking",
        any(feature = "qirlib-llvm-linking", feature = "external-llvm-linking")
    ),
))]
compile_error!("Features `qirlib/qirlib-llvm-linking`, `qirlib/external-llvm-linking`, and `qirlib/no-llvm-linking` are mutually exclusive.");

// if we are building or downloading, we cannot be externally linking
#[cfg(any(
    all(
        feature = "build-llvm",
        any(feature = "download-llvm", feature = "external-llvm-linking")
    ),
    all(
        feature = "download-llvm",
        any(feature = "build-llvm", feature = "external-llvm-linking")
    ),
))]
compile_error!("Features `qirlib/build-llvm` and `qirlib/download-llvm` are mutually exclusive.");

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=config.cmake");
    println!("cargo:rerun-if-changed=CMakeLists.txt");

    let install_dir = get_llvm_install_dir();
    println!("cargo:rerun-if-changed={install_dir:?}");

    // llvm-sys components
    println!("cargo:rerun-if-changed=external.rs");
    println!("cargo:rerun-if-changed=target.c");
    println!("cargo:rerun-if-changed=llvm-wrapper/MetadataWrapper.cpp");
    println!("cargo:rerun-if-changed=llvm-wrapper/ModuleWrapper.cpp");

    // Download vars passed to cmake
    println!("cargo:rerun-if-env-changed=QIRLIB_DOWNLOAD_LLVM");
    println!("cargo:rerun-if-env-changed=QIRLIB_LLVM_BUILDS_URL");
    println!("cargo:rerun-if-env-changed=QIRLIB_LLVM_PKG_NAME");

    // Package vars used only in here
    println!("cargo:rerun-if-env-changed=QIRLIB_PKG_DEST");

    // Build vars passed to cmake
    println!("cargo:rerun-if-env-changed=QIRLIB_LLVM_TAG");

    // maps to CPACK_PACKAGE_FILE_NAME
    println!("cargo:rerun-if-env-changed=QIRLIB_PACKAGE_FILE_NAME");

    // maps to CMAKE_INSTALL_PREFIX passed to cmake in build and download
    println!("cargo:rerun-if-env-changed=QIRLIB_CACHE_DIR");

    if cfg!(feature = "download-llvm") {
        println!("Downloading llvm");
        download_llvm()?;
    } else if cfg!(feature = "build-llvm") {
        println!("Building llvm");
        compile_llvm()?;
    }
    if cfg!(feature = "qirlib-llvm-linking") {
        println!("Linking llvm");
        link_llvm();
        let build_dir = get_build_dir()?;
        compile_target_wrappers(&build_dir)?;
    } else if cfg!(feature = "external-llvm-linking") {
        println!("LLVM_SYS_{{}}_PREFIX will provide the LLVM linking");
    } else {
        println!("No LLVM linking");
    }
    if !cfg!(feature = "no-llvm-linking") && !cfg!(feature = "no-module-metadata") {
        compile_llvm_wrapper()?;
    }

    Ok(())
}

fn download_llvm() -> Result<(), Box<dyn Error>> {
    // If the download url isn't set, we need to immediately fail.
    let url = env::var("QIRLIB_LLVM_BUILDS_URL")?;

    let enable_download = env::var("QIRLIB_DOWNLOAD_LLVM").unwrap_or_else(|_| "true".to_owned());

    let build_dir = get_build_dir()?;

    let mut config = Config::new(build_dir);
    config
        .generator("Ninja")
        .no_build_target(true)
        .env("QIRLIB_LLVM_PKG_NAME", get_package_file_name()?)
        .env("QIRLIB_LLVM_BUILDS_URL", url)
        .env("QIRLIB_DOWNLOAD_LLVM", enable_download)
        .define("CPACK_PACKAGE_FILE_NAME", get_package_name()?)
        .define("CMAKE_INSTALL_PREFIX", get_llvm_install_dir())
        .very_verbose(true);
    let _ = config.build();

    Ok(())
}

fn get_llvm_compile_target() -> String {
    // We always install unless package is chosen.
    // The user's choices for CMAKE_INSTALL_PREFIX will choose whether
    // the installation goes into the target folder for linking or
    // into another dir for potential reuse
    if cfg!(feature = "package-llvm") {
        "llvm-prefix/src/llvm-stamp/llvm-package".to_owned()
    } else {
        "llvm-prefix/src/llvm-stamp/llvm-install".to_owned()
    }
}

fn compile_llvm() -> Result<(), Box<dyn Error>> {
    let build_dir = get_build_dir()?;
    let mut config = Config::new(build_dir);

    if cfg!(target_os = "windows") {
        config
            .define("CMAKE_C_COMPILER", "clang-cl")
            .define("CMAKE_CXX_COMPILER", "clang-cl");
    }

    config
        .generator("Ninja")
        .build_target(get_llvm_compile_target().as_str())
        .env("QIRLIB_LLVM_TAG", get_llvm_tag())
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

    if let Ok(dest_dir) = env::var("QIRLIB_PKG_DEST") {
        let dest = PathBuf::from(dest_dir).join(get_package_file_name()?);
        println!(
            "Moving {} to {}.",
            output.as_path().display(),
            dest.as_path().display()
        );
        fs::rename(output, dest)?;
    } else {
        println!("Not moving package output. QIRLIB_PKG_DEST not set.");
    }

    Ok(())
}

fn get_build_dir() -> Result<PathBuf, Box<dyn Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let build_dir = PathBuf::from(manifest_dir.as_str());
    let normalized_build_dir = fs::canonicalize(build_dir)?;
    println!(
        "llvm build files dir: {}",
        normalized_build_dir.to_str().unwrap()
    );
    Ok(normalized_build_dir)
}

fn link_llvm() {
    let libdir = llvm_sys::llvm_config("--libdir");

    // Export information to other crates
    println!(
        "cargo:config_path={}",
        llvm_sys::llvm_config_path().clone().unwrap().display()
    ); // will be DEP_QIRLIB_CONFIG_PATH
    println!("cargo:libdir={libdir}"); // DEP_QIRLIB_LIBDIR

    // Link LLVM libraries
    println!("cargo:rustc-link-search=native={libdir}");
    for name in llvm_sys::get_link_libraries() {
        println!("cargo:rustc-link-lib=static={name}");
    }

    // Link system libraries
    for name in llvm_sys::get_system_libraries() {
        println!("cargo:rustc-link-lib=dylib={name}");
    }
}

fn compile_target_wrappers(build_dir: &Path) -> Result<(), Box<dyn Error>> {
    let target_c = build_dir.join("target.c").canonicalize()?;
    env::set_var("CFLAGS", llvm_sys::get_llvm_cflags());
    Build::new()
        .file(target_c)
        .prefer_clang_cl_over_msvc(true)
        .compile("targetwrappers");
    Ok(())
}

fn compile_llvm_wrapper() -> Result<(), Box<dyn Error>> {
    let mut cfg = cc::Build::new();
    cfg.warnings(false);
    let cxxflags = llvm_sys::get_llvm_cxxflags();
    for flag in cxxflags.split_whitespace() {
        if flag.starts_with("-flto") {
            continue;
        }
        cfg.flag(flag);
    }
    cfg.cpp(true)
        .cpp_link_stdlib(None)
        .static_crt(true)
        .file("llvm-wrapper/MetadataWrapper.cpp")
        .file("llvm-wrapper/ModuleWrapper.cpp")
        .prefer_clang_cl_over_msvc(true)
        .compile("llvm-wrapper");
    Ok(())
}

fn get_package_file_name() -> Result<String, Box<dyn Error>> {
    let mut base_name = get_package_name()?;

    if llvm_sys::target_os_is("windows") {
        base_name.push_str(".zip");
    } else {
        base_name.push_str(".tar.gz");
    }

    Ok(base_name)
}

fn get_llvm_tag() -> String {
    if let Ok(tag) = env::var("QIRLIB_LLVM_TAG") {
        tag
    } else if cfg!(feature = "llvm18-1") {
        "llvmorg-18.1.2".to_owned() // 26a1d66
    } else if cfg!(feature = "llvm19-1") {
        "llvmorg-19.1.0".to_owned() // a4bf6cd
    } else if cfg!(feature = "llvm20-1") {
        "llvmorg-20.1.0".to_owned() // 24a30da
    } else {
        panic!("Unsupported LLVM version. The LLVM feature flags or QIRLIB_LLVM_TAG must be set.")
    }
}

fn get_package_name() -> Result<String, Box<dyn Error>> {
    if let Ok(file_name) = env::var("QIRLIB_PACKAGE_FILE_NAME") {
        Ok(file_name)
    } else {
        let tag = get_llvm_tag();
        let triple = get_target_triple()?;
        let package_name = format!("qirlib-llvm-{triple}-{tag}");
        Ok(package_name)
    }
}

fn get_target_triple() -> Result<String, Box<dyn Error>> {
    let target = if llvm_sys::target_os_is("windows") {
        // TODO: remove static linking and just return the TARGET
        "x86_64-pc-windows-msvc-static".to_owned()
    } else {
        env::var("TARGET")?
    };
    Ok(target)
}

fn get_llvm_install_dir() -> PathBuf {
    if let Ok(path) = env::var("QIRLIB_CACHE_DIR") {
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
    let major = if cfg!(feature = "llvm18-1") {
        "18"
    } else if cfg!(feature = "llvm19-1") {
        "19"
    } else if cfg!(feature = "llvm20-1") {
        "20"
    } else {
        "unknown"
    };
    if let Ok(path) = env::var(format!("DEP_LLVM_{major}_CONFIG_PATH")) {
        Some(PathBuf::from(path))
    } else {
        let dir = get_llvm_install_dir();
        println!("Looking in {dir:?}");
        let prefix = dir.join("bin");
        let binary_name = llvm_config_name();
        let binary_path = prefix.join(binary_name);
        if binary_path.as_path().exists() {
            Some(binary_path)
        } else {
            None
        }
    }
}

pub fn llvm_config_name() -> String {
    let mut base_name = "llvm-config".to_owned();

    // On Windows, also search for llvm-config.exe
    if llvm_sys::target_os_is("windows") {
        base_name.push_str(".exe");
    }

    base_name
}
