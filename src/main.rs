extern crate structopt;
#[macro_use]
extern crate log;
extern crate cargo_fel4;

use cargo_fel4::{CargoFel4Cli, Fel4SubCmd, Logger};
use log::LevelFilter;
use structopt::StructOpt;

static LOGGER: Logger = Logger;

fn main() {
    if let Err(e) = log::set_logger(&LOGGER) {
        error!("there was an error initializing the logger:\n{}", e);
        return;
    };

    // subcommands can adjust as needed
    log::set_max_level(LevelFilter::Error);

    let subcmd = match CargoFel4Cli::from_args() {
        CargoFel4Cli::Fel4SubCmd(c) => c,
    };

    match subcmd {
        Fel4SubCmd::BuildCmd(c) => {
            if let Err(e) = cargo_fel4::handle_build_cmd(&c) {
                error!("failed to run the build command\n{}", e)
            }
        }
        Fel4SubCmd::SimulateCmd(c) => {
            if let Err(e) = cargo_fel4::handle_simulate_cmd(&c) {
                error!("failed to run the simulation command\n{}", e)
            }
        }
        Fel4SubCmd::NewCmd(c) => {
            if let Err(e) = cargo_fel4::handle_new_cmd(&c) {
                error!("failed to run the new command\n{}", e)
            }
        }
        // TODO - how to put a dependency on building in debug mode first?
        Fel4SubCmd::TestCmd(c) => {
            if let Err(e) = cargo_fel4::handle_test_cmd(&c) {
                error!("failed to run the test command\n{}", e)
            }
        }
    }
}
