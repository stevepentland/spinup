use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use sys_info;

fn main() -> Result<(), sys_info::Error> {
    let distro = sys_info::linux_os_release()?;
    let release = sys_info::os_release()?;
    println!("{}: {}", distro.pretty_name.unwrap(), distro.name.unwrap());
    println!("{}", release);

    let config = read_in_config().unwrap();
    println!("{:#?}", config);
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Configuration {
    packages: Option<Vec<String>>,
}

fn read_in_config() -> Result<Configuration, std::io::Error> {
    let mut file = File::open("./data/sample.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config = toml::from_str::<Configuration>(&contents);

    if let Ok(cfg) = config {
        return Ok(cfg);
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Could not read configuration file",
    ))
}

/*
STEPS:
- Check environment for install methods, current distro
- read in instructions file
- run commands

NEEDS:
- Config files in various formats, start with toml
- Structure to represent multiple different commands
- Custom commands
- Start with arch, move from there
*/
