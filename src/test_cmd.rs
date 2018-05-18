use super::{handle_build_cmd, Error};
use config::{BuildCmd, TestCmd, TestSubCmd};
use log;
use log::LevelFilter;
use new_cmd::generate_tests_source_files;

pub fn handle_test_cmd(test_cmd: &TestCmd) -> Result<(), Error> {
    if test_cmd.verbose {
        log::set_max_level(LevelFilter::Info);
    } else {
        log::set_max_level(LevelFilter::Error);
    }

    if let Some(ref subcmd) = test_cmd.subcmd {
        match subcmd {
            TestSubCmd::Build => {
                generate_tests_source_files(None)?;
                run_test_build(test_cmd)?;
            }
        }
    }

    Ok(())
}

fn run_test_build(test_cmd: &TestCmd) -> Result<(), Error> {
    let build_cmd = BuildCmd {
        verbose: test_cmd.verbose,
        quiet: test_cmd.quiet,
        release: false,
        tests: true,
        cargo_manifest_path: test_cmd.cargo_manifest_path.clone(),
    };

    handle_build_cmd(&build_cmd)?;

    Ok(())
}
