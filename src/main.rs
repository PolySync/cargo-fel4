extern crate structopt;
#[macro_use]
extern crate log;
extern crate cargo_fel4;

use cargo_fel4::{CargoFel4Cli, Fel4SubCmd, Logger, LoudnessOpts};
use log::LevelFilter;
use structopt::StructOpt;

static LOGGER: Logger = Logger;

fn main() {
    if let Err(e) = log::set_logger(&LOGGER) {
        error!("There was an error initializing the logger:\n{}", e);
        return;
    };
    let CargoFel4Cli::Fel4SubCmd(subcmd) = CargoFel4Cli::from_args();

    match subcmd {
        Fel4SubCmd::BuildCmd(c) => {
            set_logging_level(&c.loudness);
            if let Err(e) = cargo_fel4::handle_build_cmd(&c) {
                error!("Failed to run the build command\n{}", e)
            }
        }
        Fel4SubCmd::SimulateCmd(c) => {
            set_logging_level(&c.loudness);
            if let Err(e) = cargo_fel4::handle_simulate_cmd(&c) {
                error!("Failed to run the simulation command\n{}", e)
            }
        }
        Fel4SubCmd::NewCmd(c) => {
            set_logging_level(&c.loudness);
            if let Err(e) = cargo_fel4::handle_new_cmd(&c) {
                error!("Failed to run the new command\n{}", e)
            }
        }
        Fel4SubCmd::TestCmd(c) => {
            set_logging_level(&c.loudness);
            if let Err(e) = cargo_fel4::handle_test_cmd(&c) {
                error!("Failed to run the test command\n{}", e)
            }
        }
        Fel4SubCmd::CleanCmd(c) => {
            set_logging_level(&c.loudness);
            if let Err(e) = cargo_fel4::handle_clean_cmd(&c) {
                error!("Failed to run the clean command\n{}", e)
            }
        }
    }
}

fn set_logging_level(LoudnessOpts { verbose, quiet }: &LoudnessOpts) {
    log::set_max_level(match (verbose, quiet) {
        (true, _) => LevelFilter::Info,
        (false, true) => LevelFilter::Off,
        _ => LevelFilter::Error,
    });
}
