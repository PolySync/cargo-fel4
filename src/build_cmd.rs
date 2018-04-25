extern crate cargo_metadata;
extern crate toml;

use std::fs::{copy, create_dir};
use std::path::PathBuf;
use std::process::Command;

use common;
use common::{Config, Error};
use cpio;

pub fn handle_build_cmd(config: &Config) -> Result<(), Error> {
    if config.cli_args.flag_verbose {
        println!("\n{:#?}", config.helios_metadata);
    }

    let build_type = if config.cli_args.flag_release {
        String::from("release")
    } else {
        String::from("debug")
    };

    let target_spec = if config.cli_args.flag_target.is_empty() {
        config.helios_metadata.default_target.clone()
    } else {
        config.cli_args.flag_target.clone()
    };

    let target_build_cache_path = PathBuf::from(
        &config.root_metadata.target_directory,
    ).join(&target_spec)
        .join(&build_type);

    let root_task_path = if config.helios_metadata.root_task.is_empty() {
        PathBuf::from(&config.root_metadata.workspace_root)
    } else {
        PathBuf::from(&config.root_metadata.workspace_root)
            .join(&config.helios_metadata.root_task)
    };

    let helios_sel4_config_manifest_path =
        PathBuf::from(&config.root_metadata.workspace_root);

    let root_task_name = if config.helios_metadata.root_task.is_empty() {
        config.root_metadata.packages[0].name.clone()
    } else {
        config.helios_metadata.root_task.clone()
    };

    if config.cli_args.flag_verbose {
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
        println!();
    }

    if !config.helios_metadata.apps.is_empty() {
        let archive_name =
            format!("{}.o", config.helios_metadata.apps_lib_name);
        let archive_lib =
            format!("lib{}.a", config.helios_metadata.apps_lib_name);

        if config.cli_args.flag_verbose {
            println!(
                "\ncreating archive {:?}\n",
                target_build_cache_path.join(&archive_name)
            );
        }

        let mut append = false;
        for app_name in &config.helios_metadata.apps {
            if *app_name != config.helios_metadata.root_task {
                let app_path = PathBuf::from(
                    &config.root_metadata.workspace_root,
                ).join(&app_name);

                println!("processing application '{}'", app_name);

                let mut cmd = Command::new(&config.helios_metadata.build_cmd);

                cmd.arg("build");

                if config.cli_args.flag_release {
                    cmd.arg("--release");
                }

                if config.cli_args.flag_quiet {
                    cmd.arg("--quiet");
                }

                if config.cli_args.flag_verbose {
                    cmd.arg("--verbose");
                }

                if config.uses_root_manifest_config {
                    cmd.env(
                        "HELIOS_MANIFEST_PATH",
                        &helios_sel4_config_manifest_path.join("Cargo.toml"),
                    );
                }

                common::run_cmd(
                    cmd.current_dir(app_path)
                        .env(
                            "RUST_TARGET_PATH",
                            &config.helios_metadata.target_specs_path,
                        )
                        .env(
                            "HELIOS_ARTIFACT_PATH",
                            &config.helios_metadata.artifact_path,
                        )
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
                    config.root_metadata.target_directory,
                    target_spec,
                    build_type
                )),
        );
    }

    println!();
    println!("building root task '{}'", root_task_name);
    println!();

    let mut cmd = Command::new(&config.helios_metadata.build_cmd);

    cmd.arg("build");

    if config.cli_args.flag_release {
        cmd.arg("--release");
    }

    if config.cli_args.flag_quiet {
        cmd.arg("--quiet");
    }

    if config.cli_args.flag_verbose {
        cmd.arg("--verbose");
    }

    if config.uses_root_manifest_config {
        cmd.env(
            "HELIOS_MANIFEST_PATH",
            &helios_sel4_config_manifest_path.join("Cargo.toml"),
        );
    }

    common::run_cmd(
        cmd.current_dir(root_task_path)
            .env(
                "RUST_TARGET_PATH",
                &config.helios_metadata.target_specs_path,
            )
            .env(
                "HELIOS_ARTIFACT_PATH",
                &config.helios_metadata.artifact_path,
            )
            .arg("--target")
            .arg(&target_spec),
    );

    let sysimg_path = config
        .helios_metadata
        .artifact_path
        .join("feL4img");
    let kernel_path = config
        .helios_metadata
        .artifact_path
        .join("kernel");

    if !config.helios_metadata.artifact_path.exists() {
        create_dir(&config.helios_metadata.artifact_path)?;
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
        config.helios_metadata.artifact_path.display()
    );
    println!("  - kernel");
    println!("  - feL4img");
    println!();

    Ok(())
}
