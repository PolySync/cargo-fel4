extern crate cargo_metadata;
extern crate toml;

use std::fs;
use std::fs::File;
use std::process::Command;

use common::{run_cmd, Error};
use config::Config;
use generator::Generator;

pub fn handle_build_cmd(config: &Config) -> Result<(), Error> {
    let root_task_path = config.root_dir.join("src").join("bin");
    fs::create_dir_all(&root_task_path)?;
    let mut root_file =
        File::create(root_task_path.join("root-task.rs").as_path())?;
    let mut gen = Generator::new(&mut root_file);
    gen.generate(&config.pkg_name)?;

    let build_type = if config.cli_args.flag_release {
        String::from("release")
    } else {
        String::from("debug")
    };

    let target_spec = if config.cli_args.flag_target.is_empty() {
        config.fel4_metadata.default_target.clone()
    } else {
        config.cli_args.flag_target.clone()
    };

    let target_build_cache_path = config
        .root_dir
        .join("target")
        .join(&target_spec)
        .join(&build_type);

    info!(
        "\ntarget build cache: {:?}",
        target_build_cache_path,
    );

    let mut cmd = Command::new("xargo");

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

    let targets_path = config
        .root_dir
        .join(&config.fel4_metadata.target_specs_path);
    let artifact_path = config
        .root_dir
        .join(&config.fel4_metadata.artifact_path);

    run_cmd(
        cmd.current_dir(root_task_path)
            .env(
                "RUST_TARGET_PATH",
                &config.fel4_metadata.target_specs_path,
            )
            .env(
                "HELIOS_ARTIFACT_PATH",
                &config.fel4_metadata.artifact_path,
            )
            .env("HELIOS_ARTIFACT_PATH", &artifact_path)
            .env("RUST_TARGET_PATH", &targets_path)
            .arg("--target")
            .arg(&target_spec),
    )?;

    let sysimg_path = config
        .fel4_metadata
        .artifact_path
        .join("feL4img");
    let kernel_path = config
        .fel4_metadata
        .artifact_path
        .join("kernel");

    if !config.fel4_metadata.artifact_path.exists() {
        fs::create_dir(&config.fel4_metadata.artifact_path)?;
    }

    fs::copy(
        target_build_cache_path.join("root-task"),
        &sysimg_path,
    )?;

    if !sysimg_path.exists() {
        return Err(Error::MetadataError(
            format!("something went wrong with the build, cannot find the system image '{}'",
            target_build_cache_path.join(&sysimg_path).display())
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
        config.fel4_metadata.artifact_path.display()
    );

    info!("kernel: '{}'", kernel_path.display());
    info!("feL4img: '{}'", sysimg_path.display());

    Ok(())
}
