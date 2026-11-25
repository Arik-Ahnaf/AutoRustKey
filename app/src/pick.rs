//! "Run <script> with" — the launcher's version picker (VersionSelectGui in launcher.ahk).

use crate::app::{Action, Shared};
use crate::icons::Icon;
use crate::launcher::Interpreter;
use crate::theme::{self, Tile, MARGIN_X, MARGIN_Y};
use egui::{vec2, Key, Ui, Vec2};
use std::path::PathBuf;

const TILE_H: f32 = 38.0;

pub struct PickWin {
    script: PathBuf,
    files: Vec<Interpreter>,
    labels: Vec<String>,
    selected: Option<usize>,
}

impl PickWin {
    pub fn new(script: PathBuf, files: Vec<Interpreter>) -> Self {
        let labels = files.iter().map(|f| format!("{} {}", f.version, f.description.replace("AutoHotkey ", ""))).collect();
        Self { script, files, labels, selected: Some(0) }
    }

    pub fn title(&self) -> String {
        format!("Run {} with", self.script.file_name().unwrap_or_default().to_string_lossy())
    }

    pub fn size(&self) -> Vec2 {
        vec2(200.0 + MARGIN_X * 2.0, self.files.len().clamp(1, 8) as f32 * TILE_H + 4.0 + MARGIN_Y * 2.0)
    }

    fn confirm(&mut self, i: usize, shared: &mut Shared, actions: &mut Vec<Action>) {
        if let Some(f) = self.files.get(i) {
            *shared.picked.lock().unwrap() = Some(f.clone());
            actions.push(Action::Close);
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, shared: &mut Shared) -> Vec<Action> {
        let mut actions = Vec::new();
        let o = ui.max_rect().min;
        let tiles: Vec<Tile> = self.labels.iter().map(|l| Tile { name: l, desc: None, icon: Some(Icon::Interpreter) }).collect();
        let res = theme::tile_list(ui, "pick", o + vec2(MARGIN_X, MARGIN_Y), 200.0, TILE_H, &tiles, self.selected, self.selected, true);
        if let Some(i) = res.clicked.or(res.double_clicked) {
            self.confirm(i, shared, &mut actions);
        }
        let ctx = ui.ctx().clone();
        let keys = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9];
        for (i, k) in keys.iter().enumerate() {
            if ctx.input(|inp| inp.key_pressed(*k)) {
                self.confirm(i, shared, &mut actions);
            }
        }
        let (up, down, enter, esc) = ctx.input(|i| (i.key_pressed(Key::ArrowUp), i.key_pressed(Key::ArrowDown), i.key_pressed(Key::Enter), i.key_pressed(Key::Escape)));
        if up {
            self.selected = Some(self.selected.unwrap_or(0).saturating_sub(1));
        }
        if down {
            self.selected = Some((self.selected.unwrap_or(0) + 1).min(self.files.len().saturating_sub(1)));
        }
        if enter {
            if let Some(i) = self.selected {
                self.confirm(i, shared, &mut actions);
            }
        }
        if esc {
            actions.push(Action::Close);
        }
        actions
    }
}
