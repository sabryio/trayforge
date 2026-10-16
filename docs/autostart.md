# How auto-start works

`tray-cli` uses the [`auto-launch`](https://crates.io/crates/auto-launch) crate
behind the `AutoStartService` port. The mechanism is entirely OS-specific, which
is exactly why it lives behind a trait.

| OS | Mechanism | Location |
|----|-----------|----------|
| Windows | Registry `Run` value (per-user) | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` |
| macOS | LaunchAgent plist (SMAppService on newer systems) | `~/Library/LaunchAgents` |
| Linux/BSD | XDG autostart entry | `~/.config/autostart/*.desktop` |

## What gets registered

The value written is `current_exe()` — the path of the running binary — plus the
app name (`tray-cli`) as the entry name, **and the `run` argument**. So:

* Running `tray-cli enable-autostart` from `target/debug` registers
  `...\target\debug\tray-cli.exe run`. Install to a stable location first (see
  `packaging.md`).
* Logging in launches the **tray agent** directly. The interactive dashboard is
  only opened when a human runs the bare executable, so auto-start never pops a
  terminal UI at login.

## macOS LaunchAgent vs SMAppService

`auto-launch` 0.6 allows choosing the macOS strategy:

```rust
builder.set_macos_launch_mode(MacOSLaunchMode::LaunchAgent)
```

`LaunchAgent` writes a plist and is the most predictable for a plain binary.
`SMAppService` requires macOS 13+ and a proper `.app` bundle; it is the right
choice once you ship a bundled, notarized app. Flip the mode in
`src/autostart/desktop.rs` when you move to bundling.

## Verifying by hand

Windows:

```powershell
Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name tray-cli
```

macOS:

```sh
ls ~/Library/LaunchAgents | grep -i tray
launchctl list | grep -i tray
```

Linux:

```sh
cat ~/.config/autostart/tray-cli.desktop
```

`tray-cli status` asks the same backend whether the entry exists, so it is the
portable way to check.
