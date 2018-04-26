#[macro_use]
extern crate serde_derive;
extern crate cargo_metadata;
extern crate docopt;
extern crate toml;
#[macro_use]
extern crate log;
extern crate colored;

use build_cmd::handle_build_cmd;
use docopt::Docopt;
use log::LevelFilter;
use simulate_cmd::handle_simulate_cmd;

mod build_cmd;
mod common;
mod config;
mod simulate_cmd;
mod generator;

use config::Config;

fn main() {
    let config: Config = match config::gather() {
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
