use libc;
use std::process::Command;

use crate::{config::Configuration, system::SystemDetails};

pub fn install_packages(config: &Configuration, details: &SystemDetails) {
    if let Some(packages) = &config.packages {
        if let Some(pm) = details.package_manager() {
            let mut c = Command::new(&pm.name);
            if let Some(install_command) = &pm.install_subcommand {
                c.arg(install_command);
            }
            if let Some(autoconfirm) = &pm.autoconfirm {
                c.arg(autoconfirm);
            }
            c.args(packages);
            println!("{:?}", c); // Don't run the command, just print for now
        }
    } else {
        // Print out log message when logging setup
    }
}

pub fn process_is_root() -> bool {
    unsafe {
        let uid = libc::getuid();
        uid == 0
    }
}
