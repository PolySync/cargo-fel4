use log;
use log::LevelFilter;
use std::path::Path;
use std::process::Command;

use super::{gather_config, run_cmd, Error};
use config::{Config, Fel4SubCmd, SimulateCmd};

pub fn handle_simulate_cmd(subcmd: &SimulateCmd) -> Result<(), Error> {
    if subcmd.verbose {
        log::set_max_level(LevelFilter::Info);
    } else {
        log::set_max_level(LevelFilter::Error);
    }

    let config: Config = gather_config(&Fel4SubCmd::SimulateCmd(subcmd.clone()))?;

    let artifact_path = Path::new(&config.root_dir)
        .join(config.fel4_config.artifact_path)
        .join(config.build_profile.artifact_subdir_path());

    let sim_script_path = artifact_path.join("simulate");

    if !sim_script_path.exists() {
        return Err(Error::ConfigError(format!(
            "something went wrong with the build, cannot find the simulation script '{}'",
            sim_script_path.display()
        )));
    }

    run_cmd(&mut Command::new(&sim_script_path).current_dir(&artifact_path.parent().unwrap()))?;

    Ok(())
}
