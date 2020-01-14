use crate::configuration::{CommandSet, Configuration};
use crate::error::Result;

use super::run_command;

/// Run the custom commands and command sets that are in the configuration
pub fn run_custom_commands(config: &Configuration) -> Result<()> {
    if let Some(commands) = &config.custom_commands {
        commands
            .iter()
            .map(|c| run_command(c, config.system_details))
            .collect::<Result<Vec<()>>>()
            .map(|_| ())?;
    }

    if let Some(command_sets) = &config.command_sets {
        for command_set in command_sets {
            run_collected_commands(command_set, config)?;
        }
    }

    Ok(())
}

fn run_collected_commands(command_set: &CommandSet, config: &Configuration) -> Result<()> {
    command_set
        .get_runnable_commands()
        .iter()
        .map(|c| run_command(c, config.system_details))
        .collect::<Result<Vec<_>>>()
        .map(|_| ())
}
