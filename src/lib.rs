#[macro_use]
extern crate serde_derive;
extern crate cargo_metadata;
extern crate colored;
#[macro_use]
extern crate log;
extern crate docopt;
extern crate toml;

use colored::Colorize;
use std::fmt;
use std::io;
use std::process::Command;

mod build_cmd;
mod config;
mod generator;
mod simulate_cmd;

pub use build_cmd::handle_build_cmd;
pub use config::{gather as gather_config, Config, SubCommand};
pub use simulate_cmd::handle_simulate_cmd;

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

impl From<toml::ser::Error> for Error {
    fn from(e: toml::ser::Error) -> Self {
        Error::ConfigError(format!("{}", e))
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::ConfigError(format!("{}", e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(msg) => write!(f, "[IO error] {}", msg),
            Error::ExitStatusError(msg) => write!(f, "[command error] {}", msg),
            Error::ConfigError(msg) => write!(f, "[config error] {}\ncheck your project's toml files for invalid syntax", msg)
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
