# AutoHotkey Dash (Rust)

A Rust/egui rebuild of AutoHotkey's UX tools (the Dash, New Script, Launch
settings, Editor settings, Window Spy and the version launcher). Originally
packaged for **Arch Linux with KDE Plasma**, it is **not limited to Arch or
KDE** — it runs on other Linux distributions and partially on Windows. On
Linux, AutoHotkey itself is a Windows program, so scripts and Ahk2Exe run
through **Wine**.

## Install

### Arch Linux + KDE Plasma

```sh
sudo pacman -S --needed base-devel rust wine
makepkg -si            # in this folder
```

Optional: `kdotool` (AUR, Window Spy on Wayland), `xdotool xorg-xprop imagemagick`
(Window Spy on X11), `kdialog`.

### Other Linux Distributions

Build with `cargo build --release --locked` (in this folder) and install
`target/release/autohotkey-ux`. Criteria:

- **GUI & desktop**: `libx11`, `wayland`, `libxkbcommon`, `libglvnd`, `xdg-utils`, `xdg-desktop-portal`
- **Running scripts**: `wine`
- **Window Spy**: `xdotool`, `xprop`, and ImageMagick (`import`) on X11/XWayland (any DE); `kdotool` on Wayland (KDE Plasma only — non-KDE Wayland compositors are not supported)

### Windows (Partial)

Builds with `cargo build --release` and runs `.exe` interpreters directly without
Wine, but **Window Spy** (requires `xdotool`/`kdotool` and `/proc`) and
**registry file associations** are not implemented on Windows, and settings are
stored in JSON rather than the Windows Registry.

Put an AutoHotkey install where the app can find it (any of):
`~/.local/share/AutoHotkey`, `~/.wine/drive_c/Program Files/AutoHotkey`,
`/opt/autohotkey`, or set `AHK_ROOT=/path/to/AutoHotkey`. It should contain
`v2/AutoHotkey64.exe` (and optionally `v1.1.x/`, `Compiler/Ahk2Exe.exe`, `UX/installed-files.csv`).

## Use

| Command | What it opens |
|---|---|
| `autohotkey-ux` | the Dash |
| `autohotkey-ux --new [folder/or/file.ahk]` | New Script |
| `autohotkey-ux --launch-config` | Launch settings |
| `autohotkey-ux --editor` | Editor settings |
| `autohotkey-ux --spy` | Window Spy |
| `autohotkey-ux --run script.ahk [args]` | run a script with the right AutoHotkey version |
| `autohotkey-ux --edit script.ahk` | open a script in your chosen editor |
| `autohotkey-ux --install-desktop` | make .ahk files open with the launcher |

Settings live in `~/.config/autohotkey-ux/config.json` (the original used the
Windows registry under `HKCU\Software\AutoHotkey`).
