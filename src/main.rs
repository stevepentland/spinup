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

use config::{read_in_config, Configuration};
use process::{install_packages, process_is_root};
use system::extract_distro_details;

const DEFAULT_LOG_LEVEL: &str = "warn";

#[cfg_attr(tarpaulin, skip)]
fn main() {
    let mut app = App::new("Spinup")
        .version(crate_version!())
        .author("Steve Pentland")
        .about("Helps you spin up your new environment!")
        .arg(
            Arg::with_name("verbose")
                .short("-v")
                .help("Increase the verbosity of the program. This may be specified multiple times")
                .multiple(true),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .help("Suppress all program output")
                .multiple(false)
                .takes_value(false)
                .conflicts_with("verbose"),
        );

    #[cfg(debug_assertions)]
    {
        // TODO: Keep only for development
        app = app.arg(
            Arg::with_name("generate")
                .long("generate")
                .short("g")
                .takes_value(false),
        );
    }

    let matches = app.get_matches();

    let log_level = get_log_level(
        matches.occurrences_of("verbose"),
        matches.is_present("quiet"),
    );
    Logger::with_str(log_level).start().unwrap();
    if !process_is_root() {
        // just comment for now, it's a pain to test with root all the time
        // return Err(String::from("This program must be run as root"));
    }
    let details = extract_distro_details().unwrap();
    let config = read_in_config("./data/sample.toml").unwrap();

    #[cfg(debug_assertions)]
    {
        if matches.is_present("generate") {
            write_other_config_files(&config);
        }
    }

    let _ = install_packages(&config, &details);
}

fn get_log_level(verbosity: u64, is_quiet: bool) -> &'static str {
    if is_quiet {
        return "off";
    }
    match verbosity {
        0 => DEFAULT_LOG_LEVEL,
        1 => "info",
        2 => "debug",
        _ => "trace",
    }
}

#[cfg(debug_assertions)]
fn write_other_config_files(config: &Configuration) {
    // This is just used to generate other examples from the original toml
    // to other types
    use std::fs::File;
    use std::io::Write;
    // Write JSON
    let mut j_file = File::create("./data/sample.json").unwrap();
    let _ = j_file.write(serde_json::to_string_pretty(&config).unwrap().as_bytes());
    // Write yaml
    let mut y_file = File::create("./data/sample.yml").unwrap();
    let _ = y_file.write(serde_yaml::to_string(&config).unwrap().as_bytes());
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
                        let actual = get_log_level($count, false);
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

    #[test]
    fn test_quiet_is_off() {
        let actual = get_log_level(0, true);
        assert_eq!(actual, "off");
    }
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
