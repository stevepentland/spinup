use libc;

use std::process::Command;

use crate::error::Result;

pub mod file_downloads;
pub mod packages;

pub fn process_is_root() -> bool {
    unsafe {
        let uid = libc::getuid();
        uid == 0
    }
}

pub fn get_root() -> Result<()> {
    let exit_status = Command::new("sudo").arg("-v").spawn()?.wait()?;

    if exit_status.success() {
        Ok(())
    } else {
        Err("Unable to authenticate for sudo".into())
    }
}
