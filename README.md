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
Build, manage and simulate feL4 system images

Usage:
    cargo fel4 [options] [build | simulate]

Options:
    -h, --help                   Print this message
    --release                    Build artifacts in release mode, with optimizations
    -v, --verbose                Use verbose output (-vv very verbose/build.rs output)
    -q, --quiet                  No output printed to stdout

This cargo subcommand handles the process of building and managing feL4
system images.

Run `cargo fel4 build` from the top-level system directory.

Resulting in:
└── artifacts
    ├── feL4img
    └── kernel

Run `cargo fel4 simulate` to simulate a system image with QEMU.
```

## Command `build`

TODO

## Command `simulate`

TODO

## Command `deploy`

TODO

## Metadata Configuration

```
[fel4]
# path where output artifacts are stored, relative to this location
artifact-path = "images"

# path where target specifications are located
target-specs-path = "targets"

# the target triple to build for
target = "x86_64-sel4-fel4"
```
