//! Testing mock for running commands. When we're in test, this can be used
//! as the substitute for building `Command` objects and executing shell commands.
//!
//! The values in here will be populated with the call args (hopefully...)
use crate::error::Result;

static mut TEST_VALUES: RunnerContainer = RunnerContainer::new();

pub fn testing_runner(command: &str, args: &[String]) -> Result<()> {
    unsafe {
        TEST_VALUES.command = Some(command.into());
        TEST_VALUES.args = Some(args.into());
    }
    Ok(())
}

struct RunnerContainer {
    command: Option<String>,
    args: Option<Vec<String>>,
}

impl RunnerContainer {
    const fn new() -> Self {
        RunnerContainer {
            command: None,
            args: None,
        }
    }
}

pub fn passed_command() -> Option<String> {
    unsafe { TEST_VALUES.command.clone() }
}

pub fn passed_args() -> Option<Vec<String>> {
    unsafe { TEST_VALUES.args.clone() }
}

pub fn reset() {
    unsafe {
        TEST_VALUES = RunnerContainer::new();
    }
}
