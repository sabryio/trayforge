# tray-cli

A small, production-shaped **CLI/background utility** that shows a **system tray
icon** with a menu and can register itself to **start at login** — built in Rust
with a clean, ports-and-adapters (hexagonal) architecture.

A pure terminal program cannot draw a tray icon: the icon is created through
desktop GUI APIs. So the binary is a CLI entrypoint that either opens an
interactive **terminal dashboard** (a [ratatui](https://ratatui.rs) control
panel) or starts a long-running desktop event loop.

```text
tray-cli              # open the interactive terminal dashboard
tray-cli tui          #   …same, explicitly
tray-cli run          # start the tray process (headless, used by auto-start)
tray-cli enable-autostart
tray-cli disable-autostart
tray-cli status
```

The dashboard is a live control panel: it shows auto-start and process state,
starts/stops the tray agent, flips auto-start, opens the log folder and re-reads
status — all with the keyboard.

```text
╭ tray-cli ────────────────────────────────────────────────────────────────╮
│ a terminal control panel for the tray agent   ·   v0.1.0                 │
╰──────────────────────────────────────────────────────────────────────────╯
╭ Commands ───────────────────────────╮╭ Status ───────────────────────────╮
│ ▸ ▶  Start tray agent               ││ Auto-start    ●  enabled on login  │
│      Launch the tray icon…          ││ Tray agent    ●  running · pid …   │
│   ■  Stop tray agent                ││ Instance lock ●  available         │
│   ⇄  Toggle auto-start              ││                                    │
│   ↻  Refresh status                 ││ Executable                         │
│   ▤  Open log folder                ││ …\target\debug\tray-cli.exe        │
│   ✕  Quit dashboard                 ││                                    │
╰─────────────────────────────────────╯╰────────────────────────────────────╯
╭ Activity ────────────────────────────────────────────────────────────────╮
│ [00:00]  ✓  tray agent started (pid 4242)                                │
│ [00:00]  •  dashboard ready                                              │
╰──────────────────────────────────────────────────────────────────────────╯
⠋ live                        ↑/↓  move    ⏎  run    r  refresh    q  quit
```

## Why this shape

The interesting part is not the tray icon — it is that the tray icon, the login
registration and the single-instance guard are all *replaceable* and *testable*.
The core use-case layer never learns which OS API is behind a port, and it never
prints: it returns structured [`Outcome`](src/app.rs) values that a *driving
adapter* (the CLI or the dashboard) renders.

```text
src/
├─ main.rs            # composition root: parse, build adapters, inject
├─ lib.rs             # public module surface
├─ app.rs             # Application: use cases + Outcome (policy)
├─ cli.rs             # clap model — argv in, `Command` out
├─ presentation.rs    # render an Outcome for the CLI
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
├─ tray/              # tray runtime port + {desktop, mock} adapters
│  ├─ mod.rs          #   trait TrayRunner
│  ├─ desktop.rs      #   winit event loop + tray-icon + muda
│  ├─ menu.rs         #   menu construction
│  ├─ icon.rs         #   procedural RGBA icon (no asset/decoder deps)
│  └─ mock.rs
├─ supervisor/        # supervised tray child-process port + adapters
│  ├─ mod.rs          #   trait TraySupervisor + TrayState
│  ├─ desktop.rs      #   spawn/kill `tray-cli run` as a child
│  └─ mock.rs
└─ tui/               # interactive dashboard (driving adapter)
   ├─ mod.rs          #   terminal loop + orchestration
   ├─ command.rs      #   declarative command catalogue
   ├─ dashboard.rs    #   state machine + key handling (ratatui-free)
   └─ view.rs         #   rendering
```

Dependencies point **inward**: `app` depends on the traits, the traits know
nothing about the platform, and only `main.rs` picks real adapters.

## How it maps to SOLID

| Principle | Where |
|-----------|-------|
| **S**ingle responsibility | `cli` parses, `config` detects, `app` orchestrates, each adapter does exactly one OS thing, `tui` presents, `presentation` formats. |
| **O**pen/closed | Add `WindowsServiceAutoStart`, `HeadlessTrayRunner`, or a different lock by implementing the trait — no core change. |
| **L**iskov | Any `AutoStartService`/`TrayRunner`/`InstanceLock`/`TraySupervisor` swaps in without surprising the orchestrator; the mocks prove it. |
| **I**nterface segregation | Five tiny traits (`enable/disable/is_enabled`, `run`, `try_acquire/is_available`, `start/stop/state`) instead of one fat "system services" trait. |
| **D**ependency inversion | `Application` holds `Box<dyn ...>`; `main.rs` is the only place that names concrete platform types. |


## Usage

### Requirements

* Rust **1.99+** (`rust-version` in `Cargo.toml`; ratatui 0.30 needs 1.88+).
* **Linux**: GTK 3 and an AppIndicator implementation, plus winit's X11/Wayland
  dev libraries, to build:
  ```sh
  sudo apt install libgtk-3-dev libayatana-appindicator3-dev libxkbcommon-dev
  ```

### Build & run

```sh
cargo run                 # open the interactive dashboard
cargo run -- tui          #   …same, explicitly
cargo run -- run          # start the tray process (headless)
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

* **No subcommand ⇒ dashboard.** Running the bare executable opens the
  interactive control panel. Auto-start therefore registers the binary *with* the
  `run` argument, so logging in launches the headless tray agent rather than
  opening a terminal UI.
* **The dashboard supervises a child process.** The tray's desktop event loop
  wants the OS main thread and the dashboard owns the terminal, so "Start tray
  agent" launches `tray-cli run` as a hidden child and tracks its pid. Quitting
  the dashboard stops the child it started, so no agent is ever stranded.
* **Single instance.** `run` first acquires an [`InstanceLock`]; a second copy
  exits with `another instance is already running` instead of drawing a second
  icon. The dashboard *probes* the lock (read-only) to show whether an instance
  is already running.
* **Auto-start path.** The executable registered is whatever `current_exe()`
  returns plus `run`. During development that is `target/debug/tray-cli run` —
  install the binary somewhere stable before enabling production auto-start (see
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
| `ratatui` | 0.30 | Split into `ratatui-core`/`ratatui-widgets`/`ratatui-crossterm`; `Style` no longer implements `Styled` (its `Stylize` methods are inherent now), `Backend` gained an associated `Error` type. crossterm is used through `ratatui::crossterm` to avoid a version clash. |

Tray/menu events are delivered on background threads, so the adapters install
`set_event_handler` callbacks that forward them through the winit
`EventLoopProxy`. That marshals them onto the main thread *and* wakes the idle
event loop — the pattern the `tray-icon` docs recommend.

See [`docs/platform-notes.md`](docs/platform-notes.md) for per-OS caveats and
[`docs/autostart.md`](docs/autostart.md) for how registration works.

## License

MIT OR Apache-2.0.
