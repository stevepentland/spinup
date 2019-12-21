//! The operations module defines various operations that can be run
//! and also provides the means to run them.

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

/// The `RunnableOperation` trait represents those operations that will
/// be executed as shell processes. This includes package installs,
/// snap & flatpak packages, custom commands, and so on.
///
/// A runnable operation is required to have a command name. However,
/// additional args are optional. It also must report whether it requires
/// root permissions to run.
pub trait RunnableOperation {
    /// This is the name of the command to run, with no arguments.
    ///
    /// **Note**:
    ///
    /// It is important that this is only a single string of the actual command
    /// be included in [`args`](trait.RunnableOperation.html#tymethod.args)
    /// to run, with no arguments. If there are additional sub-commands they should
    fn command_name(&self, system_details: SystemDetails) -> Result<String>;

    /// Any additional arguments to be sent to the command that will be run.
    /// This includes subcommands, arguments, etc.
    fn args(&self, system_details: SystemDetails) -> Option<Vec<String>>;

    /// Whether this process requires root permissions (via `sudo`) to run
    fn needs_root(&self) -> bool;
}

/// Helper function that queries `libc` to check whether we're inside a
/// superuser process.
pub fn process_is_root() -> bool {
    unsafe {
        let uid = libc::getuid();
        uid == 0
    }
}

/// Helper that will run `sudo -v` to obtain a prompt to enter a user's password.
/// As a session with sudo lasts ~15 minutes, the user's authentication for this should
/// serve for the entire time this application runs. Subsequent calls will not require
/// password entry if we're still within the time limit.
fn get_root() -> Result<()> {
    let exit_status = Command::new("sudo").arg("-v").spawn()?.wait()?;

    if exit_status.success() {
        Ok(())
    } else {
        Err("Unable to authenticate for sudo".into())
    }
}

/// Run the given `RunnableOperation`, returning an empty result if there were no errors
///
/// # Arguments:
///
/// - `runnable`: The `RunnableOperation` to execute
/// - `system_details`: The current configuration's system details for which system we're running in
fn run_command(runnable: &impl RunnableOperation, system_details: SystemDetails) -> Result<()> {
    let mut command = Command::new("sh");
    command.arg("-c");

    if runnable.needs_root() {
        get_root()?;
        command.arg("sudo");
    }

    let command_name = runnable.command_name(system_details)?;

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

// TODO: Look into using this with `run_command`
fn _handle_process_output(output: std::process::Output) -> Result<()> {
    if let Some(code) = output.status.code() {
        if code == 0 {
            info!("Package install process completed successfully");
            Ok(())
        } else {
            use log::{max_level, LevelFilter};
            warn!("Package install returned status of {}", code);

            // Don't bother building trace output unless we're actually using it
            if max_level() == LevelFilter::Trace {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.len() > 0 {
                    debug!("Stdout: \n{}", stdout);
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.len() > 0 {
                    debug!("Stderr: \n{}", stderr);
                }
            }
            Err(format!("Package manager returned status of '{}'.\nRun with higher verbosity to see more output", code).into())
        }
    } else {
        Ok(())
    }
}
