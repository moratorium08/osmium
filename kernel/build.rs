extern crate cc;

use cc::Build;
use std::env;
use std::error::Error;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<Error>> {

    Build::new()
        .file("boot.s")
        .compile("asm");

    Command::new("../tools/bin/elf2bin")
        .args(
            &["bin/osmium", "bin/osmium.bin"]
        )
        .status()?;

    Ok(())
}
