//! How .ahk files open on the desktop. The original wrote the Open / RunAs /
//! Edit verbs of HKCU\Software\Classes\AutoHotkeyScript. On Linux the
//! equivalent is a MIME type for *.ahk plus .desktop entries registered as its
//! default handlers (KDE Dolphin and the Plasma menu read these).

#[cfg(not(windows))]
use std::process::{Command, Stdio};

pub const MIME: &str = "text/x-autohotkey";

/// Register `run_cmd` (a full command line with %f for the script) as the
/// way .ahk files are opened. `app_name` shows in "Open with".
pub fn set_open_command(app_name: &str, run_cmd: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        let _ = (app_name, run_cmd);
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let data = dirs::data_dir().ok_or("no data folder")?;
        let mime_dir = data.join("mime").join("packages");
        let apps = data.join("applications");
        std::fs::create_dir_all(&mime_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&apps).map_err(|e| e.to_string())?;
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<mime-info xmlns=\"http://www.freedesktop.org/standards/shared-mime-info\">\n  <mime-type type=\"{MIME}\">\n    <sub-class-of type=\"text/plain\"/>\n    <comment>AutoHotkey Script</comment>\n    <glob pattern=\"*.ahk\"/>\n  </mime-type>\n</mime-info>\n"
        );
        std::fs::write(mime_dir.join("autohotkey-ux.xml"), xml).map_err(|e| e.to_string())?;
        let exe = std::env::current_exe().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| "autohotkey-ux".into());
        let desktop = format!(
            "[Desktop Entry]\nType=Application\nName={app_name}\nComment=Run AutoHotkey scripts\nExec={run_cmd}\nMimeType={MIME};\nNoDisplay=true\nTerminal=false\nIcon=text-x-script\nActions=Edit;\n\n[Desktop Action Edit]\nName=Edit Script\nExec=\"{exe}\" --edit %f\n"
        );
        std::fs::write(apps.join("autohotkey-ux-launcher.desktop"), desktop).map_err(|e| e.to_string())?;
        let dash = format!(
            "[Desktop Entry]\nType=Application\nName=AutoHotkey Dash\nComment=Tools, settings and help for AutoHotkey\nExec=\"{exe}\"\nTerminal=false\nIcon=preferences-desktop-keyboard-shortcuts\nCategories=Utility;Development;\n"
        );
        let _ = std::fs::write(apps.join("autohotkey-ux-dash.desktop"), dash);
        let quiet = |c: &mut Command| {
            let _ = c.stdout(Stdio::null()).stderr(Stdio::null()).status();
        };
        quiet(Command::new("update-mime-database").arg(data.join("mime")));
        quiet(Command::new("update-desktop-database").arg(&apps));
        quiet(Command::new("xdg-mime").args(["default", "autohotkey-ux-launcher.desktop", MIME]));
        quiet(&mut Command::new("kbuildsycoca6"));
        Ok(())
    }
}

/// Command used when .ahk files are opened through the launcher (auto-detect).
pub fn launcher_command() -> String {
    let exe = std::env::current_exe().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| "autohotkey-ux".into());
    format!("\"{exe}\" --run %f")
}

/// Command used for a specific interpreter (run through Wine on Linux).
pub fn interpreter_command(exe: &str, switches: &str) -> String {
    if cfg!(windows) {
        format!("\"{exe}\" {switches}\"%1\" %*")
    } else {
        format!("wine \"{exe}\" {switches}%f")
    }
}
