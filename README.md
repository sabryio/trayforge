# tray-cli

A small, production-shaped **CLI/background utility** that shows a **system tray
icon** with a menu and can register itself to **start at login** — built in Rust
with a clean, ports-and-adapters (hexagonal) architecture.

A pure terminal program cannot draw a tray icon: the icon is created through
desktop GUI APIs. So the binary is a CLI entrypoint that starts a long-running
desktop event loop when asked to run.

```text
tray-cli              # same as `tray-cli run`
tray-cli run          # start the tray process
tray-cli enable-autostart
tray-cli disable-autostart
tray-cli status
```

## Why this shape

The interesting part is not the tray icon — it is that the tray icon, the login
registration and the single-instance guard are all *replaceable* and *testable*.
The core use-case layer never learns which OS API is behind a port.

```text
src/
├─ main.rs            # composition root: parse, build adapters, inject
├─ lib.rs             # public module surface
├─ app.rs             # Application: use-case orchestration (policy)
├─ cli.rs             # clap model — argv in, `Command` out
├─ config.rs          # AppConfig: identity + paths
├─ error.rs           # AppError: one error model
├─ logging.rs         # tracing → rotating file + stderr
├─ instance/          # single-instance port + {desktop, mock} adapters
│  ├─ mod.rs          #   trait InstanceLock
│  ├─ desktop.rs      #   OS mutex / abstract socket / flock
│  └─ mock.rs
├─ autostart/         # login/auto-start port + {desktop, mock} adapters
│  ├─ mod.rs          #   trait AutoStartService
│  ├─ desktop.rs      #   Run key / LaunchAgent / XDG autostart
│  └─ mock.rs
└─ tray/              # tray runtime port + {desktop, mock} adapters
   ├─ mod.rs          #   trait TrayRunner
   ├─ desktop.rs      #   winit event loop + tray-icon + muda
   ├─ menu.rs         #   menu construction
   ├─ icon.rs         #   procedural RGBA icon (no asset/decoder deps)
   └─ mock.rs
```

Dependencies point **inward**: `app` depends on the traits, the traits know
nothing about the platform, and only `main.rs` picks real adapters.

## How it maps to SOLID

| Principle | Where |
|-----------|-------|
| **S**ingle responsibility | `cli` parses, `config` detects, `app` orchestrates, each adapter does exactly one OS thing. |
| **O**pen/closed | Add `WindowsServiceAutoStart`, `HeadlessTrayRunner`, or a different lock by implementing the trait — no core change. |
| **L**iskov | Any `AutoStartService`/`TrayRunner`/`InstanceLock` swaps in without surprising the orchestrator; the mocks prove it. |
| **I**nterface segregation | Three tiny traits (`enable/disable/is_enabled`, `run`, `try_acquire`) instead of one fat "system services" trait. |
| **D**ependency inversion | `Application` holds `Box<dyn ...>`; `main.rs` is the only place that names concrete platform types. |

## Usage

### Requirements

* Rust **1.85+** (edition 2024).
* **Linux**: GTK 3 and an AppIndicator implementation, plus winit's X11/Wayland
  dev libraries, to build:
  ```sh
  sudo apt install libgtk-3-dev libayatana-appindicator3-dev libxkbcommon-dev
  ```

### Build & run

```sh
cargo run                 # start the tray process (or `cargo run -- run`)
cargo run -- enable-autostart
cargo run -- status
cargo run -- disable-autostart
cargo test
cargo clippy --all-targets -- -D warnings
```

Increase log verbosity with `-v` (debug) or `-vv` (trace); `RUST_LOG` overrides
the level. Logs go to a per-user file and to stderr.

| OS | Log file |
|----|----------|
| Windows | `%LOCALAPPDATA%\traycli\tray-cli\data\logs\tray-cli.log.<date>` |
| macOS | `~/Library/Application Support/tray-cli/logs/tray-cli.log.<date>` |
| Linux | `~/.local/state/tray-cli/logs/tray-cli.log.<date>` |

## Behaviour notes

* **No subcommand ⇒ `run`.** Auto-start launches the bare executable, so that
  case must run the tray.
* **Single instance.** `run` first acquires an [`InstanceLock`]; a second copy
  exits with `another instance is already running` instead of drawing a second
  icon.
* **Auto-start path.** The executable registered is whatever `current_exe()`
  returns. During development that is `target/debug/tray-cli` — install the
  binary somewhere stable before enabling production auto-start (see
  [`docs/packaging.md`](docs/packaging.md)).
* **Icon.** The default icon is rendered procedurally into raw RGBA so the crate
  needs no image decoder or binary assets. See [`docs/packaging.md`](docs/packaging.md)
  to ship real artwork.

## Implementation notes (library API drift)

The architecture is stable; the desktop crates move fast. This code targets:

| Crate | Version | Note |
|-------|---------|------|
| `winit` | 0.30 | `EventLoop::with_user_event()` + `run_app(ApplicationHandler)`. `ControlFlow::Exit` is gone — quit via `ActiveEventLoop::exit()`. |
| `tray-icon` | 0.26 | `TrayIconBuilder::with_menu(Box<dyn ContextMenu>)`; `menu` re-exports `muda`. |
| `muda` | 0.21 | `MenuEvent { id: MenuId }`; items are kept alive by the owning `TrayMenu`. |
| `auto-launch` | 0.6 | Builder methods take `&mut self`; `set_use_launch_agent` is deprecated → `set_macos_launch_mode`. |

Tray/menu events are delivered on background threads, so the adapters install
`set_event_handler` callbacks that forward them through the winit
`EventLoopProxy`. That marshals them onto the main thread *and* wakes the idle
event loop — the pattern the `tray-icon` docs recommend.

See [`docs/platform-notes.md`](docs/platform-notes.md) for per-OS caveats and
[`docs/autostart.md`](docs/autostart.md) for how registration works.

## License

MIT OR Apache-2.0.
