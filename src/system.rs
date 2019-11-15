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

#[derive(Debug, PartialEq, Copy, Clone)]
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
    macro_rules! os_info {
        ( $x:expr, $y:expr ) => {
            sys_info::LinuxOSReleaseInfo {
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
        };
    }

    macro_rules! os_val_test {
        ( $name:ident, $x:expr, $y:expr, $exp:expr ) => {
            #[test]
            fn $name() {
                let info = os_info!($x, $y);
                let sd = SystemDetails::from(info);
                assert_eq!($exp, sd.target_os)
            }
        };
        ( $name:ident, $x:expr, $exp:expr ) => {
            #[test]
            fn $name() {
                let info = os_info!($x, None);
                let sd = SystemDetails::from(info);
                assert_eq!($exp, sd.target_os)
            }
        };
    }

    os_val_test!(
        test_release_info_arch,
        Some(String::from("arch")),
        TargetOperatingSystem::Arch
    );
    os_val_test!(
        test_release_info_manjaro,
        Some(String::from("manjaro")),
        TargetOperatingSystem::Arch
    );
    os_val_test!(
        test_release_info_id_like_arch,
        None,
        Some(String::from("arch")),
        TargetOperatingSystem::Arch
    );
    os_val_test!(
        test_release_info_id_like_manjaro,
        None,
        Some(String::from("manjaro")),
        TargetOperatingSystem::Arch
    );
    os_val_test!(
        test_release_info_all_none,
        None,
        TargetOperatingSystem::Unknown
    );
    os_val_test!(
        test_release_info_unknown_id_arch_id_like,
        Some(String::from("bsd")),
        Some(String::from("arch")),
        TargetOperatingSystem::Arch
    );
    os_val_test!(
        test_release_info_both_unknown,
        Some(String::from("bsd")),
        Some(String::from("mac")),
        TargetOperatingSystem::Unknown
    );
}
