use std::path::Path;
use std::process::Command;

use super::Error;
use command_ext::CommandExt;
use config::{get_fel4_manifest_with_root_dir, Fel4BuildProfile, ManifestWithRootDir, SimulateCmd};
use fel4_config::SupportedPlatform;

pub fn handle_simulate_cmd(cmd: &SimulateCmd) -> Result<(), Error> {
    let ManifestWithRootDir {
        fel4_manifest,
        root_dir,
    } = get_fel4_manifest_with_root_dir(&cmd.cargo_manifest_path)?;
    let artifact_path = Path::new(&root_dir)
        .join(&fel4_manifest.artifact_path)
        .join(Fel4BuildProfile::from(cmd).artifact_subdir_path());

    if fel4_manifest.selected_platform == SupportedPlatform::Tx1 {
        return Err(Error::ConfigError(format!(
            "The selected {} platform does not support simulation",
            fel4_manifest.selected_platform.full_name()
        )));
    }

    // The simulation script is relies on being called from its parent directory
    let sim_script_path = Path::new(artifact_path.file_name().unwrap()).join("simulate");

    if !artifact_path.join("simulate").exists() {
        return Err(Error::ConfigError(format!(
            "Something went wrong with the build, cannot find the simulation script '{}'",
            artifact_path.join("simulate").display()
        )));
    }

    Command::new(&sim_script_path)
        .current_dir(&artifact_path.parent().unwrap())
        .run_cmd()
}
