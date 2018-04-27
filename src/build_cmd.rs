extern crate cargo_metadata;
extern crate toml;

use std::fs::{copy, create_dir};
use std::path::PathBuf;
use std::process::Command;

use common::{run_cmd, Config, Error};

pub fn handle_build_cmd(config: &Config) -> Result<(), Error> {
    info!("{:#?}", config.helios_metadata);

    let build_type = if config.cli_args.flag_release {
        String::from("release")
    } else {
        String::from("debug")
    };

    let target_spec = if config.cli_args.flag_target.is_empty() {
        config.helios_metadata.default_target.clone()
    } else {
        config.cli_args.flag_target.clone()
    };

    let target_build_cache_path = PathBuf::from(
        &config.root_metadata.target_directory,
    ).join(&target_spec)
        .join(&build_type);

    let root_task_path = if config.helios_metadata.root_task.is_empty() {
        PathBuf::from(&config.root_metadata.workspace_root)
    } else {
        PathBuf::from(&config.root_metadata.workspace_root)
            .join(&config.helios_metadata.root_task)
    };

    let helios_sel4_config_manifest_path =
        PathBuf::from(&config.root_metadata.workspace_root);

    let root_task_name = if config.helios_metadata.root_task.is_empty() {
        config.root_metadata.packages[0].name.clone()
    } else {
        config.helios_metadata.root_task.clone()
    };

    info!(
        "target build cache: {:?}",
        target_build_cache_path
    );
    info!("root task name: {:?}", root_task_name);
    info!("root task path: {:?}", root_task_path);
    info!(
        "seL4 configuration manifest path: {:?}",
        helios_sel4_config_manifest_path.join("Cargo.toml")
    );

    info!("building root task '{}'", root_task_name);

    let mut cmd = Command::new(&config.helios_metadata.build_cmd);

    cmd.arg("build");

    if config.cli_args.flag_release {
        cmd.arg("--release");
    }

    if config.cli_args.flag_quiet {
        cmd.arg("--quiet");
    }

    if config.cli_args.flag_verbose {
        cmd.arg("--verbose");
    }

    if config.uses_root_manifest_config {
        cmd.env(
            "HELIOS_MANIFEST_PATH",
            &helios_sel4_config_manifest_path.join("Cargo.toml"),
        );
    }

    run_cmd(
        cmd.current_dir(root_task_path)
            .env(
                "RUST_TARGET_PATH",
                &config.helios_metadata.target_specs_path,
            )
            .env(
                "HELIOS_ARTIFACT_PATH",
                &config.helios_metadata.artifact_path,
            )
            .arg("--target")
            .arg(&target_spec),
    )?;

    let sysimg_path = config
        .helios_metadata
        .artifact_path
        .join("feL4img");
    let kernel_path = config
        .helios_metadata
        .artifact_path
        .join("kernel");

    if !config.helios_metadata.artifact_path.exists() {
        create_dir(&config.helios_metadata.artifact_path)?;
    }

    // copy the image out of the Cargo workspace
    copy(
        target_build_cache_path.join(&root_task_name),
        &sysimg_path,
    )?;

    if !sysimg_path.exists() {
        return Err(Error::MetadataError(
            format!("something went wrong with the build, cannot find the system image '{}'",
            target_build_cache_path.join(&root_task_name).display())
        ));
    }

    if !kernel_path.exists() {
        return Err(Error::MetadataError(
            format!("something went wrong with the build, cannot find the kernel file '{}'",
            kernel_path.display())
        ));
    }

    info!(
        "output artifact path '{}'",
        config.helios_metadata.artifact_path.display()
    );

    info!("kernel: '{}'", kernel_path.display());
    info!("feL4img: '{}'", sysimg_path.display());

    Ok(())
}
