#[macro_use]
extern crate lazy_static;

mod config;
mod error;
mod system;

use config::read_in_config;
use error::SpinupError;
use system::extract_distro_details;

fn main() -> Result<(), SpinupError> {
    let details = extract_distro_details()?;
    let config = read_in_config("./data/sample.toml")?;
    println!("{:#?}", config);
    println!("{:#?}", details);
    Ok(())
}

/* TODO:
STEPS:
- Check environment for install methods, current distro
- read in instructions file
- run commands

NEEDS:
- Config files in various formats, start with toml
- Structure to represent multiple different commands
- Custom commands
- Start with arch, move from there
- Actually organize the code
*/
