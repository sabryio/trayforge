//! Presentation layer of the `trayforge` command-line application.
//!
//! Every piece of *policy* lives in the [`trayforge`] library; this crate only
//! decides how to talk to a human. It owns three things:
//!
//! | Module               | Responsibility                                  |
//! |----------------------|-------------------------------------------------|
//! | [`cli`]              | The clap command surface (argv → typed command) |
//! | [`presentation`]     | Rendering core outcomes as one-shot CLI text    |
//! | [`tui`]              | The interactive ratatui dashboard               |
//!
//! Splitting it this way means the library stays free of `clap` and `ratatui`,
//! while the binary stays a thin composition root (see `main.rs`).

pub mod cli;
pub mod presentation;
pub mod tui;
