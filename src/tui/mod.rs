//! Interactive terminal dashboard (a driving adapter).
//!
//! Running `tray-cli` with no subcommand opens this ratatui dashboard: a live
//! control panel that reports auto-start state and manages the tray agent.
//!
//! Design notes:
//!
//! * This module is a *driving* adapter, exactly like [`crate::cli`]. It calls
//!   into the core and the [`TraySupervisor`] port; the core has no idea it
//!   exists.
//! * The tray agent itself is started as a **child process**. The tray's desktop
//!   event loop wants the OS main thread, and the dashboard owns the terminal, so
//!   keeping them in separate processes keeps both honest.
//! * Only [`run`] is public. The state machine, catalogue and renderer stay
//!   private so the crate's public surface stays small.

mod command;
mod dashboard;
mod view;

use std::{
    io::{self, Stdout},
    path::Path,
    process::Command,
    time::Duration,
};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        cursor::{Hide, Show},
        event::{self, Event, KeyEvent},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

use crate::{
    app::{Application, UseCase},
    config::AppConfig,
    error::AppResult,
    supervisor::{TrayState, TraySupervisor},
};

use command::CommandId;
use dashboard::{Action, Dashboard, Level};

/// How long to wait for a key before redrawing (drives the spinner).
const TICK: Duration = Duration::from_millis(120);

/// Run the dashboard until the user quits.
///
/// The terminal is restored even when an error propagates, because the raw-mode
/// guard restores it on drop.
///
/// # Errors
/// Returns [`crate::error::AppError::Io`] if the terminal cannot be set up or
/// drawn.
pub fn run(
    application: &Application,
    supervisor: Box<dyn TraySupervisor>,
    config: &AppConfig,
) -> AppResult<()> {
    let mut session = TerminalSession::enter()?;
    let mut tui = Tui::new(application, supervisor, config);
    tui.refresh();

    loop {
        session.terminal.draw(|frame| tui.render(frame))?;

        if event::poll(TICK)? {
            if let Event::Key(key) = event::read()?
                && tui.on_key(key)
            {
                break;
            }
        } else {
            tui.on_tick();
        }
    }

    Ok(())
}

/// Owns the raw-mode terminal and restores it on drop.
struct TerminalSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalSession {
    fn enter() -> AppResult<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen, Show);
        let _ = self.terminal.show_cursor();
    }
}

/// Binds the state machine to its collaborators and performs the effects that a
/// [`Action`] asks for.
struct Tui<'a> {
    application: &'a Application,
    supervisor: Box<dyn TraySupervisor>,
    config: &'a AppConfig,
    dashboard: Dashboard,
}

impl<'a> Tui<'a> {
    fn new(
        application: &'a Application,
        supervisor: Box<dyn TraySupervisor>,
        config: &'a AppConfig,
    ) -> Self {
        let mut dashboard = Dashboard::new();
        dashboard.record(Level::Info, "dashboard ready");
        Self {
            application,
            supervisor,
            config,
            dashboard,
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        view::render(frame, &self.dashboard, self.config);
    }

    fn on_tick(&mut self) {
        self.dashboard.tick = self.dashboard.tick.wrapping_add(1);
    }

    /// Handle a keypress; returns `true` when the dashboard should close.
    fn on_key(&mut self, key: KeyEvent) -> bool {
        match self.dashboard.handle_key(key) {
            Some(Action::Quit) => true,
            Some(Action::Refresh) => {
                self.refresh();
                self.dashboard.record(Level::Info, "status refreshed");
                false
            }
            Some(Action::Execute(id)) => {
                self.execute(id);
                false
            }
            None => false,
        }
    }

    /// Re-read status from the adapters into the snapshot.
    fn refresh(&mut self) {
        let snapshot = &mut self.dashboard.snapshot;
        snapshot.auto_start = self.application.auto_start_enabled().ok();
        snapshot.instance_available = self.application.instance_available().ok();
        snapshot.tray = self.supervisor.state();
    }

    /// Perform a command and record what happened.
    fn execute(&mut self, id: CommandId) {
        match id {
            CommandId::StartTray => self.start_tray(),
            CommandId::StopTray => self.stop_tray(),
            CommandId::ToggleAutoStart => self.toggle_auto_start(),
            CommandId::Refresh => {
                self.refresh();
                self.dashboard.record(Level::Info, "status refreshed");
            }
            CommandId::OpenLogs => self.open_logs(),
            // Quit is handled in `on_key`, which owns the loop's control flow.
            CommandId::Quit => {}
        }
        // Status almost always changes after an action; re-read it eagerly.
        self.refresh();
    }

    fn start_tray(&mut self) {
        match self.supervisor.start() {
            Ok(TrayState::Running { pid }) => {
                self.dashboard
                    .record(Level::Success, format!("tray agent started (pid {pid})"));
            }
            Ok(TrayState::Stopped) => {
                self.dashboard
                    .record(Level::Warn, "tray agent did not start");
            }
            Err(err) => self
                .dashboard
                .record(Level::Error, format!("could not start agent: {err}")),
        }
    }

    fn stop_tray(&mut self) {
        let was_running = self.supervisor.state().is_running();
        match self.supervisor.stop() {
            Ok(()) if was_running => self.dashboard.record(Level::Info, "tray agent stopped"),
            Ok(()) => self
                .dashboard
                .record(Level::Warn, "no tray agent was running"),
            Err(err) => self
                .dashboard
                .record(Level::Error, format!("could not stop agent: {err}")),
        }
    }

    fn toggle_auto_start(&mut self) {
        let currently_enabled = self.dashboard.snapshot.auto_start.unwrap_or(false);
        let (use_case, verb) = if currently_enabled {
            (UseCase::DisableAutoStart, "disabled")
        } else {
            (UseCase::EnableAutoStart, "enabled")
        };

        match self.application.execute(use_case) {
            Ok(_) => self
                .dashboard
                .record(Level::Success, format!("auto-start {verb}")),
            Err(err) => self
                .dashboard
                .record(Level::Error, format!("auto-start change failed: {err}")),
        }
    }

    fn open_logs(&mut self) {
        let dir = self.config.log_dir.clone();
        match reveal_in_file_manager(&dir) {
            Ok(()) => self
                .dashboard
                .record(Level::Info, format!("opened {}", dir.display())),
            Err(err) => self
                .dashboard
                .record(Level::Error, format!("could not open logs: {err}")),
        }
    }
}

/// Ask the OS file manager to reveal `dir`, creating it first if needed.
fn reveal_in_file_manager(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;

    #[cfg(windows)]
    let mut command = Command::new("explorer");
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = Command::new("xdg-open");

    command.arg(dir).spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;
    use crate::{
        autostart::MockAutoStartService, instance::MockInstanceLock,
        supervisor::MockTraySupervisor, tray::MockTrayRunner,
    };

    struct Harness {
        app: Application,
        autostart: MockAutoStartService,
        supervisor: MockTraySupervisor,
        config: AppConfig,
    }

    fn harness() -> Harness {
        let autostart = MockAutoStartService::new();
        let app = Application::new(
            Box::new(autostart.clone()),
            Box::new(MockTrayRunner::new()),
            Box::new(MockInstanceLock::available()),
        );
        Harness {
            app,
            autostart,
            supervisor: MockTraySupervisor::stopped(),
            config: AppConfig::new("tray-cli", PathBuf::from("/opt/tray-cli")),
        }
    }

    fn tui<'a>(harness: &'a Harness) -> Tui<'a> {
        Tui::new(
            &harness.app,
            Box::new(harness.supervisor.clone()),
            &harness.config,
        )
    }

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn refresh_reads_state_from_both_adapters() {
        let harness = harness();
        let mut tui = tui(&harness);

        tui.refresh();

        assert_eq!(tui.dashboard.snapshot.auto_start, Some(false));
        assert_eq!(tui.dashboard.snapshot.instance_available, Some(true));
        assert_eq!(tui.dashboard.snapshot.tray, TrayState::Stopped);
    }

    #[test]
    fn starting_the_tray_updates_the_supervisor_and_the_log() {
        let harness = harness();
        let mut tui = tui(&harness);

        tui.execute(CommandId::StartTray);

        assert!(harness.supervisor.state().is_running());
        assert_eq!(
            tui.dashboard.snapshot.tray,
            TrayState::Running { pid: 4242 }
        );
        assert!(
            tui.dashboard
                .log
                .iter()
                .any(|line| line.text.contains("started")),
            "expected a start entry in {:?}",
            tui.dashboard.log
        );
    }

    #[test]
    fn toggling_auto_start_flips_the_service() {
        let harness = harness();
        let mut tui = tui(&harness);

        tui.execute(CommandId::ToggleAutoStart);
        assert!(harness.autostart.state());

        tui.execute(CommandId::ToggleAutoStart);
        assert!(!harness.autostart.state());
    }

    #[test]
    fn enter_starts_the_highlighted_command() {
        let harness = harness();
        let mut tui = tui(&harness);
        // Row 0 is "Start tray agent".
        assert!(!tui.on_key(press(KeyCode::Enter)));

        assert!(harness.supervisor.state().is_running());
    }

    #[test]
    fn quit_keys_close_the_dashboard() {
        let harness = harness();
        let mut tui = tui(&harness);

        assert!(tui.on_key(press(KeyCode::Char('q'))));
    }

    #[test]
    fn ctrl_c_also_closes_the_dashboard() {
        let harness = harness();
        let mut tui = tui(&harness);

        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(tui.on_key(key));
    }
}
