extern crate cargo_metadata;
extern crate toml;

use std::process::Command;

use common::{run_cmd, Error};
use config::Config;

pub fn handle_simulate_cmd(config: &Config) -> Result<(), Error> {
    let sim_script_path = config
        .root_dir
        .join(&config.fel4_metadata.artifact_path)
        .join("simulate");

    if !sim_script_path.exists() {
        return Err(Error::ConfigError(
        format!("something went wrong with the build, cannot find the simulation script '{}'",
        sim_script_path.display())));
    }

    run_cmd(&mut Command::new(&sim_script_path))?;

    Ok(())
}
