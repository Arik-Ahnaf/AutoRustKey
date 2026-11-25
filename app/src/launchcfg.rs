//! "AutoHotkey Launch Config" window (ui-launcherconfig.ahk).

use crate::app::{Action, Shared};
use crate::icons::{self, Icon};
use crate::launcher;
use crate::theme::{self, MessageBox, MsgIcon, LINE, MARGIN_X, MARGIN_Y};
use crate::{assoc, sys};
use egui::{pos2, vec2, Rect, Ui, Vec2};

pub struct LaunchWin {
    use_launcher: bool,
    v1: Vec<String>,
    v2: Vec<String>,
    uia_available: bool,
    version: [usize; 2],
    build: [usize; 2],
    uia: [bool; 2],
    fallback: usize,
    identify: bool,
    launcher_utf8: bool,
    exe_path: String,
    custom_utf8: bool,
    custom_utf8_visible: bool,
    msg: Option<MessageBox>,
    note: Option<String>,
}

const BUILDS: [&[&str]; 2] = [&["Unicode 64-bit", "Unicode 32-bit", "ANSI 32-bit"], &["64-bit", "32-bit"]];
const FALLBACKS: [&str; 3] = ["Use v1.x", "Use v2.x", "Ask the user"];

// Layout rows (AutoHotkey's automatic positions for these controls).
const Y_RADIO2: f32 = MARGIN_Y + LINE + MARGIN_Y;
const Y_PAGE: f32 = Y_RADIO2 + LINE + 12.0;
const Y_V1: f32 = Y_PAGE + LINE + 3.0;
const Y_V2: f32 = Y_V1 + 23.0 + MARGIN_Y;
const Y_FB_LABEL: f32 = Y_V2 + 2.0 + LINE + MARGIN_Y + 8.0;
const Y_FB: f32 = Y_FB_LABEL + LINE + 3.0;
const Y_IDENTIFY: f32 = Y_FB + 23.0 + MARGIN_Y + 8.0;
const Y_UTF8: f32 = Y_IDENTIFY + LINE + MARGIN_Y;
const Y_CUSTOM_UTF8: f32 = Y_PAGE + 23.0 + MARGIN_Y + 4.0;
const WIDTH: f32 = MARGIN_X + 326.0 + 28.0 + MARGIN_X;

impl LaunchWin {
    pub fn new(shared: &mut Shared) -> Self {
        let list = shared.interpreters().to_vec();
        let (v1, v2, uia_available) = launcher::versions_by_major(&list);
        let cfg = &shared.cfg;
        let mode = cfg.get_str("Launcher", "Mode", "launcher");
        let mut exe_path = cfg.get_str("Launcher", "Exe", "");
        if exe_path.is_empty() {
            if let Some(r) = &shared.root {
                let f = r.join("v2").join("AutoHotkey64.exe");
                if f.is_file() {
                    exe_path = f.to_string_lossy().into_owned();
                }
            }
        }
        let mut s = Self {
            use_launcher: mode != "specific",
            v1: v1.clone(),
            v2: v2.clone(),
            uia_available,
            version: [0, 0],
            build: [0, 0],
            uia: [false, false],
            fallback: 2,
            identify: cfg.get_bool("Launcher", "Identify", true),
            launcher_utf8: cfg.get_bool("Launcher\\v1", "UTF8", false),
            exe_path,
            custom_utf8: cfg.get_bool("Launcher", "CustomUTF8", false),
            custom_utf8_visible: false,
            msg: None,
            note: None,
        };
        for m in 0..2 {
            let section = format!("Launcher\\v{}", m + 1);
            let v = cfg.get_str(&section, "Version", "");
            let list = if m == 0 { &v1 } else { &v2 };
            s.version[m] = list.iter().position(|x| *x == v).map(|i| i + 1).unwrap_or(0);
            let b = cfg.get_str(&section, "Build", "");
            s.build[m] = BUILDS[m].iter().position(|x| *x == b).unwrap_or(0);
            s.uia[m] = cfg.get_bool(&section, "UIA", false);
        }
        if let Ok(f) = cfg.get_str("Launcher", "Fallback", "").parse::<usize>() {
            if (1..=3).contains(&f) {
                s.fallback = f - 1;
            }
        }
        s.changed_exe();
        s
    }

    pub fn title(&self) -> String {
        "AutoHotkey Launch Config".into()
    }

    pub fn size(&self) -> Vec2 {
        let y1 = if self.use_launcher { Y_UTF8 } else { Y_CUSTOM_UTF8 };
        vec2(WIDTH, y1 + 25.0 + 10.0)
    }

    fn changed_exe(&mut self) {
        if self.exe_path.is_empty() {
            return;
        }
        match launcher::exe_info(std::path::Path::new(&self.exe_path)) {
            Some(info) => {
                if !self.custom_utf8_visible && self.launcher_utf8 {
                    self.custom_utf8 = true;
                }
                self.custom_utf8_visible = launcher::ver_cmp(&info.version, "2").is_lt();
            }
            None => {
                self.msg = Some(MessageBox::new(
                    "AutoHotkey Launch Config",
                    &format!("The selected EXE appears to be invalid.\n\nSpecifically: {}", self.exe_path),
                    MsgIcon::Warning,
                ));
            }
        }
    }

    /// UpdateVerbs: make the desktop open .ahk files the chosen way.
    fn update_verbs(&mut self, shared: &mut Shared) {
        shared.cfg.set_str("Launcher", "Mode", if self.use_launcher { "launcher" } else { "specific" });
        let result = if self.use_launcher {
            assoc::set_open_command("AutoHotkey Launcher", &assoc::launcher_command())
        } else {
            if self.exe_path.is_empty() {
                return;
            }
            shared.cfg.set_str("Launcher", "Exe", &self.exe_path);
            shared.cfg.set_bool("Launcher", "CustomUTF8", self.custom_utf8);
            let switches = if self.custom_utf8_visible && self.custom_utf8 { "/cp65001 " } else { "" };
            let name = launcher::exe_info(std::path::Path::new(&self.exe_path)).map(|i| i.description).unwrap_or_else(|| "AutoHotkey".into());
            assoc::set_open_command(&name, &assoc::interpreter_command(&self.exe_path, switches))
        };
        self.note = result.err();
    }

    fn dropdown(ui: &mut Ui, rect: Rect, id: &str, idx: &mut usize, items: &[String]) -> bool {
        let before = *idx;
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.spacing_mut().interact_size.y = rect.height();
            ui.spacing_mut().button_padding = vec2(7.0, 3.0);
            egui::ComboBox::from_id_salt(id)
                .width(rect.width())
                .height(240.0)
                .icon(|ui, rect, _visuals, _open, _above| {
                    // Win11 drop-down chevron
                    let c = rect.center();
                    ui.painter().add(egui::Shape::line(
                        vec![pos2(c.x - 4.0, c.y - 2.0), pos2(c.x, c.y + 2.0), pos2(c.x + 4.0, c.y - 2.0)],
                        egui::Stroke::new(1.0_f32, theme::TEXT),
                    ));
                })
                .selected_text(items.get(*idx).cloned().unwrap_or_default())
                .show_ui(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                    for (i, it) in items.iter().enumerate() {
                        ui.selectable_value(idx, i, it);
                    }
                });
        });
        before != *idx
    }

    pub fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) -> Vec<Action> {
        let mut actions = Vec::new();
        let ctx = ui.ctx().clone();
        let o = ui.max_rect().min;
        let before_mode = self.use_launcher;

        if theme::radio(ui, o + vec2(MARGIN_X, MARGIN_Y), self.use_launcher, "Auto-detect version when launching script").clicked() {
            self.use_launcher = true;
        }
        if theme::radio(ui, o + vec2(MARGIN_X, Y_RADIO2), !self.use_launcher, "Run all scripts with a specific interpreter").clicked() {
            self.use_launcher = false;
        }
        if before_mode != self.use_launcher {
            actions.push(Action::Resize(self.size()));
            self.update_verbs(shared);
        }

        if self.use_launcher {
            theme::label(ui, o + vec2(MARGIN_X, Y_PAGE), "Preferred interpreter by major version", theme::BASE, theme::TEXT);
            for m in 0..2 {
                let y = if m == 0 { Y_V1 } else { Y_V2 };
                let section = format!("Launcher\\v{}", m + 1);
                let mut items = vec![format!("Latest {}.x", m + 1)];
                items.extend(if m == 0 { self.v1.clone() } else { self.v2.clone() });
                if Self::dropdown(ui, Rect::from_min_size(o + vec2(MARGIN_X, y), vec2(110.0, 23.0)), &format!("ver{m}"), &mut self.version[m], &items) {
                    let v = if self.version[m] == 0 { String::new() } else { items[self.version[m]].clone() };
                    shared.cfg.set_str(&section, "Version", &v);
                }
                let builds: Vec<String> = BUILDS[m].iter().map(|s| s.to_string()).collect();
                if Self::dropdown(ui, Rect::from_min_size(o + vec2(MARGIN_X + 110.0 + MARGIN_X, y), vec2(150.0, 23.0)), &format!("build{m}"), &mut self.build[m], &builds) {
                    shared.cfg.set_str(&section, "Build", &builds[self.build[m]]);
                }
                let mut u = self.uia[m];
                if theme::checkbox(ui, o + vec2(MARGIN_X + 110.0 + MARGIN_X + 150.0 + MARGIN_X, y + 3.0), &mut u, "UI Access", self.uia_available).changed() {
                    self.uia[m] = u;
                    shared.cfg.set_bool(&section, "UIA", u);
                }
            }
            theme::label(ui, o + vec2(MARGIN_X, Y_FB_LABEL), "When detection fails", theme::BASE, theme::TEXT);
            let fb: Vec<String> = FALLBACKS.iter().map(|s| s.to_string()).collect();
            if Self::dropdown(ui, Rect::from_min_size(o + vec2(MARGIN_X, Y_FB), vec2(110.0, 23.0)), "fallback", &mut self.fallback, &fb) {
                let v = if self.fallback == 2 { String::new() } else { (self.fallback + 1).to_string() };
                shared.cfg.set_str("Launcher", "Fallback", &v);
            }
            if theme::checkbox(ui, o + vec2(MARGIN_X, Y_IDENTIFY), &mut self.identify, "Try to identify version based on syntax", true).changed() {
                shared.cfg.set_bool("Launcher", "Identify", self.identify);
            }
            if theme::checkbox(ui, o + vec2(MARGIN_X, Y_UTF8), &mut self.launcher_utf8, "Default to UTF-8 even for v1 scripts", true).changed() {
                shared.cfg.set_bool("Launcher\\v1", "UTF8", self.launcher_utf8);
            }
        } else {
            let field = Rect::from_min_size(o + vec2(MARGIN_X, Y_PAGE), vec2(326.0, 23.0));
            theme::readonly_field(ui, field, &self.exe_path);
            let browse = Rect::from_min_size(pos2(field.right(), field.top() - 1.0), vec2(28.0, 25.0));
            if theme::icon_button(ui, browse, "lc-browse", |p, r| icons::draw(p, r, Icon::Folder)).on_hover_text("Browse").clicked() {
                let mut dlg = rfd::FileDialog::new().set_title("Select an AutoHotkey.exe").add_filter("EXE Files (*.exe)", &["exe"]);
                if let Some(dir) = std::path::Path::new(&self.exe_path).parent() {
                    dlg = dlg.set_directory(dir);
                }
                if let Some(f) = dlg.pick_file() {
                    self.exe_path = f.to_string_lossy().into_owned();
                    self.changed_exe();
                    self.update_verbs(shared);
                }
            }
            if self.custom_utf8_visible && theme::checkbox(ui, o + vec2(MARGIN_X, Y_CUSTOM_UTF8), &mut self.custom_utf8, "Default to UTF-8", true).changed() {
                self.update_verbs(shared);
            }
        }

        let y1 = if self.use_launcher { Y_UTF8 } else { Y_CUSTOM_UTF8 };
        let close = Rect::from_min_size(o + vec2(292.0, y1), vec2(70.0, 25.0));
        if theme::button(ui, close, "&Close", true, true).clicked() {
            actions.push(Action::Close);
        }
        if let Some(n) = &self.note {
            ui.painter().text(o + vec2(MARGIN_X, y1 + 12.0), egui::Align2::LEFT_CENTER, format!("Could not update file association: {n}"), theme::font(10.0), theme::GRAY_TEXT);
        }
        if self.msg.is_none() && ctx.input(|i| i.key_pressed(egui::Key::Escape) || (i.key_pressed(egui::Key::Enter) && !ctx.wants_keyboard_input())) && !ctx.memory(|m| m.any_popup_open()) {
            actions.push(Action::Close);
        }
        if let Some(m) = &self.msg {
            if m.show(&ctx).is_some() {
                self.msg = None;
            }
        }
        let _ = sys::which;
        actions
    }
}
