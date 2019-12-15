use crate::configuration::{Configuration, SnapPackage};
use crate::error::Result;

pub fn install_snap_packages(config: &Configuration) -> Result<()> {
    if let Some(snaps) = &config.snaps {
        let (standard, classic): (Vec<&SnapPackage>, Vec<&SnapPackage>) =
            snaps.packages.iter().partition(|snap| snap.classic);
    }
    Ok(())
}

fn process_standard_snaps(snaps: Vec<&SnapPackage>) -> Result<()> {
    Ok(())
}
