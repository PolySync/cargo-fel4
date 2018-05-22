use std::path::Path;
use std::process::Command;

use super::{gather_config, Error};
use command_ext::CommandExt;
use config::{Config, Fel4SubCmd, SimulateCmd};

pub fn handle_simulate_cmd(subcmd: &SimulateCmd) -> Result<(), Error> {
    let config: Config = gather_config(&Fel4SubCmd::SimulateCmd(subcmd.clone()))?;

    let artifact_profile_subdir = if let Some(p) = config.build_profile {
        p.artifact_subdir_path()
    } else {
        // TODO - better error message
        return Err(Error::ConfigError(
            "The build profile could not determined".to_string(),
        ));
    };

    let artifact_path = Path::new(&config.root_dir)
        .join(config.fel4_config.artifact_path)
        .join(artifact_profile_subdir);

    let sim_script_path = artifact_path.join("simulate");

    if !sim_script_path.exists() {
        return Err(Error::ConfigError(format!(
            "something went wrong with the build, cannot find the simulation script '{}'",
            sim_script_path.display()
        )));
    }

    Command::new(&sim_script_path)
        .current_dir(&artifact_path.parent().unwrap())
        .run_cmd()
}
