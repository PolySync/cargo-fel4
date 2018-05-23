use std::path::Path;
use std::process::Command;

use super::Error;
use command_ext::CommandExt;
use config::{get_fel4_manifest_with_root_dir, Fel4BuildProfile, ManifestWithRootDir, SimulateCmd};

pub fn handle_simulate_cmd(cmd: &SimulateCmd) -> Result<(), Error> {
    let ManifestWithRootDir {
        fel4_manifest,
        root_dir,
    } = get_fel4_manifest_with_root_dir(&cmd.cargo_manifest_path)?;
    let artifact_path = Path::new(&root_dir)
        .join(fel4_manifest.artifact_path)
        .join(Fel4BuildProfile::from(cmd).artifact_subdir_path());

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
