use std::fmt;

use serde::{Deserialize, Serialize};

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
            SnapChannel::Stable => "",
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Snaps {
    pub packages: Vec<SnapPackage>,
}

impl Snaps {
    pub fn standard_install_string(&self) -> Option<String> {
        let base = self
            .packages
            .iter()
            .filter(|pkg| !pkg.classic && pkg.channel == SnapChannel::Stable)
            .map(|pkg| format!("{}", pkg))
            .collect::<Vec<String>>()
            .join(" ");

        let names = base.trim();

        if names.len() > 0 {
            Some(String::from(names))
        } else {
            None
        }
    }

    pub fn individual_snap_install_strings(&self) -> Vec<String> {
        self.packages
            .iter()
            .filter(|pkg| pkg.classic || pkg.channel != SnapChannel::Stable)
            .map(|pkg| format!("{}", pkg))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snaps_data() -> Snaps {
        Snaps {
            packages: vec![
                SnapPackage {
                    name: String::from("code-insiders"),
                    classic: true,
                    channel: SnapChannel::Stable,
                },
                SnapPackage {
                    name: String::from("mailspring"),
                    classic: false,
                    channel: SnapChannel::Stable,
                },
                SnapPackage {
                    name: String::from("postman"),
                    classic: false,
                    channel: SnapChannel::Stable,
                },
                SnapPackage {
                    name: String::from("powershell"),
                    classic: true,
                    channel: SnapChannel::Beta,
                },
                SnapPackage {
                    name: String::from("wormhole"),
                    classic: false,
                    channel: SnapChannel::Beta,
                },
                SnapPackage {
                    name: String::from("makemkv"),
                    classic: false,
                    channel: SnapChannel::Candidate,
                },
                SnapPackage {
                    name: String::from("shotcut"),
                    classic: true,
                    channel: SnapChannel::Candidate,
                },
                SnapPackage {
                    name: String::from("darktable"),
                    classic: false,
                    channel: SnapChannel::Edge,
                },
                SnapPackage {
                    name: String::from("sublime-text"),
                    classic: true,
                    channel: SnapChannel::Edge,
                },
            ],
        }
    }

    #[test]
    fn test_build_standard_snap_names() {
        let expected = String::from("mailspring postman");
        let actual = snaps_data().standard_install_string();
        assert!(actual.is_some());
        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn test_build_other_snap_install_strings() {
        let expected = vec![
            String::from("--classic code-insiders"),
            String::from("--classic --beta powershell"),
            String::from("--beta wormhole"),
            String::from("--candidate makemkv"),
            String::from("--classic --candidate shotcut"),
            String::from("--edge darktable"),
            String::from("--classic --edge sublime-text"),
        ];
        let actual = snaps_data().individual_snap_install_strings();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_build_standard_no_snaps() {
        let subject = Snaps {
            packages: vec![SnapPackage {
                name: String::from("darktable"),
                classic: false,
                channel: SnapChannel::Edge,
            }],
        };
        let actual = subject.standard_install_string();
        assert_eq!(actual, None);
    }

    #[test]
    fn test_build_others_no_snaps() {
        let subject = Snaps {
            packages: vec![SnapPackage {
                name: String::from("darktable"),
                classic: false,
                channel: SnapChannel::Stable,
            }],
        };
        let actual = subject.individual_snap_install_strings();
        assert_eq!(actual.len(), 0);
    }
}
