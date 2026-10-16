//! Ask the OS to open a path or URL, and to run a shell command.
//!
//! Two different callers need the same "hand this to the OS" primitive: the
//! tray menu's custom entries (which can run a command or open a target) and the
//! dashboard's *Open log folder*. It lives here once so neither reimplements it,
//! and so downstream apps can reuse it.

use std::process::{Command, Stdio};

use crate::error::AppResult;

/// Open `target` with the platform's default handler — a directory, a file, or
/// a URL.
///
/// # Errors
/// Returns [`crate::error::AppError::Io`] if the launcher process cannot be
/// spawned.
pub fn open(target: &str) -> AppResult<()> {
    let mut command = open_command(target);
    null_stdio(&mut command);
    command.spawn()?;
    Ok(())
}

/// Run `command` through the platform shell (`cmd /C` on Windows, `sh -c`
/// elsewhere), so a CLI-supplied command line behaves the way users expect.
///
/// # Errors
/// Returns [`crate::error::AppError::Io`] if the shell cannot be spawned.
pub fn shell(command: &str) -> AppResult<()> {
    let mut command = shell_command(command);
    null_stdio(&mut command);
    command.spawn()?;
    Ok(())
}

/// Stop inherited handles from leaking into the spawned child, so it cannot
/// scribble over the dashboard's alternate screen.
fn null_stdio(command: &mut Command) {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
}

/// `start` treats its first quoted argument as the window title, so an empty one
/// is passed to make targets containing spaces work.
#[cfg(windows)]
fn open_command(target: &str) -> Command {
    let mut command = Command::new("cmd");
    command.args(["/C", "start", ""]).arg(target);
    command
}

#[cfg(target_os = "macos")]
fn open_command(target: &str) -> Command {
    let mut command = Command::new("open");
    command.arg(target);
    command
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_command(target: &str) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(target);
    command
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("cmd");
    shell.args(["/C", command]);
    shell
}

#[cfg(not(windows))]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("/bin/sh");
    shell.args(["-c", command]);
    shell
}
