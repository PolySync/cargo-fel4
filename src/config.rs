use std::path::{Path, PathBuf};

use cargo_metadata;
use fel4_config::{
    get_fel4_config, get_full_manifest, BuildProfile as ConfigBuildProfile, Fel4Config,
    FullFel4Manifest, SupportedTarget,
};
use structopt::StructOpt;

use super::Error;

#[derive(Debug, Clone, StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum CargoFel4Cli {
    #[structopt(
        name = "fel4", about = "A cargo subcommand for automating feL4 (seL4 for Rust) development"
    )]
    Fel4SubCmd(Fel4SubCmd),
}

#[derive(Debug, Clone, StructOpt)]
pub enum Fel4SubCmd {
    #[structopt(name = "build", about = "Build a feL4 project")]
    BuildCmd(BuildCmd),
    #[structopt(name = "simulate", about = "Simulate a feL4 project with QEMU")]
    SimulateCmd(SimulateCmd),
    #[structopt(name = "deploy", about = "Deploy a feL4 project")]
    DeployCmd(DeployCmd),
    #[structopt(name = "new", about = "Create a new feL4 project")]
    NewCmd(NewCmd),
    #[structopt(name = "test", about = "Build and run feL4 tests")]
    TestCmd(TestCmd),
    #[structopt(name = "clean", about = "Remove generated artifacts")]
    CleanCmd(CleanCmd),
}
#[derive(Debug, Clone, StructOpt)]
pub struct LoudnessOpts {
    #[structopt(name = "verbose", long = "verbose", short = "v", help = "Use verbose output")]
    pub verbose: bool,
    #[structopt(name = "quiet", long = "quiet", short = "q", help = "No output printed to stdout")]
    pub quiet: bool,
}

#[derive(Debug, Clone, StructOpt)]
pub struct BuildCmd {
    #[structopt(flatten)]
    pub loudness: LoudnessOpts,
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
    #[structopt(flatten)]
    pub loudness: LoudnessOpts,
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
pub struct DeployCmd {
    #[structopt(flatten)]
    pub loudness: LoudnessOpts,
    #[structopt(name = "release", long = "release", help = "Deploy release artifacts")]
    pub release: bool,
    #[structopt(name = "tests", long = "tests", help = "Deploy test artifacts")]
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
    #[structopt(flatten)]
    pub loudness: LoudnessOpts,
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
    #[structopt(flatten)]
    pub loudness: LoudnessOpts,
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
    #[structopt(name = "build", about = "Build the feL4 test application")]
    Build,
    #[structopt(name = "simulate", about = "Simulate the feL4 test application")]
    Simulate,
    #[structopt(name = "deploy", about = "Deploy the feL4 test application")]
    Deploy,
}

#[derive(Debug, Clone, StructOpt)]
pub struct CleanCmd {
    #[structopt(flatten)]
    pub loudness: LoudnessOpts,
    #[structopt(
        name = "cargo-manifest-path",
        long = "manifest-path",
        parse(from_os_str),
        default_value = "./Cargo.toml",
        help = "Path to the Cargo.toml manifest of the fel4 project"
    )]
    pub cargo_manifest_path: PathBuf,
}

impl<'a> From<&'a BuildCmd> for Fel4BuildProfile {
    fn from(c: &'a BuildCmd) -> Self {
        build_flags_to_profile(c.release, c.tests)
    }
}

impl<'a> From<&'a SimulateCmd> for Fel4BuildProfile {
    fn from(c: &'a SimulateCmd) -> Self {
        build_flags_to_profile(c.release, c.tests)
    }
}

impl<'a> From<&'a DeployCmd> for Fel4BuildProfile {
    fn from(c: &'a DeployCmd) -> Self {
        build_flags_to_profile(c.release, c.tests)
    }
}

impl<'a> From<&'a TestCmd> for Fel4BuildProfile {
    fn from(c: &'a TestCmd) -> Self {
        build_flags_to_profile(c.release, true)
    }
}

fn build_flags_to_profile(is_release: bool, is_test: bool) -> Fel4BuildProfile {
    match (is_release, is_test) {
        (true, true) => Fel4BuildProfile::TestRelease,
        (true, false) => Fel4BuildProfile::Release,
        (false, true) => Fel4BuildProfile::TestDebug,
        (false, false) => Fel4BuildProfile::Debug,
    }
}

/// Wraps a fully resolved Fel4Config instance, along with summarized package
/// metadata
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub root_dir: PathBuf,
    /// The end user application's package name
    pub pkg_name: String,
    /// The module name of the user application's package
    pub pkg_module_name: String,
    pub arch: Arch,
    pub fel4_config: Fel4Config,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Arch {
    X86,
    X86_64,
    Armv7,
    Aarch64,
}

impl<'a> From<&'a SupportedTarget> for Arch {
    fn from(target: &'a SupportedTarget) -> Self {
        match *target {
            SupportedTarget::X8664Sel4Fel4 => Arch::X86_64,
            SupportedTarget::Armv7Sel4Fel4 => Arch::Armv7,
            SupportedTarget::Aarch64Sel4Fel4 => Arch::Aarch64,
        }
    }
}

/// We support building and simulating four different profiles:
/// - debug
/// - release
/// - test-debug
/// - test-release
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Fel4BuildProfile {
    Debug,
    Release,
    TestDebug,
    TestRelease,
}

impl Fel4BuildProfile {
    pub fn artifact_subdir_path(&self) -> PathBuf {
        match *self {
            Fel4BuildProfile::Debug => PathBuf::from("debug"),
            Fel4BuildProfile::Release => PathBuf::from("release"),
            Fel4BuildProfile::TestDebug => PathBuf::from("test").join("debug"),
            Fel4BuildProfile::TestRelease => PathBuf::from("test").join("release"),
        }
    }

    pub fn as_fel4_config_build_profile(&self) -> ConfigBuildProfile {
        match *self {
            Fel4BuildProfile::Debug => ConfigBuildProfile::Debug,
            Fel4BuildProfile::Release => ConfigBuildProfile::Release,
            Fel4BuildProfile::TestDebug => ConfigBuildProfile::Debug,
            Fel4BuildProfile::TestRelease => ConfigBuildProfile::Release,
        }
    }
}

pub struct ManifestWithRootDir {
    pub fel4_manifest: FullFel4Manifest,
    pub root_dir: PathBuf,
}

pub fn get_fel4_manifest_with_root_dir<P: AsRef<Path>>(
    cargo_manifest_file_path: P,
) -> Result<ManifestWithRootDir, Error> {
    let fel4_manifest = get_fel4_manifest(cargo_manifest_file_path.as_ref())?;
    let root_dir = {
        let mut p = cargo_manifest_file_path.as_ref().to_path_buf();
        p.pop();
        p
    };
    Ok(ManifestWithRootDir {
        fel4_manifest,
        root_dir,
    })
}

pub fn get_fel4_manifest<P: AsRef<Path>>(
    cargo_manifest_file_path: P,
) -> Result<FullFel4Manifest, Error> {
    get_full_manifest(fel4_manifest_path_from_cargo_manifest_path(
        cargo_manifest_file_path,
    )).map_err(|ce| Error::ConfigError(format!("{}", ce)))
}

fn fel4_manifest_path_from_cargo_manifest_path<P: AsRef<Path>>(
    cargo_manifest_file_path: P,
) -> PathBuf {
    let mut p = cargo_manifest_file_path.as_ref().to_path_buf();
    p.pop();
    p.join("fel4.toml")
}

pub fn get_resolved_config<P: AsRef<Path>>(
    cargo_manifest_path: P,
    build_profile: &Fel4BuildProfile,
) -> Result<ResolvedConfig, Error> {
    let (pkg_name, pkg_module_name, root_dir) = {
        let metadata = cargo_metadata::metadata(Some(cargo_manifest_path.as_ref()))?;
        if metadata.packages.len() > 1 {
            return Err(Error::ConfigError(String::from(
                "a fel4 build currently requires a singular top-level package",
            )));
        };

        if let Some(pkg) = metadata.packages.first() {
            let root_dir = {
                let mut p = PathBuf::from(&pkg.manifest_path);
                p.pop();
                p
            };
            (pkg.name.clone(), pkg.name.replace("-", "_"), root_dir)
        } else {
            return Err(Error::ConfigError(String::from(
                "a fel4 build currently requires a singular top-level package",
            )));
        }
    };
    let fel4_config: Fel4Config = get_fel4_config(
        root_dir.join("fel4.toml"),
        &build_profile.as_fel4_config_build_profile(),
    ).map_err(|e| Error::ConfigError(format!("{}", e)))?;
    let arch = Arch::from(&fel4_config.target);
    Ok(ResolvedConfig {
        root_dir,
        pkg_name,
        pkg_module_name,
        arch,
        fel4_config,
    })
}
