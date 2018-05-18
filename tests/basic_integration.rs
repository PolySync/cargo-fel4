extern crate cargo_fel4;
#[macro_use]
extern crate log;
extern crate tempfile;

use cargo_fel4::Logger;
use std::env::{current_dir, set_current_dir};
use std::path::PathBuf;
use tempfile::TempDir;

static LOGGER: Logger = Logger;

fn enable_detailed_logging() {
    if let Err(e) = log::set_logger(&LOGGER) {
        error!("{}", e);
    };
    log::set_max_level(log::LevelFilter::Info);
}

#[test]
fn cargo_fel4_new_runs_in_temp_dir_with_success() {
    enable_detailed_logging();
    let d = TempDir::new().expect("Could not create temp dir");
    let target_dir: PathBuf = d.path().join("bar").into();
    run_fel4_new(&target_dir);

    assert!(&target_dir.is_dir(), "Should have a fresh-made dir");
    assert!(&target_dir.join("src/lib.rs").is_file());
    assert!(&target_dir.join("Cargo.toml").is_file());
    assert!(&target_dir.join("fel4.toml").is_file());
}

fn run_fel4_new(target_dir: &PathBuf) {
    let _ = cargo_fel4::handle_new_cmd(&cargo_fel4::NewCmd {
        verbose: true,
        quiet: false,
        name: Some("foo".to_string()),
        path: target_dir.clone(),
    }).expect("could not run fel4 new command");
}

#[test]
fn cargo_fel4_build_works_on_new_project() {
    enable_detailed_logging();
    let d = TempDir::new().expect("Could not create temp dir");
    let target_dir: PathBuf = d.path().join("bar").into();
    let target_cargo_manifest = target_dir.clone().join("Cargo.toml");
    run_fel4_new(&target_dir);

    let old_dir = current_dir().expect("Could get current dir");
    set_current_dir(&target_dir).expect("Could not change current dir to target test repo dir");
    let r = cargo_fel4::handle_build_cmd(&cargo_fel4::BuildCmd {
        verbose: true,
        quiet: false,
        release: false,
        tests: false,
        cargo_manifest_path: target_cargo_manifest,
    });
    set_current_dir(old_dir).expect("Could not change the current dir back to its original state");
    r.expect("could not run fel4 build command");
    assert!(&target_dir.join("src/bin/root-task.rs").is_file());
    assert!(&target_dir.join("artifacts/simulate").is_file());
    assert!(&target_dir.join("artifacts/kernel").is_file());
    assert!(&target_dir.join("artifacts/feL4img").is_file());
}
