# Packaging

## 1. Install to a stable path

Auto-start registers `current_exe() run`, so never ship `target/{debug,release}`.
Install somewhere stable:

| OS | Suggested location |
|----|--------------------|
| Windows | `%LOCALAPPDATA%\Programs\trayforge\trayforge.exe` or `C:\Program Files\trayforge\` |
| macOS | inside `Trayforge.app/Contents/MacOS/trayforge` |
| Linux | `/usr/local/bin/trayforge` or `~/.local/bin/trayforge` |

Then a first-run step (or the installer) can call `trayforge enable-autostart`.

## 2. Build a release binary

```sh
cargo build --release        # target/release/trayforge(.exe)
```

The release profile enables thin LTO, one codegen unit and symbol stripping.

## 3. Icons

The default icon is generated at runtime (`crates/trayforge/src/tray/icon.rs`), so there is no
binary asset to ship. For real artwork:

* Generate the RGBA bitmap yourself and call `Icon::from_rgba`, or
* On Windows, embed an `.ico` in the executable and load it with
  `Icon::from_resource(<ordinal>, None)`, or
* Add an image decoder (e.g. the `image` crate) and use your PNG/ICO bytes via
  `include_bytes!`.

Swap the body of `default_icon` and the rest of the app is unaffected.

## 4. Windows installer

* Sign `trayforge.exe`.
* Use WiX or NSIS to copy the binary to a stable path, optionally run
  `trayforge enable-autostart` on first launch, and remove the Run entry on
  uninstall (`trayforge disable-autostart`).

## 5. macOS bundle

* Wrap the binary in `Trayforge.app`.
* Add `LSUIElement = true` to `Info.plist` for a menu-bar-only app.
* Notarize and staple the bundle before distribution.
* Switch auto-start to `MacOSLaunchMode::SMAppService` (macOS 13+).

## 6. Linux packages

* Ship the binary `/usr/local/bin/trayforge`.
* Recommend the `ksni` tray backend (see `platform-notes.md`).
* Provide an XDG autostart entry, or let `trayforge enable-autostart` create it.

## 7. Versioning & releases

* Tag releases (`vX.Y.Z`); the version flows from `Cargo.toml` into `--version`
  via `clap`.
* Keep `rust-toolchain.toml` pinned so CI and release builds agree.
