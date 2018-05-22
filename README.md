# cargo-fel4

```
A cargo subcommand for automating fel4 (seL4 for Rust) development
```

## Dependencies

### Linux

We use Ubuntu Xenial but other versions should work.

### QEMU

```
$ sudo apt-get install qemu-system-x86

$ sudo apt-get install qemu-system-arm
```

### CMake

CMake version `3.7.2` or greater is required.

Binary releases are available from [cmake.org](https://cmake.org/download/).

### Ninja

Ninja version `1.7.1` or greater is required.

Binary releases are available from [github](https://github.com/ninja-build/ninja/releases).

### Cross compiler toolchains

```
$ sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
```

### rustup

```
# Download the install script
$ curl -f -L https://static.rust-lang.org/rustup.sh -O

# Install rustup
$ sh rustup.sh
```

### Nightly Rust

```
$ rustup default nightly
```

### Xargo

```
# Xargo requires rust-src component
$ rustup component add rust-src

# Install Xargo
$ cargo install xargo
```

## Install

cargo-fel4 can be installed with `cargo install`:

```
$ cargo install --git https://github.com/PolySync/cargo-fel4.git
```

## Usage

See the output of `cargo fel4 --help` for more details.

```
$ cargo fel4 --help

Build, manage and simulate feL4 system images

USAGE:
    cargo fel4 <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    build       Build a feL4 project
    clean       Remove generated artifacts
    help        Prints this message or the help of the given subcommand(s)
    new         Create a new feL4 project
    simulate    Simulate a feL4 project with QEMU
    test        Build and run feL4 tests
```

### Create a New feL4 Project

To create a new project using cargo-fel4:

```
$ cargo fel4 new my-project
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

### Build a feL4 Project

To build a cargo-fel4 project:

```
$ cd my-project/

$ cargo fel4 build
```

### Simulate a feL4 Project

To simulate a cargo-fel4 project with QEMU:

```
$ cd my-project/

$ cargo fel4 simulate
```

### Running Tests

cargo-fel4 will generate a basic set of property tests when creating a new project.

Build a cargo-fel4 test application:

```
$ cargo fel4 test build
```

Simulate a cargo-fel4 test application:

```
$ cargo fel4 test simulate
```

## Configuration

cargo-fel4 is configured through a `fel4.toml` manifest file.

The manifest file is responsible for prescribing a high-level configuration for cargo-fel4
infrastructure, as well as the underlying `libsel4-sys` package CMake build system.

The `fel4.toml` manifest resides in the project's root directory, and contains several properties
related to the location of input/output artifacts.
These path properties are relative to project's root directory.

For example, a newly generated cargo-fel4 project contains the following in `fel4.toml`:

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

```
$ cargo fel4 new my-project

# The fel4.toml is generated at the project's root directory
$ my-new-project/fel4.toml

# Output artifacts produced by the build
$ my-new-project/artifacts/

# Rust target specifications available to cargo-fel4
$ my-new-project/target_specs/
```

See the [fel4-config](https://github.com/PolySync/fel4-config) and
[libsel4-sys](https://github.com/PolySync/libsel4-sys) packages for more configuration information.

See the target specifications [README](target_specs/README.md) for more information about
the specifications shipped with cargo-fel4.

## License

cargo-fel4 is released under the MIT license, with additional thanks and attribution to the following:

* [Robigalia](https://gitlab.com/robigalia/sel4-start/blob/master/LICENSE-MIT), MIT License
* [seL4](https://github.com/seL4/seL4/blob/master/LICENSE_BSD2.txt), BSD 2-Clause License

Please see the [LICENSE](LICENSE) file for more details.
