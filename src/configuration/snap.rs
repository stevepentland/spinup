use std::fmt;

use serde::{Deserialize, Serialize};

use crate::operations::RunnableOperation;

use super::SystemDetails;

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq)]
#[serde(rename_all(deserialize = "lowercase", serialize = "lowercase"))]
pub enum SnapChannel {
    Stable,
    Beta,
    Candidate,
    Edge,
}

impl fmt::Display for SnapChannel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            SnapChannel::Stable => "--stable",
            SnapChannel::Beta => "--beta",
            SnapChannel::Candidate => "--candidate",
            SnapChannel::Edge => "--edge",
        };

        write!(f, "{}", text)
    }
}

impl Default for SnapChannel {
    fn default() -> Self {
        SnapChannel::Stable
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SnapPackage {
    pub name: String,
    #[serde(default)]
    pub classic: bool,
    #[serde(default)]
    pub channel: SnapChannel,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StandardSnaps {
    pub names: Vec<String>,
}

impl fmt::Display for SnapPackage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let base = format!("{} {}", self.channel, self.name);

        if self.classic {
            write!(f, "--classic {}", base.trim())
        } else {
            write!(f, "{}", base.trim())
        }
    }
}

impl RunnableOperation for &SnapPackage {
    fn command_name(&self, _system_details: SystemDetails) -> Option<String> {
        Some(String::from("snap"))
    }

    fn args(&self, _system_details: SystemDetails) -> Option<Vec<String>> {
        let mut args = vec![
            String::from("install"),
            self.name.clone(),
            format!("{}", self.channel),
        ];

        if self.classic {
            args.push(String::from("--classic"));
        }

        Some(args)
    }

    fn needs_root(&self) -> bool {
        true
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Snaps {
    /// These snaps can be installed from the stable channel and
    /// do not require `--classic` confinement
    pub standard_snaps: StandardSnaps,

    /// These snaps need to be installed from other channels and/or
    /// need `--classic` confinement
    pub alternate_snaps: Option<Vec<SnapPackage>>,
}

impl RunnableOperation for &StandardSnaps {
    fn command_name(&self, _system_details: SystemDetails) -> Option<String> {
        if self.names.is_empty() {
            None
        } else {
            Some(String::from("snap"))
        }
    }

    fn args(&self, _system_details: SystemDetails) -> Option<Vec<String>> {
        if self.names.is_empty() {
            None
        } else {
            let mut args = vec![String::from("install")];
            args.extend(self.names.clone().into_iter());
            Some(args)
        }
    }

    fn needs_root(&self) -> bool {
        true
    }
}
