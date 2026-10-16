# Platform notes

## Windows

* **Tray**: `tray-icon` creates a hidden message-only window; a running message
  pump is required. The winit event loop provides it.
* **Auto-start**: a per-user `Run` registry value. No admin rights needed.
* **Single instance**: a named kernel mutex — released automatically when the
  process exits.
* **Distribution**: sign the executable and use an installer (WiX/NSIS) so the
  registered path is stable and the icon can be loaded from the exe resources
  (`Icon::from_resource`).

## macOS

* **Tray**: the icon must be created on the main thread *after* the event loop is
  running. In winit that means the first `resumed` event, which is where
  `crates/trayforge/src/tray/desktop.rs` builds the icon.
* **Auto-start**: LaunchAgent by default; use `MacOSLaunchMode::SMAppService`
  when distributing a notarized `.app` bundle, and register the bundle rather
  than the inner executable.
* **Bundle**: for a real product, wrap the binary in a `.app` and set an
  `LSUIElement = true` entry so it appears only in the menu bar (no Dock icon).

## Linux / BSD

* **Tray backends**: `tray-icon` supports two:
  * `libappindicator` (default) — needs GTK 3 + libayatana/libappindicator. It
    drives the GTK main loop, which is **not** the same loop winit runs.
  * `ksni` — StatusNotifierItem over D-Bus, runs its service on a worker thread
    and does **not** need a GTK loop. This is the better fit for a winit-based
    app.
* If you target Linux desktop seriously, switch the dependency to the ksni
  backend:

  ```toml
  [target.'cfg(any(target_os = "linux", target_os = "freebsd"))'.dependencies]
  tray-icon = { version = "0.26", default-features = false, features = ["ksni"] }
  ```

  The builder API in `crates/trayforge/src/tray/desktop.rs` is unchanged; some event variants
  (e.g. left-click) are simply unavailable under ksni.
* **Wayland**: global menus / tray behaviour varies by compositor. Test on the
  desktops you support.
* **Auto-start**: XDG autostart `.desktop` entries under `~/.config/autostart`.

## Dashboard (all platforms)

The interactive control panel is pure terminal code (`ratatui` + `crossterm`
through `ratatui::crossterm`). It needs no desktop APIs and behaves identically
everywhere. It launches the tray agent as a **child process** rather than
in-process, because the tray's event loop wants the OS main thread while the
dashboard owns the terminal.

On Windows the child is spawned with `CREATE_NO_WINDOW` and null stdio so it
never flashes a console or scribbles over the dashboard's alternate screen.

## Headless / CI environments

The tray, auto-start and lock adapters are never exercised by the test suite —
the tests inject the mock adapters, so `cargo test` runs everywhere, including a
bare container. The dashboard's state machine and renderer are likewise tested
without a terminal, by drawing into ratatui's `TestBackend`. Only *building*
needs the platform headers.
