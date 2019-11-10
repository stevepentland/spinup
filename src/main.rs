#[macro_use]
extern crate lazy_static;

use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use sys_info;

lazy_static! {
    static ref KNOWN_ARCH_NAMES: Vec<&'static str> = vec!["manjaro", "arch"];
}

#[derive(Debug, Clone)]
enum SpinupError {
    ConfigurationReadError(String),
    SystemDetailsError,
}

#[derive(Debug, Deserialize)]
struct Configuration {
    packages: Option<Vec<String>>,
}

#[derive(Debug)]
struct SystemDetails {
    target_os: TargetOperatingSystem,
}

#[derive(Debug, PartialEq)]
enum TargetOperatingSystem {
    Arch,
    Unknown,
}

fn main() -> Result<(), SpinupError> {
    let details = extract_distro_details()?;
    let config = read_in_config("./data/sample.toml")?;
    println!("{:#?}", config);
    println!("{:#?}", details);
    Ok(())
}

fn read_in_config(config_path: &str) -> Result<Configuration, SpinupError> {
    if let Ok(mut file) = File::open(config_path) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            let config_r = toml::from_str::<Configuration>(&contents);
            if let Ok(config) = config_r {
                return Ok(config);
            }
        }
    }
    Err(SpinupError::ConfigurationReadError(String::from(
        config_path,
    )))
}

fn extract_distro_details() -> Result<SystemDetails, SpinupError> {
    if let Ok(os_release) = sys_info::linux_os_release() {
        let mut current_id: TargetOperatingSystem = TargetOperatingSystem::Unknown;
        if let Some(id) = os_release.id {
            if KNOWN_ARCH_NAMES.iter().any(|s| *s == id) {
                current_id = TargetOperatingSystem::Arch;
            }
        }
        if current_id == TargetOperatingSystem::Unknown {
            if let Some(id_like) = os_release.id_like {
                if id_like
                    .split(" ")
                    .any(|s| KNOWN_ARCH_NAMES.iter().any(|v| *v == s))
                {
                    current_id = TargetOperatingSystem::Arch;
                }
            }
        }
        return Ok(SystemDetails {
            target_os: current_id,
        });
    }
    Err(SpinupError::SystemDetailsError)
}

/* TODO:
STEPS:
- Check environment for install methods, current distro
- read in instructions file
- run commands

NEEDS:
- Config files in various formats, start with toml
- Structure to represent multiple different commands
- Custom commands
- Start with arch, move from there
- Actually organize the code
*/
