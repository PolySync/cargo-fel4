use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use cargo_metadata;
use docopt::Docopt;
use log;
use log::LevelFilter;
use toml::Value;

use super::Error;

const USAGE: &str = "
Build, manage and simulate Helios feL4 system images

Usage:
    cargo fel4 [options] [build | simulate | deploy | info] [<path>]

Options:
    -h, --help                   Print this message
    --release                    Build artifacts in release mode, with optimizations
    --target TRIPLE              Build for the target triple
    --platform PLAT              Build for the target platform (used for deployment configuration)
    -v, --verbose                Use verbose output (-vv very verbose/build.rs output)
    -q, --quiet                  No output printed to stdout

This cargo subcommand handles the process of building and managing Helios
system images.

Run `cargo fel4 build` from the top-level system directory.

Resulting in:
└── images
    └── feL4img
    └── kernel

Run `cargo fel4 simulate` to simulate a system image with QEMU.

Run `cargo fel4 info` to produce a human readable description of the system.

Run `cargo fel4 deploy` to deploy the system image to a given platform.
";

#[derive(Debug, Clone, Deserialize)]
pub struct CliArgs {
    pub flag_verbose: bool,
    pub flag_quiet: bool,
    pub flag_release: bool,
    pub flag_target: String,
    pub flag_platform: String,
    pub cmd_build: bool,
    pub cmd_simulate: bool,
    pub cmd_deploy: bool,
    pub cmd_info: bool,
    pub arg_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum SubCommand {
    Missing,
    Build,
    Simulate,
    Deploy,
    Info,
}

impl SubCommand {
    /// Converts subcommands to an enum we can match on.
    pub fn from_cli_args(args: &CliArgs) -> Result<Self, Error> {
        // fold over the product of the enum variant and its presence in the
        // cli args.  If it's present, set it as the return type, and increment
        // our "found" counter.  If it isn't leave the initial state as it was.
        let out = vec![
            (SubCommand::Build, args.cmd_build),
            (SubCommand::Simulate, args.cmd_simulate),
            (SubCommand::Deploy, args.cmd_deploy),
            (SubCommand::Info, args.cmd_info),
        ].iter()
            .fold((SubCommand::Missing, 0), |state, cmd| {
                if cmd.1 {
                    (cmd.0.clone(), state.1 + 1)
                } else {
                    state
                }
            });
        // If we found more than one, something is broken.
        if out.1 > 1 {
            return Err(Error::ConfigError(String::from(
                "more than one subcommand was provided",
            )));
        }
        Ok(out.0)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fel4Metadata {
    #[serde(rename = "artifact-path")]
    pub artifact_path: PathBuf,
    #[serde(rename = "target-specs-path")]
    pub target_specs_path: PathBuf,
    #[serde(rename = "default-target")]
    pub default_target: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub cli_args: CliArgs,
    pub subcommand: SubCommand,
    pub root_dir: PathBuf,
    pub pkg_name: String,
    pub target: String,
    pub arch: Arch,
    pub fel4_metadata: Fel4Metadata,
}

#[derive(Debug, Clone)]
pub enum Arch {
    X86,
    Arm,
}

impl Arch {
    fn from_target_str(target: &str) -> Result<Self, Error> {
        if target.contains("x86_64") {
            return Ok(Arch::X86);
        }
        if target.contains("arm") {
            return Ok(Arch::Arm);
        }
        Err(Error::ConfigError(format!(
            "could not derive architecture from target str: {}",
            target
        )))
    }
}

pub fn gather() -> Result<Config, Error> {
    let cli_args: CliArgs = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if cli_args.flag_verbose {
        log::set_max_level(LevelFilter::Info);
    } else {
        log::set_max_level(LevelFilter::Error);
    }

    let (pkg_name, root_dir) = {
        let metadata = cargo_metadata::metadata(None)?;
        if metadata.packages.len() != 1 {
            return Err(Error::ConfigError(String::from(
                "a fel4 build requires a singular top-level package",
            )));
        };
        let mut mani_path =
            Path::new(&metadata.packages[0].manifest_path).to_path_buf();
        mani_path.pop();
        let pkg = match metadata.packages.get(0) {
            Some(p) => p,
            None => {
                return Err(Error::ConfigError(String::from(
                    "couldn't retrieve package details",
                )))
            }
        };
        (pkg.name.clone(), mani_path)
    };

    let fel4_metadata: Fel4Metadata = {
        let mut fel4_conf_path = root_dir.join("fel4.toml");
        let mut fel4_conf_file = File::open(fel4_conf_path.as_path())?;
        let mut contents = String::new();
        fel4_conf_file.read_to_string(&mut contents)?;
        let fel4_conf_toml = contents.parse::<Value>()?;
        let fel4_table = match fel4_conf_toml.get("fel4") {
            Some(f) => f,
            None => {
                return Err(Error::ConfigError(String::from(
                    "fel4.toml file is missing fel4 section",
                )))
            }
        };
        fel4_table.clone().try_into()?
    };

    let target = if cli_args.flag_target.is_empty() {
        fel4_metadata.default_target.clone()
    } else {
        cli_args.flag_target.clone()
    };
    let arch = Arch::from_target_str(&target)?;
    let subcommand = SubCommand::from_cli_args(&cli_args)?;

    Ok(Config {
        cli_args,
        subcommand,
        root_dir,
        pkg_name,
        arch,
        target,
        fel4_metadata,
    })
}
