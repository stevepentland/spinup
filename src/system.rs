use std::collections::HashMap;
use sys_info;

use crate::error::SpinupError;

lazy_static! {
    static ref SUPPORTED_OS_VERSIONS: HashMap<&'static str, TargetOperatingSystem> = {
        let mut h = HashMap::new();
        h.insert("manjaro", TargetOperatingSystem::Arch);
        h.insert("arch", TargetOperatingSystem::Arch);
        h.insert("debian", TargetOperatingSystem::Debian);
        h.insert("ubuntu", TargetOperatingSystem::Ubuntu);
        h.insert("linuxmint", TargetOperatingSystem::Mint);
        h
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackageManager {
    pub name: String,
    pub install_subcommand: Option<String>,
    pub autoconfirm: Option<String>,
}

impl PackageManager {
    pub fn new(name: &str, install_subcommand: Option<&str>, autoconfirm: Option<&str>) -> Self {
        PackageManager {
            name: String::from(name),
            install_subcommand: install_subcommand.map(String::from),
            autoconfirm: autoconfirm.map(String::from),
        }
    }
}

impl From<TargetOperatingSystem> for Option<PackageManager> {
    fn from(target_os: TargetOperatingSystem) -> Self {
        match target_os {
            TargetOperatingSystem::Arch => Some(PackageManager::new(
                "pacman",
                Some("-S"),
                Some("--noconfirm"),
            )),
            TargetOperatingSystem::Debian
            | TargetOperatingSystem::Ubuntu
            | TargetOperatingSystem::Mint => {
                Some(PackageManager::new("apt-get", Some("install"), Some("-y")))
            }
            TargetOperatingSystem::Unknown => None,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub enum TargetOperatingSystem {
    Arch,
    Debian,
    Ubuntu,
    Mint,
    Unknown,
}

#[derive(Debug)]
pub struct SystemDetails {
    target_os: TargetOperatingSystem,
}

impl SystemDetails {
    pub fn new(target_os: TargetOperatingSystem) -> Self {
        SystemDetails { target_os }
    }

    pub fn package_manager(&self) -> Option<PackageManager> {
        self.target_os.into()
    }

    pub fn current_os(&self) -> TargetOperatingSystem {
        self.target_os
    }
}

impl From<sys_info::LinuxOSReleaseInfo> for SystemDetails {
    fn from(info: sys_info::LinuxOSReleaseInfo) -> Self {
        let mut current_id: TargetOperatingSystem = TargetOperatingSystem::Unknown;
        if let Some(id) = info.id {
            if let Some(ver) = SUPPORTED_OS_VERSIONS.get(&id[..]) {
                current_id = *ver;
            }
        }
        if current_id == TargetOperatingSystem::Unknown {
            if let Some(id_like) = info.id_like {
                for id in id_like.split(' ') {
                    if let Some(i) = SUPPORTED_OS_VERSIONS.get(id) {
                        current_id = *i;
                        // We found one, no need to continue
                        break;
                    }
                }
            }
        }
        SystemDetails::new(current_id)
    }
}

pub fn extract_distro_details() -> Result<SystemDetails, SpinupError> {
    if let Ok(os_release) = sys_info::linux_os_release() {
        Ok(SystemDetails::from(os_release))
    } else {
        Err(SpinupError::SystemDetailsError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paste;

    macro_rules! os_value_tests {
        ($(($name:ident, $x:expr, $y:expr, $exp:expr));+) => {
            $(
                paste::item!(
                    #[test]
                    fn [<test_release_info_ $name>]() {
                        let info = sys_info::LinuxOSReleaseInfo {
                            id: $x,
                            id_like: $y,
                            name: None,
                            pretty_name: None,
                            version: None,
                            version_id: None,
                            version_codename: None,
                            ansi_color: None,
                            cpe_name: None,
                            build_id: None,
                            variant: None,
                            variant_id: None,
                            home_url: None,
                            bug_report_url: None,
                            support_url: None,
                            documentation_url: None,
                            logo: None,
                        };
                        let sd = SystemDetails::from(info);
                        assert_eq!($exp, sd.target_os)
                    }
                );
            )*
        };
    }

    os_value_tests!(
        (
            arch,
            Some(String::from("arch")),
            None,
            TargetOperatingSystem::Arch
        );
        (
            manjaro,
            Some(String::from("manjaro")),
            None,
            TargetOperatingSystem::Arch
        );
        (
            id_like_arch,
            None,
            Some(String::from("arch")),
            TargetOperatingSystem::Arch
        );
        (
            id_like_manjaro,
            None,
            Some(String::from("manjaro")),
            TargetOperatingSystem::Arch
        );
        (all_none, None, None, TargetOperatingSystem::Unknown);
        (
            unknown_id_arch_id_like,
            Some(String::from("bsd")),
            Some(String::from("arch")),
            TargetOperatingSystem::Arch
        );
        (
            both_unknown,
            Some(String::from("bsd")),
            Some(String::from("mac")),
            TargetOperatingSystem::Unknown
        );
        (
            ubuntu,
            Some(String::from("ubuntu")),
            None,
            TargetOperatingSystem::Ubuntu
        );
        (
            ubuntu_no_fallback_to_id_like,
            Some(String::from("ubuntu")),
            Some(String::from("debian")),
            TargetOperatingSystem::Ubuntu
        );
        (
            debian_from_id,
            Some(String::from("debian")),
            None,
            TargetOperatingSystem::Debian
        );
        (
            debian_from_id_like,
            Some(String::from("somethingunknown")),
            Some(String::from("debian")),
            TargetOperatingSystem::Debian
        );
        (
            mint,
            Some(String::from("linuxmint")),
            Some(String::from("ubuntu")),
            TargetOperatingSystem::Mint
        )
    );

    macro_rules! package_manager_tests {
        ($(($name:ident, $target:expr, $expected:expr));+) => {
            $(
                paste::item!(
                    #[test]
                    fn [<test_package_manager_from_ $name>]() {
                        let actual: Option<PackageManager> = $target.into();
                        assert_eq!($expected, actual);
                    }
                );
            )*
        };
    }

    package_manager_tests!(
        (
            arch,
            TargetOperatingSystem::Arch,
            Some(PackageManager::new(
                "pacman",
                Some("-S"),
                Some("--noconfirm")
            ))
        );
        (
            debian,
            TargetOperatingSystem::Debian,
            Some(PackageManager::new(
                "apt-get",
                Some("install"),
                Some("-y")
            ))
        );
        (
            ubuntu,
            TargetOperatingSystem::Ubuntu,
            Some(PackageManager::new(
                "apt-get",
                Some("install"),
                Some("-y")
            ))
        );
        (
            mint,
            TargetOperatingSystem::Mint,
            Some(PackageManager::new(
                "apt-get", 
                Some("install"), 
                Some("-y")
            ))
        );
        (
            unknown,
            TargetOperatingSystem::Unknown,
            None
        )
    );

    #[test]
    fn test_target_os_set() {
        let expected = TargetOperatingSystem::Arch;
        let actual = SystemDetails::new(expected);
        assert_eq!(expected, actual.target_os);
    }

    #[test]
    fn test_current_os_reflects_target() {
        let expected = TargetOperatingSystem::Ubuntu;
        let actual = SystemDetails::new(expected);
        assert_eq!(expected, actual.current_os());
    }
}
