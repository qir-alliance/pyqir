// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::Parser;
use inkwell::context::Context;
use qirlib::module::load_file;
use std::path::PathBuf;

/// Simple program for combining multiple QIR programs together using standard LLVM linking.
#[derive(Parser, Debug)]
#[clap(name = "QIR Link", bin_name = "qirlink")]
struct Args {
    /// When provided, output IR instead of bitcode
    #[clap(long = "emit-ir")]
    emit_ir: bool,

    /// Output file path
    #[clap(short, long, parse(from_os_str))]
    output: PathBuf,

    /// One or more files to combine
    #[clap(short, long, required(true), parse(from_os_str))]
    file: Vec<PathBuf>,
}

pub fn main() -> Result<(), String> {
    let args = Args::parse();

    let context = Context::create();
    let module = load_file(args.file[0].as_path(), &context)?;
    module.verify().map_err(|e| e.to_string())?;

    if args.file.len() > 1 {
        for file in args.file[1..].iter() {
            let other = load_file(file.as_path(), &context)?;
            other.verify().map_err(|e| e.to_string())?;
            module.link_in_module(other).map_err(|e| e.to_string())?;
        }
    }

    if args.emit_ir {
        module.print_to_file(args.output.as_path()).map_err(|e| e.to_string())
    } else if !module.write_bitcode_to_path(args.output.as_path()) {
        Err("Failed to write bitcode.".to_string())
    } else {
        Ok(())
    }
}
