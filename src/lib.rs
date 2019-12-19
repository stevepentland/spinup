#[macro_use]
extern crate log;

use flexi_logger::Logger;

pub mod configuration;
pub mod error;
pub mod operations;

use configuration::{read_in_config, Configuration};
use error::{Error, Result};
use operations::{
    execute_download_operations, install_packages, install_snap_packages, process_is_root,
};

const DEFAULT_LOG_LEVEL: &str = "warn";

pub async fn run_app(matches: clap::ArgMatches<'_>) -> Result<()> {
    let log_level = get_log_level(
        matches.occurrences_of("verbose"),
        matches.is_present("quiet"),
    );
    Logger::with_str(log_level).start().unwrap();
    if process_is_root() {
        return Err(Error::from("spinup should not be run as root"));
    }

    let config = read_in_config(matches.value_of("CONFIG").unwrap())?;

    if matches.is_present("print-parsed") {
        println!("{:#?}", config);
    }

    if cfg!(debug_assertions) && matches.is_present("generate") {
        write_other_config_files(&config);
        return Ok(());
    }

    if !matches.is_present("no-packages") {
        debug!("Installing packages");
        install_packages(&config)?;
    }

    if !matches.is_present("no-files") {
        debug!("Downloading files");
        let dls = execute_download_operations(&config).await;
        if let Err(e) = dls {
            return Err(e);
        }
    }

    if !matches.is_present("no-snaps") {
        debug!("Installing snaps");
        install_snap_packages(&config)?;
    }

    Ok(())
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
