extern crate cc;

use cc::Build;

fn main() -> Result<(), Box<Error>> {

    Build::new().file("boot.s").compile("asm");

    Ok(())
}
