//! "New Script" window (ui-newscript.ahk): name, folder, template list,
//! Create / Edit buttons and the defaults menu.

use crate::app::{Action, Kind, Shared};
use crate::icons::{self, Icon};
use crate::sys;
use crate::theme::{self, MenuItem, MessageBox, MsgIcon, MsgResult, PopupMenu, Tile, MARGIN_X, MARGIN_Y};
use egui::{pos2, vec2, Key, Rect, Ui, Vec2};
use std::path::{Path, PathBuf};

const TILE_H: f32 = 34.0;
const MAX_ITEMS: usize = 8;
const BUILTIN_MINIMAL: &str = "/*\n[NewScriptTemplate]\nDescription = Just #Requires v2.0\n*/\n#Requires AutoHotkey v2.0\n\n";

#[derive(Clone)]
pub struct Template {
    pub name: String,
    pub desc: String,
    pub path: Option<PathBuf>,
    pub execute: bool,
}

impl Template {
    fn from_file(path: &Path) -> Self {
        let name = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
        let text = std::fs::read(path).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default();
        let desc = ini_read(&text, "NewScriptTemplate", "Description").unwrap_or_default();
        let execute = matches!(ini_read(&text, "NewScriptTemplate", "Execute").as_deref().map(|s| s.to_ascii_lowercase()), Some(ref s) if s == "1" || s == "true");
        Self { name, desc, path: Some(path.to_path_buf()), execute }
    }

    fn code(&self) -> String {
        match &self.path {
            Some(p) => std::fs::read(p).map(|b| String::from_utf8_lossy(&b).trim_start_matches('\u{feff}').to_string()).unwrap_or_default(),
            None => BUILTIN_MINIMAL.to_string(),
        }
    }
}

/// IniRead on a script file: key=value lines below a [section] header.
fn ini_read(text: &str, section: &str, key: &str) -> Option<String> {
    let mut in_sec = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            in_sec = t[1..t.len() - 1].trim().eq_ignore_ascii_case(section);
            continue;
        }
        if in_sec {
            if let Some((k, v)) = t.split_once('=') {
                if k.trim().eq_ignore_ascii_case(key) {
                    return Some(v.trim().to_string());
                }
            }
        }
    }
    None
}

pub fn user_template_folder() -> PathBuf {
    sys::ahk_user_dir().join("Templates")
}

/// GetScriptTemplates: bundled templates, then the user's (same name overrides), sorted by name.
pub fn script_templates(root: Option<&Path>) -> Vec<Template> {
    let mut map: Vec<Template> = Vec::new();
    let put = |t: Template, map: &mut Vec<Template>| {
        if let Some(e) = map.iter_mut().find(|e| e.name.eq_ignore_ascii_case(&t.name)) {
            *e = t;
        } else {
            map.push(t);
        }
    };
    put(Template { name: "Minimal for v2".into(), desc: "Just #Requires v2.0".into(), path: None, execute: false }, &mut map);
    let mut sources = Vec::new();
    if let Some(r) = root {
        sources.push(r.join("UX").join("Templates"));
    }
    sources.push(user_template_folder());
    for src in sources {
        let Ok(rd) = std::fs::read_dir(&src) else { continue };
        let mut files: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().map(|x| x.eq_ignore_ascii_case("ahk")).unwrap_or(false)).collect();
        files.sort();
        for f in files {
            put(Template::from_file(&f), &mut map);
        }
    }
    map.sort_by_key(|t| t.name.to_lowercase());
    map
}

const MENU_DEF_CREATE: u32 = 1;
const MENU_DEF_EDIT: u32 = 2;
const MENU_STAY_OPEN: u32 = 3;
const MENU_SET_DIR: u32 = 4;
const MENU_OPEN_TEMPLATES: u32 = 5;
const MENU_EDIT_TEMPLATE: u32 = 11;
const MENU_HIDE_TEMPLATE: u32 = 12;
const MENU_SET_DEFAULT: u32 = 13;

const MSG_CREATE_TEMPLATES: u32 = 1;

enum Pending {
    OpenTemplates,
    EditTemplate(Template),
}

pub struct NewScriptWin {
    name: String,
    dir: String,
    templates: Vec<Template>,
    /// rows shown: None = "Empty"
    rows: Vec<Option<usize>>,
    selected: Option<usize>,
    default_create: bool,
    menu: Option<PopupMenu>,
    menu_row: Option<usize>,
    msg: Option<MessageBox>,
    pending: Option<Pending>,
    focus_name: bool,
}

impl NewScriptWin {
    pub fn new(shared: &Shared, path: Option<PathBuf>) -> Self {
        let (mut dir, mut name) = (String::new(), String::new());
        if let Some(p) = path {
            dir = p.parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
            name = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            if p.is_dir() {
                dir = p.to_string_lossy().into_owned();
                name.clear();
            }
        }
        if name == "New AutoHotkey Script" {
            name.clear();
        }
        if dir.is_empty() {
            dir = default_dir(shared);
        }
        let templates = script_templates(shared.root.as_deref());
        let deft = shared.cfg.get_str("New", "DefaultTemplate", "");
        let mut rows = vec![None];
        let mut selected = if deft.is_empty() { Some(0) } else { None };
        for (i, t) in templates.iter().enumerate() {
            if shared.cfg.get_bool("New\\HideTemplate", &t.name, false) {
                continue;
            }
            rows.push(Some(i));
            if deft == t.name {
                selected = Some(rows.len() - 1);
            }
        }
        let default_create = shared.cfg.get_str("New", "DefaultButton", "Edit") == "Create";
        Self { name, dir, templates, rows, selected, default_create, menu: None, menu_row: None, msg: None, pending: None, focus_name: true }
    }

    pub fn title(&self) -> String {
        "New Script".into()
    }

    fn list_top() -> f32 {
        MARGIN_Y + 23.0 + MARGIN_Y + 23.0 + MARGIN_Y
    }

    fn list_h(&self) -> f32 {
        self.rows.len().min(MAX_ITEMS) as f32 * TILE_H + 4.0
    }

    pub fn size(&self) -> Vec2 {
        vec2(300.0 + MARGIN_X * 2.0, Self::list_top() + self.list_h() + MARGIN_Y + 25.0 + MARGIN_Y + 2.0)
    }

    fn row_template(&self, row: usize) -> Option<&Template> {
        self.rows.get(row).copied().flatten().map(|i| &self.templates[i])
    }

    fn confirm(&mut self, edit: bool, ctrl: bool, shared: &mut Shared, actions: &mut Vec<Action>) {
        let Some(row) = self.selected else {
            self.msg = Some(MessageBox::new("New Script", "You need to select a template first.", MsgIcon::Warning));
            return;
        };
        let t = self.row_template(row).cloned();
        let stay_open = ctrl || shared.cfg.get_bool("New", "StayOpen", false);
        let dir = PathBuf::from(&self.dir);
        let _ = std::fs::create_dir_all(&dir);
        let mut base = self.name.trim().to_string();
        if base.is_empty() {
            base = "Untitled".into();
        }
        if base.to_ascii_lowercase().ends_with(".ahk") {
            base.truncate(base.len() - 4);
        }
        let mut new_path = dir.join(format!("{base}.ahk"));
        let mut n = 1;
        while new_path.exists() {
            new_path = dir.join(format!("{base}-{n}.ahk"));
            n += 1;
        }
        let btn = if edit { "Edit" } else { "Create" };
        match &t {
            Some(t) if t.execute && t.path.is_some() => {
                let exe = std::env::current_exe().unwrap_or_default();
                let args = vec![
                    exe.to_string_lossy().into_owned(),
                    "--run".into(),
                    t.path.as_ref().unwrap().to_string_lossy().into_owned(),
                    new_path.to_string_lossy().into_owned(),
                    btn.into(),
                ];
                if let Ok(mut c) = sys::spawn(&args, Some(&dir)) {
                    let _ = c.wait();
                }
            }
            _ => {
                let code = t.as_ref().map(|t| t.code()).unwrap_or_default();
                let re = regex::Regex::new(r"(?si)/\*\s+\[NewScriptTemplate\].*\*/(\r\n|\n|\r)?").unwrap();
                let code = re.replace(&code, "").into_owned();
                let mut bytes = vec![0xEF, 0xBB, 0xBF];
                bytes.extend_from_slice(code.as_bytes());
                if let Err(e) = std::fs::write(&new_path, bytes) {
                    self.msg = Some(MessageBox::new("New Script", &format!("Could not create the script:\n\n{e}"), MsgIcon::Warning));
                    return;
                }
            }
        }
        if !edit {
            sys::show_in_file_manager(&new_path);
        } else {
            actions.push(Action::EditFile(new_path));
        }
        if !stay_open {
            actions.push(Action::Close);
        }
    }

    fn defaults_menu(&mut self, ctx: &egui::Context, pos: egui::Pos2, shared: &Shared) {
        let stay = shared.cfg.get_bool("New", "StayOpen", false);
        let is_default_dir = self.dir == default_dir(shared);
        let items = vec![
            MenuItem::new("Default to Create", MENU_DEF_CREATE).radio().checked(self.default_create),
            MenuItem::new("Default to Edit", MENU_DEF_EDIT).radio().checked(!self.default_create),
            MenuItem::new("Stay open", MENU_STAY_OPEN).checked(stay),
            MenuItem::separator(),
            MenuItem::new("Set folder as default", MENU_SET_DIR).checked(is_default_dir),
            MenuItem::new("Open templates folder", MENU_OPEN_TEMPLATES),
        ];
        self.menu = Some(PopupMenu::new(ctx, pos, items));
        self.menu_row = None;
    }

    fn template_menu(&mut self, ctx: &egui::Context, row: usize, pos: egui::Pos2, shared: &Shared) {
        let name = self.row_template(row).map(|t| t.name.clone()).unwrap_or_default();
        let mut items = Vec::new();
        if row > 0 {
            items.push(MenuItem::new("&Edit template", MENU_EDIT_TEMPLATE));
            items.push(MenuItem::new("&Hide template", MENU_HIDE_TEMPLATE));
        }
        items.push(MenuItem::new("Set as &default", MENU_SET_DEFAULT).checked(name == shared.cfg.get_str("New", "DefaultTemplate", "")));
        self.menu = Some(PopupMenu::new(ctx, pos, items));
        self.menu_row = Some(row);
    }

    fn need_templates_folder(&mut self, then: Pending) -> bool {
        let dir = user_template_folder();
        if dir.is_dir() {
            return true;
        }
        self.msg = Some(
            MessageBox::new(
                "New Script",
                &format!("User-created templates should be placed in the following folder, which does not yet exist:\n\n{}\n\nCreate it now?", dir.display()),
                MsgIcon::None,
            )
            .yes_no(MSG_CREATE_TEMPLATES),
        );
        self.pending = Some(then);
        false
    }

    fn run_pending(&mut self, p: Pending, actions: &mut Vec<Action>) {
        let dir = user_template_folder();
        match p {
            Pending::OpenTemplates => {
                sys::shell_open(&dir.to_string_lossy());
            }
            Pending::EditTemplate(t) => {
                let user_path = dir.join(format!("{}.ahk", t.name));
                if !user_path.exists() {
                    let _ = std::fs::write(&user_path, {
                        let mut b = vec![0xEF, 0xBB, 0xBF];
                        b.extend_from_slice(t.code().as_bytes());
                        b
                    });
                }
                actions.push(Action::EditFile(user_path));
            }
        }
    }

    fn handle_menu(&mut self, action: u32, shared: &mut Shared, actions: &mut Vec<Action>) {
        match action {
            MENU_DEF_CREATE | MENU_DEF_EDIT => {
                self.default_create = action == MENU_DEF_CREATE;
                shared.cfg.set_str("New", "DefaultButton", if self.default_create { "Create" } else { "Edit" });
            }
            MENU_STAY_OPEN => {
                let v = !shared.cfg.get_bool("New", "StayOpen", false);
                shared.cfg.set_bool("New", "StayOpen", v);
            }
            MENU_SET_DIR => shared.cfg.set_str("New", "DefaultDir", &self.dir.clone()),
            MENU_OPEN_TEMPLATES => {
                if self.need_templates_folder(Pending::OpenTemplates) {
                    self.run_pending(Pending::OpenTemplates, actions);
                }
            }
            MENU_EDIT_TEMPLATE => {
                if let Some(t) = self.menu_row.and_then(|r| self.row_template(r)).cloned() {
                    if self.need_templates_folder(Pending::EditTemplate(t.clone())) {
                        self.run_pending(Pending::EditTemplate(t), actions);
                    }
                }
            }
            MENU_HIDE_TEMPLATE => {
                if let Some(row) = self.menu_row {
                    if let Some(t) = self.row_template(row).cloned() {
                        shared.cfg.set_bool("New\\HideTemplate", &t.name, true);
                        self.rows.remove(row);
                        self.selected = match self.selected {
                            Some(s) if s == row => None,
                            Some(s) if s > row => Some(s - 1),
                            s => s,
                        };
                        actions.push(Action::Resize(self.size()));
                    }
                }
            }
            MENU_SET_DEFAULT => {
                if let Some(row) = self.menu_row {
                    let name = self.row_template(row).map(|t| t.name.clone()).unwrap_or_default();
                    shared.cfg.set_str("New", "DefaultTemplate", &name);
                }
            }
            _ => {}
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) -> Vec<Action> {
        let mut actions = Vec::new();
        let ctx = ui.ctx().clone();
        let o = ui.max_rect().min;
        let blocked = self.msg.is_some() || self.menu.is_some();

        // name + browse
        let name_rect = Rect::from_min_size(o + vec2(MARGIN_X, MARGIN_Y), vec2(272.0, 23.0));
        let name_resp = theme::text_edit(ui, name_rect, &mut self.name, "Untitled", "ns-name");
        if self.focus_name {
            name_resp.request_focus();
            self.focus_name = false;
        }
        if name_resp.has_focus() && !blocked {
            let (up, down) = ctx.input(|i| (i.key_pressed(Key::ArrowUp), i.key_pressed(Key::ArrowDown)));
            if up {
                self.selected = Some(self.selected.map(|s| s.saturating_sub(1)).unwrap_or(0));
            }
            if down {
                self.selected = Some(self.selected.map(|s| (s + 1).min(self.rows.len() - 1)).unwrap_or(0));
            }
        }
        let browse = Rect::from_min_size(pos2(name_rect.right(), name_rect.top() - 1.0), vec2(28.0, 25.0));
        if theme::icon_button(ui, browse, "ns-browse", |p, r| icons::draw(p, r, Icon::Folder)).on_hover_text("Browse").clicked() && !blocked {
            let file = rfd::FileDialog::new()
                .set_title("New Script")
                .set_directory(&self.dir)
                .set_file_name(format!("{}.ahk", if self.name.is_empty() { "Untitled" } else { &self.name }))
                .add_filter("Script Files (*.ahk)", &["ahk"])
                .save_file();
            if let Some(f) = file {
                self.name = f.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                self.dir = f.parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
            }
        }

        // folder
        let dir_rect = Rect::from_min_size(o + vec2(MARGIN_X, MARGIN_Y + 23.0 + MARGIN_Y), vec2(300.0, 23.0));
        theme::readonly_field(ui, dir_rect, &self.dir);

        // template list
        let top = o + vec2(MARGIN_X, Self::list_top());
        let tiles: Vec<Tile> = self
            .rows
            .iter()
            .map(|r| match r {
                None => Tile { name: "Empty", desc: Some("Clean slate"), icon: None },
                Some(i) => Tile { name: &self.templates[*i].name, desc: Some(&self.templates[*i].desc), icon: None },
            })
            .collect();
        let res = theme::tile_list(ui, "ns-list", top, 300.0, TILE_H, &tiles, self.selected, self.selected, true);
        if !blocked {
            if let Some(i) = res.clicked {
                self.selected = Some(i);
            }
            if let Some(i) = res.double_clicked {
                self.selected = Some(i);
                let ctrl = ctx.input(|i| i.modifiers.ctrl);
                self.confirm(!self.default_create, ctrl, shared, &mut actions);
            }
            if let Some((i, pos)) = res.right_clicked {
                self.selected = Some(i);
                self.template_menu(&ctx, i, pos, shared);
            }
        }

        // bottom row
        let by = o.y + Self::list_top() + self.list_h() + MARGIN_Y;
        let defaults = Rect::from_min_size(pos2(o.x + MARGIN_X, by), vec2(28.0, 25.0));
        if theme::icon_button(ui, defaults, "ns-defaults", |p, r| icons::draw(p, r, Icon::Defaults)).on_hover_text("Defaults").clicked() && !blocked {
            self.defaults_menu(&ctx, defaults.left_bottom(), shared);
        }
        let create = Rect::from_min_size(pos2(o.x + 150.0, by), vec2(75.0, 25.0));
        let edit = Rect::from_min_size(pos2(o.x + MARGIN_X + 225.0, by), vec2(75.0, 25.0));
        let ctrl = ctx.input(|i| i.modifiers.ctrl);
        if theme::button(ui, create, "&Create", self.default_create, true).clicked() && !blocked {
            self.confirm(false, ctrl, shared, &mut actions);
        }
        if theme::button(ui, edit, "&Edit", !self.default_create, true).clicked() && !blocked {
            self.confirm(true, ctrl, shared, &mut actions);
        }
        if !blocked {
            let (enter, esc, alt_c, alt_e) = ctx.input(|i| {
                (
                    i.key_pressed(Key::Enter),
                    i.key_pressed(Key::Escape),
                    i.modifiers.alt && i.key_pressed(Key::C),
                    i.modifiers.alt && i.key_pressed(Key::E),
                )
            });
            if enter {
                self.confirm(!self.default_create, ctrl, shared, &mut actions);
            } else if alt_c {
                self.confirm(false, ctrl, shared, &mut actions);
            } else if alt_e {
                self.confirm(true, ctrl, shared, &mut actions);
            }
            if esc {
                actions.push(Action::Close);
            }
        }

        if let Some(m) = &mut self.menu {
            if let Some(a) = m.show(&ctx) {
                self.menu = None;
                if a != 0 {
                    self.handle_menu(a, shared, &mut actions);
                }
            }
        }
        if let Some(m) = &self.msg {
            if let Some(r) = m.show(&ctx) {
                let tag = m.tag;
                self.msg = None;
                if tag == MSG_CREATE_TEMPLATES {
                    if r == MsgResult::Yes {
                        let _ = std::fs::create_dir_all(user_template_folder());
                        if let Some(p) = self.pending.take() {
                            self.run_pending(p, &mut actions);
                        }
                    }
                    self.pending = None;
                }
            }
        }
        let _ = Kind::Dash;
        actions
    }
}

fn default_dir(shared: &Shared) -> String {
    let d = shared.cfg.get_str("New", "DefaultDir", "");
    if d.is_empty() {
        sys::ahk_user_dir().to_string_lossy().into_owned()
    } else {
        d
    }
}
