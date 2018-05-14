use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // We default to using SSH, once our repositories go public
    // we'll switch over to HTTTPS
    let git_repo_url = "git@github.com:PolySync/feL4-targets.git";

    let out_dir = env::var("OUT_DIR").unwrap();

    // Clone the feL4 target specs, so we can compile them into cargo-fel4
    if !Path::new(&out_dir).join("feL4-targets").exists() {
        assert!(
            Command::new("git")
                .current_dir(&out_dir)
                .arg("clone")
                .arg("--depth")
                .arg("1")
                .arg("-b")
                .arg("master")
                .arg(git_repo_url)
                .arg("feL4-targets")
                .status()
                .unwrap()
                .success()
        );
    };

    // Export paths to target specs as environment variables
    println!(
        "cargo:rustc-env=TARGET_SPEC_PATH_X86_64_SEL4_FEL4={}/feL4-targets/x86_64-sel4-fel4.json",
        out_dir
    );

    println!(
        "cargo:rustc-env=TARGET_SPEC_PATH_ARM_SEL4_FEL4={}/feL4-targets/arm-sel4-fel4.json",
        out_dir
    );
}
