//! CLI contract tests: the argv surface is what auto-start launchers and users
//! depend on, so it is pinned here.

use clap::Parser;
use tray_cli::cli::{Cli, Command};

#[test]
fn a_bare_executable_is_equivalent_to_run() {
    // Auto-start launches the binary with no arguments, so this must hold.
    let cli = Cli::parse_from(["tray-cli"]);
    assert_eq!(cli.command_or_default(), Command::Run);
}

#[test]
fn every_subcommand_round_trips() {
    let cases = [
        ("run", Command::Run),
        ("enable-autostart", Command::EnableAutostart),
        ("disable-autostart", Command::DisableAutostart),
        ("status", Command::Status),
    ];

    for (arg, expected) in cases {
        let cli = Cli::parse_from(["tray-cli", arg]);
        assert_eq!(cli.command, Some(expected), "parsing `{arg}`");
    }
}

#[test]
fn verbose_is_global_and_repeatable() {
    let cli = Cli::parse_from(["tray-cli", "status", "-vv"]);
    assert_eq!(cli.verbose, 2);
    assert_eq!(cli.command, Some(Command::Status));
}
