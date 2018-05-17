use log;
use log::LevelFilter;
use std::process::Command;

use super::{gather_config, run_cmd, Error};
use config::{Config, SimulateCmd};
use fel4_config::BuildProfile;

pub fn handle_simulate_cmd(subcmd: &SimulateCmd) -> Result<(), Error> {
    if subcmd.verbose {
        log::set_max_level(LevelFilter::Info);
    } else {
        log::set_max_level(LevelFilter::Error);
    }
    let build_profile = match subcmd.release {
        true => BuildProfile::Release,
        false => BuildProfile::Debug,
    };

    let config: Config = gather_config(&build_profile)?;

    let sim_script_path = config
        .root_dir
        .join(&config.fel4_config.artifact_path)
        .join("simulate");

    if !sim_script_path.exists() {
        return Err(Error::ConfigError(format!(
            "something went wrong with the build, cannot find the simulation script '{}'",
            sim_script_path.display()
        )));
    }

    run_cmd(&mut Command::new(&sim_script_path))?;

    Ok(())
}
