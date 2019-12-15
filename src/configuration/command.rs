use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomCommand {
    pub command: String,
    pub args: Option<Vec<String>>,
}
