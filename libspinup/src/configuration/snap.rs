//! The `snap` module contains elements and implementations for
//! working with and installing snap packages.
//!
//! The elements in here are generally only going to be loaded from
//! the parent module.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::operations::RunnableOperation;

use super::SystemDetails;

/// Represents a channel that a snap can be installed from. This does
/// not guarantee that a snap can be installed from the given channel,
/// and any usage of channels other than stable can fail.
#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq)]
#[serde(rename_all(deserialize = "lowercase", serialize = "lowercase"))]
pub enum SnapChannel {
    /// Install snap from channel `--stable` (default)
    Stable,

    /// Install snap from channel `--beta`
    Beta,

    /// Install snap from channel `--candidate`
    Candidate,

    /// Install snap from channel `--edge`
    Edge,
}

impl fmt::Display for SnapChannel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            SnapChannel::Stable => "stable",
            SnapChannel::Beta => "beta",
            SnapChannel::Candidate => "candidate",
            SnapChannel::Edge => "edge",
        };

        write!(f, "--{}", text)
    }
}

impl Default for SnapChannel {
    fn default() -> Self {
        SnapChannel::Stable
    }
}

/// Represents a single snap package. Generally used to indicate snaps
/// which need to be installed from channels other than stable or with
/// classic confinement.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SnapPackage {
    /// The name of the snap, this will be used to install via snapd
    pub name: String,

    /// Whether this snap requires `--classic` confinement, defaults to `false`
    #[serde(default)]
    pub classic: bool,

    /// The channel from which to install this snap, defaults to [`SnapChannel::Stable`](enum.SnapChannel.html#variant.Stable)
    #[serde(default)]
    pub channel: SnapChannel,
}

/// A container for a set of snaps that can all be installed from the
/// stable channel without classic confinement
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StandardSnaps {
    /// The names of the snaps to install
    pub names: Vec<String>,
}

impl RunnableOperation for SnapPackage {
    fn command_name(&self, _system_details: SystemDetails) -> Result<String> {
        match self.name.len() {
            0 => Err(Error::from("Cannot install a snap with no name")),
            _ => Ok(String::from("snap")),
        }
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

/// Upper-most container for snap install directives.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Snaps {
    /// These snaps can be installed from the stable channel and
    /// do not require `--classic` confinement
    pub standard_snaps: StandardSnaps,

    /// These snaps need to be installed from other channels and/or
    /// need `--classic` confinement
    pub alternate_snaps: Option<Vec<SnapPackage>>,
}

impl RunnableOperation for StandardSnaps {
    fn command_name(&self, _system_details: SystemDetails) -> Result<String> {
        if self.names.is_empty() {
            Err(Error::from("Snap list was present but no names were given"))
        } else {
            Ok(String::from("snap"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::{SystemDetails, TargetOperatingSystem};
    use crate::operations::RunnableOperation;

    #[test]
    fn ensure_default_for_snap_channel() {
        let channel = SnapChannel::default();

        assert_eq!(channel, SnapChannel::Stable);
    }

    #[test]
    fn test_get_expected_executable_from_standard_snaps() {
        let standard_snaps = StandardSnaps {
            names: vec![String::from("dummy")],
        };

        let actual_res =
            standard_snaps.command_name(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_ok());
        let actual_cmd = actual_res.unwrap();
        assert_eq!(actual_cmd, String::from("snap"));
    }

    #[test]
    fn test_get_expected_arguments_from_standard_snaps() {
        let standard_snaps = StandardSnaps {
            names: vec![String::from("spotify"), String::from("code")],
        };
        let actual_opt = standard_snaps.args(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_opt.is_some());
        let actual_args = actual_opt.unwrap();
        let expected_args = vec![
            String::from("install"),
            String::from("spotify"),
            String::from("code"),
        ];
        assert_eq!(actual_args, expected_args);
    }

    #[test]
    fn ensure_no_names_gives_none_args() {
        let standard_snaps = StandardSnaps { names: vec![] };
        let actual_opt = standard_snaps.args(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_opt.is_none());
    }

    #[test]
    fn ensure_empty_snap_list_is_command_err() {
        let standard_snaps = StandardSnaps { names: Vec::new() };
        let actual_res =
            standard_snaps.command_name(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_err());
    }

    #[test]
    fn ensure_standard_snaps_needs_root() {
        let standard_snaps = StandardSnaps {
            names: vec![String::from("dummy")],
        };
        assert!(standard_snaps.needs_root());
    }

    #[test]
    fn test_package_command() {
        let package = SnapPackage {
            name: String::from("spotify"),
            classic: true,
            channel: SnapChannel::default(),
        };
        let actual_res = package.command_name(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_ok());
        let actual_cmd = actual_res.unwrap();
        assert_eq!(actual_cmd, String::from("snap"));
    }

    #[test]
    fn test_package_stable_classic_args() {
        let package = SnapPackage {
            name: String::from("spotify"),
            classic: true,
            channel: SnapChannel::default(),
        };
        let actual_res = package.args(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_some());
        let actual_args = actual_res.unwrap();
        assert_eq!(
            actual_args,
            vec![
                String::from("install"),
                String::from("spotify"),
                String::from("--stable"),
                String::from("--classic")
            ]
        );
    }

    #[test]
    fn ensure_nameless_snap_is_cmd_err() {
        let package = SnapPackage {
            name: String::new(),
            classic: true,
            channel: SnapChannel::default(),
        };
        let actual_res = package.command_name(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_err());
    }

    #[test]
    fn test_package_stable_no_classic_args() {
        let package = SnapPackage {
            name: String::from("spotify"),
            classic: false,
            channel: SnapChannel::default(),
        };
        let actual_res = package.args(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_some());
        let actual_args = actual_res.unwrap();
        assert_eq!(
            actual_args,
            vec![
                String::from("install"),
                String::from("spotify"),
                String::from("--stable")
            ]
        );
    }

    #[test]
    fn test_package_beta_args() {
        let package = SnapPackage {
            name: String::from("spotify"),
            classic: false,
            channel: SnapChannel::Beta,
        };
        let actual_res = package.args(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_some());
        let actual_args = actual_res.unwrap();
        assert_eq!(
            actual_args,
            vec![
                String::from("install"),
                String::from("spotify"),
                String::from("--beta")
            ]
        );
    }

    #[test]
    fn test_package_candidate_args() {
        let package = SnapPackage {
            name: String::from("spotify"),
            classic: false,
            channel: SnapChannel::Candidate,
        };
        let actual_res = package.args(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_some());
        let actual_args = actual_res.unwrap();
        assert_eq!(
            actual_args,
            vec![
                String::from("install"),
                String::from("spotify"),
                String::from("--candidate")
            ]
        );
    }

    #[test]
    fn test_package_edge_args() {
        let package = SnapPackage {
            name: String::from("spotify"),
            classic: false,
            channel: SnapChannel::Edge,
        };
        let actual_res = package.args(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_some());
        let actual_args = actual_res.unwrap();
        assert_eq!(
            actual_args,
            vec![
                String::from("install"),
                String::from("spotify"),
                String::from("--edge")
            ]
        );
    }

    #[test]
    fn test_package_needs_root() {
        let package = SnapPackage {
            name: String::from("spotify"),
            classic: false,
            channel: SnapChannel::Edge,
        };
        assert!(package.needs_root());
    }
}
