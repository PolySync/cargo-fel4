# cargo-fel4

Tool for automating fel4 (seL4 for Rust) development

## Install

```
cargo install --git git@bitbucket.org:PolySync/cargo-fel4.git
```

## Example

```
cargo fel4 new my-fel4-project

cd my-fel4-project

cargo fel4 build

cargo fel4 simulate
```

## Usage

```
Build, manage and simulate feL4 system images

Usage:
    cargo fel4 [options] [build | simulate | new]

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

Run `cargo fel4 new` to create a new fel4 package.
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

## License

cargo-fel4 is released under the MIT license, with additional thanks and attribution to the following:
* [Robigalia](https://gitlab.com/robigalia/sel4-start/blob/master/LICENSE-MIT), MIT License
* [seL4](https://github.com/seL4/seL4/blob/master/LICENSE_BSD2.txt), BSD 2-Clause License
