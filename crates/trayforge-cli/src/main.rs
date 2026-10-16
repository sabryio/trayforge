//! Thin binary entrypoint / composition root.
//!
//! This file contains no business logic. Its only job is to:
//!
//! 1. parse the CLI,
//! 2. detect configuration,
//! 3. choose the concrete adapters, and
//! 4. inject them into [`Application`] (or hand them to the dashboard).

use clap::Parser;

use trayforge::{
    app::Application, autostart::DesktopAutoStartService, config::AppConfig, error::AppResult,
    instance::DesktopInstanceLock, logging, supervisor::DesktopTraySupervisor,
    tray::DesktopTrayRunner,
};
use trayforge_cli::{
    cli::{Cli, Command},
    presentation, tui,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("trayforge: {err}");
        std::process::exit(1);
    }
}

/// Parse arguments and dispatch.
///
/// Keeping this fallible and separate from [`main`] means the error is formatted
/// once, with a sensible exit code, and the happy path stays linear.
fn run() -> AppResult<()> {
    let cli = Cli::parse();
    let config = AppConfig::detect()?;

    // The guard owns the log writer thread for the whole process lifetime.
    let _logging = logging::init(&config, cli.verbose)?;

    // Composition root: pick the concrete platform adapters here and nowhere else.
    let autostart = DesktopAutoStartService::new(config.clone());
    let tray = DesktopTrayRunner::new(config.clone());
    let instance = DesktopInstanceLock::new(config.app_id.clone());

    let application = Application::new(Box::new(autostart), Box::new(tray), Box::new(instance));

    match cli.command_or_default() {
        // Presentation-only verb: the dashboard drives the core itself.
        Command::Tui => {
            let supervisor = DesktopTraySupervisor::new(config.clone());
            tui::run(&application, Box::new(supervisor), &config)
        }
        command => {
            let use_case = command
                .use_case()
                .expect("only the `tui` command lacks a core use case");
            let outcome = application.execute(use_case)?;
            println!("{}", presentation::describe(&outcome));
            Ok(())
        }
    }
}
