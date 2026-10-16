//! `tray-cli` — a small CLI/background utility with a system tray icon and
//! cross-platform auto-start, wired together with a clean, ports-and-adapters
//! (hexagonal) architecture.
//!
//! The dependency direction always points *inward*, toward policy:
//!
//! ```text
//!                    main.rs  (composition root)
//!                       |
//!                       v
//!              app::Application          <- use cases / policy
//!                       |
//!        +--------------+--------------+
//!        v              v              v
//!  autostart::*    tray::*     instance::*     <- abstractions (ports)
//!        ^              ^              ^
//!        |              |              |
//!   desktop adapter, desktop adapter, desktop adapter
//!   mock adapter     mock adapter     mock adapter   <- test doubles
//! ```
//!
//! Nothing in [`app`], [`cli`] or [`config`] knows how a tray icon is drawn, how
//! the OS registers a login item, or how single-instance locking works. Those are
//! details supplied at the edge by the concrete adapters and injected in
//! `main.rs`.
//!
//! # Layout
//!
//! | Module        | Responsibility                                     |
//! |---------------|----------------------------------------------------|
//! | [`app`]       | Use-case orchestration (the "what")                |
//! | [`cli`]       | Command-line surface (clap model)                  |
//! | [`config`]    | Runtime configuration detection                    |
//! | [`error`]     | Shared error model                                 |
//! | [`instance`]  | Single-instance guard port + adapters              |
//! | [`autostart`] | Login/auto-start port + adapters                   |
//! | [`tray`]      | Tray icon/event-loop port + adapters               |
//! | [`logging`]   | Structured, file-backed logging setup              |

pub mod app;
pub mod autostart;
pub mod cli;
pub mod config;
pub mod error;
pub mod instance;
pub mod logging;
pub mod tray;
