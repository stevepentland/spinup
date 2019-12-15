use crate::configuration::{Configuration, SnapPackage};
use crate::error::Result;

pub fn _install_snap_packages(config: &Configuration) -> Result<()> {
    if let Some(snaps) = &config.snaps {
        let (_standard, _classic): (Vec<&SnapPackage>, Vec<&SnapPackage>) =
            snaps.packages.iter().partition(|snap| snap.classic);
    }
    Ok(())
}

fn _process_standard_snaps(_snaps: Vec<&SnapPackage>) -> Result<()> {
    Ok(())
}
