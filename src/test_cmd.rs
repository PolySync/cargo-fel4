use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use super::{run_cmd, Error};
use config::{TestCmd, TestSubCmd};
use log;
use log::LevelFilter;

/// TODO
/// probably should renanme other top-level parameter to something
/// more consistent (<...>_cmd ?)
pub fn handle_test_cmd(test_cmd: &TestCmd) -> Result<(), Error> {
    if test_cmd.verbose {
        log::set_max_level(LevelFilter::Info);
    } else {
        log::set_max_level(LevelFilter::Error);
    }

    if let Some(ref subcmd) = test_cmd.subcmd {
        match subcmd {
            TestSubCmd::Build => generate_source_files(test_cmd)?,
        }
    }

    Ok(())
}

fn generate_source_files(test_cmd: &TestCmd) -> Result<(), Error> {
    let src_path = Path::new("src").join("fel4_test.rs");

    if !src_path.exists() {
        let mut test_src_file = File::create(&src_path)?;
        test_src_file.write_all(TEST_LIB_CODE.as_bytes())?;
    }

    Ok(())
}

const TEST_LIB_CODE: &'static str = include_str!("../templates/fel4_test.rs");
