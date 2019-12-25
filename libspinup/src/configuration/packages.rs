use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::operations::RunnableOperation;

use super::{SystemDetails, TargetOperatingSystem};

/// `DistroPackages` refer to collections of packages that may only
/// exist on individual distributions or that have differing names.
#[derive(Debug, Deserialize, Serialize)]
pub struct DistroPackages {
    /// The target OS that these packages should install on, such as:
    /// - debian
    /// - arch
    /// - ubuntu
    ///
    /// And so on.
    pub target_os: String,

    /// The names of the packages to install on the given distribution
    pub packages: Option<Vec<String>>,
}

impl DistroPackages {
    /// Helper to check whether there are any named packages in this struct
    fn has_packages(&self) -> bool {
        match &self.packages {
            Some(pkg) => !pkg.is_empty(),
            None => false,
        }
    }
}

/// The `PackageList` represents a batch of packages to install
/// via the distro's package manager. This is split into common ones
/// that usually have the same name across distros and distro specific
/// packages whose names vary across distributions.
#[derive(Debug, Deserialize, Serialize)]
pub struct PackageList {
    /// A collection of common packages that have the same name across distributions
    /// such as vim, git, etc.
    pub base_packages: Option<Vec<String>>,

    /// Packages that may only exist on particular distros, or those whose names
    /// change across distributions.
    pub distro_packages: Option<Vec<DistroPackages>>,
}

impl PackageList {
    /// Helper that checks whether there are any packages listed in the `base_packages` field
    fn has_base_packages(&self) -> bool {
        match &self.base_packages {
            Some(pkg) => !pkg.is_empty(),
            None => false,
        }
    }

    /// Helper that will indicate whether there are packages listed for the current Linux
    /// distribution in `distro_packages`
    fn has_distro_packages(&self, system_details: SystemDetails) -> bool {
        match &self.distro_packages {
            Some(pkgs) => pkgs.iter().any(|it| {
                TargetOperatingSystem::from(&it.target_os[..]) == system_details.current_os()
                    && it.has_packages()
            }),
            None => false,
        }
    }
}

impl RunnableOperation for PackageList {
    fn needs_root(&self) -> bool {
        true
    }

    fn command_name(&self, system_details: SystemDetails) -> Result<String> {
        if !self.has_base_packages() && !self.has_distro_packages(system_details) {
            return Err(Error::from(
                "Packages present in config, but no packages were listed",
            ));
        }

        system_details.package_manager().name().ok_or_else(|| {
            Error::from("Spinup does not have a package manager configuration for this platform")
        })
    }

    fn args(&self, system_details: SystemDetails) -> Option<Vec<String>> {
        if !self.has_base_packages() && !self.has_distro_packages(system_details) {
            return None;
        }

        let package_manager = system_details.package_manager();

        if !package_manager.can_run() {
            return None;
        }

        let mut install_args = vec![];
        if let Some(install_command) = package_manager.install_subcommand() {
            install_args.push(install_command);
        }
        if let Some(autoconfirm) = package_manager.autoconfirm() {
            install_args.push(autoconfirm);
        }

        if let Some(packages) = &self.base_packages {
            install_args.extend(packages.clone());
        }

        if let Some(distro_packages) = &self.distro_packages {
            if let Some(package_def) = distro_packages.iter().find(|it| {
                TargetOperatingSystem::from(&it.target_os[..]) == system_details.current_os()
                    && it.has_packages()
            }) {
                if let Some(packages) = &package_def.packages {
                    install_args.extend(packages.clone());
                }
            }
        }

        Some(install_args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::{SystemDetails, TargetOperatingSystem};

    #[test]
    fn test_package_install_needs_root() {
        let package_list = PackageList {
            base_packages: None,
            distro_packages: None,
        };

        assert!(package_list.needs_root());
    }

    #[test]
    fn test_unknown_system_is_err() {
        let package_list = PackageList {
            base_packages: Some(vec![String::from("git")]),
            distro_packages: None,
        };
        let actual_res =
            package_list.command_name(SystemDetails::new(TargetOperatingSystem::Unknown));
        assert!(actual_res.is_err());
    }

    #[test]
    fn test_command_name_err_no_packages() {
        let package_list = PackageList {
            base_packages: None,
            distro_packages: None,
        };
        let actual_res =
            package_list.command_name(SystemDetails::new(TargetOperatingSystem::Ubuntu));
        assert!(actual_res.is_err());
    }

    #[test]
    fn test_command_name_err_distro_package_none() {
        let package_list = PackageList {
            base_packages: None,
            distro_packages: Some(vec![DistroPackages {
                target_os: String::from("manjaro"),
                packages: None,
            }]),
        };
        let actual_res = package_list.command_name(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_err());
    }

    #[test]
    fn test_command_name_err_distro_package_empty() {
        let package_list = PackageList {
            base_packages: None,
            distro_packages: Some(vec![DistroPackages {
                target_os: String::from("ubuntu"),
                packages: Some(vec![]),
            }]),
        };
        let actual_res =
            package_list.command_name(SystemDetails::new(TargetOperatingSystem::Ubuntu));
        assert!(actual_res.is_err());
    }

    #[test]
    fn test_command_name_base_packages() {
        let package_list = PackageList {
            base_packages: Some(vec![String::from("git")]),
            distro_packages: None,
        };
        let actual_res =
            package_list.command_name(SystemDetails::new(TargetOperatingSystem::Debian));
        assert!(actual_res.is_ok());
        let actual = actual_res.unwrap();
        assert_eq!(actual, String::from("apt-get"));
    }

    #[test]
    fn test_command_name_distro_packages() {
        let package_list = PackageList {
            base_packages: None,
            distro_packages: Some(vec![DistroPackages {
                target_os: String::from("arch"),
                packages: Some(vec![String::from("git")]),
            }]),
        };
        let actual_res = package_list.command_name(SystemDetails::new(TargetOperatingSystem::Arch));
        assert!(actual_res.is_ok());
        let actual = actual_res.unwrap();
        assert_eq!(actual, String::from("pacman"));
    }
}
