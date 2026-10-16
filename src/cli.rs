//! The command-line surface of the application.
//!
//! This module owns exactly one thing: translating argv into a typed
//! [`Command`]. It contains no behaviour — it never touches the tray, the
//! registry or the file system. It does, however, know how to map each verb onto
//! a core [`UseCase`], which keeps that wiring in the driving adapter rather than
//! in the policy.

use clap::{Parser, Subcommand};

use crate::app::UseCase;

/// CLI entrypoint for `tray-cli`.
///
/// Running the binary with no subcommand opens the interactive dashboard. The
/// long-lived tray process is started with an explicit `run`, which is what the
/// auto-start registration points at.
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

    /// If omitted, the interactive dashboard opens.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// The set of verbs the CLI understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Open the interactive terminal dashboard (the default).
    Tui,

    /// Run the tray application in the foreground.
    Run,

    /// Register the app to start automatically at login.
    EnableAutostart,

    /// Remove the app from the login startup items.
    DisableAutostart,

    /// Print whether auto-start is currently enabled.
    Status,
}

impl Command {
    /// The core use case this verb triggers, or `None` for presentation-only
    /// verbs such as [`Command::Tui`].
    ///
    /// The dashboard is a driving adapter, not a use case, so it is dispatched by
    /// the composition root rather than by [`crate::app::Application`].
    #[must_use]
    pub fn use_case(self) -> Option<UseCase> {
        match self {
            Command::Tui => None,
            Command::Run => Some(UseCase::Run),
            Command::EnableAutostart => Some(UseCase::EnableAutoStart),
            Command::DisableAutostart => Some(UseCase::DisableAutoStart),
            Command::Status => Some(UseCase::ReportStatus),
        }
    }
}

impl Cli {
    /// The effective command: the parsed subcommand or [`Command::Tui`].
    #[must_use]
    pub fn command_or_default(&self) -> Command {
        self.command.unwrap_or(Command::Tui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn no_arguments_opens_the_dashboard() {
        let cli = Cli::parse_from(["tray-cli"]);
        assert_eq!(cli.command, None);
        assert_eq!(cli.command_or_default(), Command::Tui);
    }

    #[test]
    fn explicit_subcommands_parse() {
        let cases = [
            ("tui", Command::Tui),
            ("run", Command::Run),
            ("enable-autostart", Command::EnableAutostart),
            ("disable-autostart", Command::DisableAutostart),
            ("status", Command::Status),
        ];

        for (arg, expected) in cases {
            assert_eq!(
                Cli::parse_from(["tray-cli", arg]).command,
                Some(expected),
                "parsing `{arg}`"
            );
        }
    }

    #[test]
    fn dashboard_has_no_use_case_but_the_rest_do() {
        assert_eq!(Command::Tui.use_case(), None);
        assert_eq!(Command::Run.use_case(), Some(UseCase::Run));
        assert_eq!(
            Command::EnableAutostart.use_case(),
            Some(UseCase::EnableAutoStart)
        );
        assert_eq!(
            Command::DisableAutostart.use_case(),
            Some(UseCase::DisableAutoStart)
        );
        assert_eq!(Command::Status.use_case(), Some(UseCase::ReportStatus));
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
