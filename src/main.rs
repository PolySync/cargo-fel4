#[macro_use]
extern crate log;
extern crate cargo_fel4;

use cargo_fel4::{Config, Logger, SubCommand};

static LOGGER: Logger = Logger;

fn main() {
    if let Err(e) = log::set_logger(&LOGGER) {
        error!(
            "there was an error initializing the logger:\n{}",
            e
        );

        return;
    };
    let config: Config = match cargo_fel4::gather_config() {
        Ok(c) => c,
        Err(e) => {
            error!("failed to parse configuration\n{}", e);
            return;
        }
    };

    match config.subcommand {
        SubCommand::Build => {
            if let Err(e) = cargo_fel4::handle_build_cmd(&config) {
                error!("failed to run the build command\n{}", e)
            }
        }
        SubCommand::Simulate => {
            if let Err(e) = cargo_fel4::handle_simulate_cmd(&config) {
                error!("failed to run the simulation command\n{}", e)
            }
        }
        _ => error!("not implemented!"),
    }
}
