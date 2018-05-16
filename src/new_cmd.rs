use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use super::{run_cmd, Error};
use config::NewCmd;
use log;
use log::LevelFilter;

/// Create a new feL4 project.
///
/// Generates all of the scaffolding files for a new
/// feL4 project.
pub fn handle_new_cmd(subcmd: &NewCmd) -> Result<(), Error> {
    if subcmd.verbose {
        log::set_max_level(LevelFilter::Info);
    } else {
        log::set_max_level(LevelFilter::Error);
    }

    generate_baseline_cargo_package(subcmd)?;

    generate_fel4_project_files(subcmd)?;

    generate_target_specs(subcmd)?;

    Ok(())
}

fn generate_baseline_cargo_package(subcmd: &NewCmd) -> Result<(), Error> {
    let mut cmd = Command::new("cargo");
    cmd.arg("new");

    // We passthrough these to cargo
    if subcmd.verbose {
        cmd.arg("--verbose");
    } else if subcmd.quiet {
        cmd.arg("--quiet");
    }

    if let Some(ref n) = subcmd.name {
        cmd.arg("--name").arg(&n);
    }

    run_cmd(cmd.arg("--lib").arg(&subcmd.path))?;

    Ok(())
}

fn generate_fel4_project_files(subcmd: &NewCmd) -> Result<(), Error> {
    // Create example feL4 application thread run function
    let mut lib_src_file = File::create(Path::new(&subcmd.path).join("src").join("lib.rs"))?;
    lib_src_file.write_all(APP_LIB_CODE.as_bytes())?;

    // Add feL4 dependencies to Cargo.toml
    let mut cargo_toml_file = OpenOptions::new()
        .append(true)
        .open(Path::new(&subcmd.path).join("Cargo.toml"))?;

    // Add feL4 dev-dependencies
    cargo_toml_file.write_all(CARGO_TOML_TEXT.as_bytes())?;

    let mut fel4_toml_file = File::create(Path::new(&subcmd.path).join("fel4.toml"))?;
    fel4_toml_file.write_all(FEL4_TOML_TEXT.as_bytes())?;

    // Create Xargo.toml with our target features
    let mut xargo_toml_file = File::create(Path::new(&subcmd.path).join("Xargo.toml"))?;
    xargo_toml_file.write_all(XARGO_TOML_TEXT.as_bytes())?;

    Ok(())
}

fn generate_target_specs(subcmd: &NewCmd) -> Result<(), Error> {
    // Create target specifications directory and specification files
    let target_specs_path = Path::new(&subcmd.path).join("target_specs");
    fs::create_dir(Path::new(&target_specs_path))?;

    let mut target_spec_readme_file = File::create(&target_specs_path.join("README.md"))?;
    target_spec_readme_file.write_all(FEL4_TARGET_SPEC_README.as_bytes())?;

    let mut target_spec_x86_64_file =
        File::create(&target_specs_path.join("x86_64-sel4-fel4.json"))?;
    target_spec_x86_64_file.write_all(FEL4_TARGET_SPEC_X86_64_SEL4_FEL4.as_bytes())?;

    let mut target_spec_arm_file = File::create(&target_specs_path.join("arm-sel4-fel4.json"))?;
    target_spec_arm_file.write_all(FEL4_TARGET_SPEC_ARM_SEL4_FEL4.as_bytes())?;

    Ok(())
}

const FEL4_TARGET_SPEC_README: &'static str = include_str!("../target_specs/README.md");

const FEL4_TARGET_SPEC_X86_64_SEL4_FEL4: &'static str =
    include_str!("../target_specs/x86_64-sel4-fel4.json");

const FEL4_TARGET_SPEC_ARM_SEL4_FEL4: &'static str =
    include_str!("../target_specs/arm-sel4-fel4.json");

const FEL4_TOML_TEXT: &'static str = include_str!("../configs/fel4.toml");

const APP_LIB_CODE: &'static str = include_str!("../templates/lib.rs");

const XARGO_TOML_TEXT: &'static str = include_str!("../templates/Xargo.toml");

const CARGO_TOML_TEXT: &'static str = include_str!("../templates/Cargo.toml.part");
