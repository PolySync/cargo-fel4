extern crate cargo_metadata;
extern crate toml;

use std::borrow::Borrow;
use std::env;
use std::fs::{self, File};
use std::path::Path;
use std::process::Command;

use super::{run_cmd, Error};
use cmake_codegen::{cache_to_interesting_flags, truthy_boolean_flags_as_rust_identifiers};
use config::{CliArgs, Config};
use generator::Generator;
use heck::ShoutySnakeCase;

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

    info!("\ntarget build cache: {:?}", target_build_cache_path,);

    let cross_layer_locations = CrossLayerLocations {
        fel4_artifact_path: config.root_dir.join(&config.fel4_metadata.artifact_path),
        fel4_manifest_path: config.root_dir.join("fel4.toml"),
        rust_target_path: config
            .root_dir
            .join(&config.fel4_metadata.target_specs_path),
    };

    // Initial build of libsel4-sys to construct kernel, bindings and resolve CMake config
    let mut libsel4_build = construct_libsel4_build_command(config, &cross_layer_locations);
    run_cmd(&mut libsel4_build)?;

    // Extract the resolved CMake config details and filter down to ones that might be useful
    let interesting_flags = cache_to_interesting_flags(
        config
            .root_dir
            .join(&config.fel4_metadata.artifact_path)
            .join("CMakeCache.txt"),
    )?;
    let truthy_cmake_feature_flags = truthy_boolean_flags_as_rust_identifiers(&interesting_flags)?;
    let rustflags_env_var = merge_feature_flags_with_rustflags_env_var(&truthy_cmake_feature_flags);

    run_cmd(
        &mut construct_libsel4_build_command(config, &cross_layer_locations)
            .add_as_rustc_feature_flags(&truthy_cmake_feature_flags)
            .env("RUSTFLAGS", &rustflags_env_var),
    )?;

    // Generate the source code entry point (root task) for the application
    // that will wrap the end-user's code as executing within a sub-thread
    let root_task_path = config.root_dir.join("src").join("bin");
    fs::create_dir_all(&root_task_path)?;
    let mut root_file = File::create(root_task_path.join("root-task.rs").as_path())?;
    Generator::new(&mut root_file, config, &interesting_flags).generate()?;

    // Build the generated root task binary
    run_cmd(
        &mut construct_root_task_build_command(&config, &cross_layer_locations)
            .add_as_rustc_feature_flags(&truthy_cmake_feature_flags)
            .env("RUSTFLAGS", &rustflags_env_var),
    )?;

    let sysimg_path = config.fel4_metadata.artifact_path.join("feL4img");
    let kernel_path = config.fel4_metadata.artifact_path.join("kernel");

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
            construct_libsel4_build_command(&config, &cross_layer_locations)
                .env(
                    "FEL4_ROOT_TASK_IMAGE_PATH",
                    target_build_cache_path.join("root-task"),
                )
                .arg("-p")
                .arg("libsel4-sys")
                .add_as_rustc_feature_flags(&truthy_cmake_feature_flags)
                .env("RUSTFLAGS", &rustflags_env_var),
        )?;

        // seL4 CMake rules will just output everything to `kernel`
        // we copy it so it's consistent with our image name but
        // won't trigger a rebuild (as it would if we were to move it)
        fs::copy(&kernel_path, &sysimg_path)?;
    } else {
        fs::copy(target_build_cache_path.join("root-task"), &sysimg_path)?;
    }

    if !sysimg_path.exists() {
        return Err(Error::ConfigError(format!(
            "something went wrong with the build, cannot find the system image '{}'",
            target_build_cache_path.join(&sysimg_path).display()
        )));
    }

    if !kernel_path.exists() {
        return Err(Error::ConfigError(format!(
            "something went wrong with the build, cannot find the kernel file '{}'",
            kernel_path.display()
        )));
    }

    info!(
        "output artifact path '{}'",
        config.fel4_metadata.artifact_path.display()
    );

    info!("kernel: '{}'", kernel_path.display());
    info!("feL4img: '{}'", sysimg_path.display());

    Ok(())
}


fn construct_libsel4_build_command<P>(
    config: &Config,
    CrossLayerLocations {
        fel4_manifest_path,
        fel4_artifact_path,
        rust_target_path,
    }: &CrossLayerLocations<P>,
) -> Command
where
    P: Borrow<Path>,
{
    let mut libsel4_build = Command::new("xargo");

    libsel4_build.arg("rustc");

    if config.cli_args.flag_release {
        libsel4_build.arg("--release");
    }

    libsel4_build.add_loudness_args(&config.cli_args);

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
        libsel4_build.env("CC_arm-sel4-fel4", "arm-linux-gnueabihf-gcc");
    }

    libsel4_build
        .env("FEL4_MANIFEST_PATH", fel4_manifest_path.borrow())
        .env("FEL4_ARTIFACT_PATH", fel4_artifact_path.borrow())
        .env("RUST_TARGET_PATH", rust_target_path.borrow())
        .arg("--target")
        .arg(&config.target)
        .arg("-p")
        .arg("libsel4-sys");
    libsel4_build
}

/// Create a Command instance that, when run,
/// will build the root task binary
///
/// Note: Does NOT include application of Rust/Cargo feature flags
fn construct_root_task_build_command<P>(
    config: &Config,
    cross_layer_locations: &CrossLayerLocations<P>,
) -> Command
    where
        P: Borrow<Path>,
{
    let mut root_task_build = Command::new("xargo");
    root_task_build.arg("rustc").arg("--bin").arg("root-task");
    if config.cli_args.flag_release {
        root_task_build.arg("--release");
    }
    root_task_build.add_loudness_args(&config.cli_args);
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
        root_task_build.env("CC_arm-sel4-fel4", "arm-linux-gnueabihf-gcc");
    }
    root_task_build.arg("--target").arg(&config.target);
    root_task_build.add_as_env_vars(cross_layer_locations);
    root_task_build
}

/// Common-cause struct for the path data associated with the environment variables
/// used by cargo-fel4 to communicate across package and process boundaries.
#[derive(Clone, Debug, PartialEq)]
pub struct CrossLayerLocations<P: Borrow<Path>> {
    fel4_manifest_path: P,
    fel4_artifact_path: P,
    rust_target_path: P,
}

trait CommandExt {
    /// Populate the command with the environment variables tracked by CrossLayerLocations
    fn add_as_env_vars<'c, 'l, P: Borrow<Path>>(
        &'c mut self,
        cross_layer_locations: &'l CrossLayerLocations<P>,
    ) -> &'c mut Self;

    /// If any flags are present, adds an `--` arg and then adds new arguments
    /// of the form `--cfg` and  `feature=\"FOO\"`
    fn add_as_rustc_feature_flags<'c, 'f>(&'c mut self, flags: &'f [String]) -> &'c mut Self;

    /// Configures the presence of `--verbose` and `--quiet` flags
    fn add_loudness_args<'c, 'f>(&'c mut self, args: &'f CliArgs) -> &'c mut Self;
}

impl CommandExt for Command {
    fn add_as_env_vars<'c, 'l, P: Borrow<Path>>(
        &'c mut self,
        locations: &'l CrossLayerLocations<P>,
    ) -> &'c mut Self {
        self.env("FEL4_MANIFEST_PATH", locations.fel4_manifest_path.borrow())
            .env("FEL4_ARTIFACT_PATH", locations.fel4_artifact_path.borrow())
            .env("RUST_TARGET_PATH", locations.rust_target_path.borrow());
        self
    }

    fn add_as_rustc_feature_flags<'c, 'f>(&'c mut self, flags: &[String]) -> &mut Self {
        if flags.is_empty() {
            return self;
        }
        self.arg("--");
        for feature in flags {
            self.arg("--cfg");
            self.arg(format!("feature=\"{}\"", feature));

            // TODO - remove once libsel4-sys updates its feature-flag casing for the temporary debug shim
            self.arg("--cfg");
            self.arg(format!("feature=\"{}\"", feature.to_shouty_snake_case()));
        }
        self
    }

    fn add_loudness_args<'c, 'f>(&'c mut self, args: &CliArgs) -> &mut Self {
        if args.flag_quiet {
            self.arg("--quiet");
        }
        if args.flag_verbose {
            self.arg("--verbose");
        }
        self
    }
}

fn merge_feature_flags_with_rustflags_env_var(feature_flags: &[String]) -> String {
    let mut output: String = match env::var("RUSTFLAGS") {
        Ok(s) => s.into(),
        Err(env::VarError::NotUnicode(_)) => String::new(),
        Err(env::VarError::NotPresent) => String::new(),
    };
    if !output.is_empty() {
        output.push(' ');
    }
    for feature in feature_flags {
        output.push_str("--cfg ");
        output.push_str(&format!("feature=\"{}\" ", feature));

        // TODO - remove once libsel4-sys updates its feature-flag casing for the temporary debug shim
        output.push_str("--cfg ");
        output.push_str(&format!("feature=\"{}\" ", feature.to_shouty_snake_case()));
    }
    output
}
