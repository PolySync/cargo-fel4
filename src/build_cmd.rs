extern crate cargo_metadata;
extern crate toml;

use std::fs::{copy, create_dir};
use std::path::Path;
use std::process::Command;

use toml::Value;

use common;
use common::{Config, DeepLookup, Error};
use cpio;

pub fn handle_build_cmd(config: &Config) -> Result<(), Error> {
    let default_target = match config
        .mf
        .lookup("metadata.helios.default-target")?
    {
        Value::String(s) => s,
        _ => {
            return Err(Error::MetadataError(
                "default-target is malformed",
            ))
        }
    };
    let target_specs_dir = match config
        .mf
        .lookup("metadata.helios.target-specs-path")?
    {
        Value::String(s) => s,
        _ => {
            return Err(Error::MetadataError(
                "target-specs-path is malformed",
            ))
        }
    };
    let artifact_dir = match config
        .mf
        .lookup("metadata.helios.artifact-path")?
    {
        Value::String(s) => s,
        _ => {
            return Err(Error::MetadataError(
                "artifact-path is malformed",
            ))
        }
    };
    let fel4_apps = match config.mf.lookup("metadata.helios.apps")? {
        Value::Array(s) => Some(s),
        _ => return Err(Error::MetadataError("apps is malformed")),
    };
    let root_task = match config.mf.lookup("metadata.helios.root-task")? {
        Value::String(s) => s,
        _ => return Err(Error::MetadataError("root-task is malformed")),
    };
    let build_cmd = match config.mf.lookup("metadata.helios.build-cmd")? {
        Value::String(s) => s,
        _ => return Err(Error::MetadataError("build-cmd is malformed")),
    };
    let build_type = if config.args.flag_release {
        String::from("release")
    } else {
        String::from("debug")
    };

    let target_spec = if config.args.flag_target.is_empty() {
        default_target
    } else {
        &config.args.flag_target
    };

    let helios_target_specs_path =
        Path::new(&config.md.workspace_root).join(target_specs_dir);

    let helios_artifact_path =
        Path::new(&config.md.workspace_root).join(artifact_dir);

    if let Some(apps) = fel4_apps {
        if !apps.is_empty() {
            let apps_lib_name = match config
                .mf
                .lookup("metadata.helios.apps-lib-name")?
            {
                Value::String(s) => s,
                _ => {
                    return Err(Error::MetadataError(
                        "apps-lib-name is malformed",
                    ))
                }
            };

            let archive_name = format!("{}.o", apps_lib_name);
            let archive_lib = format!("lib{}.a", apps_lib_name);
            let archive_dir = format!(
                "{}/{}/{}",
                config.md.target_directory, target_spec, build_type
            );

            println!();
            println!(
                "archiving applications in '{}/{}'",
                archive_dir, archive_name
            );
            println!();

            let mut append = false;
            for app in apps {
                let app_name = match app {
                    Value::String(s) => s,
                    _ => {
                        return Err(Error::MetadataError(
                            "couldn't parse app name",
                        ))
                    }
                };

                if app_name != root_task {
                    println!("processing member '{}'", app_name);

                    let mut cmd = Command::new(build_cmd);

                    cmd.arg("build");

                    if config.args.flag_release {
                        cmd.arg("--release");
                    }

                    if config.args.flag_quiet {
                        cmd.arg("--quiet");
                    }

                    if config.args.flag_verbose {
                        cmd.arg("--verbose");
                    }

                    common::run_cmd(
                        cmd.current_dir(format!(
                            "{}/{}",
                            config.md.workspace_root, app_name
                        )).env("RUST_TARGET_PATH", &helios_target_specs_path)
                            .env("HELIOS_ARTIFACT_PATH", &helios_artifact_path)
                            .arg("--target")
                            .arg(target_spec),
                    );

                    cpio::make_cpio_archive(
                        &Path::new(&format!(
                            "{}/{}/{}/{}",
                            config.md.target_directory,
                            target_spec,
                            build_type,
                            app_name
                        )),
                        &archive_name.to_string(),
                        &Path::new(&archive_dir),
                        append,
                    );

                    append = true;

                    println!();
                }
            }

            println!();
            println!(
                "creating applications library '{}'",
                archive_lib
            );
            println!();

            // archive the apps ELF archive into a static library
            let mut cmd = Command::new("ar");
            common::run_cmd(
                cmd.current_dir(&archive_dir)
                    .arg("rcs")
                    .arg(&archive_lib)
                    .arg(&archive_name),
            );

            // copy the applications archive library into pre-existing linker
            // directory
            let mut cmd = Command::new("cp");
            common::run_cmd(
                cmd.current_dir(&archive_dir)
                    .arg("-f")
                    .arg(&archive_lib)
                    .arg(&format!(
                        "{}/{}/{}/deps/",
                        config.md.target_directory, target_spec, build_type
                    )),
            );
        }
    }

    println!();
    println!(
        "building root task '{}/{}'",
        config.md.workspace_root, root_task
    );
    println!();

    let mut cmd = Command::new(build_cmd);

    cmd.arg("build");

    if config.args.flag_release {
        cmd.arg("--release");
    }

    if config.args.flag_quiet {
        cmd.arg("--quiet");
    }

    if config.args.flag_verbose {
        cmd.arg("--verbose");
    }

    common::run_cmd(
        cmd.current_dir(format!(
            "{}/{}",
            config.md.workspace_root, root_task
        )).env("RUST_TARGET_PATH", &helios_target_specs_path)
            .env("HELIOS_ARTIFACT_PATH", &helios_artifact_path)
            .arg("--target")
            .arg(target_spec),
    );

    let sysimg_path = helios_artifact_path.join("feL4img");
    let kernel_path = helios_artifact_path.join("kernel");

    if !helios_artifact_path.exists() {
        create_dir(&helios_artifact_path)?;
    }

    // copy the image out of the Cargo workspace
    copy(
        format!(
            "{}/{}/{}/{}",
            config.md.target_directory, target_spec, build_type, root_task
        ),
        &sysimg_path,
    ).unwrap();

    if !sysimg_path.exists() {
        common::fail(
            "something went wrong with the build, cannot find the system image",
        );
    }

    if !kernel_path.exists() {
        common::fail(
            "something went wrong with the build, cannot find the kernel file",
        );
    }

    println!();
    println!(
        "artifacts in '{}'",
        helios_artifact_path.display()
    );
    println!("  - kernel");
    println!("  - feL4img");
    println!();
    Ok(())
}
