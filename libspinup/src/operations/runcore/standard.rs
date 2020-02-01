use std::process::{Command, Stdio};

use crate::error::{Error, Result};

#[allow(dead_code)]
pub fn internal_runner(command: &str, args: &[String]) -> Result<()> {
    let status = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args)
        .spawn()?
        .wait_with_output()?;

    handle_process_output(
        command,
        status.status.code(),
        &status.stdout,
        &status.stderr,
    )
}

/// Helper that will run `sudo -v` to obtain a prompt to enter a user's password.
/// As a session with sudo lasts ~15 minutes, the user's authentication for this should
/// serve for the entire time this application runs. Subsequent calls will not require
/// password entry if we're still within the time limit.
#[allow(dead_code)]
pub fn get_root() -> Result<()> {
    let exit_status = Command::new("sudo").arg("-v").spawn()?.wait()?;

    if exit_status.success() {
        Ok(())
    } else {
        Err("Unable to authenticate for sudo".into())
    }
}

/// Helper that handles the process output of any run commands and offers logging capabilities.
fn handle_process_output(
    cmd: &str,
    status: Option<i32>,
    stdout: &[u8],
    stderr: &[u8],
) -> Result<()> {
    if let Some(code) = status {
        if code == 0 {
            info!("Command execution of {} completed successfully", cmd);
            Ok(())
        } else {
            use log::{max_level, LevelFilter};
            warn!("Command execution of {} returned status of {}", cmd, code);

            // Don't bother building trace output unless we're actually using it
            if max_level() == LevelFilter::Trace {
                let stdout = String::from_utf8_lossy(stdout);
                if stdout.len() > 0 {
                    debug!("Stdout: \n{}", stdout);
                }
                let stderr = String::from_utf8_lossy(stderr);
                if stderr.len() > 0 {
                    debug!("Stderr: \n{}", stderr);
                }
            }
            Err(Error::from(format!("Package manager returned status of '{}'.\nRun with higher verbosity to see more output", code)))
        }
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_output_ok_none_status() {
        let actual = handle_process_output("test", None, &[], &[]);
        assert!(actual.is_ok());
    }
}
