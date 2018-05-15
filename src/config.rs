use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use cargo_metadata;
use toml::Value;

use super::Error;

#[derive(Debug, Clone, StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum CargoFel4Cli {
    #[structopt(name = "fel4", about = "Build, manage and simulate feL4 system images")]
    Fel4SubCmd(Fel4SubCmd),
}

#[derive(Debug, Clone, StructOpt)]
pub enum Fel4SubCmd {
    #[structopt(name = "build", about = "Build a feL4 project")]
    BuildCmd(BuildCmd),
    #[structopt(name = "simulate", about = "Simulate a feL4 project with QEMU")]
    SimulateCmd(SimulateCmd),
    #[structopt(name = "new", about = "Create a new feL4 project")]
    NewCmd(NewCmd),
}

#[derive(Debug, Clone, StructOpt)]
pub struct BuildCmd {
    #[structopt(name = "verbose", long = "verbose", short = "v", help = "Use verbose output")]
    pub verbose: bool,
    #[structopt(name = "quiet", long = "quiet", short = "q", help = "No output printed to stdout")]
    pub quiet: bool,
    #[structopt(name = "release", long = "release", help = "Build artifacts in release mode")]
    pub release: bool,
}

#[derive(Debug, Clone, StructOpt)]
pub struct SimulateCmd {
    #[structopt(name = "verbose", long = "verbose", short = "v", help = "Use verbose output")]
    pub verbose: bool,
    #[structopt(name = "quiet", long = "quiet", short = "q", help = "No output printed to stdout")]
    pub quiet: bool,
}

#[derive(Debug, Clone, StructOpt)]
pub struct NewCmd {
    #[structopt(name = "verbose", long = "verbose", short = "v", help = "Use verbose output")]
    pub verbose: bool,
    #[structopt(name = "quiet", long = "quiet", short = "q", help = "No output printed to stdout")]
    pub quiet: bool,
    #[structopt(
        name = "name",
        long = "name",
        help = "Set the resulting package name, defaults to the directory name"
    )]
    pub name: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fel4Metadata {
    #[serde(rename = "artifact-path")]
    pub artifact_path: PathBuf,
    #[serde(rename = "target-specs-path")]
    pub target_specs_path: PathBuf,
    pub target: String,
    pub platform: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub root_dir: PathBuf,
    /// The end user application's package name
    pub pkg_name: String,
    pub pkg_module_name: String,
    pub target: String,
    pub arch: Arch,
    pub fel4_metadata: Fel4Metadata,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Arch {
    X86,
    X86_64,
    Arm,
}

impl Arch {
    fn from_target_str(target: &str) -> Result<Self, Error> {
        if target.contains("x86_64") {
            return Ok(Arch::X86_64);
        }
        if target.contains("arm") {
            return Ok(Arch::Arm);
        }
        if target.contains("i686") {
            return Ok(Arch::X86);
        }
        Err(Error::ConfigError(format!(
            "could not derive architecture from target str: {}",
            target
        )))
    }
}

pub fn gather() -> Result<Config, Error> {
    let (pkg_name, pkg_module_name, root_dir) = {
        let metadata = cargo_metadata::metadata(None)?;
        if metadata.packages.len() != 1 {
            return Err(Error::ConfigError(String::from(
                "a fel4 build requires a singular top-level package",
            )));
        };
        let mut mani_path = Path::new(&metadata.packages[0].manifest_path).to_path_buf();
        mani_path.pop();
        let pkg = match metadata.packages.get(0) {
            Some(p) => p,
            None => {
                return Err(Error::ConfigError(String::from(
                    "couldn't retrieve package details",
                )))
            }
        };
        (pkg.name.clone(), pkg.name.replace("-", "_"), mani_path)
    };

    let fel4_metadata: Fel4Metadata = {
        let mut fel4_conf_path = root_dir.join("fel4.toml");
        let mut fel4_conf_file = File::open(fel4_conf_path.as_path())?;
        let mut contents = String::new();
        fel4_conf_file.read_to_string(&mut contents)?;
        let fel4_conf_toml = contents.parse::<Value>()?;
        let fel4_table = match fel4_conf_toml.get("fel4") {
            Some(f) => f,
            None => {
                return Err(Error::ConfigError(String::from(
                    "fel4.toml file is missing fel4 section",
                )))
            }
        };
        fel4_table.clone().try_into()?
    };

    let target = fel4_metadata.target.clone();
    let arch = Arch::from_target_str(&target)?;

    Ok(Config {
        root_dir,
        pkg_name,
        pkg_module_name,
        arch,
        target,
        fel4_metadata,
    })
}
