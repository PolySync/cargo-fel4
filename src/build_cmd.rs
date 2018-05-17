extern crate cargo_metadata;

use cmake_config::{Key, SimpleFlag};
use fel4_config::{BuildProfile, FlatTomlValue, SupportedTarget};
use log;
use log::LevelFilter;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::env::{self, current_dir};
use std::ffi::OsStr;
use std::fs::{self, canonicalize, File};
use std::path::Path;
use std::process::Command;

use super::{gather_config, run_cmd, Error};
use cmake_codegen::{cache_to_interesting_flags, truthy_boolean_flags_as_rust_identifiers};
use config::{BuildCmd, Config};
use generator::Generator;

pub fn handle_build_cmd(subcmd: &BuildCmd) -> Result<(), Error> {
    if subcmd.verbose {
        log::set_max_level(LevelFilter::Info);
    } else {
        log::set_max_level(LevelFilter::Error);
    }

    let build_profile = if subcmd.release {
        BuildProfile::Release
    } else {
        BuildProfile::Debug
    };

    let config: Config = gather_config(&subcmd.cargo_manifest_path, &build_profile)?;
    let artifact_path = &config.root_dir.join(&config.fel4_config.artifact_path);

    let target_build_cache_path = config
        .root_dir
        .join("target")
        .join(config.fel4_config.target.full_name())
        .join(&build_profile.full_name());

    info!("\ntarget build cache: {:?}", target_build_cache_path,);

    let cross_layer_locations = CrossLayerLocations {
        fel4_artifact_path: config.root_dir.join(&artifact_path),
        fel4_manifest_path: config.root_dir.join("fel4.toml"),
        rust_target_path: config.root_dir.join(&config.fel4_config.target_specs_path),
    };

    let fel4_flags: Vec<SimpleFlag> = config
        .fel4_config
        .properties
        .iter()
        .map(|(k, v): (&String, &FlatTomlValue)| {
            let key = Key(k.to_string());
            match v {
                FlatTomlValue::Boolean(b) => SimpleFlag::Boolish(key, *b),
                FlatTomlValue::String(s) => SimpleFlag::Stringish(key, s.to_string()),
                FlatTomlValue::Integer(s) => SimpleFlag::Stringish(key, s.to_string()),
                FlatTomlValue::Float(s) => SimpleFlag::Stringish(key, s.to_string()),
                FlatTomlValue::Datetime(s) => SimpleFlag::Stringish(key, s.to_string()),
            }
        })
        .collect();
    let rustflags_env_var = merge_feature_flags_with_rustflags_env_var(
        &truthy_boolean_flags_as_rust_identifiers(&fel4_flags)?,
    );

    // Generate the source code entry point (root task) for the application
    // that will wrap the end-user's code as executing within a sub-thread
    let root_task_path = config.root_dir.join("src").join("bin");
    fs::create_dir_all(&root_task_path)?;
    let mut root_file = File::create(root_task_path.join("root-task.rs").as_path())?;
    Generator::new(&mut root_file, &config, &fel4_flags).generate()?;

    if canonicalize(&config.root_dir)? != canonicalize(current_dir()?)? {
        return Err(Error::ExitStatusError("The build command does not work with a cargo manifest directory that differs from the current working directory due to limitations of Xargo".to_string()));
    }
    // Build the generated root task binary
    run_cmd(
        &mut construct_root_task_build_command(subcmd, &config, &cross_layer_locations)
            .env("RUSTFLAGS", &rustflags_env_var),
    )?;

    let sysimg_path = artifact_path.join("feL4img");
    let kernel_path = artifact_path.join("kernel");

    if !artifact_path.exists() {
        fs::create_dir_all(&artifact_path)?;
    }

    // For ARM targets, we currently take advantage of the
    // seL4 elfloader-tool to bootstrap the system and kick
    // things off.
    // To accomplish this, we just re-build libsel4-sys
    // with an extra environment variable which gives
    // elfloader-tool a path to the root-task binary
    match &config.fel4_config.target {
        &SupportedTarget::ArmSel4Fel4 => {
            run_cmd(
                construct_libsel4_build_command(subcmd, &config, &cross_layer_locations)
                    .env(
                        "FEL4_ROOT_TASK_IMAGE_PATH",
                        target_build_cache_path.join("root-task"),
                    )
                    .env("RUSTFLAGS", &rustflags_env_var),
            )?;

            // seL4 CMake rules will just output everything to `kernel`
            // we copy it so it's consistent with our image name but
            // won't trigger a rebuild (as it would if we were to move it)
            fs::copy(&kernel_path, &sysimg_path)?;
        }
        _ => {
            fs::copy(target_build_cache_path.join("root-task"), &sysimg_path)?;
        }
    }

    {
        // Extract the resolved CMake config details and filter down to ones that might
        // be useful for cross-reference with the fel4-config derived values
        let interesting_raw_flags_from_cmake = cache_to_interesting_flags(
            config.root_dir.join(&artifact_path).join("CMakeCache.txt"),
        )?;
        let simple_cmake_flags: HashSet<SimpleFlag> = interesting_raw_flags_from_cmake
            .iter()
            .map(SimpleFlag::from)
            .collect();
        let simple_fel4_flags: HashSet<SimpleFlag> = fel4_flags.into_iter().collect();
        if !&simple_fel4_flags.is_subset(&simple_cmake_flags) {
            for s in &simple_fel4_flags {
                if simple_cmake_flags.contains(s) {
                    continue;
                }
                println!("Found a fel4 flag {:?} that was not in the cmake flags", s);
                let key = match s {
                    SimpleFlag::Boolish(Key(k), _) | SimpleFlag::Stringish(Key(k), _) => k.clone(),
                };
                for raw_flag in &interesting_raw_flags_from_cmake {
                    if raw_flag.key == key {
                        println!(
                            "    But there was a flag with the same key in CMakeCache.txt: {:?}",
                            raw_flag
                        );
                    }
                }
            }
            return Err(Error::ConfigError("Unexpected mismatch between the fel4.toml config values and seL4's CMakeCache.txt config values".to_string()));
        }
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

    info!("output artifact path '{}'", artifact_path.display());

    info!("kernel: '{}'", kernel_path.display());
    info!("feL4img: '{}'", sysimg_path.display());

    Ok(())
}

fn construct_libsel4_build_command<P>(
    subcmd: &BuildCmd,
    config: &Config,
    locations: &CrossLayerLocations<P>,
) -> Command
where
    P: Borrow<Path>,
{
    let mut libsel4_build = Command::new("xargo");

    libsel4_build
        .arg("rustc")
        .arg("--manifest-path")
        .arg(&subcmd.cargo_manifest_path)
        .arg_if(|| subcmd.release, "--release")
        .add_loudness_args(&subcmd)
        .handle_arm_edge_case(&config.fel4_config.target)
        .add_locations_as_env_vars(locations)
        .arg("--target")
        .arg(&config.fel4_config.target.full_name())
        .arg("-p")
        .arg("libsel4-sys");

    libsel4_build
}

/// Create a Command instance that, when run,
/// will build the root task binary
///
/// Note: Does NOT include application of Rust/Cargo feature flags
///
/// TODO: Replace our optional dependency usage with proper
/// test feature flagging when custom test frameworks are
/// more feasible in our environment
fn construct_root_task_build_command<P>(
    subcmd: &BuildCmd,
    config: &Config,
    cross_layer_locations: &CrossLayerLocations<P>,
) -> Command
where
    P: Borrow<Path>,
{
    let mut root_task_build = Command::new("xargo");
    root_task_build
        .arg("rustc")
        .arg("--bin")
        .arg("root-task")
        .arg("--manifest-path")
        .arg(&subcmd.cargo_manifest_path)
        .arg_if(|| subcmd.release, "--release")
        .add_loudness_args(&subcmd)
        .handle_arm_edge_case(&config.fel4_config.target)
        .arg_if(|| subcmd.tests, "--features")
        .arg_if(|| subcmd.tests, "test")
        .arg("--target")
        .arg(&config.fel4_config.target.full_name())
        .add_locations_as_env_vars(cross_layer_locations);
    root_task_build
}

/// Common-cause struct for the path data associated with the environment
/// variables used by cargo-fel4 to communicate across package and process
/// boundaries.
#[derive(Clone, Debug, PartialEq)]
pub struct CrossLayerLocations<P: Borrow<Path>> {
    fel4_manifest_path: P,
    fel4_artifact_path: P,
    rust_target_path: P,
}

/// Extension methods for `Command` instances to supply common parameters or
/// metadata
trait CommandExt
where
    Self: Into<Command>,
{
    /// Add an argument if a predicate returns true, largely for easier chaining
    fn arg_if<'c, P, S: AsRef<OsStr>>(&'c mut self, predicate: P, arg: S) -> &'c mut Self
    where
        P: FnOnce() -> bool;

    /// Populate the command with the environment variables tracked by
    /// CrossLayerLocations
    fn add_locations_as_env_vars<'c, 'l, P: Borrow<Path>>(
        &'c mut self,
        cross_layer_locations: &'l CrossLayerLocations<P>,
    ) -> &'c mut Self;

    /// Configures the presence of `--verbose` and `--quiet` flags
    fn add_loudness_args<'c, 'f>(&'c mut self, args: &'f BuildCmd) -> &'c mut Self;

    /// Handle a possible edge case in cross-compiling for arm
    fn handle_arm_edge_case<'c, 'f>(&'c mut self, config: &'f SupportedTarget) -> &'c mut Self;
}

impl CommandExt for Command {
    fn arg_if<'c, P, S: AsRef<OsStr>>(&'c mut self, predicate: P, arg: S) -> &'c mut Self
    where
        P: FnOnce() -> bool,
    {
        if predicate() {
            self.arg(arg);
        }
        self
    }

    fn add_locations_as_env_vars<'c, 'l, P: Borrow<Path>>(
        &'c mut self,
        locations: &'l CrossLayerLocations<P>,
    ) -> &'c mut Self {
        self.env("FEL4_MANIFEST_PATH", locations.fel4_manifest_path.borrow())
            .env("FEL4_ARTIFACT_PATH", locations.fel4_artifact_path.borrow())
            .env("RUST_TARGET_PATH", locations.rust_target_path.borrow());
        self
    }

    fn add_loudness_args<'c, 'f>(&'c mut self, args: &BuildCmd) -> &mut Self {
        self.arg_if(|| args.quiet, "--quiet")
            .arg_if(|| args.verbose, "--verbose")
    }

    fn handle_arm_edge_case<'c, 'f>(&'c mut self, target: &SupportedTarget) -> &mut Self {
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
        match target {
            &SupportedTarget::ArmSel4Fel4 => {
                self.env("CC_arm-sel4-fel4", "arm-linux-gnueabihf-gcc")
            }
            _ => self,
        }
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
    }
    output
}
