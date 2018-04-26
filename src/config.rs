use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use cargo_metadata;
use cargo_metadata::Package;
use docopt::Docopt;
use toml::Value;

use common::Error;

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
    pub target_dir: PathBuf,
    pub pkg_metadata: Package,
    pub fel4_metadata: Fel4Metadata,
}

pub fn gather() -> Result<Config, Error> {
    let cli_args: CliArgs = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let (pkg_metadata, target_dir) = {
        let metadata = cargo_metadata::metadata(None)?;
        if metadata.packages.len() != 1 {
            return Err(Error::ConfigError(
                "a fel4 build requires a singular top-level package",
            ));
        };
        (
            metadata.packages[0].clone(),
            PathBuf::from(metadata.target_directory),
        )
    };

    let fel4_metadata = {
        let mut fel4_conf_path = PathBuf::from(&pkg_metadata.manifest_path);
        fel4_conf_path.set_file_name("fel4.toml");
        println!("fel4 path: {:?}", fel4_conf_path);
        let mut fel4_conf_file = File::open(fel4_conf_path.as_path())?;
        let mut contents = String::new();
        fel4_conf_file.read_to_string(&mut contents)?;
        let fel4_conf_toml = contents.parse::<Value>()?;
        fel4_conf_toml.try_into()?
    };

    Ok(Config {
        cli_args,
        target_dir,
        pkg_metadata,
        fel4_metadata,
    })
}
