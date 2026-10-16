//! CLI contract tests: the argv surface is what auto-start launchers and users
//! depend on, so it is pinned here.

use clap::Parser;
use trayforge::app::UseCase;
use trayforge_cli::cli::{Cli, Command};

#[test]
fn a_bare_executable_opens_the_dashboard() {
    let cli = Cli::parse_from(["trayforge"]);
    assert_eq!(cli.command_or_default(), Command::Tui);
}

#[test]
fn every_subcommand_round_trips() {
    let cases = [
        ("tui", Command::Tui),
        ("run", Command::Run),
        ("enable-autostart", Command::EnableAutostart),
        ("disable-autostart", Command::DisableAutostart),
        ("status", Command::Status),
    ];

    for (arg, expected) in cases {
        let cli = Cli::parse_from(["trayforge", arg]);
        assert_eq!(cli.command, Some(expected), "parsing `{arg}`");
    }
}

#[test]
fn commands_map_onto_the_core_use_cases() {
    // The dashboard is a presentation concern and must not reach the core.
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
fn verbose_is_global_and_repeatable() {
    let cli = Cli::parse_from(["trayforge", "status", "-vv"]);
    assert_eq!(cli.verbose, 2);
    assert_eq!(cli.command, Some(Command::Status));
}

#[test]
fn unknown_subcommands_are_rejected() {
    assert!(Cli::try_parse_from(["trayforge", "not-a-command"]).is_err());
}

#[test]
fn item_is_global_and_repeatable() {
    let cli = Cli::parse_from([
        "trayforge",
        "--item",
        "action|Restart|make restart",
        "--item",
        "sep",
        "run",
    ]);

    assert_eq!(cli.command, Some(Command::Run));
    assert_eq!(
        cli.item,
        vec!["action|Restart|make restart".to_string(), "sep".to_string()],
        "--item must be accepted on either side of the subcommand"
    );
}
