# AutoRustKey

**AutoRustKey** is a complete rebuild of the AutoHotkey UX suite (Dash, New Script, Launch settings, Editor settings, Window Spy, and version launcher) written entirely in **Rust** (`eframe` / `egui`). While it was built and packaged for **Arch Linux with KDE Plasma**, it is **not limited to Arch or KDE** — it can be built and run on other Linux distributions and partially on Windows.

This project was rebuilt from the original Windows AutoHotkey UX application using **CraftAnything**.

## Project Structure

- [`app/`](./app) — Rust source code (`src/`), `Cargo.toml`, Arch Linux `PKGBUILD`, desktop/MIME packaging files (`packaging/`), and screenshots (`shots/`). See [`app/README.md`](./app/README.md) for full CLI flags and setup details.
- `craft.json`, `investigation.cany/`, `checks/` — CraftAnything project metadata, investigation notes, and verification artifacts.

## Platform Support & Criteria

### Other Linux Distributions
You can build and run `autohotkey-ux` on any Linux distro with `cargo build --release` as long as the following criteria are met:

- **Build & GUI libraries**: Rust/Cargo toolchain plus standard `eframe`/`egui` system dependencies (`libx11`, `wayland`, `libxkbcommon`, `libglvnd`, `xdg-utils`, `xdg-desktop-portal`).
- **Running scripts**: `wine` is required to execute `.ahk` scripts and `Ahk2Exe.exe`, along with a Windows AutoHotkey installation (e.g., under `~/.local/share/AutoHotkey`, `~/.wine/drive_c/Program Files/AutoHotkey`, `/opt/autohotkey`, or `AHK_ROOT`).
- **Desktop integration**: Uses standard freedesktop tools (`xdg-open`, `gio`, `xdg-mime`, `update-mime-database`, `update-desktop-database`, `org.freedesktop.FileManager1`, `notify-send`); KDE tools (`kbuildsycoca6`, `dolphin`, `kdialog`) are used if present.
- **Window Spy**:
  - **X11 / XWayland** (any DE/distro): requires `xdotool`, `xprop` (`xorg-xprop`), and ImageMagick's `import` (for pixel color).
  - **Wayland**: requires **KDE Plasma** and `kdotool` (non-KDE Wayland compositors such as GNOME or Sway are not supported by Window Spy).

### Arch Linux + KDE Plasma (Packaged)

```sh
cd app
sudo pacman -S --needed base-devel rust wine
makepkg -si
```

### Windows (Partial Support)
The binary compiles and runs natively on Windows (running AutoHotkey `.exe`s directly without Wine, using `explorer /select` and `hh.exe` for `.chm` docs), with these limitations:
- **Window Spy** is not implemented for Windows (it relies on `xdotool`/`kdotool` and `/proc`).
- **File associations** (`--install-desktop` / Dash association settings) are a no-op on Windows and do not write registry verbs.
- **Editor picker** only auto-lists Notepad (custom editor commands can still be configured manually).
- **Settings** are stored in JSON (`%APPDATA%\autohotkey-ux\config.json` via `dirs::config_dir()`) instead of the Windows Registry.

For full details and CLI flags, see [`app/README.md`](./app/README.md).
