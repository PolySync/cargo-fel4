extern crate cargo_metadata;
extern crate cmake_config;
extern crate colored;
extern crate fel4_config;
#[macro_use]
extern crate log;
#[macro_use]
extern crate structopt;

use colored::Colorize;
use std::fmt;
use std::io;
use std::process::Command;

mod build_cmd;
mod clean_cmd;
mod cmake_codegen;
mod config;
mod generator;
mod new_cmd;
mod simulate_cmd;
mod test_cmd;

pub use build_cmd::handle_build_cmd;
pub use clean_cmd::handle_clean_cmd;
pub use config::{
    gather as gather_config, BuildCmd, CargoFel4Cli, Config, Fel4SubCmd, NewCmd, SimulateCmd,
    TestCmd, TestSubCmd,
};
pub use new_cmd::handle_new_cmd;
pub use simulate_cmd::handle_simulate_cmd;
pub use test_cmd::handle_test_cmd;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Error {
    ConfigError(String),
    IO(String),
    ExitStatusError(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(format!("{}", e))
    }
}

impl From<cargo_metadata::Error> for Error {
    fn from(e: cargo_metadata::Error) -> Self {
        Error::ConfigError(format!("{}", e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(msg) => write!(f, "[IO error] {}", msg),
            Error::ExitStatusError(msg) => write!(f, "[command error] {}", msg),
            Error::ConfigError(msg) => write!(
                f,
                "[config error] {}\n\nCheck your project's toml files for invalid syntax",
                msg
            ),
        }
    }
}

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    /// Error/Warn are colored red/brigh-yellow to match Cargo/rustc
    /// Info is colored bright-green
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{}: {}",
                match record.level() {
                    log::Level::Error => "error".red(),
                    log::Level::Warn => "warn".bright_yellow(),
                    log::Level::Info => "info".bright_green(),
                    l => l.to_string().to_lowercase().normal(),
                },
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

pub fn run_cmd(cmd: &mut Command) -> Result<(), Error> {
    info!("running: {:?}", cmd);
    let status = match cmd.status() {
        Ok(status) => status,
        Err(e) => {
            return Err(Error::ExitStatusError(format!(
                "failed to execute the command: {}",
                e
            )));
        }
    };

    if !status.success() {
        return Err(Error::ExitStatusError(format!(
            "command status returned: {}",
            status
        )));
    }

    Ok(())
}
