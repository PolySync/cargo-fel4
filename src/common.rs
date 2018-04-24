extern crate cargo_metadata;
extern crate toml;

use cargo_metadata::Metadata;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::io;
use std::process::Command;
use toml::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct Args {
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

pub struct Config {
    pub args: Args,
    pub md: Metadata,
    pub mf: Value,
}

pub enum Error {
    MetadataError(&'static str),
    IO(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IO(format!("{}", e))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MetadataError(msg) => write!(f, "metadata error: {}", msg),
            Error::IO(msg) => write!(f, "IO error: {}", msg),
        }
    }
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
                    None => Err(Error::MetadataError("couldn't find key")),
                },
                _ => Err(Error::MetadataError("couldn't find key")),
            },
            Err(e) => Err(e),
        })
    }
}

pub fn update_git_submodules(cwd: &str) {
    let mut cmd = Command::new("git");

    run_cmd(
        cmd.current_dir(cwd)
            .arg("submodule")
            .arg("update")
            .arg("--init")
            .arg("--recursive"),
    );
}

pub fn read_manifest(path: &str) -> toml::Value {
    let manifest_path = format!("{}/Cargo.toml", path);
    let mut file = File::open(manifest_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    contents.parse::<Value>().unwrap()
}

pub fn run_cmd(cmd: &mut Command) {
    println!("running: {:?}", cmd);
    let status = match cmd.status() {
        Ok(status) => status,
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            fail(&format!("failed to execute command: {}\ndoes the program exist on the system?", e));
        }
        Err(e) => fail(&format!("failed to execute command: {}", e)),
    };
    if !status.success() {
        fail(&format!(
            "command did not execute successfully, got: {}",
            status
        ));
    }
}

pub fn fail(s: &str) -> ! {
    panic!("\n{}\n\nbuild script failed, must exit now", s)
}
