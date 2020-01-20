//! `libspinup` provides a common entry point for all functionality
//! used by the `spinup` binary.
//!
//! Currently, there is only one operation, which is to run the app. However
//! in the future this could change.
#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use flexi_logger::Logger;

pub mod configuration;
pub mod error;
pub mod operations;
pub mod runconfig;

use configuration::read_in_config;
use error::{Error, Result};
use operations::{
    execute_download_operations, install_packages, install_snap_packages, process_is_root,
    run_custom_commands,
};

pub use runconfig::RunConfig;

pub async fn run_app(run_config: RunConfig) -> Result<()> {
    Logger::with_str(run_config.log_level).start().unwrap();

    if process_is_root() {
        return Err(Error::from("spinup should not be run as root"));
    }

    let config = read_in_config(&run_config.config_file_path)?;

    if run_config.print_parsed {
        println!("{:#?}", config);
    }

    if run_config.run_package_installs {
        debug!("Installing packages");
        install_packages(&config)?;
    }

    if run_config.run_file_downloads {
        debug!("Downloading files");
        let dls = execute_download_operations(&config).await;
        if let Err(e) = dls {
            return Err(e);
        }
    }

    if run_config.run_custom_commands {
        debug!("Running custom commands");
        run_custom_commands(&config)?;
    }

    if run_config.run_snap_installs {
        debug!("Installing snaps");
        install_snap_packages(&config)?;
    }

    Ok(())
}

#[doc(hidden)]
pub fn generate_configurations(source_format: &str) {
    use std::fs::File;
    use std::io::Write;

    let config =
        configuration::read_in_config(&format!("examples/sample.{}", source_format)).unwrap();

    println!("{:#?}", config);
    match source_format {
        "json" => {
            let mut t_file = File::create("examples/sample.toml").unwrap();
            let _ = t_file.write(toml::to_string_pretty(&config).unwrap().as_bytes());
            let mut y_file = File::create("examples/sample.yml").unwrap();
            let _ = y_file.write(serde_yaml::to_string(&config).unwrap().as_bytes());
        }
        "yml" => {
            let mut t_file = File::create("examples/sample.toml").unwrap();
            let _ = t_file.write(toml::to_string_pretty(&config).unwrap().as_bytes());
            let mut j_file = File::create("examples/sample.json").unwrap();
            let _ = j_file.write(serde_json::to_string_pretty(&config).unwrap().as_bytes());
        }
        _ => {
            let mut y_file = File::create("examples/sample.yml").unwrap();
            let _ = y_file.write(serde_yaml::to_string(&config).unwrap().as_bytes());
            let mut j_file = File::create("examples/sample.json").unwrap();
            let _ = j_file.write(serde_json::to_string_pretty(&config).unwrap().as_bytes());
        }
    }
}
