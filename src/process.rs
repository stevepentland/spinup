use libc;
// use std::io::Read;
// use std::process::Command;

use crate::config::Configuration;

pub fn install_packages(config: &Configuration) {
    if let Some(_packages) = &config.packages {
        // TODO: Use packages as the args for cmd to
        // Command::new("ls")
        //     .args(vec!["-al"])
        //     .spawn()
        //     .expect("Whoops");
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
