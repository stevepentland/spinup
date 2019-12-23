use crate::configuration::Configuration;
use crate::error::Result;

use super::run_command;

pub fn install_snap_packages(config: &Configuration) -> Result<()> {
    match &config.snaps {
        Some(snaps) => {
            run_command(&snaps.standard_snaps, config.system_details)?;

            if let Some(additional_snaps) = &snaps.alternate_snaps {
                for snap in additional_snaps {
                    run_command(snap, config.system_details)?;
                }
            }
            Ok(())
        }
        None => Ok(()),
    }
}
