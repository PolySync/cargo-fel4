use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use super::{run_cmd, Error};
use config::Config;

pub fn handle_new_cmd(config: &Config) -> Result<(), Error> {
    // TODO - name/path, subcommands for context help?
    let package_name = "fel4-project";

    // base cargo new command to construct project scaffolding
    let mut cmd = Command::new("cargo");
    cmd.arg("new");

    // passthrough to cargo
    if config.cli_args.flag_verbose {
        cmd.arg("--verbose");
    } else if config.cli_args.flag_quiet {
        cmd.arg("--quiet");
    }

    // project name and directory are the same
    cmd.arg("--name").arg(&package_name);

    // run the command, building a bare library project
    run_cmd(cmd.arg("--lib").arg(&package_name))?;

    // create example feL4 application thread run function
    let mut lib_src_file = File::create(Path::new(package_name).join("src").join("lib.rs"))?;
    lib_src_file.write_all(
        b"#![no_std]
extern crate sel4_sys;

use sel4_sys::DebugOutHandle;

macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        DebugOutHandle.write_fmt(format_args!($($arg)*)).unwrap();
    });
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, \"\\n\")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, \"\\n\"), $($arg)*));
}

pub fn run() {
    println!(\"\\nhello from a fel4 app!\\n\");
}",
    )?;

    // TODO - use a toml object to read in, modify, write out?
    // add feL4 dependencies to Cargo.toml
    let mut cargo_toml_file = OpenOptions::new()
        .append(true)
        .open(Path::new(package_name).join("Cargo.toml"))?;
    cargo_toml_file.write_all(
        b"libsel4-sys = {git = \"ssh://github.com/PolySync/fel4-dependencies.git\", branch = \"devel\"}",
    )?;

    // TODO - use new fel4-config package?
    let mut fel4_toml_file = File::create(Path::new(package_name).join("fel4.toml"))?;
    fel4_toml_file.write_all(FEL4_TOML_TEXT.as_bytes())?;

    // create Xargo.toml with our target features
    let mut xargo_toml_file = File::create(Path::new(package_name).join("Xargo.toml"))?;
    xargo_toml_file.write_all(
        b"[target.x86_64-sel4-fel4.dependencies]
alloc = {}
[target.arm-sel4-fel4.dependencies]
alloc = {}
",
    )?;

    Ok(())
}

const FEL4_TOML_TEXT: &'static str = r##"[fel4]
artifact-path = "artifacts"
target-specs-path = "target_specs"
target = "x86_64-sel4-fel4"
platform = "pc99"

[x86_64-sel4-fel4]
BuildWithCommonSimulationSettings = true
KernelOptimisation = "-02"
KernelVerificationBuild = false
KernelBenchmarks = "none"
KernelDangerousCodeInjection = false
KernelFastpath = true
LibSel4FunctionAttributes = "public"
KernelNumDomains = 1
HardwareDebugAPI = false
KernelColourPrinting = true
KernelFWholeProgram = false
KernelResetChunkBits = 8
LibSel4DebugAllocBufferEntries = 0
LibSel4DebugFunctionInstrumentation = "none"
KernelNumPriorities = 256
KernelStackBits = 12
KernelTimeSlice = 5
KernelTimerTickMS = 2
KernelUserStackTraceLength = 16
# the following keys are specific to x86_64-sel4-fel4 targets
KernelArch = "x86"
KernelX86Sel4Arch = "x86_64"
KernelMaxNumNodes = 1
KernelRetypeFanOutLimit = 256
KernelRootCNodeSizeBits = 19
KernelMaxNumBootinfoUntypedCaps = 230
KernelSupportPCID = false
KernelCacheLnSz = 64
KernelDebugDisablePrefetchers = false
KernelExportPMCUser = false
KernelFPU = "FXSAVE"
KernelFPUMaxRestoresSinceSwitch = 64
KernelFSGSBase = "msr"
KernelHugePage = true
KernelIOMMU = false
KernelIRQController = "IOAPIC"
KernelIRQReporting = true
KernelLAPICMode = "XAPIC"
KernelMaxNumIOAPIC = 1
KernelMaxNumWorkUnitsPerPreemption= 100
KernelMultiboot1Header = true
KernelMultiboot2Header = true
KernelMultibootGFXMode = "none"
KernelSkimWindow = true
KernelSyscall = "syscall"
KernelVTX = false
KernelX86DangerousMSR = false
KernelX86IBPBOnContextSwitch = false
KernelX86IBRSMode = "ibrs_none"
KernelX86RSBOnContextSwitch = false
KernelXSaveSize = 576
LinkPageSize = 4096
UserLinkerGCSections = false

[x86_64-sel4-fel4.pc99]
KernelX86MicroArch = "nehalem"
LibPlatSupportX86ConsoleDevice = "com1"

[x86_64-sel4-fel4.debug]
KernelDebugBuild = true
KernelPrinting = true

[x86_64-sel4-fel4.release]
KernelDebugBuild = false
KernelPrinting = false

[arm-sel4-fel4]
BuildWithCommonSimulationSettings = true
KernelOptimisation = "-02"
KernelVerificationBuild = false
KernelBenchmarks = "none"
KernelDangerousCodeInjection = false
KernelFastpath = true
LibSel4FunctionAttributes = "public"
KernelNumDomains = 1
HardwareDebugAPI = false
KernelColourPrinting = true
KernelFWholeProgram = false
KernelResetChunkBits = 8
LibSel4DebugAllocBufferEntries = 0
LibSel4DebugFunctionInstrumentation = "none"
KernelNumPriorities = 256
KernelStackBits = 12
KernelTimeSlice = 5
KernelTimerTickMS = 2
KernelUserStackTraceLength = 16
# the following keys are specific to arm-sel4-fel4 targets
CROSS_COMPILER_PREFIX = "arm-linux-gnueabihf-"
KernelArch = "arm"
KernelArmSel4Arch = "aarch32"
KernelMaxNumNodes = 1
KernelRetypeFanOutLimit = 256
KernelRootCNodeSizeBits = 19
KernelMaxNumBootinfoUntypedCaps = 230
KernelAArch32FPUEnableContextSwitch = true
KernelDebugDisableBranchPrediction = false
KernelFPUMaxRestoresSinceSwitch = 64
KernelIPCBufferLocation = "threadID_register"
KernelMaxNumWorkUnitsPerPreemption = 100
LinkPageSize = 4096
UserLinkerGCSections = false

[arm-sel4-fel4.debug]
KernelDebugBuild = true
KernelPrinting = true

[arm-sel4-fel4.release]
KernelDebugBuild = false
KernelPrinting = false

[arm-sel4-fel4.sabre]
KernelARMPlatform = "sabre"
ElfloaderImage = "elf"
ElfloaderMode = "secure supervisor"
ElfloaderErrata764369 = true
KernelArmEnableA9Prefetcher = false
KernelArmExportPMUUser = false
KernelDebugDisableL2Cache = false
"##;
