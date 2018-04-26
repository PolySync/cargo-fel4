extern crate cargo_metadata;
extern crate toml;

use cargo_metadata::{metadata_deps, Metadata};
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct CliArgs {
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

#[derive(Debug, Clone, Deserialize)]
pub struct HeliosMetadata {
    #[serde(rename = "root-task")]
    pub root_task: String,
    pub apps: Vec<String>,
    #[serde(rename = "artifact-path")]
    pub artifact_path: PathBuf,
    #[serde(rename = "apps-lib-name")]
    pub apps_lib_name: String,
    #[serde(rename = "build-cmd")]
    pub build_cmd: String,
    #[serde(rename = "target-specs-path")]
    pub target_specs_path: PathBuf,
    #[serde(rename = "default-target")]
    pub default_target: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub cli_args: CliArgs,
    pub root_metadata: Metadata,
    pub root_manifest: Value,
    pub helios_metadata: HeliosMetadata,
    pub is_workspace_build: bool,
    pub uses_root_manifest_config: bool,
}

pub enum Error {
    MetadataError(&'static str),
    IO(String),
    CargoMetadataError(String),
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
        Error::CargoMetadataError(format!("{}", e))
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
            Error::MetadataError(msg) => write!(f, "metadata error: {}", msg),
            Error::IO(msg) => write!(f, "IO error: {}", msg),
            Error::CargoMetadataError(msg) => {
                write!(f, "cargo metadata error: {}", msg)
            }
            Error::TomlSerError(msg) => {
                write!(f, "toml serialize error: {}", msg)
            }
            Error::TomlDeError(msg) => {
                write!(f, "toml deserialize error: {}", msg)
            }
            Error::ExitStatusError(msg) => write!(f, "command error: {}", msg),
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

pub fn parse_config(cli_args: &CliArgs) -> Result<Config, Error> {
    let root_manifest_path = cli_args.arg_path.as_ref().map(Path::new);

    let root_manifest_metadata = metadata_deps(root_manifest_path, false)?;

    let root_manifest = read_manifest(&root_manifest_metadata.workspace_root)?;

    let is_workspace_build =
        match &root_manifest_metadata.workspace_members.len() {
            0 => {
                return Err(Error::MetadataError(
                    "metadata reports there are no packages",
                ))
            }
            1 => false,
            _ => true,
        };

    let base_key = if is_workspace_build {
        String::new()
    } else {
        String::from("package.")
    };

    let uses_root_config = match &root_manifest.lookup(&format!(
        "{}metadata.sel4-cmake-options",
        base_key
    )) {
        Ok(_) => true,
        _ => false,
    };

    let mut helios_metadata: HeliosMetadata = {
        let meta_val = match &root_manifest
            .lookup(&format!("{}metadata.helios", base_key))?
        {
            Value::Table(t) => Value::try_from(t)?,
            _ => {
                return Err(Error::MetadataError(
                    "metadata table is malformed",
                ));
            }
        };
        meta_val.try_into()?
    };

    // turn the relative paths into absolute
    helios_metadata.artifact_path = PathBuf::new()
        .join(&root_manifest_metadata.workspace_root)
        .join(&helios_metadata.artifact_path);

    helios_metadata.target_specs_path = PathBuf::new()
        .join(&root_manifest_metadata.workspace_root)
        .join(helios_metadata.target_specs_path);

    Ok(Config {
        cli_args: cli_args.clone(),
        root_metadata: root_manifest_metadata,
        root_manifest,
        helios_metadata,
        is_workspace_build,
        uses_root_manifest_config: uses_root_config,
    })
}

pub fn read_manifest(path: &str) -> Result<Value, Error> {
    let manifest_path = format!("{}/Cargo.toml", path);
    let mut file = File::open(manifest_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.parse::<Value>()?)
}

pub fn run_cmd(verbose: bool, cmd: &mut Command) -> Result<(), Error> {
    if verbose {
        println!("running: {:?}", cmd);
    }

    let status = match cmd.status() {
        Ok(status) => status,
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            return Err(Error::ExitStatusError(format!(
                "failed to execute command: {}\ndoes the program exist on the system?",
                e)));
        }
        Err(e) => {
            return Err(Error::ExitStatusError(format!(
                "failed to execute command: {}",
                e
            )));
        }
    };

    if !status.success() {
        return Err(Error::ExitStatusError(format!(
            "command did not execute successfully, got: {}",
            status
        )));
    }

    Ok(())
}
