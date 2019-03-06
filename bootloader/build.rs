extern crate cc;

use cc::Build;
use std::error::Error;

fn main() -> Result<(), Box<Error>> {

    Build::new().file("boot.s").compile("asm");

    Ok(())
}
