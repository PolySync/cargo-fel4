use std::path::Path;
use std::process::Command;

use super::Error;
use command_ext::CommandExt;
use config::{get_fel4_manifest_with_root_dir, DeployCmd, Fel4BuildProfile, ManifestWithRootDir};
use fel4_config::SupportedPlatform;

pub fn handle_deploy_cmd(cmd: &DeployCmd) -> Result<(), Error> {
    let ManifestWithRootDir {
        fel4_manifest,
        root_dir,
    } = get_fel4_manifest_with_root_dir(&cmd.cargo_manifest_path)?;
    let artifact_path = Path::new(&root_dir)
        .join(&fel4_manifest.artifact_path)
        .join(Fel4BuildProfile::from(cmd).artifact_subdir_path());

    if fel4_manifest.selected_platform != SupportedPlatform::Tx1 {
        return Err(Error::ConfigError(format!(
            "The selected {} platform does not support deployment",
            fel4_manifest.selected_platform.full_name()
        )));
    }

    let fel4img_path = artifact_path.join("feL4img");

    if !fel4img_path.exists() {
        return Err(Error::ConfigError(format!(
            "Something went wrong with the build, cannot find the deploy image '{}'",
            fel4img_path.display()
        )));
    }

    // We currently only support deploying to the TX1 platform,
    // so the deploy-mode is hard-coded to DFU and
    // the DFU device is hard-coded to the TX1
    Command::new("dfu-util")
        .arg("--device")
        .arg("0955:701a")
        .arg("-a")
        .arg("kernel")
        .arg("-D")
        .arg(fel4img_path)
        .run_cmd()
}
