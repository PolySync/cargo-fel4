extern crate cargo_fel4;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate tempfile;

use cargo_fel4::Logger;
use std::env::{current_dir, set_current_dir};
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tempfile::TempDir;

static LOGGER: Logger = Logger;

lazy_static! {
    static ref SEQUENTIAL_TEST_MUTEX: Mutex<()> = Mutex::new(());
}
macro_rules! sequential_test {
    (fn $name:ident() $body:block) => {
        #[test]
        fn $name() {
            let _guard = $crate::SEQUENTIAL_TEST_MUTEX.lock();
            {
                $body
            }
        }
    };
}

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
        loudness: cargo_fel4::LoudnessOpts {
            verbose: true,
            quiet: false,
        },
        name: Some("foo".to_string()),
        path: target_dir.clone(),
    }).expect("could not run fel4 new command");
}

sequential_test! {
    fn cargo_fel4_x86_64_build_works_on_new_project() {
        enable_detailed_logging();
        let d = TempDir::new().expect("Could not create temp dir");
        let target_dir: PathBuf = d.path().join("bar").into();
        run_fel4_new(&target_dir);
        let target_cargo_manifest = target_dir.clone().join("Cargo.toml");
        let old_dir = current_dir().expect("Could get current dir");
        set_current_dir(&target_dir).expect("Could not change current dir to target test repo dir");
        let r = cargo_fel4::handle_build_cmd(&cargo_fel4::BuildCmd {
            loudness: cargo_fel4::LoudnessOpts {
                verbose: true,
                quiet: false,
            },
            release: false,
            tests: false,
            cargo_manifest_path: target_cargo_manifest.clone(),
        });
        r.expect("could not run fel4 build command");
        assert!(&target_dir.join("src/bin/root-task.rs").is_file());
        assert!(&target_dir.join("artifacts/debug/simulate").is_file());
        assert!(&target_dir.join("artifacts/debug/kernel").is_file());
        assert!(&target_dir.join("artifacts/debug/feL4img").is_file());

        let _t = cargo_fel4::handle_test_cmd(&cargo_fel4::TestCmd {
            loudness: cargo_fel4::LoudnessOpts {
                verbose: true,
                quiet: false,
            },
            release: false,
            subcmd: cargo_fel4::TestSubCmd::Build,
            cargo_manifest_path: target_cargo_manifest.clone(),
        }).expect("Could not run handle_test_command");
        assert!(&target_dir.join("artifacts/test/debug/simulate").is_file());
        assert!(&target_dir.join("artifacts/test/debug/kernel").is_file());
        assert!(&target_dir.join("artifacts/test/debug/feL4img").is_file());

        cargo_fel4::handle_clean_cmd(&cargo_fel4::CleanCmd {
            loudness: cargo_fel4::LoudnessOpts {
                verbose: true,
                quiet: false,
            },
            cargo_manifest_path: target_cargo_manifest.clone(),
        }).expect("Could not run clean command");
        assert!(!&target_dir.join("artifacts/debug/simulate").is_file());
        assert!(!&target_dir.join("artifacts/debug/kernel").is_file());
        assert!(!&target_dir.join("artifacts/debug/feL4img").is_file());
        assert!(!&target_dir.join("artifacts/test/debug/simulate").is_file());
        assert!(!&target_dir.join("artifacts/test/debug/kernel").is_file());
        assert!(!&target_dir.join("artifacts/test/debug/feL4img").is_file());

        set_current_dir(old_dir).expect("Could not change the current dir back to its original state");
    }
}

fn replace_target_with_arm(fel4_manifest_path: &Path) {
    // Replace the default x86_64 target and pc99 platform with arm and sabre,
    // respectively
    let original = {
        let mut file = File::open(&fel4_manifest_path).expect("Could not open fel4.toml");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Could not read fel4.toml");
        contents
    };
    let contents = original.replace(
        "target = \"x86_64-sel4-fel4\"",
        "target = \"arm-sel4-fel4\"",
    );
    let contents = contents.replace("platform = \"pc99\"", "platform = \"sabre\"");
    let mut file = File::create(&fel4_manifest_path).expect("Could not recreate fel4 manifest");
    file.write(&contents.as_bytes())
        .expect("Could not write new content");
}

sequential_test! {
    fn cargo_fel4_arm_build_works_on_new_project() {
        enable_detailed_logging();
        let d = TempDir::new().expect("Could not create temp dir");
        let target_dir: PathBuf = d.path().join("arm").into();
        let target_cargo_manifest = target_dir.clone().join("Cargo.toml");
        run_fel4_new(&target_dir);

        let fel4_manifest_path = target_dir.join("fel4.toml");
        replace_target_with_arm(&fel4_manifest_path);

        let old_dir = current_dir().expect("Could get current dir");
        set_current_dir(&target_dir).expect("Could not change current dir to target test repo dir");


        let r = cargo_fel4::handle_build_cmd(&cargo_fel4::BuildCmd {
            loudness: cargo_fel4::LoudnessOpts {
                verbose: true,
                quiet: false,
            },
            release: false,
            tests: false,
            cargo_manifest_path: target_cargo_manifest.clone(),
        });
        r.expect("could not run fel4 build command");
        assert!(&target_dir.join("src/bin/root-task.rs").is_file());
        assert!(&target_dir.join("artifacts/debug/simulate").is_file());
        assert!(&target_dir.join("artifacts/debug/kernel").is_file());
        assert!(&target_dir.join("artifacts/debug/feL4img").is_file());

        let _t = cargo_fel4::handle_test_cmd(&cargo_fel4::TestCmd {
            loudness: cargo_fel4::LoudnessOpts {
                verbose: true,
                quiet: false,
            },
            release: false,
            subcmd: cargo_fel4::TestSubCmd::Build,
            cargo_manifest_path: target_cargo_manifest.clone(),
        }).expect("Could not run handle_test_command");
        assert!(&target_dir.join("artifacts/test/debug/simulate").is_file());
        assert!(&target_dir.join("artifacts/test/debug/kernel").is_file());
        assert!(&target_dir.join("artifacts/test/debug/feL4img").is_file());

        cargo_fel4::handle_clean_cmd(&cargo_fel4::CleanCmd {
            loudness: cargo_fel4::LoudnessOpts {
                verbose: true,
                quiet: false,
            },
            cargo_manifest_path: target_cargo_manifest.clone(),
        }).expect("Could not run clean command");
        assert!(!&target_dir.join("artifacts/debug/simulate").is_file());
        assert!(!&target_dir.join("artifacts/debug/kernel").is_file());
        assert!(!&target_dir.join("artifacts/debug/feL4img").is_file());
        assert!(!&target_dir.join("artifacts/test/debug/simulate").is_file());
        assert!(!&target_dir.join("artifacts/test/debug/kernel").is_file());
        assert!(!&target_dir.join("artifacts/test/debug/feL4img").is_file());

        set_current_dir(old_dir).expect("Could not change the current dir back to its original state");

        assert!(&target_dir.join("src/fel4_test.rs").is_file());
    }
}
