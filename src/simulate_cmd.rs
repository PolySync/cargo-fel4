extern crate cargo_metadata;
extern crate toml;

use std::path::Path;
use std::process::Command;

use common::{run_cmd, Error};
use config::Config;

pub fn handle_simulate_cmd(config: &Config) -> Result<(), Error> {
    let mut mani_path =
        Path::new(&config.pkg_metadata.manifest_path).to_path_buf();
    mani_path.pop();
    let sim_script_path = mani_path
        .join(&config.fel4_metadata.artifact_path)
        .join("simulate");

    if !sim_script_path.exists() {
        return Err(Error::MetadataError(
        format!("something went wrong with the build, cannot find the simulation script '{}'",
        sim_script_path.display())));
    }

    run_cmd(&mut Command::new(&sim_script_path))?;

    Ok(())
}
