use super::{handle_build_cmd, handle_simulate_cmd, Error};
use config::{BuildCmd, SimulateCmd, TestCmd, TestSubCmd};
use new_cmd::generate_tests_source_files;

pub fn handle_test_cmd(test_cmd: &TestCmd) -> Result<(), Error> {
    match test_cmd.subcmd {
        TestSubCmd::Build => {
            generate_tests_source_files(test_cmd.cargo_manifest_path.parent())?;
            run_test_build(test_cmd)?;
        }
        TestSubCmd::Simulate => run_test_simulation(test_cmd)?,
        _ => {
            generate_tests_source_files(test_cmd.cargo_manifest_path.parent())?;
            run_test_build(test_cmd)?;
            run_test_simulation(test_cmd)?
        }
    };

    Ok(())
}

fn run_test_build(test_cmd: &TestCmd) -> Result<(), Error> {
    let build_cmd = BuildCmd {
        loudness: test_cmd.loudness.clone(),
        release: test_cmd.release,
        tests: true,
        cargo_manifest_path: test_cmd.cargo_manifest_path.clone(),
    };

    handle_build_cmd(&build_cmd)?;

    Ok(())
}

fn run_test_simulation(test_cmd: &TestCmd) -> Result<(), Error> {
    let sim_cmd = SimulateCmd {
        loudness: test_cmd.loudness.clone(),
        release: test_cmd.release,
        tests: true,
        cargo_manifest_path: test_cmd.cargo_manifest_path.clone(),
    };

    handle_simulate_cmd(&sim_cmd)?;

    Ok(())
}
