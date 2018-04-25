extern crate cargo_metadata;
extern crate toml;

use std::path::PathBuf;
use std::process::Command;

use common;
use common::Config;

/// TODO - need to use configs
pub fn handle_simulate_cmd(config: &Config) {
    let sim_script_rel_path: PathBuf = ["images", "simulate"].iter().collect();

    let workspace_root = PathBuf::from(&config.md.workspace_root);

    if !workspace_root
        .join(&sim_script_rel_path)
        .exists()
    {
        common::fail("something went wrong with the build, cannot find the simulation script");
    }

    common::run_cmd(
        Command::new(&sim_script_rel_path).current_dir(workspace_root),
    );
}
