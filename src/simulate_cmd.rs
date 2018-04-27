extern crate cargo_metadata;
extern crate toml;

use std::path::PathBuf;
use std::process::Command;

use common::{run_cmd, Config, Error};

pub fn handle_simulate_cmd(config: &Config) -> Result<(), Error> {
    let sim_script_path =
        PathBuf::from(&config.helios_metadata.artifact_path).join("simulate");

    if !sim_script_path.exists() {
        return Err(Error::MetadataError(
        format!("something went wrong with the build, cannot find the simulation script '{}'",
        sim_script_path.display())));
    }

    let run_from_path = match sim_script_path.parent() {
        Some(p) => match p.parent() {
            Some(nextp) => nextp,
            _ => {
                return Err(Error::IO(format!(
                "failed to navigate simulation script parent directory '{}'",
                p.display()
            )))
            }
        },
        _ => {
            return Err(Error::IO(format!(
                "failed to navigate simulation script parent directory '{}'",
                sim_script_path.display()
            )))
        }
    };

    run_cmd(Command::new(&sim_script_path).current_dir(run_from_path))?;

    Ok(())
}
