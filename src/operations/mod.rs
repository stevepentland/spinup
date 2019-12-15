use libc;

use std::process::{Command, Stdio};

use crate::configuration::SystemDetails;
use crate::error::{Error, Result};

pub mod file_downloads;
pub mod packages;
pub mod snap;

pub trait RunnableOperation {
    fn command_name(&self, system_details: &SystemDetails) -> Result<String>;

    fn args(&self, system_details: &SystemDetails) -> Result<Vec<String>>;

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

pub fn run_command(runnable: impl RunnableOperation, system_details: &SystemDetails) -> Result<()> {
    if runnable.needs_root() {
        get_root()?;
    }

    let mut command = Command::new("sh");
    command.arg("-c");

    if runnable.needs_root() {
        command.arg("sudo");
    }

    let command_name = runnable.command_name(system_details)?;

    let status = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(&command_name)
        .args(runnable.args(system_details)?)
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
