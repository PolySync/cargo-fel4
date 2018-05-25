# cargo-fel4

```
A cargo subcommand for automating feL4 (seL4 for Rust) development
```

## Overview

`cargo-fel4` seeks to accelerate the pace of Rust development for seL4 environments
by automating away the annoyances of building the underlying seL4 codebase,
generating useable Rust bindings, and providing a way to get your code
into a runnable seL4 application.

## Getting Started

Once installed, use `cargo fel4 new my-project` to create a new feL4 project, which is a regular
Rust `no_std` library project with a few additional configuration frills.

In that project, running `cargo fel4 build` will generate a seL4 application
wrapping your library code from `src/lib.rs`, and `cargo fel4 simulate` will run it.

Access to seL4 capabilities is presently through the [libsel4-sys library](https://github.com/PolySync/libsel4-sys),
a thin binding layer around seL4. This wrapper is built and configured according to your
feL4 project settings, stored in your project's `fel4.toml` manifest file.

feL4 projects come with a example [property-based](https://github.com/AltSysrq/proptest) test suite to demonstrate how to conduct
tests in the feL4 context. Try it out with `cargo fel4 test build && cargo fel4 test simulate`

### Dependencies

`cargo-fel4` works on top of several other tools to operate, so you'll need Rust with Cargo, Xargo,
CMake, Ninja, and QEMU to build and run feL4 projects.

#### Linux

cargo-fel4 was developed using Ubuntu Xenial, but other Linux versions should work.

#### QEMU

```bash
sudo apt-get install qemu-system-x86

sudo apt-get install qemu-system-arm
```

#### DFU USB Programmer

```bash
sudo apt-get install dfu-util
```

#### CMake

CMake version `3.7.2` or greater is required due to the seL4 build system.

Binary releases are available from [cmake.org](https://cmake.org/download/).

An example workflow for a recent binary installation on Ubuntu
[can be found on StackExchange's askUbuntu](https://askubuntu.com/questions/355565/how-do-i-install-the-latest-version-of-cmake-from-the-command-line/865294#865294).

#### Ninja

Ninja version `1.7.1` or greater is required or greater is required due to the seL4 build system.

Binary releases are available from [github](https://github.com/ninja-build/ninja/releases).

Ubuntu users can typically install ninja using apt-get.

```bash
sudo apt-get install ninja-build
```

#### Python Tooling

The underlying seL4 build system also makes use of some Python tools.

```bash
# Install python and pip, if you don't have them already
sudo apt-get install python-pip

pip install sel4-deps
```

#### seL4 Tooling

The underlying seL4 build system also requires `xmlint`.

```bash
sudo apt-get install libxml2-utils
```

#### Cross Compiler Toolchains

```bash
# Used by the armv7-sel4-fel4 target
sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf

# Used by the aarch64-sel4-fel4 target
sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
```

#### rustup

```bash
# Download the install script
wget https://static.rust-lang.org/rustup/rustup-init.sh

# Install rustup
chmod +x rustup-init.sh
sh rustup-init.sh
```

#### Nightly Rust

```bash
rustup install nightly
```

#### Xargo

```bash
# Xargo requires rust-src component
rustup component add rust-src

# Install Xargo
cargo install xargo
```

## Building

cargo-fel4 can be built and installed with `cargo install`:

```bash
cargo +nightly install --git https://github.com/PolySync/cargo-fel4.git
```

If you intend on developing `cargo-fel4`, a build is as simple
as running `cargo build` after getting the repo and installing
dependencies.

## Usage

See the output of `cargo fel4 --help` for more details.

```bash
cargo fel4 --help

A cargo subcommand for automating feL4 (seL4 for Rust) development

USAGE:
    cargo fel4 <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    build       Build a feL4 project
    clean       Remove generated artifacts
    deploy      Deploy a feL4 project
    help        Prints this message or the help of the given subcommand(s)
    new         Create a new feL4 project
    simulate    Simulate a feL4 project with QEMU
    test        Build and run feL4 tests
```

### Examples

#### Create a New feL4 Project

To create a new project using cargo-fel4:

```bash
cargo fel4 new my-project
    Created library `my-project` project

$ tree my-project/
my-project/
├── Cargo.toml
├── fel4.toml
├── src
│   ├── fel4_test.rs
│   └── lib.rs
├── target_specs
│   ├── armv7-sel4-fel4.json
│   ├── README.md
│   └── x86_64-sel4-fel4.json
└── Xargo.toml
```

#### Build a feL4 Project

To build a feL4 project using cargo-fel4:

```bash
cd my-project/

cargo fel4 build
```

#### Simulate a feL4 Project

To simulate a feL4 project with QEMU via cargo-fel4:

```bash
cd my-project/

cargo fel4 simulate
```

#### Deploy a feL4 Project

To deploy a feL4 project on to the target platform using cargo-fel4:

```bash
cd my-project/

cargo fel4 deploy
```

#### Running Tests

cargo-fel4 will generate a basic set of property tests when creating a new project.

Build a feL4 test application:

```bash
cargo fel4 test build
```

Simulate a feL4 test application:

```bash
cargo fel4 test simulate
```

Deploy a feL4 test application:

```bash
cargo fel4 test deploy
```

#### Configuration

cargo-fel4 is configured through a `fel4.toml` manifest file.

The manifest file is responsible for prescribing a high-level configuration for cargo-fel4
infrastructure, as well as the underlying `libsel4-sys` package CMake build system.

Boolean properties specified in the `fel4.toml` are applied as Rust features
to feL4 projects during `cargo fel4 build`, so it's possible to
do compile-time configuration to account for variations in available seL4 options.

The `fel4.toml` manifest resides in the project's root directory, and contains several properties
related to the location of input/output artifacts.
These path properties are relative to project's root directory.

For example, a newly generated feL4 project contains the following in `fel4.toml`:

```
[fel4]
artifact-path = "artifacts"
target-specs-path = "target_specs"
...
```

Output artifacts produced during a cargo-fel4 build will be placed in the directory
specified by the `artifact-path` property.

Target specification files available to cargo-fel4 are located in the directory
specified by the `target-specs-path` property.

```bash
cargo fel4 new my-project

# The fel4.toml is generated at the project's root directory
my-new-project/fel4.toml

# Output artifacts produced by the build
my-new-project/artifacts/

# Rust target specifications available to cargo-fel4
my-new-project/target_specs/
```

See the [fel4-config](https://github.com/PolySync/fel4-config) and
[libsel4-sys](https://github.com/PolySync/libsel4-sys) packages for more configuration information.

See the target specifications [README](target_specs/README.md) for more information about
the specifications shipped with cargo-fel4.

## Tests

### Test Dependencies

The tests for `cargo-fel4` (as opposed to the tests within a given feL4 project)
requires installing the standard dependencies listed earlier.


### Running Tests
`cargo-fel4`'s internal tests can be exercised by running `cargo test`

## Deployment

### DFU Deployment on the TX1 Platform

To deploy a feL4 application via DFU, be sure to have a serial connection set up in order to
interact with the U-Boot boot loader.

Attach the USB-mini end of a USB cable to the USB-mini port on the TX1.
Then plug in the power supply for the TX1 and power it on.

Once the TX1 is powered on, watch the serial output so you can stop the boot process at the
U-boot command prompt.

Then at the U-boot command prompt, enter the following:

```bash
setenv dfu_alt_info "kernel ram 0x83000000 0x1000000"
setenv bootcmd_dfu "dfu 0 ram 0; go 0x83000000"
saveenv
```

To make U-boot enter its DFU server mode, type:

```bash
run bootcmd_dfu
```

U-boot will wait until an image has been uploaded.

You can now deploy a cargo-fel4 application image from the host machine:

```bash
cargo fel4 deploy
```

# License

cargo-fel4 is released under the MIT license, with additional thanks and attribution to the following:

* [Robigalia](https://gitlab.com/robigalia/sel4-start/blob/master/LICENSE-MIT), MIT License
* [seL4](https://github.com/seL4/seL4/blob/master/LICENSE_BSD2.txt), BSD 2-Clause License

Please see the [LICENSE](LICENSE) file for more details.
