//! The catalogue of actions the dashboard can run.
//!
//! Keeping this as plain data means the list is declarative and the renderer has
//! nothing to hard-code.

/// Stable identity of a dashboard action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandId {
    /// Spawn the tray agent as a supervised child process.
    StartTray,
    /// Stop the agent this dashboard started.
    StopTray,
    /// Flip the login auto-start registration.
    ToggleAutoStart,
    /// Re-read the current status from the adapters.
    Refresh,
    /// Reveal the log directory in the OS file manager.
    OpenLogs,
    /// Leave the dashboard.
    Quit,
}

/// A single row in the command list.
#[derive(Debug, Clone, Copy)]
pub struct CommandSpec {
    /// What running this row does.
    pub id: CommandId,
    /// A compact leading glyph.
    pub glyph: &'static str,
    /// Short title shown in the list.
    pub title: &'static str,
    /// One-line explanation shown beneath the title.
    pub summary: &'static str,
}

/// Every command, in display order.
pub const COMMANDS: [CommandSpec; 6] = [
    CommandSpec {
        id: CommandId::StartTray,
        glyph: "▶",
        title: "Start tray agent",
        summary: "Launch the tray icon in the background",
    },
    CommandSpec {
        id: CommandId::StopTray,
        glyph: "■",
        title: "Stop tray agent",
        summary: "Stop the agent started from here",
    },
    CommandSpec {
        id: CommandId::ToggleAutoStart,
        glyph: "⇄",
        title: "Toggle auto-start",
        summary: "Enable or disable launching at login",
    },
    CommandSpec {
        id: CommandId::Refresh,
        glyph: "↻",
        title: "Refresh status",
        summary: "Re-read status from the OS",
    },
    CommandSpec {
        id: CommandId::OpenLogs,
        glyph: "▤",
        title: "Open log folder",
        summary: "Reveal the log folder",
    },
    CommandSpec {
        id: CommandId::Quit,
        glyph: "✕",
        title: "Quit dashboard",
        summary: "Close the dashboard",
    },
];
