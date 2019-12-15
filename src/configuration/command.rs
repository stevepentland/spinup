use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::operations::RunnableOperation;

use super::SystemDetails;

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomCommand {
    pub command: String,
    pub args: Option<Vec<String>>,
}

impl RunnableOperation for &CustomCommand {
    fn command_name(&self, _system_details: SystemDetails) -> Result<String> {
        Ok(self.command.clone())
    }

    fn args(&self, _system_details: SystemDetails) -> Result<Vec<String>> {
        match &self.args {
            Some(a) => Ok(a.clone()),
            None => Ok(vec![]),
        }
    }

    fn needs_root(&self) -> bool {
        self.command.starts_with("sudo")
    }
}
