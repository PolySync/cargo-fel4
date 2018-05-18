use std::path::{Path, PathBuf};

use cargo_metadata;
use fel4_config::{get_fel4_config, BuildProfile as ConfigBuildProfile, Fel4Config};

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
    #[structopt(name = "clean", about = "Remove generated artifacts")]
    CleanCmd(CleanCmd),
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
    #[structopt(
        name = "cargo-manifest-path",
        long = "manifest-path",
        parse(from_os_str),
        default_value = "./Cargo.toml",
        help = "Path to the Cargo.toml manifest of the fel4 project"
    )]
    pub cargo_manifest_path: PathBuf,
}

#[derive(Debug, Clone, StructOpt)]
pub struct SimulateCmd {
    #[structopt(name = "verbose", long = "verbose", short = "v", help = "Use verbose output")]
    pub verbose: bool,
    #[structopt(name = "quiet", long = "quiet", short = "q", help = "No output printed to stdout")]
    pub quiet: bool,
    #[structopt(name = "release", long = "release", help = "Simulate release artifacts")]
    pub release: bool,
    #[structopt(name = "tests", long = "tests", help = "Simulate test artifacts")]
    pub tests: bool,
    #[structopt(
        name = "cargo-manifest-path",
        long = "manifest-path",
        parse(from_os_str),
        default_value = "./Cargo.toml",
        help = "Path to the Cargo.toml manifest of the fel4 project"
    )]
    pub cargo_manifest_path: PathBuf,
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
    #[structopt(parse(from_os_str))]
    pub path: PathBuf,
}

#[derive(Debug, Clone, StructOpt)]
pub struct TestCmd {
    #[structopt(name = "verbose", long = "verbose", short = "v", help = "Use verbose output")]
    pub verbose: bool,
    #[structopt(name = "quiet", long = "quiet", short = "q", help = "No output printed to stdout")]
    pub quiet: bool,
    #[structopt(name = "release", long = "release", help = "Build artifacts in release mode")]
    pub release: bool,
    #[structopt(subcommand)]
    pub subcmd: TestSubCmd,
    #[structopt(
        name = "cargo-manifest-path",
        long = "manifest-path",
        parse(from_os_str),
        default_value = "./Cargo.toml",
        help = "Path to the Cargo.toml manifest of the fel4 project"
    )]
    pub cargo_manifest_path: PathBuf,
}

#[derive(Debug, Clone, StructOpt)]
pub enum TestSubCmd {
    #[structopt(name = "build", about = "Build the feL4 test suite")]
    Build,
    #[structopt(name = "simulate", about = "Simulate the feL4 test suite")]
    Simulate,
}

#[derive(Debug, Clone, StructOpt)]
pub struct CleanCmd {
    #[structopt(name = "verbose", long = "verbose", short = "v", help = "Use verbose output")]
    pub verbose: bool,
    #[structopt(name = "quiet", long = "quiet", short = "q", help = "No output printed to stdout")]
    pub quiet: bool,
}

impl Fel4SubCmd {
    /// Determine the appropriate feL4 build profile from the given subcommand.
    pub fn build_profile(&self) -> Fel4BuildProfile {
        match *self {
            Fel4SubCmd::BuildCmd(ref c) => self.build_mode_to_profile(c.release, c.tests),
            Fel4SubCmd::SimulateCmd(ref c) => self.build_mode_to_profile(c.release, c.tests),
            Fel4SubCmd::NewCmd(_) => Fel4BuildProfile::NotApplicable,
            Fel4SubCmd::TestCmd(ref c) => self.build_mode_to_profile(c.release, true),
            Fel4SubCmd::CleanCmd(_) => Fel4BuildProfile::NotApplicable,
        }
    }

    fn build_mode_to_profile(&self, is_release: bool, is_test: bool) -> Fel4BuildProfile {
        if is_test {
            if is_release {
                Fel4BuildProfile::TestRelease
            } else {
                Fel4BuildProfile::TestDebug
            }
        } else if is_release {
            Fel4BuildProfile::Release
        } else {
            Fel4BuildProfile::Debug
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub root_dir: PathBuf,
    /// The end user application's package name
    pub pkg_name: String,
    /// The module name of the user application's package
    pub pkg_module_name: String,
    pub arch: Arch,
    pub build_profile: Fel4BuildProfile,
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

/// We support building and simulating four different profiles:
/// - debug
/// - release
/// - test-debug
/// - test-release
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Fel4BuildProfile {
    NotApplicable,
    Debug,
    Release,
    TestDebug,
    TestRelease,
}

impl Fel4BuildProfile {
    pub fn artifact_subdir_path(&self) -> PathBuf {
        match *self {
            Fel4BuildProfile::NotApplicable => PathBuf::new(),
            Fel4BuildProfile::Debug => PathBuf::from("debug"),
            Fel4BuildProfile::Release => PathBuf::from("release"),
            Fel4BuildProfile::TestDebug => PathBuf::from("test").join("debug"),
            Fel4BuildProfile::TestRelease => PathBuf::from("test").join("release"),
        }
    }

    pub fn as_fel4_config_build_profile(&self) -> ConfigBuildProfile {
        match *self {
            Fel4BuildProfile::NotApplicable => ConfigBuildProfile::Debug,
            Fel4BuildProfile::Debug => ConfigBuildProfile::Debug,
            Fel4BuildProfile::Release => ConfigBuildProfile::Release,
            Fel4BuildProfile::TestDebug => ConfigBuildProfile::Debug,
            Fel4BuildProfile::TestRelease => ConfigBuildProfile::Release,
        }
    }
}

pub fn gather(cmd: &Fel4SubCmd) -> Result<Config, Error> {
    let cargo_manifest_path = match cmd {
        Fel4SubCmd::BuildCmd(c) => &c.cargo_manifest_path,
        Fel4SubCmd::SimulateCmd(c) => &c.cargo_manifest_path,
        Fel4SubCmd::TestCmd(c) => &c.cargo_manifest_path,
        Fel4SubCmd::NewCmd(_) => Path::new("./Cargo.toml"),
        Fel4SubCmd::CleanCmd(_) => Path::new("./Cargo.toml"),
    };

    let (pkg_name, pkg_module_name, root_dir) = {
        let metadata = cargo_metadata::metadata(Some(cargo_manifest_path))?;
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

    let build_profile = cmd.build_profile();

    let fel4_config: Fel4Config = match get_fel4_config(
        root_dir.join("fel4.toml"),
        &build_profile.as_fel4_config_build_profile(),
    ) {
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
        build_profile,
        fel4_config,
    })
}
