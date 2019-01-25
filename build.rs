extern crate cc;

use cc::Build;
use std::env;
use std::error::Error;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<Error>> {

    Build::new()
        .file("boot.s")
        //.flag("-mabi=ilp32")
        .compile("asm");

    let out_dir = env::var("OUT_DIR").unwrap();
    let input_path = Path::new(&out_dir).join("../../../osmium");
    let output_path = Path::new(&out_dir).join("../../../osmium.bin");

    Command::new("tools/bin/elf2bin")
        .args(&[input_path.to_str().unwrap(), output_path.to_str().unwrap()])
        .status()?;

    Ok(())
}
