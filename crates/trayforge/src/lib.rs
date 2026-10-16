//! `trayforge` — reusable building blocks for cross-platform system-tray
//! background applications, wired together with a clean, ports-and-adapters
//! (hexagonal) architecture.
//!
//! This crate is the *reusable* half of the project: it contains policy and the
//! adapters behind small traits, but **no** command-line or terminal-ui code.
//! The batteries-included CLI/TUI lives in the companion `trayforge-cli` crate;
//! you can depend on `trayforge` to build a different front-end (a GUI, a
//! service, your own CLI) without pulling in `clap` or `ratatui`.
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
//! Nothing in [`app`] or [`config`] knows how a tray icon is drawn, how the OS
//! registers a login item, or how single-instance locking works. Those are
//! details supplied at the edge by the concrete adapters and injected by the
//! binary's composition root.
//!
//! # Layout
//!
//! | Module          | Responsibility                                   |
//! |-----------------|--------------------------------------------------|
//! | [`app`]         | Use-case orchestration (the "what")              |
//! | [`config`]      | Runtime configuration detection                  |
//! | [`error`]       | Shared error model                               |
//! | [`instance`]    | Single-instance guard port + adapters            |
//! | [`autostart`]   | Login/auto-start port + adapters                 |
//! | [`tray`]        | Tray icon/menu/event-loop port + adapters        |
//! | [`supervisor`]  | Tray child-process supervision port + adapters   |
//! | [`system`]      | "Open this" / "run that" primitives for the OS   |
//! | [`logging`]     | Structured, file-backed logging setup            |
//!
//! # Example
//!
//! ```
//! use trayforge::{app::Application, autostart::MockAutoStartService,
//!                 instance::MockInstanceLock, tray::MockTrayRunner};
//!
//! // In production you would inject the `Desktop*` adapters instead.
//! let application = Application::new(
//!     Box::new(MockAutoStartService::new()),
//!     Box::new(MockTrayRunner::new()),
//!     Box::new(MockInstanceLock::available()),
//! );
//! # let _ = application;
//! ```

pub mod app;
pub mod autostart;
pub mod config;
pub mod error;
pub mod instance;
pub mod logging;
pub mod supervisor;
pub mod system;
pub mod tray;
