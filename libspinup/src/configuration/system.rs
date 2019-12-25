//! The system configuration module contains definitions and implementations
//! for dealing with the host operating system. This includes package managers,
//! update & upgrade commands, etc.

use sys_info;

use crate::error::Result;
use crate::operations::RunnableOperation;

/// Defines the set of commands required to interact with the
/// package manager of the host OS.
#[derive(Debug, Clone, PartialEq)]
pub struct PackageManager {
    /// This is the name of the package manager (e.g. `apt-get`, `pacman`)
    name: String,

    /// The command passed to [`name`](struct.PackageManager.html#structfield.name) to install packages
    install_subcommand: String,

    /// The command passed to [`name`](struct.PackageManager.html#structfield.name) to update package lists
    update_subcommand: String,

    /// The command passed to [`name`](struct.PackageManager.html#structfield.name) to upgrade system packages
    upgrade_subcommand: String,

    /// The argument that will bypass confirmation requests
    autoconfirm: String,
}

/// Internal struct used to make runnable operations on the fly
#[derive(Debug, Clone)]
struct SystemRefreshOperation {
    /// The command to run, generally will be the backing [`name`](struct.PackageManager.html#structfield.name)
    pub command_name: String,

    /// The subcommand, which is generally [`update_subcommand`](struct.PackageManager.html#structfield.update_subcommand)
    /// or [`upgrade_subcommand`](struct.PackageManager.html#structfield.upgrade_subcommand)
    pub target_subcommand: String,

    /// The autoconfirm argument value
    pub autoconfirm: String,
}

impl RunnableOperation for SystemRefreshOperation {
    fn command_name(&self, _system_details: SystemDetails) -> Result<String> {
        Ok(self.command_name.clone())
    }

    fn args(&self, _system_details: SystemDetails) -> Option<Vec<String>> {
        Some(vec![
            self.target_subcommand.clone(),
            self.autoconfirm.clone(),
        ])
    }

    fn needs_root(&self) -> bool {
        true
    }
}

impl PackageManager {
    fn new(
        name: &str,
        install_subcommand: &str,
        update_subcommand: &str,
        upgrade_subcommand: &str,
        autoconfirm: &str,
    ) -> Self {
        PackageManager {
            name: name.to_string(),
            install_subcommand: install_subcommand.to_string(),
            update_subcommand: update_subcommand.to_string(),
            upgrade_subcommand: upgrade_subcommand.to_string(),
            autoconfirm: autoconfirm.to_string(),
        }
    }

    /// Get the system update operation for this package manager
    pub fn update_operation(&self) -> impl RunnableOperation {
        SystemRefreshOperation {
            command_name: self.name.clone(),
            target_subcommand: self.update_subcommand.clone(),
            autoconfirm: self.autoconfirm.clone(),
        }
    }

    /// Get the system upgrade operation for this package manager
    pub fn upgrade_operation(&self) -> impl RunnableOperation {
        SystemRefreshOperation {
            command_name: self.name.clone(),
            target_subcommand: self.upgrade_subcommand.clone(),
            autoconfirm: self.autoconfirm.clone(),
        }
    }

    /// This is the name of the package manager (e.g. `apt-get`, `pacman`)
    pub fn name(&self) -> Option<String> {
        if self.name.is_empty() {
            None
        } else {
            Some(self.name.clone())
        }
    }

    /// The command passed to [`name`](struct.PackageManager.html#method.name) to install packages
    pub fn install_subcommand(&self) -> Option<String> {
        if self.install_subcommand.is_empty() {
            None
        } else {
            Some(self.install_subcommand.clone())
        }
    }

    /// The argument that will bypass confirmation requests
    pub fn autoconfirm(&self) -> Option<String> {
        if self.autoconfirm.is_empty() {
            None
        } else {
            Some(self.autoconfirm.clone())
        }
    }

    /// Whether this package manager has a configured setup
    pub fn can_run(&self) -> bool {
        self.name().is_some()
    }
}

impl From<TargetOperatingSystem> for PackageManager {
    fn from(target_os: TargetOperatingSystem) -> Self {
        match target_os {
            TargetOperatingSystem::Arch => {
                PackageManager::new("pacman", "-S", "-Sy", "-Syu", "--noconfirm")
            }
            TargetOperatingSystem::Debian => {
                PackageManager::new("apt-get", "install", "update", "upgrade", "-y")
            }
            TargetOperatingSystem::Unknown => PackageManager::new("", "", "", "", ""),
        }
    }
}

/// An identified operating system
#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub enum TargetOperatingSystem {
    Arch,
    Debian,
    Unknown,
}

/// Collection of details for the current host system
#[derive(Debug, Copy, Clone)]
pub struct SystemDetails {
    target_os: TargetOperatingSystem,
}

impl SystemDetails {
    pub fn new(target_os: TargetOperatingSystem) -> Self {
        SystemDetails { target_os }
    }

    pub fn package_manager(self) -> PackageManager {
        PackageManager::from(self.target_os)
    }

    pub fn current_os(self) -> TargetOperatingSystem {
        self.target_os
    }
}

impl From<sys_info::LinuxOSReleaseInfo> for SystemDetails {
    fn from(info: sys_info::LinuxOSReleaseInfo) -> Self {
        let mut current_id: TargetOperatingSystem = TargetOperatingSystem::Unknown;
        if let Some(id) = info.id {
            current_id = TargetOperatingSystem::from(&id[..])
        }
        if current_id == TargetOperatingSystem::Unknown {
            if let Some(id_like) = info.id_like {
                if let Some(target) = id_like.split(' ').find(|&name| {
                    TargetOperatingSystem::from(name) != TargetOperatingSystem::Unknown
                }) {
                    current_id = TargetOperatingSystem::from(target);
                }
            }
        }
        SystemDetails::new(current_id)
    }
}

impl From<&str> for TargetOperatingSystem {
    fn from(name: &str) -> Self {
        match &name.to_lowercase()[..] {
            "arch" | "archlinux" | "manjaro" => TargetOperatingSystem::Arch,
            "debian" | "linuxmint" | "mint" | "ubuntu" => TargetOperatingSystem::Debian,
            _ => TargetOperatingSystem::Unknown,
        }
    }
}

impl Default for SystemDetails {
    fn default() -> Self {
        match sys_info::linux_os_release() {
            Ok(release) => SystemDetails::from(release),
            Err(_) => SystemDetails::new(TargetOperatingSystem::Unknown),
        }
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
            TargetOperatingSystem::Debian
        );
        (
            ubuntu_no_fallback_to_id_like,
            Some(String::from("ubuntu")),
            Some(String::from("debian")),
            TargetOperatingSystem::Debian
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
            TargetOperatingSystem::Debian
        )
    );

    macro_rules! package_manager_tests {
        ($(($name:ident, $target:expr, $expected:expr, $can_run:expr));+) => {
            $(
                paste::item!(
                    #[test]
                    fn [<test_package_manager_from_ $name>]() {
                        let actual: PackageManager = $target.into();
                        assert_eq!($expected, actual);
                        assert_eq!($can_run, actual.can_run())
                    }
                );
            )*
        };
    }

    package_manager_tests!(
        (
            arch,
            TargetOperatingSystem::Arch,
            PackageManager::new(
                "pacman",
                "-S",
                "-Sy",
                "-Syu",
                "--noconfirm",
            ),
            true
        );
        (
            debian,
            TargetOperatingSystem::Debian,
            PackageManager::new(
                "apt-get",
                "install",
                "update",
                "upgrade",
                "-y"
            ),
            true
        );
        (
            unknown,
            TargetOperatingSystem::Unknown,
            PackageManager::new(
                "",
                "",
                "",
                "",
                ""
            ),
            false
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
        let expected = TargetOperatingSystem::Debian;
        let actual = SystemDetails::new(expected);
        assert_eq!(expected, actual.current_os());
    }
}
