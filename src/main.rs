#[macro_use]
extern crate serde_derive;
extern crate cargo_metadata;
extern crate docopt;
extern crate toml;
#[macro_use]
extern crate log;
extern crate colored;

mod build_cmd;
mod common;
mod config;
mod generator;
mod simulate_cmd;

use build_cmd::handle_build_cmd;
use common::Logger;
use config::Config;
use simulate_cmd::handle_simulate_cmd;

static LOGGER: Logger = Logger;

fn main() {
    if let Err(e) = log::set_logger(&LOGGER) {
        error!(
            "there was an error initializing the logger:\n{}",
            e
        );

        return;
    };
    let config: Config = match config::gather() {
        Ok(c) => c,
        Err(e) => {
            error!("failed to parse configuration\n{}", e);
            return;
        }
    };

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
