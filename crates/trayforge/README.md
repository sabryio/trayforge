# trayforge

**Reusable building blocks for cross-platform system-tray background apps.**

`trayforge` gives you the parts every tray/background app needs — a tray runtime,
login auto-start, single-instance locking and child-process supervision — behind
small, replaceable traits. You supply the front-end; the library supplies the
platform plumbing and stays out of your way.

```text
cargo add trayforge
```

The crate is deliberately free of presentation dependencies: there is **no
`clap` and no `ratatui`** here. The batteries-included CLI/TUI lives in the
companion [`trayforge-cli`](https://crates.io/crates/trayforge-cli) crate.

## Design

Ports-and-adapters (hexagonal). The core depends only on abstractions, and the
binary decides which concrete adapters satisfy them.

| Port | Purpose | Desktop adapter |
|------|---------|-----------------|
| `autostart::AutoStartService` | register/unregister/query login items | `DesktopAutoStartService` |
| `tray::TrayRunner` | run the tray icon + event loop | `DesktopTrayRunner` |
| `instance::InstanceLock` | guarantee a single running instance | `DesktopInstanceLock` |
| `supervisor::TraySupervisor` | start/stop/supervise a tray child process | `DesktopTraySupervisor` |

Every port also ships a `mock` adapter so your tests never touch the OS.

`app::Application` is the only place that knows *what* the program does. It
never prints — it returns a structured `app::Outcome` for the front-end to
render.

## Example

```rust
use trayforge::app::{Application, UseCase};
use trayforge::autostart::MockAutoStartService;
use trayforge::instance::MockInstanceLock;
use trayforge::tray::MockTrayRunner;

let application = Application::new(
    Box::new(MockAutoStartService::new()),
    Box::new(MockTrayRunner::new()),
    Box::new(MockInstanceLock::available()),
);

application.execute(UseCase::EnableAutoStart)?;
# Ok::<(), trayforge::error::AppError>(())
```

In production you would inject the `Desktop*` adapters instead — see the
[`trayforge-cli`](https://crates.io/crates/trayforge-cli) composition root.

## Platform notes

| OS | Auto-start | Locking | Tray |
|----|-----------|---------|------|
| Windows | `HKCU\...\Run` value | named mutex | `tray-icon` + `muda` |
| macOS | LaunchAgent | abstract socket | `tray-icon` |
| Linux | XDG autostart `.desktop` | file lock | `tray-icon` (AppIndicator) |

Building the tray backend on Linux needs GTK 3 and an AppIndicator
implementation, plus winit's X11/Wayland dev libraries:

```sh
sudo apt install libgtk-3-dev libayatana-appindicator3-dev libxkbcommon-dev
```

## MSRV

Rust **1.99+**.

## License

MIT OR Apache-2.0.
