extern crate cargo_metadata;
extern crate toml;

use std::fs::{copy, create_dir};
use std::path::PathBuf;
use std::process::Command;

use toml::Value;

use common;
use common::{Config, Error};
use cpio;

#[derive(Debug, Clone)]
pub struct HeliosMetadata {
    root_task: String,
    apps: Vec<String>,
    artifact_path: PathBuf,
    apps_lib_name: String,
    build_cmd: String,
    target_specs_path: PathBuf,
    default_target: String,
    uses_root_manifest_config: bool,
}

pub fn parse_helios_metadata(config: &Config) -> Result<HeliosMetadata, Error> {
    let package_table = match &config.mf {
        Value::Table(t) => match t.get("package") {
            Some(ht) => match ht {
                Value::Table(h) => h,
                _ => {
                    return Err(Error::MetadataError(
                        "package section is malformed",
                    ))
                }
            },
            None => return Err(Error::MetadataError("missing package section")),
        },
        _ => {
            return Err(Error::MetadataError(
                "package section is malformed",
            ))
        }
    };

    let metadata_table = match package_table.get("metadata") {
        Some(ht) => match ht {
            Value::Table(h) => h,
            _ => {
                return Err(Error::MetadataError(
                    "metadata section is malformed",
                ))
            }
        },
        None => return Err(Error::MetadataError("missing metadata section")),
    };

    let helios_table = match metadata_table.get("helios") {
        Some(ht) => match ht {
            Value::Table(h) => h,
            _ => {
                return Err(Error::MetadataError(
                    "helios section is malformed",
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

    let helios_apps_array = match helios_apps {
        Some(v) => v,
        _ => {
            return Err(Error::MetadataError(
                "helios apps array is malformed",
            ))
        }
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

    let helios_apps_lib_name = match helios_table.get("apps-lib-name") {
        Some(v) => match v {
            Value::String(s) => s,
            _ => {
                return Err(Error::MetadataError(
                    "apps-lib-name value isn't a string",
                ))
            }
        },
        None => {
            return Err(Error::MetadataError(
                "missing apps-lib-name key",
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

    let uses_root_config = match metadata_table.get("sel4-cmake-options") {
        Some(_) => true,
        None => false,
    };

    // TODO - error handling
    Ok(HeliosMetadata {
        root_task: helios_root_task.to_string(),
        apps: helios_apps_array
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            //.map(|x| x.to_string())
            .collect::<Vec<_>>(),
        artifact_path: PathBuf::from(config.md.workspace_root.clone())
            .join(helios_artifact_dir),
        apps_lib_name: helios_apps_lib_name.to_string(),
        build_cmd: helios_build_cmd.to_string(),
        target_specs_path: PathBuf::from(config.md.workspace_root.clone())
            .join(helios_target_specs_dir),
        default_target: helios_default_target.to_string(),
        uses_root_manifest_config: uses_root_config,
    })
}

pub fn handle_build_cmd(config: &Config) -> Result<(), Error> {
    let helios_md = parse_helios_metadata(config)?;

    let build_type = match config.args.flag_release {
        true => String::from("release"),
        false => String::from("debug"),
    };

    let target_spec = match config.args.flag_target.is_empty() {
        true => helios_md.default_target.clone(),
        false => config.args.flag_target.clone(),
    };

    let target_build_cache_path = PathBuf::from(&config.md.target_directory)
        .join(&target_spec)
        .join(&build_type);

    // TODO - use root-task and apps as names to lookup their path in
    // config.md.packages
    let root_task_path = match helios_md.root_task.is_empty() {
        true => PathBuf::from(&config.md.workspace_root),
        false => {
            PathBuf::from(&config.md.workspace_root).join(&helios_md.root_task)
        }
    };

    let root_task_name = match helios_md.root_task.is_empty() {
        true => config.md.packages[0].name.clone(),
        false => helios_md.root_task.clone(),
    };

    if !helios_md.apps.is_empty() {
        let archive_name = format!("{}.o", helios_md.apps_lib_name);
        let archive_lib = format!("lib{}.a", helios_md.apps_lib_name);

        println!();
        println!(
            "creating archive '{}'",
            target_build_cache_path
                .join(&archive_name)
                .display()
        );
        println!();

        let mut append = false;
        for app_name in helios_md.apps {
            if app_name != helios_md.root_task {
                // TODO - use root-task and apps as names to lookup their path
                // in config.md.packages
                let app_path =
                    PathBuf::from(&config.md.workspace_root).join(&app_name);

                println!("processing application '{}'", app_name);

                let mut cmd = Command::new(&helios_md.build_cmd);

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

                if helios_md.uses_root_manifest_config {
                    cmd.env(
                        "HELIOS_MANIFEST_PATH",
                        &root_task_path.join("Cargo.toml"),
                    );
                }

                common::run_cmd(
                    cmd.current_dir(app_path)
                        .env("RUST_TARGET_PATH", &helios_md.target_specs_path)
                        .env("HELIOS_ARTIFACT_PATH", &helios_md.artifact_path)
                        .arg("--target")
                        .arg(&target_spec),
                );

                cpio::make_cpio_archive(
                    &target_build_cache_path.join(app_name),
                    &archive_name.to_string(),
                    &target_build_cache_path,
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
            cmd.current_dir(&target_build_cache_path)
                .arg("rcs")
                .arg(&archive_lib)
                .arg(&archive_name),
        );

        // copy the applications archive library into pre-existing linker
        // directory
        let mut cmd = Command::new("cp");
        common::run_cmd(
            cmd.current_dir(&target_build_cache_path)
                .arg("-f")
                .arg(&archive_lib)
                .arg(&format!(
                    "{}/{}/{}/deps/",
                    config.md.target_directory, target_spec, build_type
                )),
        );
    }

    println!();
    println!("building root task '{}'", root_task_name);
    println!();

    let mut cmd = Command::new(&helios_md.build_cmd);

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

    if helios_md.uses_root_manifest_config {
        cmd.env(
            "HELIOS_MANIFEST_PATH",
            &root_task_path.join("Cargo.toml"),
        );
    }

    common::run_cmd(
        cmd.current_dir(root_task_path)
            .env("RUST_TARGET_PATH", &helios_md.target_specs_path)
            .env("HELIOS_ARTIFACT_PATH", &helios_md.artifact_path)
            .arg("--target")
            .arg(&target_spec),
    );

    let sysimg_path = helios_md.artifact_path.join("feL4img");
    let kernel_path = helios_md.artifact_path.join("kernel");

    if !helios_md.artifact_path.exists() {
        create_dir(&helios_md.artifact_path)?;
    }

    // copy the image out of the Cargo workspace
    copy(
        target_build_cache_path.join(&root_task_name),
        &sysimg_path,
    )?;

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
        helios_md.artifact_path.display()
    );
    println!("  - kernel");
    println!("  - feL4img");
    println!();

    Ok(())
}
