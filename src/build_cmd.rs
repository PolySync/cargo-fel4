extern crate cargo_metadata;
extern crate toml;

use std::fs;
use std::fs::File;
use std::process::Command;

use super::{run_cmd, Error};
use config::Config;
use generator::Generator;

pub fn handle_build_cmd(config: &Config) -> Result<(), Error> {
    let root_task_path = config.root_dir.join("src").join("bin");
    fs::create_dir_all(&root_task_path)?;
    let mut root_file =
        File::create(root_task_path.join("root-task.rs").as_path())?;
    let mut gen = Generator::new(&mut root_file, config);
    gen.generate()?;

    let build_type = if config.cli_args.flag_release {
        String::from("release")
    } else {
        String::from("debug")
    };

    let target_build_cache_path = config
        .root_dir
        .join("target")
        .join(&config.target)
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

    // If anything depends on `alloc`, then we seem to be locked
    // into using whatever features `compiler-builtins` selects.
    // One of which is the `c` feature, which is why a cross-compiler
    // is now needed to build the sysroot.
    // Prevously users were able to control the features in `Xargo.toml`,
    // however for the time being we are no longer able to do so.
    // See the following issues:
    // `xargo/issues/216`
    // `cargo-fel4/issues/18`
    if config.target == "arm-sel4-fel4" {
        cmd.env("CC_arm-sel4-fel4", "arm-linux-gnueabihf-gcc");
    }

    let targets_path = config
        .root_dir
        .join(&config.fel4_metadata.target_specs_path);
    let artifact_path = config
        .root_dir
        .join(&config.fel4_metadata.artifact_path);
    let mani_path = config.root_dir.join("fel4.toml");

    run_cmd(
        cmd.env("FEL4_MANIFEST_PATH", &mani_path)
            .env("FEL4_ARTIFACT_PATH", &artifact_path)
            .env("RUST_TARGET_PATH", &targets_path)
            .arg("--target")
            .arg(&config.target),
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

    // For ARM targets, we currently take advantage of the
    // seL4 elfloader-tool to bootstrap the system and kick
    // things off.
    // To accomplish this, we just re-build libsel4-sys
    // with an extra environment variable which gives
    // elfloader-tool a path to the root-task binary
    if config.target == "arm-sel4-fel4" {
        run_cmd(
            cmd.env(
                "FEL4_ROOT_TASK_IMAGE_PATH",
                target_build_cache_path.join("root-task"),
            ).arg("-p")
                .arg("libsel4-sys"),
        )?;

        // seL4 CMake rules will just output everything to `kernel`
        // we copy it so it's consistent with our image name but
        // won't trigger a rebuild (as it would if we were to move it)
        fs::copy(&kernel_path, &sysimg_path)?;
    } else {
        fs::copy(
            target_build_cache_path.join("root-task"),
            &sysimg_path,
        )?;
    }

    if !sysimg_path.exists() {
        return Err(Error::ConfigError(
            format!("something went wrong with the build, cannot find the system image '{}'",
            target_build_cache_path.join(&sysimg_path).display())
        ));
    }

    if !kernel_path.exists() {
        return Err(Error::ConfigError(
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
