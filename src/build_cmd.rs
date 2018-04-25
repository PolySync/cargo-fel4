extern crate cargo_metadata;
extern crate toml;

use std::fs::{copy, create_dir};
use std::path::PathBuf;
use std::process::Command;

use toml::Value;

use common;
use common::{Config, DeepLookup, Error};
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
    is_workspace_build: bool,
    uses_root_manifest_config: bool,
}

pub fn parse_helios_metadata(config: &Config) -> Result<HeliosMetadata, Error> {
    let is_workspace_build = match config.md.workspace_members.len() {
        0 => {
            return Err(Error::MetadataError(
                "metadata reports there are no packages",
            ))
        }
        1 => false,
        _ => true,
    };

    let base_key = match is_workspace_build {
        true => String::new(),
        false => String::from("package."),
    };

    let default_target = match config.mf.lookup(&format!(
        "{}metadata.helios.default-target",
        base_key
    ))? {
        Value::String(s) => s,
        _ => {
            return Err(Error::MetadataError(
                "default-target is malformed",
            ))
        }
    };
    let target_specs_dir = match config.mf.lookup(&format!(
        "{}metadata.helios.target-specs-path",
        base_key
    ))? {
        Value::String(s) => s,
        _ => {
            return Err(Error::MetadataError(
                "target-specs-path is malformed",
            ))
        }
    };
    let artifact_dir = match config.mf.lookup(&format!(
        "{}metadata.helios.artifact-path",
        base_key
    ))? {
        Value::String(s) => s,
        _ => {
            return Err(Error::MetadataError(
                "artifact-path is malformed",
            ))
        }
    };
    let apps_lib_name = match config.mf.lookup(&format!(
        "{}metadata.helios.apps-lib-name",
        base_key
    ))? {
        Value::String(s) => s,
        _ => {
            return Err(Error::MetadataError(
                "apps-lib-name is malformed",
            ))
        }
    };
    let helios_apps = match config
        .mf
        .lookup(&format!("{}metadata.helios.apps", base_key))?
    {
        Value::Array(s) => s,
        _ => return Err(Error::MetadataError("apps is malformed")),
    };
    let root_task = match config.mf.lookup(&format!(
        "{}metadata.helios.root-task",
        base_key
    ))? {
        Value::String(s) => s,
        _ => return Err(Error::MetadataError("root-task is malformed")),
    };
    let build_cmd = match config.mf.lookup(&format!(
        "{}metadata.helios.build-cmd",
        base_key
    ))? {
        Value::String(s) => s,
        _ => return Err(Error::MetadataError("build-cmd is malformed")),
    };

    let uses_root_config = match config.mf.lookup(&format!(
        "{}metadata.sel4-cmake-options",
        base_key
    )) {
        Ok(_) => true,
        _ => false,
    };

    Ok(HeliosMetadata {
        root_task: root_task.to_string(),
        apps: helios_apps
            .iter()
            .take_while(|x| x.as_str().is_some())
            .map(|x| x.as_str().unwrap().to_string())
            .collect::<Vec<_>>(),
        artifact_path: PathBuf::from(config.md.workspace_root.clone())
            .join(artifact_dir),
        apps_lib_name: apps_lib_name.to_string(),
        build_cmd: build_cmd.to_string(),
        target_specs_path: PathBuf::from(config.md.workspace_root.clone())
            .join(target_specs_dir),
        default_target: default_target.to_string(),
        uses_root_manifest_config: uses_root_config,
        is_workspace_build: is_workspace_build,
    })
}

pub fn handle_build_cmd(config: &Config) -> Result<(), Error> {
    let helios_md = parse_helios_metadata(config)?;

    if config.args.flag_verbose {
        println!("\n{:#?}", helios_md);
    }

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

    let helios_sel4_config_manifest_path =
        PathBuf::from(&config.md.workspace_root);

    let root_task_name = match helios_md.root_task.is_empty() {
        true => config.md.packages[0].name.clone(),
        false => helios_md.root_task.clone(),
    };

    if config.args.flag_verbose {
        println!(
            "\ntarget build cache: {:?}",
            target_build_cache_path
        );
        println!("root task name: {:?}", root_task_name);
        println!("root task path: {:?}", root_task_path);
        println!(
            "seL4 configuration manifest path: {:?}",
            helios_sel4_config_manifest_path.join("Cargo.toml")
        );
        println!("");
    }

    if !helios_md.apps.is_empty() {
        let archive_name = format!("{}.o", helios_md.apps_lib_name);
        let archive_lib = format!("lib{}.a", helios_md.apps_lib_name);

        if config.args.flag_verbose {
            println!(
                "\ncreating archive {:?}\n",
                target_build_cache_path.join(&archive_name)
            );
        }

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
                        &helios_sel4_config_manifest_path.join("Cargo.toml"),
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
            &helios_sel4_config_manifest_path.join("Cargo.toml"),
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
