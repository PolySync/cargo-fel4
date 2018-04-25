use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};

use common;

/// create a relocable ELF cpio archive that can be linked into another target
pub fn make_cpio_archive(
    input_file: &Path,
    output_name: &str,
    output_dir: &Path,
    append: bool,
) {
    let dirname = input_file.parent().unwrap();
    let basename = input_file.file_name().unwrap();

    if append {
        println!(
            "archiving '{}' +> '{}'",
            basename.to_str().unwrap(),
            output_name
        );
    } else {
        println!(
            "archiving '{}' -> '{}'",
            basename.to_str().unwrap(),
            output_name
        );
    }

    // we could just pipe a string into stdin of our cpio command instead of
    // this
    let file_echo = Command::new("echo")
        .arg(basename.to_str().unwrap().to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let echo_out = file_echo
        .stdout
        .expect("failed to open echo stdout");

    println!("generating 'archive.{}.cpio'", output_name);

    let mut cmd = Command::new("cpio");

    cmd.current_dir(dirname);
    cmd.stdin(Stdio::from(echo_out));

    if append {
        cmd.arg("--append");
    }

    cmd.arg("--quiet");
    cmd.arg("-o");
    cmd.arg("-H");
    cmd.arg("newc");
    cmd.arg(&format!("--file=archive.{}.cpio", output_name));

    common::run_cmd(&mut cmd);

    let linker_file = dirname.join(format!("link.{}.ld", output_name));

    let mut linker_file_buffer = File::create(linker_file).unwrap();

    linker_file_buffer.write_all(
        b"SECTIONS { ._archive_cpio : ALIGN(4) { _cpio_archive = . ; *(.*) ; _cpio_archive_end = . ; } }\n").unwrap();

    println!("generating 'link.{}.ld'", output_name);

    // TODO - configs
    //   - triple/prefix
    //   - link oformat, using 'elf64-x86-64' for now
    let mut cmd = Command::new("ld");
    common::run_cmd(
        cmd.current_dir(dirname)
            .arg("-T")
            .arg(&format!("link.{}.ld", output_name))
            .arg("--oformat")
            .arg("elf64-x86-64")
            .arg("-r")
            .arg("-b")
            .arg("binary")
            .arg(output_name.to_string())
            .arg("-o")
            .arg(output_name.to_string()),
    );

    if dirname != Path::new(output_dir) {
        let mut cmd = Command::new("mv");

        common::run_cmd(cmd.current_dir(dirname).arg(output_name).arg(
            &format!("{}/{}", output_dir.display(), output_name),
        ));
    }
}
