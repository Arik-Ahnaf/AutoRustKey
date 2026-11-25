//! "Select an editor" window (ui-editor.ahk): pick the program that edits scripts.

use crate::app::{Action, Shared};
use crate::icons::{self, Icon};
use crate::sys;
use crate::theme::{self, Tile, LINE, MARGIN_X, MARGIN_Y};
use egui::{pos2, vec2, Key, Rect, Ui, Vec2};
use std::path::{Path, PathBuf};

const TILE_H: f32 = 38.0;
const MAX_ITEMS: usize = 8;

#[derive(Clone)]
pub struct EditorApp {
    pub name: String,
    pub cmd: String,
    pub seed: u32,
}

fn seed_of(s: &str) -> u32 {
    s.bytes().fold(7u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32))
}

/// Read Name / Exec / MimeType / Categories / NoDisplay from a .desktop file.
fn parse_desktop(path: &Path) -> Option<(String, String, String, String, bool)> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut in_main = false;
    let (mut name, mut exec, mut mime, mut cats, mut hidden) = (String::new(), String::new(), String::new(), String::new(), false);
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_main = t == "[Desktop Entry]";
            continue;
        }
        if !in_main {
            continue;
        }
        if let Some((k, v)) = t.split_once('=') {
            match k.trim() {
                "Name" => name = v.trim().to_string(),
                "Exec" => exec = v.trim().to_string(),
                "MimeType" => mime = v.trim().to_string(),
                "Categories" => cats = v.trim().to_string(),
                "NoDisplay" | "Hidden" => hidden |= v.trim().eq_ignore_ascii_case("true"),
                "Type" if v.trim() != "Application" => return None,
                _ => {}
            }
        }
    }
    (!name.is_empty() && !exec.is_empty()).then_some((name, exec, mime, cats, hidden))
}

/// Turn a desktop Exec line into the original's command format ("prog" "%l").
fn exec_to_cmd(exec: &str) -> String {
    let mut parts = Vec::new();
    let mut has_file = false;
    for a in sys::split_args(exec) {
        match a.as_str() {
            "%f" | "%F" | "%u" | "%U" => {
                has_file = true;
                parts.push("\"%l\"".to_string());
            }
            "%i" | "%c" | "%k" | "%d" | "%D" | "%n" | "%N" | "%v" | "%m" => {}
            other => {
                let other = other.replace("%%", "%");
                parts.push(if other.contains(' ') { format!("\"{other}\"") } else { other });
            }
        }
    }
    let mut cmd = parts.join(" ");
    if !has_file {
        cmd.push_str(" \"%l\"");
    }
    cmd
}

/// GetEditorApps: programs that open text files.
pub fn editor_apps() -> Vec<EditorApp> {
    let mut apps: Vec<EditorApp> = Vec::new();
    let add = |name: String, cmd: String, apps: &mut Vec<EditorApp>| {
        let lname = name.to_ascii_lowercase();
        if lname.starts_with("autohotkey") || lname.starts_with("ahk2exe") || apps.iter().any(|a| a.name.eq_ignore_ascii_case(&name)) {
            return;
        }
        let seed = seed_of(&name);
        apps.push(EditorApp { name, cmd, seed });
    };
    if cfg!(windows) {
        add("Notepad".into(), "\"notepad.exe\" \"%l\"".into(), &mut apps);
        return apps;
    }
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(d) = dirs::data_dir() {
        dirs.push(d.join("applications"));
        dirs.push(d.join("flatpak/exports/share/applications"));
    }
    let xdg = std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    for d in xdg.split(':').filter(|d| !d.is_empty()) {
        dirs.push(Path::new(d).join("applications"));
    }
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    // The desktop's default text editor comes first (like the MRU list in the original).
    let default_id = std::process::Command::new("xdg-mime")
        .args(["query", "default", "text/plain"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let mut found: Vec<(bool, String, String)> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();
    for dir in dirs {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let id = p.file_name().unwrap_or_default().to_string_lossy().into_owned();
            if !id.ends_with(".desktop") || !seen_ids.insert(id.clone()) {
                continue;
            }
            let Some((name, exec, mime, cats, hidden)) = parse_desktop(&p) else { continue };
            let is_editor = mime.split(';').any(|m| m == "text/plain" || m == crate::assoc::MIME) || cats.split(';').any(|c| c == "TextEditor");
            if !is_editor || (hidden && id != default_id) || id.starts_with("autohotkey-ux") {
                continue;
            }
            found.push((id == default_id, name, exec_to_cmd(&exec)));
        }
    }
    found.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase())));
    for (_, name, cmd) in found {
        add(name, cmd, &mut apps);
    }
    if apps.is_empty() {
        for (n, c) in [("Kate", "kate"), ("KWrite", "kwrite")] {
            if sys::which(c).is_some() {
                add(n.into(), format!("{c} \"%l\""), &mut apps);
            }
        }
    }
    apps
}

/// FindExecutable for the first word of a command.
pub fn find_executable(cmd: &str) -> Option<PathBuf> {
    let first = sys::split_args(cmd).into_iter().next()?;
    sys::which(&first).or_else(|| if cfg!(windows) { sys::which(&format!("{first}.exe")) } else { None })
}

pub struct EditorWin {
    apps: Vec<EditorApp>,
    selected: Option<usize>,
    cmd: String,
    script_to_edit: Option<PathBuf>,
}

impl EditorWin {
    pub fn new(shared: &Shared, script_to_edit: Option<PathBuf>) -> Self {
        let cmd = shared.cfg.get_str("Editor", "Command", "");
        let mut s = Self { apps: editor_apps(), selected: None, cmd: cmd.clone(), script_to_edit };
        s.select_by_cmd(&cmd);
        s
    }

    fn select_by_cmd(&mut self, cmd: &str) {
        if cmd.is_empty() {
            return;
        }
        if let Some(i) = self.apps.iter().position(|a| a.cmd == cmd) {
            self.selected = Some(i);
            return;
        }
        let args = sys::split_args(cmd);
        if let Some(first) = args.first() {
            let name = Path::new(first).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| first.clone());
            let name = args.iter().skip(1).find(|a| !a.starts_with('/') && !a.starts_with('-')).filter(|a| a.to_ascii_lowercase().ends_with(".ahk")).cloned().unwrap_or(name);
            self.apps.push(EditorApp { seed: seed_of(&name), name, cmd: cmd.to_string() });
            self.selected = Some(self.apps.len() - 1);
        }
    }

    pub fn title(&self) -> String {
        "Select an editor".into()
    }

    fn list_h(&self) -> f32 {
        self.apps.len().clamp(1, MAX_ITEMS) as f32 * TILE_H + 4.0
    }

    fn rows(&self) -> (f32, f32, f32, f32) {
        let text_y = MARGIN_Y + self.list_h();
        let edit_y = text_y + LINE + MARGIN_Y;
        let frame_y = edit_y + 38.0 + MARGIN_Y;
        let btn_y = frame_y + 25.0 + MARGIN_Y;
        (text_y, edit_y, frame_y, btn_y)
    }

    pub fn size(&self) -> Vec2 {
        vec2(300.0 + MARGIN_X * 2.0, self.rows().3 + 25.0 + MARGIN_Y + 2.0)
    }

    fn ok_enabled(&self) -> bool {
        !self.cmd.trim().is_empty() && find_executable(&self.cmd).is_some()
    }

    fn confirm(&mut self, shared: &mut Shared, actions: &mut Vec<Action>) {
        shared.cfg.set_str("Editor", "Command", &self.cmd);
        if let Some(f) = self.script_to_edit.take() {
            let _ = sys::spawn(&sys::expand_command(&self.cmd, &f), None);
        }
        actions.push(Action::Close);
    }

    pub fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) -> Vec<Action> {
        let mut actions = Vec::new();
        let ctx = ui.ctx().clone();
        let o = ui.max_rect().min;
        let (text_y, edit_y, frame_y, btn_y) = self.rows();

        let tiles: Vec<Tile> = self.apps.iter().map(|a| Tile { name: &a.name, desc: None, icon: Some(Icon::App(a.seed)) }).collect();
        let res = theme::tile_list(ui, "ed-list", o + vec2(MARGIN_X, MARGIN_Y), 300.0, TILE_H, &tiles, self.selected, self.selected, true);
        if let Some(i) = res.clicked.or(res.double_clicked) {
            self.selected = Some(i);
            self.cmd = self.apps[i].cmd.clone();
        }

        theme::label(ui, o + vec2(MARGIN_X, text_y), "Command line", theme::BASE, theme::TEXT);
        let before = self.cmd.clone();
        let edit_rect = Rect::from_min_size(o + vec2(MARGIN_X, edit_y), vec2(300.0, 38.0));
        theme::text_area(ui, edit_rect, &mut self.cmd, "ed-cmd", false);
        theme::scroll_arrows(ui, edit_rect);
        let mut enter = false;
        if self.cmd.contains('\n') {
            self.cmd = self.cmd.replace(['\r', '\n'], "");
            enter = true;
        }
        if before != self.cmd && !enter {
            self.selected = None;
        }

        // etched frame with the help link
        let frame = Rect::from_min_size(o + vec2(MARGIN_X, frame_y), vec2(300.0, 25.0));
        let p = ui.painter();
        p.rect_stroke(frame.shrink(0.5), egui::CornerRadius::ZERO, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0xA0, 0xA0, 0xA0)), egui::StrokeKind::Middle);
        p.rect_stroke(frame.shrink(1.5), egui::CornerRadius::ZERO, egui::Stroke::new(1.0_f32, egui::Color32::WHITE), egui::StrokeKind::Middle);
        icons::draw(p, Rect::from_min_size(pos2(o.x + 56.0, frame.top() + 4.0), vec2(16.0, 16.0)), Icon::Info);
        if theme::link(ui, pos2(o.x + 76.0, frame.top() + 4.0), "Editors with AutoHotkey support").clicked() {
            crate::dash::open_doc_page(shared, "misc/Editors.htm");
        }

        let browse = Rect::from_min_size(o + vec2(MARGIN_X, btn_y), vec2(80.0, 25.0));
        let right = MARGIN_X + 300.0;
        let ok = Rect::from_min_size(o + vec2(right - 160.0 - MARGIN_Y, btn_y), vec2(80.0, 25.0));
        let cancel = Rect::from_min_size(o + vec2(right - 80.0, btn_y), vec2(80.0, 25.0));
        if theme::button(ui, browse, "&Browse", false, true).clicked() {
            if let Some(app) = rfd::FileDialog::new().set_title("Select an editor").pick_file() {
                self.cmd = if app.extension().map(|e| e.eq_ignore_ascii_case("ahk")).unwrap_or(false) {
                    let me = std::env::current_exe().unwrap_or_default();
                    format!("\"{}\" --run \"{}\" \"%l\"", me.display(), app.display())
                } else {
                    format!("\"{}\" \"%l\"", app.display())
                };
                self.selected = None;
            }
        }
        let ok_enabled = self.ok_enabled();
        if theme::button(ui, ok, "&OK", true, ok_enabled).clicked() || (enter && ok_enabled) || (ok_enabled && !ctx.wants_keyboard_input() && ctx.input(|i| i.key_pressed(Key::Enter))) {
            self.confirm(shared, &mut actions);
        }
        if theme::button(ui, cancel, "&Cancel", false, true).clicked() || ctx.input(|i| i.key_pressed(Key::Escape)) {
            actions.push(Action::Close);
        }
        actions
    }
}
