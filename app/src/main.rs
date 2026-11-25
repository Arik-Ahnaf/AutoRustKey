//! AutoHotkey Dash and tools, rebuilt in Rust for Linux (Arch + KDE Plasma).
//!
//! Usage:
//!   autohotkey-ux                      the Dash
//!   autohotkey-ux --new [path]         New Script window
//!   autohotkey-ux --launch-config      Launch settings
//!   autohotkey-ux --editor             Editor settings
//!   autohotkey-ux --edit <script>      edit a script with the chosen editor
//!   autohotkey-ux --spy                Window Spy
//!   autohotkey-ux --run [switches] <script.ahk> [args]   run a script with the right AutoHotkey
//!   autohotkey-ux --screenshot <file.png> [--window dash|new|launch|editor|spy]

#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod assoc;
mod config;
mod dash;
mod editor;
mod icons;
mod launchcfg;
mod launcher;
mod newscript;
mod pick;
mod spy;
mod sys;
mod theme;

use app::{App, Kind, Shared, Win};
use std::path::PathBuf;

fn run_window(kind: Kind, screenshot: Option<PathBuf>, mut shared: Shared) -> Shared {
    let win = Win::create(kind, &mut shared);
    let mut vb = win.builder();
    if screenshot.is_some() {
        // Checks must never steal focus or cover what the person is doing.
        vb = vb.with_active(false);
        #[cfg(windows)]
        {
            vb = vb.with_window_level(egui::WindowLevel::AlwaysOnBottom);
        }
    }
    let options = eframe::NativeOptions { viewport: vb, ..Default::default() };
    let picked = shared.picked.clone();
    let result = eframe::run_native("autohotkey-ux", options, Box::new(move |cc| Ok(Box::new(App::new(cc, shared, win, screenshot)))));
    if let Err(e) = result {
        eprintln!("autohotkey-ux: {e}");
    }
    let mut s = Shared::new();
    s.picked = picked;
    s
}

/// Tell the person something outside a window (KDE dialog when available).
fn notify(title: &str, text: &str) {
    eprintln!("{title}: {text}");
    #[cfg(not(windows))]
    {
        if sys::which("kdialog").is_some() {
            let _ = std::process::Command::new("kdialog").args(["--title", title, "--sorry", text]).status();
        } else if sys::which("notify-send").is_some() {
            let _ = std::process::Command::new("notify-send").args([title, text]).status();
        }
    }
}

/// launcher.ahk Main(): switches, script path, then IdentifyAndLaunch.
fn run_launcher(mut args: Vec<String>) -> i32 {
    let mut switches = Vec::new();
    let mut run_with: Option<String> = None;
    let mut which = false;
    let mut no_wait = false;
    let mut script: Option<String> = None;
    while !args.is_empty() {
        let a = args.remove(0);
        if !a.starts_with('/') {
            script = Some(a);
            break;
        }
        let next = |args: &mut Vec<String>| -> Option<String> { (!args.is_empty()).then(|| args.remove(0)) };
        match a.to_ascii_lowercase().as_str() {
            "/runwith" => run_with = next(&mut args),
            "/launch" => no_wait = true,
            "/which" => which = true,
            "/ilib" | "/include" => {
                switches.push(a.clone());
                if let Some(v) = next(&mut args) {
                    switches.push(v);
                }
            }
            _ => switches.push(a),
        }
    }
    let shared0 = Shared::new();
    let script = match script {
        Some(s) => PathBuf::from(s),
        None => {
            let candidates = [sys::ahk_user_dir().join("AutoHotkey.ahk"), sys::documents_dir().join("AutoHotkey.ahk")];
            match candidates.into_iter().find(|p| p.is_file()) {
                Some(p) => p,
                None => {
                    run_window(Kind::Dash, None, shared0);
                    return 0;
                }
            }
        }
    };
    let mut shared = shared0;
    let list = shared.interpreters().to_vec();
    let lp = launcher::launch_parameters(&shared.cfg, &list, &script, run_with.as_deref());
    if which {
        println!("{}\n{}\n{}", lp.version, lp.exe.as_ref().map(|e| e.path.to_string_lossy().into_owned()).unwrap_or_default(), lp.switches.first().cloned().unwrap_or_default());
        return lp.exe.as_ref().and_then(|e| launcher::major(&e.version)).unwrap_or(0) as i32;
    }
    if let Some(err) = &lp.error {
        notify("AutoHotkey", err);
        return 2;
    }
    let exe = match lp.exe {
        Some(e) => e,
        None if lp.ask.len() > 1 => {
            let s = run_window(Kind::Pick(script.clone(), lp.ask.clone()), None, shared);
            let picked = s.picked.lock().unwrap().clone();
            match picked {
                Some(e) => e,
                None => return 0,
            }
        }
        None => {
            notify("AutoHotkey", &format!("No AutoHotkey interpreter was found.\n\nSet the AutoHotkey folder with AHK_ROOT or in {}.", config::config_path().display()));
            return 2;
        }
    };
    let mut sw = switches;
    if launcher::major(&exe.version) == Some(1) && shared_v1_utf8() {
        sw.push("/CP65001".into());
    }
    if no_wait {
        let mut cmd = sys::windows_program_command(&exe.path);
        cmd.extend(sw);
        cmd.push(script.to_string_lossy().into_owned());
        cmd.extend(args);
        return sys::spawn(&cmd, None).map(|c| c.id() as i32).unwrap_or(2);
    }
    launcher::launch_script(&exe.path, &script, &args, &sw)
}

fn shared_v1_utf8() -> bool {
    config::Config::load().get_bool("Launcher\\v1", "UTF8", false)
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut screenshot: Option<PathBuf> = None;
    let mut kind = Kind::Dash;
    let mut i = 0;
    while i < args.len() {
        let a = args[i].clone();
        let take_opt = |args: &Vec<String>, i: usize| args.get(i + 1).filter(|n| !n.starts_with("--")).cloned();
        match a.as_str() {
            "--screenshot" => {
                screenshot = args.get(i + 1).map(PathBuf::from);
                i += 2;
                continue;
            }
            "--window" => {
                kind = match args.get(i + 1).map(|s| s.as_str()) {
                    Some("new") => Kind::NewScript(None),
                    Some("launch") => Kind::Launch,
                    Some("editor") => Kind::Editor(None),
                    Some("spy") => Kind::Spy,
                    _ => Kind::Dash,
                };
                i += 2;
                continue;
            }
            "--new" => {
                let p = take_opt(&args, i).map(PathBuf::from);
                i += if p.is_some() { 2 } else { 1 };
                kind = Kind::NewScript(p);
                continue;
            }
            "--launch-config" => kind = Kind::Launch,
            "--editor" => {
                let p = take_opt(&args, i).map(PathBuf::from);
                i += if p.is_some() { 2 } else { 1 };
                kind = Kind::Editor(p);
                continue;
            }
            "--spy" => kind = Kind::Spy,
            "--edit" => {
                let Some(f) = args.get(i + 1).map(PathBuf::from) else {
                    eprintln!("--edit needs a file");
                    std::process::exit(1);
                };
                let cfg = config::Config::load();
                let cmd = cfg.get_str("Editor", "Command", "");
                if !cmd.is_empty() && editor::find_executable(&cmd).is_some() {
                    let _ = sys::spawn(&sys::expand_command(&cmd, &f), None);
                    return;
                }
                kind = Kind::Editor(Some(f));
                i += 2;
                continue;
            }
            "--run" => {
                let rest = args.split_off(i + 1);
                std::process::exit(run_launcher(rest));
            }
            "--install-desktop" => {
                match assoc::set_open_command("AutoHotkey Launcher", &assoc::launcher_command()) {
                    Ok(()) => println!("Registered .ahk files and the AutoHotkey Dash menu entry."),
                    Err(e) => eprintln!("{e}"),
                }
                return;
            }
            "--help" | "-h" => {
                println!("{}", include_str!("main.rs").lines().take(12).skip(2).map(|l| l.trim_start_matches("//!")).collect::<Vec<_>>().join("\n"));
                return;
            }
            other if other.to_ascii_lowercase().ends_with(".ahk") || other.starts_with('/') => {
                let rest = args.split_off(i);
                std::process::exit(run_launcher(rest));
            }
            _ => {}
        }
        i += 1;
    }
    run_window(kind, screenshot, Shared::new());
}
