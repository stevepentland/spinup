//! The operations module defines various operations that can be run
//! and also provides the means to run them.

use libc;

use std::process::Command;

use crate::configuration::SystemDetails;
use crate::error::Result;

mod custom_commands;
mod file_downloads;
mod packages;
mod runcore;
mod snap;

pub use custom_commands::run_custom_commands;
pub use file_downloads::execute_download_operations;
pub use packages::install_packages;
pub use snap::install_snap_packages;

use runcore::internal_runner;

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
pub(crate) fn process_is_root() -> bool {
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
    let command_name = runnable.command_name(system_details)?;
    let (base_command, first_arg) = {
        if runnable.needs_root() {
            get_root()?;
            ("sudo", Some(&command_name[..]))
        } else {
            (&command_name[..], None)
        }
    };

    let mut args: Vec<String> = Vec::new();

    if let Some(arg) = first_arg {
        args.push(arg.to_string());
    }

    args.extend(
        runnable
            .args(system_details)
            .unwrap_or_default()
            .into_iter(),
    );

    internal_runner(base_command, &args)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct DummyRunnable {
        command: String,
        args: Option<Vec<String>>,
        root: bool,
    }

    impl RunnableOperation for DummyRunnable {
        fn command_name(&self, _system_details: SystemDetails) -> Result<String> {
            Ok(self.command.clone())
        }
        fn args(&self, _system_details: SystemDetails) -> Option<Vec<String>> {
            self.args.clone()
        }
        fn needs_root(&self) -> bool {
            self.root
        }
    }

    #[test]
    fn test_run_call_basic() {
        let runnable = DummyRunnable {
            command: "testing".to_string(),
            args: Some(vec!["one".to_string(), "two".to_string()]),
            root: false,
        };

        let res = run_command(&runnable, SystemDetails::default());
        assert!(res.is_ok());

        let cmd = runcore::passed_command();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap(), "testing".to_string());

        let args = runcore::passed_args();
        assert!(args.is_some());
        assert_eq!(args.unwrap(), vec!["one".to_string(), "two".to_string()]);

        runcore::reset();
    }
}
