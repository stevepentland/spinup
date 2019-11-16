use std::collections::HashMap;
use sys_info;

use crate::error::SpinupError;

lazy_static! {
    static ref SUPPORTED_OS_VERSIONS: HashMap<&'static str, TargetOperatingSystem> = {
        let mut h = HashMap::new();
        h.insert("manjaro", TargetOperatingSystem::Arch);
        h.insert("arch", TargetOperatingSystem::Arch);
        h
    };
}

lazy_static! {
    static ref PACKAGE_MANAGERS: HashMap<TargetOperatingSystem, Option<PackageManager>> = {
        let mut h = HashMap::new();
        h.insert(
            TargetOperatingSystem::Arch,
            Some(PackageManager::new(
                "pacman",
                Some("-S"),
                Some("--noconfirm"),
            )),
        );
        h.insert(TargetOperatingSystem::Unknown, None);
        h
    };
}

#[derive(Debug, Clone)]
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

#[derive(Debug, PartialEq, Copy, Clone, Eq, std::hash::Hash)]
pub enum TargetOperatingSystem {
    Arch,
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

    #[allow(dead_code)]
    pub fn package_manager(&self) -> Option<&PackageManager> {
        match PACKAGE_MANAGERS.get(&self.target_os) {
            Some(pm) => pm.as_ref(),
            None => None,
        }
    }
    // pub fn is_supported(&self) -> bool {
    //     self.target_os != TargetOperatingSystem::Unknown
    // }

    // pub fn current_os(&self) -> TargetOperatingSystem {
    //     self.target_os
    // }
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
    macro_rules! os_val_test {
        ( $name:ident, $x:expr, $y:expr, $exp:expr ) => {
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
        };
        ( $name:ident, $x:expr, $exp:expr ) => {
            os_val_test!($name, $x, None, $exp);
        };
    }

    os_val_test!(
        arch,
        Some(String::from("arch")),
        TargetOperatingSystem::Arch
    );
    os_val_test!(
        manjaro,
        Some(String::from("manjaro")),
        TargetOperatingSystem::Arch
    );
    os_val_test!(
        id_like_arch,
        None,
        Some(String::from("arch")),
        TargetOperatingSystem::Arch
    );
    os_val_test!(
        id_like_manjaro,
        None,
        Some(String::from("manjaro")),
        TargetOperatingSystem::Arch
    );
    os_val_test!(all_none, None, TargetOperatingSystem::Unknown);
    os_val_test!(
        unknown_id_arch_id_like,
        Some(String::from("bsd")),
        Some(String::from("arch")),
        TargetOperatingSystem::Arch
    );
    os_val_test!(
        both_unknown,
        Some(String::from("bsd")),
        Some(String::from("mac")),
        TargetOperatingSystem::Unknown
    );
}
