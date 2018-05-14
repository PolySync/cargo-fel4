use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use super::{run_cmd, Error};
use config::Config;

pub fn handle_new_cmd(config: &Config) -> Result<(), Error> {
    // TODO
    //
    // name/path, subcommands for context help?
    let package_name = "fel4-project";

    // base cargo new command to construct project scaffolding
    let mut cmd = Command::new("cargo");
    cmd.arg("new");

    // passthrough to cargo
    if config.cli_args.flag_verbose {
        cmd.arg("--verbose");
    } else if config.cli_args.flag_quiet {
        cmd.arg("--quiet");
    }

    // project name and directory are the same
    cmd.arg("--name").arg(package_name);

    // build a bare library project
    run_cmd(cmd.arg("--lib").arg("fel4-project"))?;

    // create/truncate the generated src/lib.rs with our new contents
    let mut lib_src_file = File::create(Path::new(package_name).join("src").join("lib.rs"))?;

    lib_src_file.write_all(
        b"#![no_std]
extern crate sel4_sys;

use sel4_sys::DebugOutHandle;

macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        DebugOutHandle.write_fmt(format_args!($($arg)*)).unwrap();
    });
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, \"\\n\")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, \"\\n\"), $($arg)*));
}

pub fn run() {
    println!(\"\\nhello from a fel4 app!\\n\");
}",
    )?;

    Ok(())
}
