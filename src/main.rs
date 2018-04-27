#[macro_use]
extern crate serde_derive;
extern crate cargo_metadata;
extern crate docopt;
extern crate toml;
#[macro_use]
extern crate log;
extern crate colored;

use build_cmd::handle_build_cmd;
use common::{parse_config, CliArgs, Config, Logger};
use docopt::Docopt;
use log::LevelFilter;
use simulate_cmd::handle_simulate_cmd;

mod build_cmd;
mod common;
mod simulate_cmd;

static LOGGER: Logger = Logger;

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

fn main() {
    if let Err(e) = log::set_logger(&LOGGER) {
        error!(
            "somehow the logger has already been initialized:\n{}",
            e
        );

        return;
    };

    let cli_args: CliArgs = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if cli_args.flag_verbose {
        log::set_max_level(LevelFilter::Info);
    } else {
        log::set_max_level(LevelFilter::Error);
    }

    let config: Config = match parse_config(&cli_args) {
        Ok(c) => c,
        Err(e) => {
            error!("failed to parse configuration\n{}", e);
            return;
        }
    };

    info!(
        "using workspace {:?}",
        config.root_metadata.workspace_root,
    );

    if config.cli_args.cmd_build {
        if let Err(e) = handle_build_cmd(&config) {
            error!("failed to run the build command\n{}", e)
        }
    } else if config.cli_args.cmd_simulate {
        if let Err(e) = handle_simulate_cmd(&config) {
            error!("failed to run the simulation command\n{}", e)
        }
    }
}
