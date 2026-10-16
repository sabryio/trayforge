# trayforge-cli

**A terminal control panel and CLI for cross-platform tray-icon background apps.**

This is the application half of the [`trayforge`](https://crates.io/crates/trayforge)
project: a `clap` command surface plus a live [`ratatui`](https://ratatui.rs)
dashboard. All policy lives in the `trayforge` library; this crate only decides
how to talk to a human.

```text
cargo install trayforge-cli     # installs a `trayforge` binary
```

## Commands

```text
trayforge                       # open the interactive terminal dashboard
trayforge tui                   #   …same, explicitly
trayforge run                   # start the tray process (headless, used by auto-start)
trayforge enable-autostart      # register the app to start at login
trayforge disable-autostart     # remove it from the login items
trayforge status                # print whether auto-start is enabled
```

Running the bare executable opens the dashboard:

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

Keys: `↑`/`↓` (or `j`/`k`) move, `Home`/`End` jump, `⏎` runs, `r` refreshes,
`q`/`Esc`/`Ctrl-C` quit.

## Notes

* **No subcommand ⇒ dashboard.** Auto-start therefore registers the binary *with*
  the `run` argument, so logging in launches the headless tray agent rather than
  opening a terminal UI.
* **The dashboard supervises a child process.** The tray's desktop event loop
  wants the OS main thread and the dashboard owns the terminal, so "Start tray
  agent" launches `trayforge run` as a hidden child and tracks its pid. Quitting
  the dashboard stops the child it started.
* Increase log verbosity with `-v` (debug) or `-vv` (trace); `RUST_LOG`
  overrides the level.

## Building on Linux

The tray backend needs GTK 3, an AppIndicator implementation and winit's
X11/Wayland dev libraries:

```sh
sudo apt install libgtk-3-dev libayatana-appindicator3-dev libxkbcommon-dev
```

## MSRV

Rust **1.99+**.

## License

MIT OR Apache-2.0.
