extern crate cargo_metadata;
extern crate toml;

use std::fs::{copy, create_dir};
use std::path::Path;
use std::process::Command;

use toml::Value;

use common;
use common::{Config, Error};
use cpio;

pub fn handle_build_cmd(config: &Config) -> Result<(), Error> {
    let metadata_table = match &config.mf {
        Value::Table(t) => match t.get("metadata") {
            Some(ht) => match ht {
                Value::Table(h) => h,
                _ => {
                    return Err(Error::MetadataError(
                        "metadata section is malformed",
                    ))
                }
            },
            None => {
                return Err(Error::MetadataError("missing metadata section"))
            }
        },
        _ => return Err(Error::MetadataError("metadata is malformed")),
    };
    let helios_table = match metadata_table.get("helios") {
        Some(ht) => match ht {
            Value::Table(h) => h,
            _ => {
                return Err(Error::MetadataError(
                    "metadata section is malformed",
                ))
            }
        },
        None => {
            return Err(Error::MetadataError(
                "missing helios metadata section",
            ))
        }
    };
    let helios_root_task = match helios_table.get("root-task") {
        Some(v) => match v {
            Value::String(s) => s,
            _ => {
                return Err(Error::MetadataError(
                    "root-task value isn't a string",
                ))
            }
        },
        None => return Err(Error::MetadataError("missing root-task key")),
    };
    let helios_apps = match helios_table.get("apps") {
        Some(v) => match v {
            Value::Array(vec) => Some(vec),
            _ => {
                return Err(Error::MetadataError(
                    "helios apps value isn't an array",
                ))
            }
        },
        None => None,
    };
    let helios_artifact_dir = match helios_table.get("artifact-path") {
        Some(v) => match v {
            Value::String(s) => s,
            _ => {
                return Err(Error::MetadataError(
                    "artifact-path value isn't a string",
                ))
            }
        },
        None => {
            return Err(Error::MetadataError(
                "missing artifact-path key",
            ))
        }
    };
    let helios_build_cmd = match helios_table.get("build-cmd") {
        Some(v) => match v {
            Value::String(s) => s,
            _ => {
                return Err(Error::MetadataError(
                    "build-cmd value isn't a string",
                ))
            }
        },
        None => return Err(Error::MetadataError("missing build-cmd key")),
    };
    let helios_target_specs_dir = match helios_table.get("target-specs-path") {
        Some(v) => match v {
            Value::String(s) => s,
            _ => {
                return Err(Error::MetadataError(
                    "target-specs-path value isn't a string",
                ))
            }
        },
        None => {
            return Err(Error::MetadataError(
                "missing target-specs-path key",
            ))
        }
    };
    let helios_default_target = match helios_table.get("default-target") {
        Some(v) => match v {
            Value::String(s) => s,
            _ => {
                return Err(Error::MetadataError(
                    "default-target value isn't a string",
                ))
            }
        },
        None => {
            return Err(Error::MetadataError(
                "missing default-target key",
            ))
        }
    };

    let build_type = match config.args.flag_release {
        true => String::from("release"),
        false => String::from("debug"),
    };

    let target_spec = match config.args.flag_target.is_empty() {
        true => helios_default_target,
        false => &config.args.flag_target,
    };

    let helios_target_specs_path =
        Path::new(&config.md.workspace_root).join(helios_target_specs_dir);

    let helios_artifact_path =
        Path::new(&config.md.workspace_root).join(helios_artifact_dir);

    if let Some(apps) = helios_apps {
        if !apps.is_empty() {
            let apps_lib_name = match helios_table.get("apps-lib-name") {
                Some(v) => match v {
                    Value::String(s) => s,
                    _ => {
                        return Err(Error::MetadataError(
                            "apps-lib-name value isn't a string",
                        ))
                    }
                },
                _ => {
                    return Err(Error::MetadataError(
                        "apps-lib-name value isn't a string",
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

                if app_name != helios_root_task {
                    println!("processing member '{}'", app_name);

                    let mut cmd = Command::new(helios_build_cmd);

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
        config.md.workspace_root, helios_root_task
    );
    println!();

    let mut cmd = Command::new(helios_build_cmd);

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
            config.md.workspace_root, helios_root_task
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
            config.md.target_directory,
            target_spec,
            build_type,
            helios_root_task
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
