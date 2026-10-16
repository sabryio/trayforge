//! Real, desktop-backed tray adapter.
//!
//! A pure terminal process cannot draw a tray icon: the icon and its menu are
//! created through desktop GUI APIs and kept alive by an event loop. This
//! adapter therefore owns a [`winit`] event loop for the lifetime of the process.
//!
//! Menu and tray events are delivered on background threads by the underlying
//! libraries, so we install handlers that forward them through the winit
//! [`winit::event_loop::EventLoopProxy`]. That both marshals them onto the main
//! thread *and* wakes the otherwise-idle event loop.
//!
//! # Platform notes
//!
//! * **Windows / macOS** work with the default backend.
//! * **Linux** — the default `libappindicator` backend expects a GTK main loop
//!   rather than winit's; see `docs/platform-notes.md` for using the `ksni`
//!   (StatusNotifierItem) backend instead.

use muda::MenuEvent;
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

use super::{
    TrayRunner,
    icon::{DEFAULT_ICON_SIZE, default_icon},
    menu::TrayMenu,
    spec::TrayAction,
};
use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    system,
};

/// Events forwarded from the tray/menu backends into the winit loop.
#[derive(Debug)]
enum UserEvent {
    Menu(muda::MenuId),
    Tray(TrayIconEvent),
}

/// Runs the tray process using [`winit`] as the desktop event loop.
pub struct DesktopTrayRunner {
    config: AppConfig,
}

impl DesktopTrayRunner {
    /// Create the adapter for the given configuration.
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}

impl TrayRunner for DesktopTrayRunner {
    fn run(&self) -> AppResult<()> {
        let event_loop = EventLoop::<UserEvent>::with_user_event()
            .build()
            .map_err(|err| AppError::Tray(format!("could not create event loop: {err}")))?;

        forward_menu_events(&event_loop);
        forward_tray_events(&event_loop);

        let mut application = TrayApplication::new(self.config.clone());
        let loop_result = event_loop
            .run_app(&mut application)
            .map_err(|err| AppError::Tray(format!("tray event loop failed: {err}")));

        // Prefer the in-loop failure (e.g. icon creation) over the loop result.
        match application.take_error() {
            Some(err) => Err(err),
            None => loop_result,
        }
    }
}

/// Route menu activations to the event loop.
fn forward_menu_events(event_loop: &EventLoop<UserEvent>) {
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let _ = proxy.send_event(UserEvent::Menu(event.id));
    }));
}

/// Route tray icon events (clicks, hover, ...) to the event loop.
fn forward_tray_events(event_loop: &EventLoop<UserEvent>) {
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
        let _ = proxy.send_event(UserEvent::Tray(event));
    }));
}

/// The winit application that owns the live tray icon and menu.
struct TrayApplication {
    config: AppConfig,
    started: bool,
    tray_icon: Option<TrayIcon>,
    menu: Option<TrayMenu>,
    error: Option<AppError>,
}

impl TrayApplication {
    fn new(config: AppConfig) -> Self {
        Self {
            config,
            started: false,
            tray_icon: None,
            menu: None,
            error: None,
        }
    }

    /// Take the failure captured inside the event loop, if any.
    fn take_error(&mut self) -> Option<AppError> {
        self.error.take()
    }

    /// Create the icon and menu. Must run *inside* the event loop so a GUI loop
    /// is already pumping (required on Windows and macOS).
    fn start(&mut self) -> AppResult<()> {
        let menu = TrayMenu::build(&self.config.tray)?;
        let icon = default_icon(DEFAULT_ICON_SIZE)?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.menu().clone()))
            .with_tooltip(&self.config.tooltip)
            .with_icon(icon)
            .build()
            .map_err(|err| AppError::Tray(format!("could not create tray icon: {err}")))?;

        tracing::info!(tooltip = %self.config.tooltip, "tray icon ready");
        self.tray_icon = Some(tray_icon);
        self.menu = Some(menu);
        Ok(())
    }

    /// Tear down GUI resources on the thread that created them.
    fn shutdown(&mut self) {
        self.tray_icon = None;
        self.menu = None;
    }
}

impl ApplicationHandler<UserEvent> for TrayApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.started {
            return;
        }
        self.started = true;

        match self.start() {
            Ok(()) => tracing::info!("tray process started"),
            Err(err) => {
                tracing::error!(%err, "failed to start tray process");
                self.error = Some(err);
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
        // This application is tray-only: it never creates windows, so there is
        // nothing to handle here.
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Menu(id) => match self.menu.as_ref().and_then(|menu| menu.resolve(&id)) {
                Some(TrayAction::Quit) => {
                    tracing::info!("quit requested from tray menu");
                    event_loop.exit();
                }
                Some(TrayAction::Shell(command)) => match system::shell(command) {
                    Ok(()) => tracing::info!(%command, "tray action launched"),
                    Err(err) => tracing::error!(%err, %command, "tray action failed"),
                },
                Some(TrayAction::Open(target)) => match system::open(target) {
                    Ok(()) => tracing::info!(%target, "tray entry opened"),
                    Err(err) => tracing::error!(%err, %target, "could not open tray entry"),
                },
                None => tracing::trace!(?id, "unhandled tray menu event"),
            },
            UserEvent::Tray(event) => {
                tracing::trace!(?event, "tray icon event");
            }
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        tracing::info!("tray event loop exiting");
        self.shutdown();
    }
}
