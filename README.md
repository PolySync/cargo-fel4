# Helios/Cargo System Image Subcommand Crate

- [seL4 workspace](https://bitbucket.org/PolySync/sel4-workspace/overview)

## Install

```
cargo install --git git@bitbucket.org:PolySync/cargo-fel4.git
```

## Example

```
git clone git@bitbucket.org:PolySync/sel4-workspace.git sel4-workspace

cargo fel4 build sel4-workspace/Cargo.toml

cargo fel4 simulate sel4-workspace/Cargo.toml
```

## Usage

```
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
```

## Command `build`

TODO

## Command `simulate`

TODO

## Command `deploy`

TODO

## Example System-Level Cargo.toml

The system-level Cargo.toml is a virtual manifest.

```
[metadata.helios]
root-task = "root-task"
apps = []
artifact-path = "images"
apps-lib-name = "fel4_apps"
build-cmd = "xargo"
target-specs-path = "targets"
default-target = "x86_64-sel4-helios"
```
