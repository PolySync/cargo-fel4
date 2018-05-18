use log;
use log::LevelFilter;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::{gather_config, run_cmd, Error};
use config::{CleanCmd, Config, Fel4SubCmd};

pub fn handle_clean_cmd(clean_cmd: &CleanCmd) -> Result<(), Error> {
    if clean_cmd.verbose {
        log::set_max_level(LevelFilter::Info);
    } else {
        log::set_max_level(LevelFilter::Error);
    }

    let config: Config = gather_config(&Fel4SubCmd::CleanCmd(clean_cmd.clone()))?;

    let artifact_path = Path::new(&config.root_dir).join(config.fel4_config.artifact_path);

    clean_cargo_build_cache(clean_cmd)?;

    if artifact_path.exists() {
        if clean_cmd.verbose {
            info!("Removing {}", artifact_path.display());
        }

        fs::remove_dir_all(&artifact_path)?;
    }

    Ok(())
}

fn clean_cargo_build_cache(clean_cmd: &CleanCmd) -> Result<(), Error> {
    let mut cmd = Command::new("cargo");

    if clean_cmd.verbose {
        cmd.arg("--verbose");
    } else if clean_cmd.quiet {
        cmd.arg("--quiet");
    }

    cmd.arg("clean");

    run_cmd(&mut cmd)?;

    Ok(())
}
