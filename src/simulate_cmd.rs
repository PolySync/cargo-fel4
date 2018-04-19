extern crate cargo_metadata;
extern crate toml;

use std::path::Path;
use std::process::Command;

use common;
use common::Config;

/// TODO
pub fn handle_simulate_cmd(config: &Config) {
    let simulation_script_path = Path::new(&config.md.workspace_root)
        .join("images")
        .join("simulate");

    if !simulation_script_path.exists() {
        common::fail("something went wrong with the build, cannot find the simulation script");
    }

    let mut cmd = Command::new(simulation_script_path);

    common::run_cmd(cmd.current_dir(&config.md.workspace_root));
}
