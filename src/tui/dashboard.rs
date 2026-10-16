//! Dashboard state and input handling.
//!
//! This is deliberately free of any `ratatui` type: it is a plain state machine
//! over a status [`Snapshot`] and a rolling activity log. Input arrives as
//! [`KeyEvent`]s and leaves as a small [`Action`], so the whole thing is unit
//! testable without a terminal.

use std::time::{Duration, Instant};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::command::{COMMANDS, CommandId};
use crate::supervisor::TrayState;

/// How many activity lines are retained.
const LOG_CAPACITY: usize = 200;

/// Severity of an activity line, used only for colouring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Neutral progress.
    Info,
    /// Something completed well.
    Success,
    /// Something to notice but not a failure.
    Warn,
    /// Something failed.
    Error,
}

/// A timestamped line in the activity log.
#[derive(Debug, Clone)]
pub struct LogLine {
    /// Seconds elapsed since the dashboard opened.
    pub at: Duration,
    /// Severity of the line.
    pub level: Level,
    /// The message itself.
    pub text: String,
}

/// Everything the status panel needs, gathered from the adapters.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    /// `None` until the first successful poll.
    pub auto_start: Option<bool>,
    /// `None` until the first successful poll.
    pub instance_available: Option<bool>,
    /// Lifecycle of the supervised tray child.
    pub tray: TrayState,
}

/// A high-level effect requested by a keypress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Leave the dashboard.
    Quit,
    /// Re-read status.
    Refresh,
    /// Run the given command.
    Execute(CommandId),
}

/// The mutable dashboard state.
#[derive(Debug)]
pub struct Dashboard {
    /// Index of the highlighted command.
    pub selected: usize,
    /// Latest observed status.
    pub snapshot: Snapshot,
    /// Rolling activity log, oldest first.
    pub log: Vec<LogLine>,
    /// Monotonic frame counter driving the spinner.
    pub tick: u64,
    started: Instant,
}

impl Dashboard {
    /// Create an empty dashboard anchored at the current instant.
    #[must_use]
    pub fn new() -> Self {
        Self {
            selected: 0,
            snapshot: Snapshot::default(),
            log: Vec::new(),
            tick: 0,
            started: Instant::now(),
        }
    }

    /// Seconds elapsed since the dashboard opened.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// The id of the highlighted command.
    #[must_use]
    pub fn selected_command(&self) -> CommandId {
        COMMANDS[self.selected].id
    }

    /// Advance the highlight by `delta` rows, wrapping at both ends.
    pub fn move_selection(&mut self, delta: isize) {
        let len = COMMANDS.len() as isize;
        let next = (self.selected as isize + delta).rem_euclid(len);
        self.selected = next as usize;
    }

    /// Jump the highlight to an absolute index.
    pub fn select(&mut self, index: usize) {
        self.selected = index.min(COMMANDS.len() - 1);
    }

    /// Append a line to the activity log, dropping the oldest when full.
    pub fn record(&mut self, level: Level, text: impl Into<String>) {
        self.log.push(LogLine {
            at: self.elapsed(),
            level,
            text: text.into(),
        });
        if self.log.len() > LOG_CAPACITY {
            let overflow = self.log.len() - LOG_CAPACITY;
            self.log.drain(0..overflow);
        }
    }

    /// A spinner frame for the header, advancing with the tick counter.
    #[must_use]
    pub fn spinner(&self) -> &'static str {
        const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        FRAMES[(self.tick as usize / 2) % FRAMES.len()]
    }

    /// Translate a keypress into an [`Action`].
    ///
    /// Navigation is handled in place; only effectful keys are reported back.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> {
        // Ignore key-release/repeat noise, and the Windows "Enhanced" flag.
        if key.kind != KeyEventKind::Press {
            return None;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Some(Action::Quit);
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection(-1);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection(1);
                None
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.select(0);
                None
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.select(COMMANDS.len() - 1);
                None
            }
            KeyCode::Enter => Some(Action::Execute(self.selected_command())),
            KeyCode::Char('r') => Some(Action::Refresh),
            _ => None,
        }
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyEvent;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn navigation_wraps_in_both_directions() {
        let mut dashboard = Dashboard::new();

        dashboard.move_selection(-1);
        assert_eq!(dashboard.selected, COMMANDS.len() - 1);

        dashboard.move_selection(1);
        assert_eq!(dashboard.selected, 0);

        dashboard.move_selection(1);
        assert_eq!(dashboard.selected, 1);
    }

    #[test]
    fn enter_executes_the_highlighted_command() {
        let mut dashboard = Dashboard::new();
        dashboard.select(2);

        assert_eq!(
            dashboard.handle_key(press(KeyCode::Enter)),
            Some(Action::Execute(COMMANDS[2].id))
        );
    }

    #[test]
    fn q_and_escape_quit_but_arrows_do_not() {
        let mut dashboard = Dashboard::new();

        assert_eq!(
            dashboard.handle_key(press(KeyCode::Char('q'))),
            Some(Action::Quit)
        );
        assert_eq!(
            dashboard.handle_key(press(KeyCode::Esc)),
            Some(Action::Quit)
        );
        assert_eq!(dashboard.handle_key(press(KeyCode::Down)), None);
    }

    #[test]
    fn release_events_are_ignored() {
        let mut dashboard = Dashboard::new();
        let mut event = press(KeyCode::Enter);
        event.kind = KeyEventKind::Release;

        assert_eq!(dashboard.handle_key(event), None);
    }

    #[test]
    fn the_log_is_capped() {
        let mut dashboard = Dashboard::new();
        for i in 0..(LOG_CAPACITY + 25) {
            dashboard.record(Level::Info, format!("line {i}"));
        }

        assert_eq!(dashboard.log.len(), LOG_CAPACITY);
        assert!(
            dashboard
                .log
                .last()
                .expect("non-empty")
                .text
                .ends_with("224")
        );
        assert!(
            dashboard
                .log
                .first()
                .expect("non-empty")
                .text
                .ends_with("25")
        );
    }

    #[test]
    fn select_clamps_to_the_last_row() {
        let mut dashboard = Dashboard::new();
        dashboard.select(usize::MAX);
        assert_eq!(dashboard.selected, COMMANDS.len() - 1);
    }
}
