# AutoHotkey Dash for Linux (Rust)

A Rust/egui rebuild of AutoHotkey's UX tools (the Dash, New Script, Launch
settings, Editor settings, Window Spy and the version launcher), made for Arch
Linux with KDE Plasma. AutoHotkey itself is a Windows program, so scripts and
Ahk2Exe run through **Wine**.

## Install on Arch

```sh
sudo pacman -S --needed base-devel rust wine
makepkg -si            # in this folder
```

Optional: `kdotool` (AUR, Window Spy on Wayland), `xdotool xorg-xprop imagemagick`
(Window Spy on X11), `kdialog`.

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
