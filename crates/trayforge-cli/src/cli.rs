//! The command-line surface of the application.
//!
//! This module owns exactly one thing: translating argv into a typed
//! [`Command`]. It contains no behaviour — it never touches the tray, the
//! registry or the file system. It does, however, know how to map each verb onto
//! a core [`UseCase`], which keeps that wiring in the driving adapter rather than
//! in the policy.

use clap::{Parser, Subcommand};

use trayforge::app::UseCase;

/// CLI entrypoint for `trayforge`.
///
/// Running the binary with no subcommand opens the interactive dashboard. The
/// long-lived tray process is started with an explicit `run`, which is what the
/// auto-start registration points at.
#[derive(Debug, Parser)]
#[command(
    name = "trayforge",
    version,
    about = "CLI background app with a tray icon and auto-start support",
    long_about = None,
)]
pub struct Cli {
    /// Increase log verbosity (`-v` = debug, `-vv` = trace).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Add a tray menu item; repeat the flag to add several.
    ///
    /// Grammar:
    ///
    /// * `--item 'sep'`                        a divider
    /// * `--item 'label|TEXT'`                 a disabled informational line
    /// * `--item 'action|LABEL|COMMAND'`       run `COMMAND` in the platform shell
    /// * `--item 'open|LABEL|TARGET'`          open a path or URL with the OS
    /// * `--item 'TEXT'`                       shorthand for a label
    ///
    /// Passing any `--item` **replaces** the default menu. A single *Quit* entry
    /// is always present, so the tray stays escapable.
    #[arg(short = 'i', long = "item", global = true, value_name = "SPEC")]
    pub item: Vec<String>,

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
    /// the composition root rather than by [`trayforge::app::Application`].
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
