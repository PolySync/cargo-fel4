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

    // There seems to be an issue with `compiler_builtins` imposing
    // a default compiler used by the `c` feature/dependency; where
    // it no longer picks up a sane cross-compiler (when host != target triple).
    // This results in compiler_builtin_shims being compiled with the
    // host's default compiler (likely x86_64) rather than using
    // what our target specification (or even Xargo.toml) has prescribed.
    //
    // This fix is a band aid, and will be addressed properly at a later point.
    // However we can still force/control which cross compiler will
    // get used to build the shims through the use of CC's envirnoment variables.
    //
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

    fs::copy(
        target_build_cache_path.join("root-task"),
        &sysimg_path,
    )?;

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
