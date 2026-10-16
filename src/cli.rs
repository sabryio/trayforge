//! The command-line surface of the application.
//!
//! This module owns exactly one thing: translating argv into a typed
//! [`Command`]. It contains no behaviour — it never touches the tray, the
//! registry or the file system.

use clap::{Parser, Subcommand};

/// CLI entrypoint for `tray-cli`.
///
/// Running the binary with no subcommand is equivalent to `tray-cli run`, which
/// is important because auto-start launchers usually execute the bare
/// executable with no arguments.
#[derive(Debug, Parser)]
#[command(
    name = "tray-cli",
    version,
    about = "CLI background app with a tray icon and auto-start support",
    long_about = None,
)]
pub struct Cli {
    /// Increase log verbosity (`-v` = debug, `-vv` = trace).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// If omitted, the app runs the tray process.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// The set of use cases the CLI can trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Run the tray application (the default when no subcommand is given).
    Run,

    /// Register the app to start automatically at login.
    EnableAutostart,

    /// Remove the app from the login startup items.
    DisableAutostart,

    /// Print whether auto-start is currently enabled.
    Status,
}

impl Cli {
    /// The effective command: the parsed subcommand or [`Command::Run`].
    #[must_use]
    pub fn command_or_default(&self) -> Command {
        self.command.unwrap_or(Command::Run)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn no_arguments_defaults_to_run() {
        let cli = Cli::parse_from(["tray-cli"]);
        assert_eq!(cli.command, None);
        assert_eq!(cli.command_or_default(), Command::Run);
    }

    #[test]
    fn explicit_run_subcommand_parses() {
        let cli = Cli::parse_from(["tray-cli", "run"]);
        assert_eq!(cli.command, Some(Command::Run));
    }

    #[test]
    fn autostart_subcommands_parse() {
        assert_eq!(
            Cli::parse_from(["tray-cli", "enable-autostart"]).command,
            Some(Command::EnableAutostart)
        );
        assert_eq!(
            Cli::parse_from(["tray-cli", "disable-autostart"]).command,
            Some(Command::DisableAutostart)
        );
        assert_eq!(
            Cli::parse_from(["tray-cli", "status"]).command,
            Some(Command::Status)
        );
    }

    #[test]
    fn verbose_flag_counts_occurrences() {
        assert_eq!(Cli::parse_from(["tray-cli", "-vv", "status"]).verbose, 2);
    }

    #[test]
    fn unknown_subcommand_is_rejected() {
        assert!(Cli::try_parse_from(["tray-cli", "not-a-command"]).is_err());
    }
}
