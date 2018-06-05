# cargo-fel4

## Overview

A cargo subcommand for automating feL4 (seL4 for Rust) development

`cargo-fel4` seeks to accelerate the pace of Rust development for seL4 environments
by automating away the annoyances of building the underlying seL4 codebase,
generating useable Rust bindings, and providing a way to get your code
into a runnable seL4 application.

Once installed, use `cargo fel4 new my-project` to create a new feL4 project, which is a regular
Rust `no_std` library project with a few additional configuration frills.

In that project, running `cargo fel4 build` will generate a seL4 application
wrapping your library code from `src/lib.rs`, and `cargo fel4 simulate` will run it.

Access to seL4 capabilities is presently through the [libsel4-sys library](https://github.com/PolySync/libsel4-sys),
a thin binding layer around seL4. This wrapper is built and configured according to your
feL4 project settings, stored in your project's `fel4.toml` manifest file.

feL4 projects come with a example [property-based](https://github.com/AltSysrq/proptest) test suite to demonstrate how to conduct
tests in the feL4 context. Try it out with `cargo fel4 test build && cargo fel4 test simulate`

cargo-fel4 is released with additional special thanks and attribution to
[Robigalia](https://gitlab.com/robigalia/sel4-start/blob/master/LICENSE-MIT),
particularly for their startup assembly code and example conventions W.R.T.
language items, and of course, to Data61, et al for
[seL4](https://github.com/seL4/seL4/blob/master/LICENSE_BSD2.txt).

## Getting Started

### Dependencies

`cargo-fel4` works on top of several other tools to operate, so you'll need Rust with Cargo, Xargo,
and QEMU to build and run feL4 projects. Additionally, feL4 depends on the [libsel4-sys](https://github.com/PolySync/libsel4-sys) crate, which has its own set of dependencies. Some of the "Building" steps below are actually specific to satisfying `libsel4-sys` dependencies. `cargo-fel4` was developed using Ubuntu Xenial, but other Linux variants should work.

* [rust](https://github.com/rust-lang-nursery/rustup.rs) (nightly)
* [xargo](https://github.com/japaric/xargo) (for cross-compiling)
* [gcc/g++ cross compilers](https://gcc.gnu.org/) (for ARM targets)
* [qemu](https://www.qemu.org/) (for simulation)
* [dfu-util](http://dfu-util.sourceforge.net/) (for device deployment)

### Building

These instructions cover installing both `libsel4-sys` and `cargo-fel4` dependencies as well as building `cargo-fel4`.

* Install system package dependencies:
  ```bash
  sudo apt-get install python-pip ninja-build libxml2-utils dfu-util curl
  sudo apt-get install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
  sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
  sudo apt-get install qemu-system-x86 qemu-system-arm
  ```
* Install pip package dependencies:
  ```bash
  sudo pip install cmake sel4-deps
  ```
* Install Rust nightly and additional components:
  ```bash
  curl https://sh.rustup.rs -sSf | sh
  rustup install nightly
  rustup component add rust-src
  cargo install xargo
  ```
* Building `cargo-fel4`:
  ```bash
  cargo build
  ```

### Installation

After building, `cargo-fel4` can be installed with `cargo install`.

* Install under the nightly toolchain:
  ```bash
  cargo +nightly install --git https://github.com/PolySync/cargo-fel4.git
  ```

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

* #### Create a New feL4 Project

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

* #### Build a feL4 Project

  To build a feL4 project using cargo-fel4:

  ```bash
  cd my-project/

  cargo fel4 build
  ```

* #### Simulate a feL4 Project

  To simulate a feL4 project with QEMU via cargo-fel4:

  ```bash
  cd my-project/

  cargo fel4 simulate
  ```

* #### Deploy a feL4 Project

  To deploy a feL4 project on to the target platform using cargo-fel4:

  ```bash
  cd my-project/

  cargo fel4 deploy
  ```

* #### Running Tests

  cargo-fel4 will generate a basic set of property tests when creating a new project.

  ```bash
  cargo fel4 test
  ```

  ##### Just build a feL4 test application:

  ```bash
  cargo fel4 test build
  ```

  ##### Simulate a previously-built feL4 test application:

  ```bash
  cargo fel4 test simulate
  ```

  ##### Deploy a feL4 test application:

  ```bash
  cargo fel4 test deploy
  ```
* #### DFU Deployment on the TX1 Platform

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
* #### Configuration

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

  It is advisable to clean the build cache when changing either the Rust target triple or
  the platform configuration.  This can be done with cargo-fel4:

  ```bash
  cargo fel4 clean
  ```

  See the [fel4-config](https://github.com/PolySync/fel4-config) and
  [libsel4-sys](https://github.com/PolySync/libsel4-sys) packages for more configuration information.

  See the target specifications [README](target_specs/README.md) for more information about
  the specifications shipped with cargo-fel4.

## Tests

`cargo-fel4` manages its own tests with the standard Rust test framework, plus `proptest`
for property-based testing.

### Building

Building the tests is as simple as:

```bash
cargo build --tests
```

### Running

Running the tests for `cargo-fel4` (as opposed to the tests within a given feL4 project)
requires installing the standard dependencies listed earlier. `cargo-fel4`'s internal tests can be exercised by running:

```bash
cargo test
```

# License

© 2018, PolySync Technologies, Inc.

* Jon Lamb [email](mailto:jlamb@polysync.io)
* Zack Pierce [email](mailto:zpierce@polysync.io)
* Dan Pittman [email](mailto:dpittman@polysync.io)

Please see the [LICENSE](./LICENSE) file for more details
