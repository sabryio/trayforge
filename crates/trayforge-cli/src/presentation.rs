//! Human-readable rendering of core [`Outcome`]s for the CLI.
//!
//! Presentation lives at the edge, not in the core: [`trayforge::app::Application`]
//! returns structured results and this module turns them into the text a
//! one-shot command prints. The interactive dashboard ignores this and draws its
//! own view of the same values.

use trayforge::app::Outcome;

/// Render an [`Outcome`] as the single line a CLI command prints.
#[must_use]
pub fn describe(outcome: &Outcome) -> String {
    match outcome {
        Outcome::TrayStarted => "Starting tray process. Use the tray menu to quit.".to_string(),
        Outcome::AutoStartEnabled => "Auto-start enabled.".to_string(),
        Outcome::AutoStartDisabled => "Auto-start disabled.".to_string(),
        Outcome::AutoStartStatus { enabled } => format!("Auto-start enabled: {enabled}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_outcome_has_a_line() {
        assert_eq!(describe(&Outcome::AutoStartEnabled), "Auto-start enabled.");
        assert_eq!(
            describe(&Outcome::AutoStartDisabled),
            "Auto-start disabled."
        );
        assert_eq!(
            describe(&Outcome::AutoStartStatus { enabled: true }),
            "Auto-start enabled: true"
        );
        assert!(describe(&Outcome::TrayStarted).contains("tray"));
    }
}
