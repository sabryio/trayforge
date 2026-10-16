//! The tray context menu, described as data.
//!
//! The menu is *data*, not code: it is parsed from the CLI (`--item`), carried in
//! [`AppConfig`](crate::config::AppConfig) and rendered by whichever tray backend
//! is in use. That is what keeps the tray generic — a downstream app, or a user
//! at the command line, can describe their own entries without this crate knowing
//! anything about them.

use crate::error::{AppError, AppResult};

/// One entry in the tray context menu.
///
/// Labels and targets must not contain `|`: the separator is part of the CLI
/// grammar and would break round-tripping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayItem {
    /// A horizontal divider.
    Separator,
    /// A disabled informational line.
    Label {
        /// Text to show.
        text: String,
    },
    /// Exits the tray process. A spec always contains exactly one.
    Quit,
    /// Runs [`Self::Action::command`] through the platform shell.
    Action {
        /// Text to show.
        label: String,
        /// Shell command line to run.
        command: String,
    },
    /// Asks the OS to open [`Self::Open::target`] with its default handler.
    Open {
        /// Text to show.
        label: String,
        /// A path or a URL.
        target: String,
    },
}

impl TrayItem {
    /// What selecting this entry does, or [`None`] for inert entries such as
    /// dividers and labels.
    #[must_use]
    pub fn action(&self) -> Option<TrayAction> {
        match self {
            Self::Quit => Some(TrayAction::Quit),
            Self::Action { command, .. } => Some(TrayAction::Shell(command.clone())),
            Self::Open { target, .. } => Some(TrayAction::Open(target.clone())),
            Self::Separator | Self::Label { .. } => None,
        }
    }

    /// Parse one `--item` value.
    ///
    /// The grammar is `sep` | `quit` | `label|TEXT` | `action|LABEL|COMMAND` |
    /// `open|LABEL|TARGET`, with a bare `TEXT` shorthand for a label.
    fn parse(spec: &str) -> AppResult<Self> {
        let trimmed = spec.trim();
        let hint = "expected `sep`, `quit`, `label|TEXT`, `action|LABEL|COMMAND`, \
                    `open|LABEL|TARGET` or a plain label";

        match trimmed.splitn(3, '|').collect::<Vec<_>>().as_slice() {
            ["sep" | "separator", ..] => Ok(Self::Separator),
            ["quit", ..] => Ok(Self::Quit),
            ["label", text] if !text.is_empty() => Ok(Self::Label {
                text: (*text).to_string(),
            }),
            ["action", label, command] if !label.is_empty() && !command.is_empty() => {
                Ok(Self::Action {
                    label: (*label).to_string(),
                    command: (*command).to_string(),
                })
            }
            ["open", label, target] if !label.is_empty() && !target.is_empty() => Ok(Self::Open {
                label: (*label).to_string(),
                target: (*target).to_string(),
            }),
            // A bare string is a convenient shorthand for a disabled label.
            [text] if !text.is_empty() => Ok(Self::Label {
                text: (*text).to_string(),
            }),
            _ => Err(AppError::Tray(format!("invalid --item `{spec}`; {hint}"))),
        }
    }

    /// The inverse of [`Self::parse`].
    fn encode(&self) -> String {
        match self {
            Self::Separator => "sep".to_string(),
            Self::Quit => "quit".to_string(),
            Self::Label { text } => format!("label|{text}"),
            Self::Action { label, command } => format!("action|{label}|{command}"),
            Self::Open { label, target } => format!("open|{label}|{target}"),
        }
    }
}

/// What selecting an interactive tray entry should do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayAction {
    /// Exit the tray process.
    Quit,
    /// Run a shell command line.
    Shell(String),
    /// Open a path or URL with the OS default handler.
    Open(String),
}

/// The whole tray menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraySpec {
    items: Vec<TrayItem>,
}

impl Default for TraySpec {
    /// The smallest safe menu: just *Quit*. A tray icon with no way out would be
    /// hostile, so even the default is usable.
    fn default() -> Self {
        Self::custom(Vec::new())
    }
}

impl TraySpec {
    /// The out-of-the-box menu: an "`<app>` is running" line, a divider and
    /// *Quit*.
    #[must_use]
    pub fn standard(app_name: &str) -> Self {
        Self::custom(vec![
            TrayItem::Label {
                text: format!("{app_name} is running"),
            },
            TrayItem::Separator,
        ])
    }

    /// A menu built from `items`, normalised so it always ends up with exactly
    /// one *Quit* entry (extra ones are dropped, and a missing one is appended).
    #[must_use]
    pub fn custom(items: Vec<TrayItem>) -> Self {
        let mut quit_seen = false;
        let mut items: Vec<TrayItem> = items
            .into_iter()
            .filter(|item| {
                if matches!(item, TrayItem::Quit) {
                    if quit_seen {
                        return false;
                    }
                    quit_seen = true;
                }
                true
            })
            .collect();

        if !quit_seen {
            items.push(TrayItem::Quit);
        }

        Self { items }
    }

    /// Parse the CLI `--item` arguments into a menu.
    ///
    /// # Errors
    /// Returns [`AppError::Tray`] if any argument does not match the grammar.
    pub fn from_cli_args(args: &[String]) -> AppResult<Self> {
        let mut items = Vec::with_capacity(args.len());
        for arg in args {
            items.push(TrayItem::parse(arg)?);
        }
        Ok(Self::custom(items))
    }

    /// Re-encode the menu as `--item` arguments, so a supervisor can hand the
    /// same menu to the tray child process it spawns.
    #[must_use]
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = Vec::with_capacity(self.items.len());
        for item in &self.items {
            args.push(item.encode());
        }
        args
    }

    /// The entries, in display order.
    #[must_use]
    pub fn items(&self) -> &[TrayItem] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shorthand for building the `&[String]` the CLI would hand over.
    fn args(specs: &[&str]) -> Vec<String> {
        specs.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn the_standard_menu_is_a_label_a_divider_and_quit() {
        let spec = TraySpec::standard("trayforge");

        assert_eq!(
            spec.items(),
            &[
                TrayItem::Label {
                    text: "trayforge is running".to_string()
                },
                TrayItem::Separator,
                TrayItem::Quit,
            ]
        );
    }

    #[test]
    fn a_custom_menu_always_ends_with_quit() {
        let spec = TraySpec::from_cli_args(&args(&["action|Restart|make restart"])).expect("parse");

        assert_eq!(
            spec.items(),
            &[
                TrayItem::Action {
                    label: "Restart".to_string(),
                    command: "make restart".to_string()
                },
                TrayItem::Quit,
            ]
        );
    }

    #[test]
    fn quit_is_deduplicated() {
        let spec = TraySpec::from_cli_args(&args(&["quit", "sep", "quit"])).expect("parse");

        assert_eq!(
            spec.items(),
            &[TrayItem::Quit, TrayItem::Separator],
            "only the first Quit is kept, and none is appended"
        );
    }

    #[test]
    fn every_kind_parses() {
        let spec =
            TraySpec::from_cli_args(&args(&["sep", "separator", "Status", "label|Up", "quit"]))
                .expect("parse");

        assert_eq!(
            spec.items(),
            &[
                TrayItem::Separator,
                TrayItem::Separator,
                TrayItem::Label {
                    text: "Status".to_string()
                },
                TrayItem::Label {
                    text: "Up".to_string()
                },
                TrayItem::Quit,
            ]
        );
    }

    #[test]
    fn action_and_open_keep_their_whole_payload() {
        // The payload may itself contain `|`, e.g. a shell pipeline.
        let spec = TraySpec::from_cli_args(&args(&[
            "action|Logs|cat /var/log/app.log | less",
            "open|Docs|https://docs.rs/trayforge",
        ]))
        .expect("parse");

        assert_eq!(
            spec.items()[0],
            TrayItem::Action {
                label: "Logs".to_string(),
                command: "cat /var/log/app.log | less".to_string()
            }
        );
        assert_eq!(
            spec.items()[1],
            TrayItem::Open {
                label: "Docs".to_string(),
                target: "https://docs.rs/trayforge".to_string()
            }
        );
    }

    #[test]
    fn only_interactive_entries_have_an_action() {
        assert_eq!(TrayItem::Quit.action(), Some(TrayAction::Quit));
        assert_eq!(
            TrayItem::Separator.action(),
            None,
            "a divider must not be clickable"
        );
        assert_eq!(
            TrayItem::Label {
                text: "hi".to_string()
            }
            .action(),
            None
        );
    }

    #[test]
    fn specs_round_trip_through_cli_args() {
        let spec = TraySpec::from_cli_args(&args(&[
            "label|trayforge is running",
            "sep",
            "action|Restart|make restart",
            "open|Docs|https://docs.rs/trayforge",
            "quit",
        ]))
        .expect("parse");

        let reparsed = TraySpec::from_cli_args(&spec.to_cli_args()).expect("reparse");
        assert_eq!(
            reparsed, spec,
            "the dashboard can forward the menu unchanged"
        );
    }

    #[test]
    fn malformed_items_are_rejected() {
        for bad in [
            "",
            "   ",
            "action|no-command",
            "open|no-target",
            "|",
            "label|",
        ] {
            assert!(
                TraySpec::from_cli_args(&args(&[bad])).is_err(),
                "`{bad}` should not parse"
            );
        }
    }

    #[test]
    fn the_default_menu_is_just_quit() {
        assert_eq!(TraySpec::default().items(), &[TrayItem::Quit]);
    }
}
