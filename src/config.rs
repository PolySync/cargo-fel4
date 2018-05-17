use std::path::{Path, PathBuf};

use cargo_metadata;
use fel4_config::{get_fel4_config, BuildProfile, Fel4Config};

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
    #[structopt(name = "test", about = "Build and run feL4 tests")]
    TestCmd(TestCmd),
}

#[derive(Debug, Clone, StructOpt)]
pub struct BuildCmd {
    #[structopt(name = "verbose", long = "verbose", short = "v", help = "Use verbose output")]
    pub verbose: bool,
    #[structopt(name = "quiet", long = "quiet", short = "q", help = "No output printed to stdout")]
    pub quiet: bool,
    #[structopt(name = "release", long = "release", help = "Build artifacts in release mode")]
    pub release: bool,
    #[structopt(name = "tests", long = "tests", help = "Build with feL4 test features enabled")]
    pub tests: bool,
}

#[derive(Debug, Clone, StructOpt)]
pub struct SimulateCmd {
    #[structopt(name = "verbose", long = "verbose", short = "v", help = "Use verbose output")]
    pub verbose: bool,
    #[structopt(name = "quiet", long = "quiet", short = "q", help = "No output printed to stdout")]
    pub quiet: bool,
    #[structopt(name = "release", long = "release", help = "Use artifacts built in release mode")]
    pub release: bool,
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

#[derive(Debug, Clone, StructOpt)]
pub struct TestCmd {
    #[structopt(name = "verbose", long = "verbose", short = "v", help = "Use verbose output")]
    pub verbose: bool,
    #[structopt(name = "quiet", long = "quiet", short = "q", help = "No output printed to stdout")]
    pub quiet: bool,
    #[structopt(subcommand)]
    pub subcmd: Option<TestSubCmd>,
}

#[derive(Debug, Clone, StructOpt)]
pub enum TestSubCmd {
    #[structopt(name = "build", about = "Build the feL4 test suite")]
    Build,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub root_dir: PathBuf,
    /// The end user application's package name
    pub pkg_name: String,
    pub pkg_module_name: String,
    pub arch: Arch,
    pub fel4_config: Fel4Config,
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

pub fn gather(build_profile: &BuildProfile) -> Result<Config, Error> {
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

    let fel4_config: Fel4Config = match get_fel4_config(root_dir.join("fel4.toml"), build_profile) {
        Ok(f) => f,
        Err(e) => return Err(Error::ConfigError(format!("{}", e))),
    };

    // TODO - skip the trip through strings!
    let arch = Arch::from_target_str(fel4_config.target.full_name())?;

    Ok(Config {
        root_dir,
        pkg_name,
        pkg_module_name,
        arch,
        fel4_config,
    })
}
