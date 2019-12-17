use serde::{Deserialize, Serialize};

use crate::operations::RunnableOperation;

use super::SystemDetails;

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomCommand {
    pub command: String,
    pub args: Option<Vec<String>>,
}

impl RunnableOperation for &CustomCommand {
    fn command_name(&self, _system_details: SystemDetails) -> Option<String> {
        Some(self.command.clone())
    }

    fn args(&self, _system_details: SystemDetails) -> Option<Vec<String>> {
        self.args.clone()
    }

    fn needs_root(&self) -> bool {
        self.command.starts_with("sudo")
    }
}
