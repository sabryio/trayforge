# trayforge

A small, production-shaped **CLI/background utility** that shows a **system tray
icon** with a menu and can register itself to **start at login** — built in Rust
with a clean, ports-and-adapters (hexagonal) architecture.

The project is a **Cargo workspace** with a reusable library and a thin
application on top of it:

| Crate | Published as | What it is |
|-------|--------------|------------|
| [`crates/trayforge`](crates/trayforge) | **`trayforge`** | The reusable library: use-case orchestration, ports and platform adapters. No `clap`, no `ratatui`. |
| [`crates/trayforge-cli`](crates/trayforge-cli) | **`trayforge-cli`** | The application: the `clap` command surface, the `ratatui` dashboard and human-readable output. Installs a `trayforge` binary. |

A pure terminal program cannot draw a tray icon: the icon is created through
desktop GUI APIs. So the binary is a CLI entrypoint that either opens an
interactive **terminal dashboard** (a [ratatui](https://ratatui.rs) control
panel) or starts a long-running desktop event loop.

```text
trayforge              # open the interactive terminal dashboard
trayforge tui          #   …same, explicitly
trayforge run          # start the tray process (headless, used by auto-start)
trayforge enable-autostart
trayforge disable-autostart
trayforge status
```

The dashboard is a live control panel: it shows auto-start and process state,
starts/stops the tray agent, flips auto-start, opens the log folder and re-reads
status — all with the keyboard.

```text
╭ trayforge ───────────────────────────────────────────────────────────────╮
│ a terminal control panel for the tray agent   ·   v0.1.0                 │
╰──────────────────────────────────────────────────────────────────────────╯
╭ Commands ───────────────────────────╮╭ Status ───────────────────────────╮
│ ▸ ▶  Start tray agent               ││ Auto-start    ●  enabled on login  │
│      Launch the tray icon…          ││ Tray agent    ●  running · pid …   │
│   ■  Stop tray agent                ││ Instance lock ●  available         │
│   ⇄  Toggle auto-start              ││                                    │
│   ↻  Refresh status                 ││ Executable                         │
│   ▤  Open log folder                ││ …\target\debug\trayforge.exe       │
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
prints: it returns structured [`Outcome`](crates/trayforge/src/app.rs) values
that a *driving adapter* (the CLI or the dashboard) renders.

The library/application split is what makes the domain reusable: another
front-end (a GUI, a service, a different CLI) can depend on `trayforge` and reuse
all of the policy without dragging a terminal toolkit along.

```text
trayforge/                         # workspace root
├─ Cargo.toml                      # [workspace] manifest
├─ crates/
│  ├─ trayforge/                   # ── reusable library (publishable) ──
│  │  ├─ src/
│  │  │  ├─ lib.rs                 # public module surface
│  │  │  ├─ app.rs                 # Application: use cases + Outcome (policy)
│  │  │  ├─ config.rs              # AppConfig: identity + paths
│  │  │  ├─ error.rs               # AppError: one error model
│  │  │  ├─ logging.rs             # tracing → rotating file + stderr
│  │  │  ├─ instance/              # single-instance port + {desktop, mock}
│  │  │  │  ├─ mod.rs              #   trait InstanceLock
│  │  │  │  ├─ desktop.rs          #   OS mutex / abstract socket / flock
│  │  │  │  └─ mock.rs
│  │  │  ├─ autostart/             # login/auto-start port + {desktop, mock}
│  │  │  │  ├─ mod.rs              #   trait AutoStartService
│  │  │  │  ├─ desktop.rs          #   Run key / LaunchAgent / XDG autostart
│  │  │  │  └─ mock.rs
│  │  │  ├─ tray/                  # tray runtime port + {desktop, mock}
│  │  │  │  ├─ mod.rs              #   trait TrayRunner
│  │  │  │  ├─ desktop.rs          #   winit event loop + tray-icon + muda
│  │  │  │  ├─ menu.rs             #   menu construction
│  │  │  │  ├─ icon.rs             #   procedural RGBA icon (no asset deps)
│  │  │  │  └─ mock.rs
│  │  │  └─ supervisor/            # supervised tray child (port + adapters)
│  │  │     ├─ mod.rs              #   trait TraySupervisor + TrayState
│  │  │     ├─ desktop.rs          #   spawn/kill `trayforge run` as a child
│  │  │     └─ mock.rs
│  │  └─ tests/application.rs      # library used through its public surface
│  └─ trayforge-cli/               # ── application (publishable) ──
│     └─ src/
│        ├─ main.rs                # composition root: parse, build, inject
│        ├─ lib.rs                 # presentation modules (for tests)
│        ├─ cli.rs                 # clap model — argv in, `Command` out
│        ├─ presentation.rs        # render an Outcome for the CLI
│        └─ tui/                   # interactive dashboard (driving adapter)
│           ├─ mod.rs              #   terminal loop + orchestration
│           ├─ command.rs          #   declarative command catalogue
│           ├─ dashboard.rs        #   state machine + keys (ratatui-free)
│           └─ view.rs             #   rendering
├─ docs/                           # platform notes, packaging, autostart
└─ .github/workflows/ci.yml
```

Dependencies point **inward**: `app` depends on the traits, the traits know
nothing about the platform, and only `main.rs` picks real adapters. The
application crate adds a second, outer layer of inversion — it depends on
`trayforge`, never the other way around.

## How it maps to SOLID

| Principle | Where |
|-----------|-------|
| **S**ingle responsibility | `trayforge-cli`'s `cli` parses, `presentation` formats, `tui` presents; `trayforge`'s `config` detects, `app` orchestrates, each adapter does exactly one OS thing. |
| **O**pen/closed | Add `WindowsServiceAutoStart`, `HeadlessTrayRunner`, or a different lock by implementing the trait — no core change. |
| **L**iskov | Any `AutoStartService`/`TrayRunner`/`InstanceLock`/`TraySupervisor` swaps in without surprising the orchestrator; the mocks prove it. |
| **I**nterface segregation | Five tiny traits (`enable/disable/is_enabled`, `run`, `try_acquire/is_available`, `start/stop/state`) instead of one fat "system services" trait. |
| **D**ependency inversion | `Application` holds `Box<dyn ...>`; `main.rs` is the only place that names concrete platform types — and the *published library* depends on no presentation crate at all. |

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
cargo run -p trayforge-cli              # open the interactive dashboard
cargo run -p trayforge-cli -- tui       #   …same, explicitly
cargo run -p trayforge-cli -- run       # start the tray process (headless)
cargo run -p trayforge-cli -- enable-autostart
cargo run -p trayforge-cli -- status
cargo run -p trayforge-cli -- disable-autostart

cargo test --workspace                  # unit + integration + doc tests
cargo clippy --workspace --all-targets -- -D warnings
```

Increase log verbosity with `-v` (debug) or `-vv` (trace); `RUST_LOG` overrides
the level. Logs go to a per-user file and to stderr.

| OS | Log file |
|----|----------|
| Windows | `%LOCALAPPDATA%\trayforge\trayforge\logs\trayforge.log.<date>` |
| macOS | `~/Library/Application Support/trayforge/logs/trayforge.log.<date>` |
| Linux | `~/.local/state/trayforge/logs/trayforge.log.<date>` |

## Behaviour notes

* **No subcommand ⇒ dashboard.** Running the bare executable opens the
  interactive control panel. Auto-start therefore registers the binary *with* the
  `run` argument, so logging in launches the headless tray agent rather than
  opening a terminal UI.
* **The dashboard supervises a child process.** The tray's desktop event loop
  wants the OS main thread and the dashboard owns the terminal, so "Start tray
  agent" launches `trayforge run` as a hidden child and tracks its pid. Quitting
  the dashboard stops the child it started, so no agent is ever stranded.
* **Single instance.** `run` first acquires an `InstanceLock`; a second copy
  exits with `another instance is already running` instead of drawing a second
  icon. The dashboard *probes* the lock (read-only) to show whether an instance
  is already running.
* **Auto-start path.** The executable registered is whatever `current_exe()`
  returns plus `run`. During development that is `target/debug/trayforge run` —
  install the binary somewhere stable before enabling production auto-start (see
  [`docs/packaging.md`](docs/packaging.md)).
* **Icon.** The default icon is rendered procedurally into raw RGBA so the crate
  needs no image decoder or binary assets. See [`docs/packaging.md`](docs/packaging.md)
  to ship real artwork.

## Publishing

Both crates carry `description`, `keywords`, `categories`, `license` and their
own README. `trayforge-cli` depends on `trayforge` by *path + version*, so they
can be published in dependency order:

```sh
cargo publish -p trayforge          # the library first
cargo publish -p trayforge-cli      # then the app
```

Installing the app gives you the `trayforge` binary:

```sh
cargo install trayforge-cli
```

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
