//! The command module defines various commands that can be run on
//! their own, or alongside other operations.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::operations::RunnableOperation;

use super::SystemDetails;

/// The `CustomCommand` defines a shell command that consists
/// of a command name and arguments to pass to it.
///
/// **Note:** This command will be passed to `sh` via the `-c` option.
#[derive(Debug, Deserialize, Serialize)]
pub struct CustomCommand {
    /// The command to run, it will be passed to `sh`
    pub command: String,

    /// Any arguments to pass to the command
    pub args: Option<Vec<String>>,

    /// Whether this command needs root privileges to run
    #[serde(default)]
    pub needs_root: bool,
}

impl CustomCommand {
    /// Create a new `CustomCommand`
    ///
    /// # Arguments:
    ///
    /// - `command`: The command to run
    /// - `args`: Any arguments that should be passed to `command` when run
    /// - `needs_root`: Whether this command should be run via `sudo`
    pub fn new(command: String, args: Option<Vec<String>>, needs_root: bool) -> Self {
        CustomCommand {
            command,
            args,
            needs_root,
        }
    }
}

impl RunnableOperation for CustomCommand {
    fn command_name(&self, _system_details: SystemDetails) -> Result<String> {
        if self.command.is_empty() {
            Err(Error::from("Cannot process a zero-length shell command"))
        } else {
            Ok(self.command.clone())
        }
    }

    fn args(&self, _system_details: SystemDetails) -> Option<Vec<String>> {
        self.args.clone()
    }

    fn needs_root(&self) -> bool {
        self.needs_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::{SystemDetails, TargetOperatingSystem};
    use crate::operations::RunnableOperation;

    #[test]
    fn test_get_command_name() {
        let command = CustomCommand::new(String::from("fc-cache"), None, false);
        let actual_res = command.command_name(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_ok());
        let actual = actual_res.unwrap();
        assert_eq!(actual, String::from("fc-cache"));
    }

    #[test]
    fn test_empty_command_err() {
        let command = CustomCommand::new(String::new(), None, false);
        let actual_res = command.command_name(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_err());
    }

    #[test]
    fn test_no_root_without_sudo() {
        let command = CustomCommand::new(String::from("git"), None, false);
        assert!(!command.needs_root());
    }

    #[test]
    fn test_root() {
        let command = CustomCommand::new(String::from("systemctl"), None, true);
        assert!(command.needs_root());
    }

    #[test]
    fn test_not_root() {
        let command = CustomCommand::new(String::from("systemctl"), None, false);
        assert!(!command.needs_root());
    }

    #[test]
    fn test_get_args() {
        let command = CustomCommand::new(
            String::from("mv"),
            Some(vec![
                String::from("/tmp/file.txt"),
                String::from("/tmp/file.txt.bak"),
            ]),
            false,
        );
        let actual_opt = command.args(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_opt.is_some());
        let actual = actual_opt.unwrap();
        assert_eq!(
            actual,
            vec![
                String::from("/tmp/file.txt"),
                String::from("/tmp/file.txt.bak")
            ]
        );
    }

    #[test]
    fn test_get_none_args() {
        let command = CustomCommand::new(String::from("git"), None, false);
        let actual = command.args(SystemDetails::new(TargetOperatingSystem::Debian));
        assert!(actual.is_none());
    }
}
