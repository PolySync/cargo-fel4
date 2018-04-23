#[macro_use]

extern crate serde_derive;
extern crate cargo_metadata;
extern crate docopt;
extern crate toml;

use cargo_metadata::metadata_deps;
use docopt::Docopt;
use std::path::Path;

mod build_cmd;
mod common;
mod cpio;
mod simulate_cmd;

const USAGE: &str = "
Build, manage and simulate Helios feL4 system images

Usage:
    cargo fel4 [options] [build | simulate | deploy | info] [<path>]

Options:
    -h, --help                   Print this message
    --release                    Build artifacts in release mode, with optimizations
    --target TRIPLE              Build for the target triple
    --platform PLAT              Build for the target platform (used for deployment configuration)
    -v, --verbose                Use verbose output (-vv very verbose/build.rs output)
    -q, --quiet                  No output printed to stdout

This cargo subcommand handles the process of building and managing Helios
system images.

Run `cargo fel4 build` from the top-level system directory.

Resulting in:
└── images
    └── feL4img
    └── kernel

Run `cargo fel4 simulate` to simulate a system image with QEMU.

Run `cargo fel4 info` to produce a human readable description of the system.

Run `cargo fel4 deploy` to deploy the system image to a given platform.
";

fn main() {
    let margs: common::Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let md_path = margs.arg_path.as_ref().map(Path::new);

    let m_md = metadata_deps(md_path, false).unwrap();

    let m_mf = common::read_manifest(&m_md.workspace_root);

    let config = common::Config {
        args: margs.clone(),
        md: m_md,
        mf: m_mf,
    };

    println!("using workspace '{}'", config.md.workspace_root);

    if config.args.cmd_build {
        if let Err(e) = build_cmd::handle_build_cmd(&config) {
            println!("failure: {}", e)
        }
    } else if config.args.cmd_simulate {
        simulate_cmd::handle_simulate_cmd(&config);
    }
}
