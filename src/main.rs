#![deny(clippy::all)]

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

const DEFAULT_LOG_LEVEL: &str = "warn";

#[cfg_attr(tarpaulin, skip)]
fn main() -> Result<(), String> {
    let matches = App::new("Spinup")
        .version(crate_version!())
        .author("Steve Pentland")
        .about("Helps you spin up your new environment!")
        .arg(
            Arg::with_name("verbose")
                .short("-v")
                .help("Increase the verbosity of the program. This may be specified multiple times")
                .multiple(true),
        )
        .get_matches();

    // Create the logger, hardcode debug for now
    let log_level = get_log_level(matches.occurrences_of("verbose"));
    Logger::with_str(log_level).start().unwrap();
    if !process_is_root() {
        // just comment for now, it's a pain to test with root all the time
        // return Err(String::from("This program must be run as root"));
    }
    let details = extract_distro_details().unwrap();
    let config = read_in_config("./data/sample.toml").unwrap();
    let _ = install_packages(&config, &details);
    Ok(())
}

fn get_log_level(verbosity: u64) -> &'static str {
    match verbosity {
        0 => DEFAULT_LOG_LEVEL,
        1 => "info",
        2 => "debug",
        _ => "trace",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paste;

    macro_rules! log_level_tests {
        ($(($suffix:ident, $count:expr, $expected:expr));+) => {
            $(
                paste::item!(
                    #[test]
                    fn [<test_ensure_ $suffix>]() {
                        let actual = get_log_level($count);
                        assert_eq!(actual, $expected);
                    }
                );
            )*
        };
    }

    log_level_tests!(
        (default_as_defined, 0, DEFAULT_LOG_LEVEL);
        (single_verbose_info, 1, "info" );
        (double_verbose_is_debug, 2, "debug");
        (triple_verbose_is_trace, 3,"trace");
        (more_than_three_is_trace, 10, "trace")
    );
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
