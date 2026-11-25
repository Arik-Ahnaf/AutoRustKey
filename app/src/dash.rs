//! The Dash: AutoHotkey's "main menu" (ui-dash.ahk).

use crate::app::{Action, Kind, Shared};
use crate::icons::Icon;
use crate::sys;
use crate::theme::{self, MenuItem, MessageBox, MsgIcon, MsgResult, PopupMenu, Tile, BASE, LINE, MARGIN_X, MARGIN_Y};
use egui::{pos2, vec2, Key, Pos2, Ui, Vec2};
use std::path::PathBuf;

const LIST_W: f32 = 250.0;
const TILE_H: f32 = 38.0;

const ITEMS: [(&str, &str, Icon); 6] = [
    ("New script", "Create a script or manage templates", Icon::NewScript),
    ("Compile", "Open Ahk2Exe - convert .ahk to .exe", Icon::Compile),
    ("Help files (F1)", "", Icon::Help),
    ("Window spy", "", Icon::Spy),
    ("Launch settings", "Configure how .ahk files are opened", Icon::Launch),
    ("Editor settings", "Set your default script editor", Icon::Editor),
];

const LINKS: [(&str, &str); 6] = [
    ("Using the Program", "Program.htm"),
    ("How to Write Hotkeys", "howto/WriteHotkeys.htm"),
    ("How to Send Keystrokes", "howto/SendKeys.htm"),
    ("How to Run Programs", "howto/RunPrograms.htm"),
    ("How to Manage Windows", "howto/ManageWindows.htm"),
    ("Quick Reference", "index.htm#Quick_Reference"),
];

const MSG_GET_AHK2EXE: u32 = 1;

pub struct DashWin {
    intro: bool,
    show_info_next_time: bool,
    focus: usize,
    keyboard: bool,
    menu: Option<PopupMenu>,
    menu_targets: Vec<String>,
    msg: Option<MessageBox>,
}

impl DashWin {
    pub fn new(shared: &Shared) -> Self {
        let suppressed = shared.cfg.get_bool("Dash", "SuppressIntro", false);
        Self {
            intro: !suppressed,
            show_info_next_time: true,
            focus: 0,
            keyboard: false,
            menu: None,
            menu_targets: vec![],
            msg: None,
        }
    }

    fn list_height() -> f32 {
        ITEMS.len() as f32 * TILE_H + 4.0
    }

    pub fn title(&self) -> String {
        "AutoHotkey Dash".into()
    }

    pub fn size(&self) -> Vec2 {
        let h = Self::list_height() + MARGIN_Y * 2.0;
        let w = if self.intro { 624.0 } else { LIST_W + MARGIN_X * 2.0 };
        vec2(w, h)
    }

    fn activate(&mut self, ui: &Ui, item: usize, shared: &mut Shared, actions: &mut Vec<Action>) {
        match item {
            0 => actions.push(Action::Open(Kind::NewScript(None))),
            1 => {
                let compiler = shared.root.as_ref().map(|r| r.join("Compiler").join("Ahk2Exe.exe"));
                match compiler {
                    Some(c) if c.is_file() => {
                        let _ = sys::spawn(&sys::windows_program_command(&c), c.parent());
                    }
                    _ => {
                        self.msg = Some(
                            MessageBox::new(
                                "AutoHotkey",
                                "Ahk2Exe is not installed, but you can download it.\n\nOpen the Ahk2Exe download page now?",
                                MsgIcon::Info,
                            )
                            .yes_no(MSG_GET_AHK2EXE),
                        );
                    }
                }
            }
            2 => {
                let pos = ui.ctx().input(|i| i.pointer.hover_pos()).unwrap_or(pos2(40.0, 40.0));
                self.show_help(ui.ctx(), pos, shared);
            }
            3 => actions.push(Action::SpawnSelf(vec!["--spy".into()])),
            4 => actions.push(Action::Open(Kind::Launch)),
            5 => actions.push(Action::Open(Kind::Editor(None))),
            _ => {}
        }
    }

    /// ShowHelpFile: offline .chm files per major version, then the online docs.
    fn show_help(&mut self, ctx: &egui::Context, pos: Pos2, shared: &Shared) {
        let help = HelpFiles::find(shared.root.as_deref());
        if help.versions.len() == 1 && help.other.is_empty() {
            sys::shell_open(&help.versions[0].1.to_string_lossy());
            return;
        }
        let mut items = Vec::new();
        let mut targets = vec![String::new()];
        let mut add = |label: String, target: String, items: &mut Vec<MenuItem>| {
            targets.push(target);
            items.push(MenuItem::new(label, (targets.len() - 1) as u32));
        };
        // newest per major in the main menu, the rest under "More"
        let mut mains: Vec<(String, PathBuf)> = Vec::new();
        let mut subs: Vec<(String, PathBuf)> = Vec::new();
        for (v, p) in &help.versions {
            let m = crate::launcher::major(v);
            if let Some(slot) = mains.iter_mut().find(|(mv, _)| crate::launcher::major(mv) == m) {
                if crate::launcher::ver_cmp(&slot.0, v).is_lt() {
                    subs.push(std::mem::replace(slot, (v.clone(), p.clone())));
                } else {
                    subs.push((v.clone(), p.clone()));
                }
            } else {
                mains.push((v.clone(), p.clone()));
            }
        }
        if !mains.is_empty() {
            items.push(MenuItem::header("Offline help"));
        }
        for (v, p) in &mains {
            add(format!("v&{v}"), p.to_string_lossy().into_owned(), &mut items);
        }
        if !subs.is_empty() {
            let mut sub = Vec::new();
            for (v, p) in subs.iter().rev() {
                add(format!("v{v}"), p.to_string_lossy().into_owned(), &mut sub);
            }
            let mut more = MenuItem::new("More", 0);
            more.submenu = sub;
            items.push(more);
        }
        items.push(MenuItem::header("Online help"));
        let prefix = if mains.is_empty() { "v&" } else { "v" };
        add(format!("{prefix}1.1"), "https://www.autohotkey.com/docs/v1/".into(), &mut items);
        add(format!("{prefix}2.0"), "https://www.autohotkey.com/docs/v2/".into(), &mut items);
        if !help.other.is_empty() {
            items.push(MenuItem::header("Other files"));
        }
        for (label, p) in &help.other {
            add(label.clone(), p.to_string_lossy().into_owned(), &mut items);
        }
        self.menu_targets = targets;
        self.menu = Some(PopupMenu::new(ctx, pos, items));
    }

    pub fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) -> Vec<Action> {
        let mut actions = Vec::new();
        let o = ui.max_rect().min;
        let ctx = ui.ctx().clone();
        let modal_open = self.msg.is_some() || self.menu.is_some();

        // ---- keyboard: arrows move focus, Enter activates, F1 help, Esc closes
        if !modal_open && !ctx.wants_keyboard_input() {
            let (up, down, enter, f1, esc) = ctx.input(|i| {
                (i.key_pressed(Key::ArrowUp), i.key_pressed(Key::ArrowDown), i.key_pressed(Key::Enter), i.key_pressed(Key::F1), i.key_pressed(Key::Escape))
            });
            if up {
                self.focus = self.focus.saturating_sub(1);
                self.keyboard = true;
            }
            if down {
                self.focus = (self.focus + 1).min(ITEMS.len() - 1);
                self.keyboard = true;
            }
            if f1 {
                let pos = o + vec2(MARGIN_X + 40.0, MARGIN_Y + 2.0 * TILE_H + 20.0);
                self.show_help(&ctx, pos, shared);
            }
            if enter {
                self.activate(ui, self.focus, shared, &mut actions);
            }
            if esc {
                actions.push(Action::Close);
            }
        }

        // ---- the list menu
        let tiles: Vec<Tile> = ITEMS.iter().map(|(n, d, i)| Tile { name: n, desc: Some(d), icon: Some(*i) }).collect();
        let res = theme::tile_list(ui, "dash", o + vec2(MARGIN_X, MARGIN_Y), LIST_W, TILE_H, &tiles, None, Some(self.focus), self.keyboard);
        if !modal_open {
            if let Some(i) = res.clicked.or(res.double_clicked) {
                self.focus = i;
                self.keyboard = false;
                self.activate(ui, i, shared, &mut actions);
            }
        }

        // ---- intro panel
        if self.intro {
            let x = o.x + MARGIN_X + LIST_W + 8.0;
            let mut y = o.y + MARGIN_Y;
            theme::label(ui, pos2(x, y - 1.0), "Welcome!", 16.0, theme::TEXT);
            y += 22.0 + MARGIN_Y;
            theme::label(ui, pos2(x, y), "This is the Dash. It provides access to tools, settings and help files.", BASE, theme::TEXT);
            y += LINE + MARGIN_Y;
            theme::label(ui, pos2(x, y), "To learn how to use AutoHotkey, refer to:", BASE, theme::TEXT);
            y += LINE + MARGIN_Y;
            for (text, href) in LINKS {
                theme::label(ui, pos2(x + 13.0, y), "\u{2022}", BASE, theme::TEXT);
                if theme::link(ui, pos2(x + 21.0, y), text).clicked() {
                    open_doc_page(shared, href);
                }
                y += LINE;
            }
            let list_bottom = o.y + MARGIN_Y + Self::list_height();
            let before = self.show_info_next_time;
            theme::checkbox(ui, pos2(x, list_bottom - LINE - 2.0), &mut self.show_info_next_time, "Show this info next time", true);
            if before != self.show_info_next_time {
                if self.show_info_next_time {
                    shared.cfg.delete("Dash", "SuppressIntro");
                } else {
                    shared.cfg.set_bool("Dash", "SuppressIntro", true);
                }
            }
        }

        // ---- popups
        if let Some(menu) = &mut self.menu {
            if let Some(a) = menu.show(&ctx) {
                if a != 0 {
                    if let Some(t) = self.menu_targets.get(a as usize) {
                        sys::shell_open(t);
                    }
                }
                self.menu = None;
            }
        }
        if let Some(m) = &self.msg {
            if let Some(r) = m.show(&ctx) {
                if m.tag == MSG_GET_AHK2EXE && r == MsgResult::Yes {
                    sys::shell_open("https://github.com/AutoHotkey/Ahk2Exe/releases/latest");
                }
                self.msg = None;
            }
        }
        actions
    }
}

/// Open a documentation page: the local CHM on Windows when present, otherwise
/// the online documentation (Linux has no ms-its: protocol handler).
pub fn open_doc_page(shared: &Shared, href: &str) {
    if cfg!(windows) {
        if let Some(chm) = shared.root.as_ref().map(|r| r.join("v2").join("AutoHotkey.chm")) {
            if chm.is_file() {
                let _ = std::process::Command::new("hh.exe").arg(format!("ms-its:{}::docs/{}", chm.display(), href)).spawn();
                return;
            }
        }
    }
    sys::shell_open(&format!("https://www.autohotkey.com/docs/v2/{href}"));
}

pub struct HelpFiles {
    /// (version, path) of AutoHotkey.chm files
    pub versions: Vec<(String, PathBuf)>,
    /// (label, path) of other .chm files
    pub other: Vec<(String, PathBuf)>,
}

impl HelpFiles {
    pub fn find(root: Option<&std::path::Path>) -> Self {
        let mut out = HelpFiles { versions: vec![], other: vec![] };
        let Some(root) = root else { return out };
        let hashes = crate::launcher::read_hashes(root);
        let mut stack = vec![(root.to_path_buf(), 0)];
        while let Some((dir, depth)) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    if depth < 4 {
                        stack.push((p, depth + 1));
                    }
                    continue;
                }
                if !p.extension().map(|x| x.eq_ignore_ascii_case("chm")).unwrap_or(false) {
                    continue;
                }
                let name = p.file_stem().unwrap_or_default().to_string_lossy().into_owned();
                let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('/', "\\");
                let reldir = p.parent().and_then(|d| d.strip_prefix(root).ok()).map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
                if name == "AutoHotkey" {
                    let v = hashes
                        .iter()
                        .find(|h| h.get("Path").map(|x| x.eq_ignore_ascii_case(&rel)).unwrap_or(false))
                        .and_then(|h| h.get("Version").cloned())
                        .or_else(|| {
                            // no install record: take the version from the folder name (v2, v1.1.37.02)
                            let d = p.parent()?.file_name()?.to_string_lossy().into_owned();
                            let d = d.trim_start_matches(['v', 'V']).to_string();
                            d.starts_with(|c: char| c.is_ascii_digit()).then_some(d)
                        });
                    if let Some(v) = v {
                        out.versions.push((v, p.clone()));
                    }
                } else {
                    let label = if !reldir.is_empty() && name != reldir { format!("{name} ({reldir})") } else { name };
                    out.other.push((label, p.clone()));
                }
            }
        }
        out.versions.sort_by(|a, b| crate::launcher::ver_cmp(&b.0, &a.0));
        out
    }
}
