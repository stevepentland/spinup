use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::operations::RunnableOperation;

use super::command::CustomCommand;
use super::{SystemDetails, Validatable};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderedCommand {
    /// The ordering id for this command, all commands will be executed in ascending order
    pub id: u8,

    #[serde(flatten)]
    /// The command to run
    pub command: CustomCommand,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommandSet {
    /// The set of commands to run in the specified order
    pub(crate) commands: Vec<OrderedCommand>,
}

impl Validatable for CommandSet {
    fn validate(&self) -> Result<()> {
        let mut indexes: HashMap<u8, u8> = HashMap::new();

        for command in &self.commands {
            *indexes.entry(command.id).or_insert(0) += 1;
        }

        let filtered = indexes
            .iter()
            .filter(|kv| *kv.1 > 1)
            .map(|kv| *kv.0)
            .collect::<Vec<u8>>();

        match filtered.len() {
            0 => Ok(()),
            _ => Err(Error::from(format!(
                "The following id values are not unique: {}",
                filtered
                    .iter()
                    .map(|n| (*n).to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ))),
        }
    }
}

impl CommandSet {
    pub(crate) fn get_runnable_commands(&self) -> Vec<impl RunnableOperation> {
        let mut t = self.commands.clone();
        t.sort_by(|a, b| b.id.cmp(&a.id));
        t
    }
}

impl RunnableOperation for OrderedCommand {
    fn command_name(&self, system_details: SystemDetails) -> Result<String> {
        self.command.command_name(system_details)
    }

    fn args(&self, system_details: SystemDetails) -> Option<Vec<String>> {
        self.command.args(system_details)
    }

    fn needs_root(&self) -> bool {
        self.command.needs_root()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_flattened_custom_command() {
        // Just to make sure `flatten` doesn't get removed
        let text = r#"
            {
                "id": 1,
                "command": "ls",
                "args": ["-a", "-l"],
                "needs_root": false
            }
        "#;
        let command: OrderedCommand = serde_json::from_str(text).unwrap();
        assert_eq!(command.id, 1);
        assert_eq!(command.command.command, "ls");
        assert_eq!(
            command.command.args,
            Some(vec![String::from("-a"), String::from("-l")])
        );
        assert_eq!(command.command.needs_root, false);
    }

    #[test]
    fn check_validate_duplicate_ids() {
        let command_set = CommandSet {
            commands: vec![
                OrderedCommand {
                    id: 1,
                    command: CustomCommand::new("ls".to_string(), None, false),
                },
                OrderedCommand {
                    id: 2,
                    command: CustomCommand::new("cd".to_string(), None, false),
                },
                OrderedCommand {
                    id: 1,
                    command: CustomCommand::new("mv".to_string(), None, false),
                },
            ],
        };
        let actual = command_set.validate();
        assert!(actual.is_err());
    }

    #[test]
    fn check_validate_ok() {
        let command_set = CommandSet {
            commands: vec![
                OrderedCommand {
                    id: 1,
                    command: CustomCommand::new("ls".to_string(), None, false),
                },
                OrderedCommand {
                    id: 2,
                    command: CustomCommand::new("cd".to_string(), None, false),
                },
            ],
        };
        let actual = command_set.validate();
        assert!(actual.is_ok());
    }
}
