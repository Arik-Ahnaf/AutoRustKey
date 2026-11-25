//! Operating-system helpers: folders, opening files and links, starting programs.
//! Windows-only actions of the original (ShellExecute, registry verbs, Explorer)
//! are mapped to their Linux / KDE Plasma equivalents here.

use crate::config::Config;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The AutoHotkey installation folder (the original's ROOT_DIR).
pub fn root_dir(cfg: &Config) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(p) = std::env::var("AHK_ROOT") {
        candidates.push(PathBuf::from(p));
    }
    let configured = cfg.get_str("Install", "Dir", "");
    if !configured.is_empty() {
        candidates.push(PathBuf::from(configured));
    }
    // Next to our own program: <root>/UX/autohotkey-ux, like AutoHotkeyUX.exe.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(ux) = exe.parent() {
            if let Some(root) = ux.parent() {
                if root.join("v2").is_dir() || root.join("UX").join("installed-files.csv").is_file() {
                    candidates.push(root.to_path_buf());
                }
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join(".local/share/AutoHotkey"));
        candidates.push(home.join(".wine/drive_c/Program Files/AutoHotkey"));
        candidates.push(home.join(".wine/drive_c/users").join(whoami()).join("AppData/Local/Programs/AutoHotkey"));
    }
    candidates.push(PathBuf::from("/opt/autohotkey"));
    candidates.push(PathBuf::from("/usr/share/autohotkey"));
    if cfg!(windows) {
        if let Some(local) = dirs::data_local_dir() {
            candidates.push(local.join("Programs").join("AutoHotkey"));
        }
        candidates.push(PathBuf::from(r"C:\Program Files\AutoHotkey"));
        candidates.push(PathBuf::from(r"D:\AutoHotKey"));
    }
    candidates.into_iter().find(|p| p.is_dir())
}

fn whoami() -> String {
    std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_default()
}

pub fn documents_dir() -> PathBuf {
    dirs::document_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Documents")))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// <Documents>/AutoHotkey — where new scripts and user templates go.
pub fn ahk_user_dir() -> PathBuf {
    documents_dir().join("AutoHotkey")
}


/// Open a file, folder or web address with the desktop's default handler.
pub fn shell_open(target: &str) -> bool {
    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", "start", "", target])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
    }
    #[cfg(not(windows))]
    {
        for opener in ["xdg-open", "kde-open", "gio"] {
            let mut c = Command::new(opener);
            if opener == "gio" {
                c.arg("open");
            }
            if c.arg(target).stdout(Stdio::null()).stderr(Stdio::null()).spawn().is_ok() {
                return true;
            }
        }
        false
    }
}

/// Show a file selected in the file manager (Dolphin on KDE), like `explorer /select`.
pub fn show_in_file_manager(path: &Path) {
    #[cfg(windows)]
    {
        let _ = Command::new("explorer").arg(format!("/select,{}", path.display())).spawn();
    }
    #[cfg(not(windows))]
    {
        let uri = format!("file://{}", path.display());
        let ok = Command::new("dbus-send")
            .args([
                "--session",
                "--print-reply",
                "--dest=org.freedesktop.FileManager1",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:{uri}"),
                "string:",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            if which("dolphin").is_some() {
                let _ = Command::new("dolphin").arg("--select").arg(path).spawn();
            } else if let Some(dir) = path.parent() {
                shell_open(&dir.to_string_lossy());
            }
        }
    }
}

/// Find a program on PATH (or accept an existing path).
pub fn which(name: &str) -> Option<PathBuf> {
    if name.is_empty() {
        return None;
    }
    let p = Path::new(name);
    if p.components().count() > 1 || p.is_absolute() {
        return if p.is_file() { Some(p.to_path_buf()) } else { None };
    }
    let path = std::env::var_os("PATH")?;
    let exts: Vec<String> = if cfg!(windows) {
        vec!["".into(), ".exe".into(), ".cmd".into(), ".bat".into()]
    } else {
        vec!["".into()]
    };
    for dir in std::env::split_paths(&path) {
        for ext in &exts {
            let candidate = dir.join(format!("{name}{ext}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Split a command line into arguments (double quotes group, like CommandLineToArgvW).
pub fn split_args(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut has_any = false;
    let mut chars = cmd.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                has_any = true;
            }
            '\\' if chars.peek() == Some(&'"') => {
                chars.next();
                cur.push('"');
                has_any = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if has_any {
                    args.push(std::mem::take(&mut cur));
                    has_any = false;
                }
            }
            c => {
                cur.push(c);
                has_any = true;
            }
        }
    }
    if has_any {
        args.push(cur);
    }
    args
}

/// Start a program detached.
pub fn spawn(args: &[String], cwd: Option<&Path>) -> std::io::Result<std::process::Child> {
    let (prog, rest) = args.split_first().ok_or_else(|| std::io::Error::other("empty command"))?;
    let mut c = Command::new(prog);
    c.args(rest);
    if let Some(d) = cwd {
        c.current_dir(d);
    }
    c.spawn()
}

/// Build the command that runs a Windows program: directly on Windows, through Wine elsewhere.
pub fn windows_program_command(exe: &Path) -> Vec<String> {
    if cfg!(windows) {
        vec![exe.to_string_lossy().into_owned()]
    } else {
        vec!["wine".to_string(), exe.to_string_lossy().into_owned()]
    }
}

/// Expand an editor command ("prog" "%l") for one file.
pub fn expand_command(cmd: &str, file: &Path) -> Vec<String> {
    let f = file.to_string_lossy();
    let mut out: Vec<String> = split_args(cmd)
        .into_iter()
        .map(|a| a.replace("%l", &f).replace("%L", &f).replace("%1", &f).replace("%f", &f).replace("%F", &f))
        .collect();
    if !cmd.contains("%l") && !cmd.contains("%L") && !cmd.contains("%1") && !cmd.contains("%f") && !cmd.contains("%F") {
        out.push(f.into_owned());
    }
    out.retain(|a| a != "%*");
    out
}
