# AutoRustKey

**AutoRustKey** is a complete rebuild of the AutoHotkey UX suite (Dash, New Script, Launch settings, Editor settings, Window Spy, and version launcher) written entirely in **Rust** (`eframe` / `egui`) for **Arch Linux with KDE Plasma**.

This project was rebuilt from the original Windows AutoHotkey UX application using **CraftAnything**.

## Project Structure

- [`app/`](./app) — Rust source code (`src/`), `Cargo.toml`, Arch Linux `PKGBUILD`, desktop/MIME packaging files (`packaging/`), and screenshots (`shots/`). See [`app/README.md`](./app/README.md) for build, installation, and usage instructions on Arch Linux.
- `craft.json`, `investigation.cany/`, `checks/` — CraftAnything project metadata, investigation notes, and verification artifacts.

## Quick Start (Arch Linux + KDE Plasma)

```sh
cd app
sudo pacman -S --needed base-devel rust wine
makepkg -si
```

For full details and CLI flags, see [`app/README.md`](./app/README.md).
