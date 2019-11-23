#![deny(clippy::all)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

mod config;
mod error;
mod process;
mod system;

use clap::{App, Arg};
use flexi_logger::Logger;

use config::read_in_config;
use process::{install_packages, process_is_root};
use system::extract_distro_details;

fn main() -> Result<(), String> {
    let _matches = App::new("Spinup")
        .version(crate_version!())
        .author("Steve Pentland")
        .about("Helps you spin up your new environment!")
        .arg(Arg::with_name("verbose").short("-v"))
        .get_matches();

    // Create the logger, hardcode debug for now
    Logger::with_str("trace").start().unwrap();
    if !process_is_root() {
        // just comment for now, it's a pain to test with root all the time
        // return Err(String::from("This program must be run as root"));
    }
    let details = extract_distro_details().unwrap();
    let config = read_in_config("./data/sample.toml").unwrap();
    let _ = install_packages(&config, &details);
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
