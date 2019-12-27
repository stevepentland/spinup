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

    if run_config.run_snap_installs {
        debug!("Installing snaps");
        install_snap_packages(&config)?;
    }

    Ok(())
}
