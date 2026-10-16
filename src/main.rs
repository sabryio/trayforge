//! Thin binary entrypoint / composition root.
//!
//! This file contains no business logic. Its only job is to:
//!
//! 1. parse the CLI,
//! 2. detect configuration,
//! 3. choose the concrete adapters, and
//! 4. inject them into [`Application`].

use clap::Parser;

use tray_cli::{
    app::Application, autostart::DesktopAutoStartService, cli::Cli, config::AppConfig,
    error::AppResult, instance::DesktopInstanceLock, logging, tray::DesktopTrayRunner,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("tray-cli: {err}");
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

    application.execute(cli.command_or_default())
}
