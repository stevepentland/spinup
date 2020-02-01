//! Testing mock for running commands. When we're in test, this can be used
//! as the substitute for building `Command` objects and executing shell commands.
//!
//! The values in here will be populated with the call args (hopefully...)
//!
//! I don't really like this though, the static values container means that the
//! tests can't be run on more than one thread. I need to find a better way to create
//! a test context.
#![allow(dead_code)]
#![cfg_attr(tarpaulin, skip)]

use crate::error::Result;

static mut TEST_VALUES: RunnerContainer = RunnerContainer::new();

pub fn internal_runner(command: &str, args: &[String]) -> Result<()> {
    unsafe {
        TEST_VALUES.command = Some(command.into());
        TEST_VALUES.args = Some(args.into());
    }
    Ok(())
}

pub fn get_root() -> Result<()> {
    unsafe {
        TEST_VALUES.root_called = true;
    }
    Ok(())
}

struct RunnerContainer {
    command: Option<String>,
    args: Option<Vec<String>>,
    root_called: bool,
}

impl RunnerContainer {
    const fn new() -> Self {
        RunnerContainer {
            command: None,
            args: None,
            root_called: false,
        }
    }
}

pub fn passed_command() -> Option<String> {
    unsafe { TEST_VALUES.command.clone() }
}

pub fn passed_args() -> Option<Vec<String>> {
    unsafe { TEST_VALUES.args.clone() }
}

pub fn called_root() -> bool {
    unsafe { TEST_VALUES.root_called }
}

pub fn reset() {
    unsafe {
        TEST_VALUES = RunnerContainer::new();
    }
}
