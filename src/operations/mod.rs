use libc;

use std::process::{Command, Stdio};

use crate::configuration::SystemDetails;
use crate::error::{Error, Result};

mod file_downloads;
mod packages;
mod snap;

pub use file_downloads::execute_download_operations;
pub use packages::install_packages;
pub use snap::install_snap_packages;

pub trait RunnableOperation {
    fn command_name(&self, system_details: SystemDetails) -> Option<String>;

    fn args(&self, system_details: SystemDetails) -> Option<Vec<String>>;

    fn needs_root(&self) -> bool;
}

pub fn process_is_root() -> bool {
    unsafe {
        let uid = libc::getuid();
        uid == 0
    }
}

fn get_root() -> Result<()> {
    let exit_status = Command::new("sudo").arg("-v").spawn()?.wait()?;

    if exit_status.success() {
        Ok(())
    } else {
        Err("Unable to authenticate for sudo".into())
    }
}

fn run_command(runnable: impl RunnableOperation, system_details: SystemDetails) -> Result<()> {
    let mut command = Command::new("sh");
    command.arg("-c");

    if runnable.needs_root() {
        get_root()?;
        command.arg("sudo");
    }

    let command_name = runnable
        .command_name(system_details)
        .ok_or_else(|| Error::from("Was expecting a command, but got None instead"))?;

    let command_args = runnable.args(system_details).unwrap_or_default();

    let status = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(&command_name)
        .args(command_args)
        .spawn()?
        .wait()?;

    if status.success() {
        Ok(())
    } else {
        let code = match status.code() {
            Some(code) => code.to_string(),
            None => "unknown".to_string(),
        };
        Err(Error::from(format!(
            "While running command {}, received exit status code {}",
            command_name, code
        )))
    }
}
