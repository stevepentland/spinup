use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::operations::RunnableOperation;

use super::{SystemDetails, TargetOperatingSystem};

#[derive(Debug, Deserialize, Serialize)]
pub struct DistroPackages {
    pub target_os: String,
    pub packages: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageList {
    pub base_packages: Option<Vec<String>>,
    pub distro_packages: Option<Vec<DistroPackages>>,
}

impl RunnableOperation for &PackageList {
    fn needs_root(&self) -> bool {
        true
    }

    fn command_name(&self, system_details: SystemDetails) -> Result<String> {
        system_details
            .package_manager()
            .map(|pm| pm.name)
            .ok_or_else(|| Error::from("This system has no package manager"))
    }

    fn args(&self, system_details: SystemDetails) -> Result<Vec<String>> {
        let package_manager = system_details
            .package_manager()
            .ok_or_else(|| Error::from("This system has no package manager"))?;

        let mut install_args = vec![];
        if let Some(install_command) = package_manager.install_subcommand {
            install_args.push(install_command);
        }
        if let Some(autoconfirm) = package_manager.autoconfirm {
            install_args.push(autoconfirm);
        }

        if let Some(packages) = &self.base_packages {
            install_args.extend(packages.clone());
        }

        if let Some(distro_packages) = &self.distro_packages {
            if let Some(package_def) = distro_packages.iter().find(|it| {
                TargetOperatingSystem::from(&it.target_os[..]) == system_details.current_os()
            }) {
                if let Some(packages) = &package_def.packages {
                    install_args.extend(packages.clone());
                }
            }
        }

        Ok(install_args)
    }
}
