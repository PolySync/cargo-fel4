extern crate cargo_metadata;
extern crate toml;

use std::fs;
use std::fs::File;
use std::process::Command;

use super::{run_cmd, Error};
use config::Config;
use generator::Generator;
use cmake_codegen::{cache_to_interesting_flags, truthy_boolean_flags_as_rust_identifiers};

pub fn handle_build_cmd(config: &Config) -> Result<(), Error> {
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

    let mut stage_1 = Command::new("xargo");

    stage_1.arg("build");

    if config.cli_args.flag_release {
        stage_1.arg("--release");
    }

    if config.cli_args.flag_quiet {
        stage_1.arg("--quiet");
    }

    if config.cli_args.flag_verbose {
        stage_1.arg("--verbose");
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
    // get used to build the shims through the use of CC's envirnoment
    // variables.
    //
    // See the following issues:
    // `xargo/issues/216`
    // `cargo-fel4/issues/18`
    if config.target == "arm-sel4-fel4" {
        stage_1.env("CC_arm-sel4-fel4", "arm-linux-gnueabihf-gcc");
    }

    let targets_path = config
        .root_dir
        .join(&config.fel4_metadata.target_specs_path);
    let artifact_path = config
        .root_dir
        .join(&config.fel4_metadata.artifact_path);
    let mani_path = config.root_dir.join("fel4.toml");

    run_cmd(
        stage_1
            .env("FEL4_MANIFEST_PATH", &mani_path)
            .env("FEL4_ARTIFACT_PATH", &artifact_path)
            .env("RUST_TARGET_PATH", &targets_path)
            .arg("--target")
            .arg(&config.target)
            .arg("-p")
            .arg("libsel4-sys"),
    )?;
    let interesting_flags = cache_to_interesting_flags(config
        .root_dir
        .join(&config.fel4_metadata.artifact_path)
        .join("CMakeCache.txt"))?;

    let root_task_path = config.root_dir.join("src").join("bin");
    fs::create_dir_all(&root_task_path)?;
    let mut root_file =
        File::create(root_task_path.join("root-task.rs").as_path())?;
    let mut gen = Generator::new(&mut root_file, config, &interesting_flags);
    gen.generate()?;

    let mut stage_2 = Command::new("xargo");

    stage_2.arg("rustc").arg("--bin").arg("root-task");


    if config.cli_args.flag_release {
        stage_2.arg("--release");
    }

    if config.cli_args.flag_quiet {
        stage_2.arg("--quiet");
    }

    if config.cli_args.flag_verbose {
        stage_2.arg("--verbose");
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
    // get used to build the shims through the use of CC's envirnoment
    // variables.
    //
    // See the following issues:
    // `xargo/issues/216`
    // `cargo-fel4/issues/18`
    if config.target == "arm-sel4-fel4" {
        stage_2.env("CC_arm-sel4-fel4", "arm-linux-gnueabihf-gcc");
    }

    stage_2.arg("--target")
           .arg(&config.target);

    let truthy_cmake_feature_flags = truthy_boolean_flags_as_rust_identifiers(&interesting_flags)?;
    if !truthy_cmake_feature_flags.is_empty() {
        stage_2.arg("--");
        for feature in truthy_cmake_feature_flags {
            stage_2.arg("--cfg");
            stage_2.arg(format!("feature=\"{}\"", feature));
        }
    }

    run_cmd(
        stage_2
            .env("FEL4_MANIFEST_PATH", &mani_path)
            .env("FEL4_ARTIFACT_PATH", &artifact_path)
            .env("RUST_TARGET_PATH", &targets_path)
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
            stage_2
                .env(
                    "FEL4_ROOT_TASK_IMAGE_PATH",
                    target_build_cache_path.join("root-task"),
                )
                .arg("-p")
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
