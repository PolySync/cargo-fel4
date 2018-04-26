extern crate cargo_metadata;
extern crate colored;
extern crate log;
extern crate toml;

use colored::Colorize;
use std::fmt;
use std::io;
use std::process::Command;
use toml::Value;

pub enum Error {
    ConfigError(&'static str),
    IO(String),
    MetadataError(String),
    TomlSerError(String),
    TomlDeError(String),
    ExitStatusError(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(format!("{}", e))
    }
}

impl From<cargo_metadata::Error> for Error {
    fn from(e: cargo_metadata::Error) -> Self {
        Error::MetadataError(format!("{}", e))
    }
}

impl From<toml::ser::Error> for Error {
    fn from(e: toml::ser::Error) -> Self {
        Error::TomlSerError(format!("{}", e))
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::TomlDeError(format!("{}", e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MetadataError(msg) => write!(
                f,
                "[toml metadata error] {}\n\ncheck your project's Cargo.toml [metadata] table(s)",
                msg
            ),
            Error::IO(msg) => write!(f, "IO error: {}", msg),
            Error::MetadataError(msg) => {
                write!(f, "[cargo metadata error] {}\n\ncheck your project's Cargo.toml for invalid syntax", msg)
            }
            Error::TomlSerError(msg) => {
                write!(f, "[toml serialize error] {}\n\ncheck your project's Cargo.toml [metadata.helios] table", msg)
            }
            Error::TomlDeError(msg) => {
                write!(f, "[toml deserialize error] {}\n\ncheck your project's Cargo.toml [metadata.helios] table", msg)
            }
            Error::ExitStatusError(msg) => {
                write!(f, "[command execution error] {}\n\ntry running with verbose flag for more information", msg)
            }
            Error::ExitStatusError(msg) => write!(f, "command error: {}", msg),
            Error::ConfigError(msg) => write!(f, "config error: {}", msg),
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

pub trait DeepLookup {
    /// `lookup` takes a namespace in dot-separated format: `"a.b.c" and uses
    /// it to traverse a trie-like structure.
    fn lookup(&self, ns: &str) -> Result<&Self, Error>;
}

impl DeepLookup for Value {
    fn lookup(&self, ns: &str) -> Result<&Self, Error> {
        let ns_iter = ns.split('.');
        ns_iter.fold(Ok(self), |state, key| match state {
            Ok(v) => match v {
                Value::Table(t) => match t.get(key) {
                    Some(next) => Ok(next),
                    None => Err(Error::MetadataError(format!(
                        "failed to lookup toml key '{}'",
                        ns
                    ))),
                },
                _ => Err(Error::MetadataError(format!(
                    "failed to lookup toml key '{}'",
                    ns
                ))),
            },
            Err(e) => Err(e),
        })
    }
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
