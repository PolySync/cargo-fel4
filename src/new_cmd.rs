use std::process::Command;

use super::{run_cmd, Error};
use config::Config;

pub fn handle_new_cmd(config: &Config) -> Result<(), Error> {
    // TODO
    //
    // name fel4 or feL4?
    // name/path, subcommands for context help?
    //
    // cargo new --lib

    let package_name = "fel4-project";

    // base cargo new command to construct project scaffolding
    let mut cmd = Command::new("cargo");
    cmd.arg("new");

    // passthrough to cargo
    if config.cli_args.flag_verbose {
        cmd.arg("--verbose");
    } else if config.cli_args.flag_quiet {
        cmd.arg("--quiet");
    }

    // project name and directory are the same
    cmd.arg("--name").arg(package_name);

    // build a bare library project
    run_cmd(cmd.arg("--lib").arg("fel4-project"))?;

    Ok(())
}
