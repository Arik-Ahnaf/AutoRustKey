//! The application: one main window plus tool windows, each its own native
//! window like the separate GUIs of the original.

use crate::config::Config;
use crate::dash::DashWin;
use crate::editor::EditorWin;
use crate::launchcfg::LaunchWin;
use crate::launcher::Interpreter;
use crate::newscript::NewScriptWin;
use crate::pick::PickWin;
use crate::spy::SpyWin;
use crate::{icons, sys, theme};
use egui::{ViewportBuilder, ViewportCommand, ViewportId};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct Shared {
    pub cfg: Config,
    pub root: Option<PathBuf>,
    interpreters: Option<Vec<Interpreter>>,
    pub picked: Arc<Mutex<Option<Interpreter>>>,
}

impl Shared {
    pub fn new() -> Self {
        let cfg = Config::load();
        let root = sys::root_dir(&cfg);
        Self { cfg, root, interpreters: None, picked: Arc::new(Mutex::new(None)) }
    }

    pub fn interpreters(&mut self) -> &[Interpreter] {
        if self.interpreters.is_none() {
            self.interpreters = Some(crate::launcher::usable_interpreters(self.root.as_deref()));
        }
        self.interpreters.as_deref().unwrap_or(&[])
    }
}

#[derive(Clone, Debug)]
pub enum Kind {
    Dash,
    NewScript(Option<PathBuf>),
    Launch,
    Editor(Option<PathBuf>),
    Spy,
    Pick(PathBuf, Vec<Interpreter>),
}

pub enum Action {
    Close,
    Open(Kind),
    Resize(egui::Vec2),
    SpawnSelf(Vec<String>),
    EditFile(PathBuf),
}

pub enum Win {
    Dash(DashWin),
    New(NewScriptWin),
    Launch(LaunchWin),
    Editor(EditorWin),
    Spy(SpyWin),
    Pick(PickWin),
}

impl Win {
    pub fn create(kind: Kind, shared: &mut Shared) -> Self {
        match kind {
            Kind::Dash => Win::Dash(DashWin::new(shared)),
            Kind::NewScript(p) => Win::New(NewScriptWin::new(shared, p)),
            Kind::Launch => Win::Launch(LaunchWin::new(shared)),
            Kind::Editor(s) => Win::Editor(EditorWin::new(shared, s)),
            Kind::Spy => Win::Spy(SpyWin::new(shared)),
            Kind::Pick(script, list) => Win::Pick(PickWin::new(script, list)),
        }
    }

    pub fn tag(&self) -> &'static str {
        match self {
            Win::Dash(_) => "dash",
            Win::New(_) => "new",
            Win::Launch(_) => "launch",
            Win::Editor(_) => "editor",
            Win::Spy(_) => "spy",
            Win::Pick(_) => "pick",
        }
    }

    pub fn title(&self) -> String {
        match self {
            Win::Dash(w) => w.title(),
            Win::New(w) => w.title(),
            Win::Launch(w) => w.title(),
            Win::Editor(w) => w.title(),
            Win::Spy(w) => w.title(),
            Win::Pick(w) => w.title(),
        }
    }

    pub fn size(&self) -> egui::Vec2 {
        match self {
            Win::Dash(w) => w.size(),
            Win::New(w) => w.size(),
            Win::Launch(w) => w.size(),
            Win::Editor(w) => w.size(),
            Win::Spy(w) => w.size(),
            Win::Pick(w) => w.size(),
        }
    }

    pub fn resizable(&self) -> bool {
        matches!(self, Win::Spy(_))
    }

    pub fn builder(&self) -> ViewportBuilder {
        let mut b = ViewportBuilder::default()
            .with_title(self.title())
            .with_app_id("autohotkey-ux")
            .with_inner_size(self.size())
            .with_resizable(self.resizable())
            .with_maximize_button(self.resizable())
            .with_icon(Arc::new(icons::window_icon()));
        if let Win::Spy(_) = self {
            b = b.with_window_level(egui::WindowLevel::AlwaysOnTop).with_min_inner_size([200.0, 300.0]);
        }
        if let Win::Pick(_) = self {
            b = b.with_minimize_button(false);
        }
        b
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, shared: &mut Shared) -> Vec<Action> {
        match self {
            Win::Dash(w) => w.ui(ui, shared),
            Win::New(w) => w.ui(ui, shared),
            Win::Launch(w) => w.ui(ui, shared),
            Win::Editor(w) => w.ui(ui, shared),
            Win::Spy(w) => w.ui(ui, shared),
            Win::Pick(w) => w.ui(ui, shared),
        }
    }
}

struct Child {
    win: Win,
    id: ViewportId,
    close: bool,
    focus: bool,
}

pub struct Screenshot {
    pub path: PathBuf,
    pub frames: u32,
    pub requested: bool,
}

pub struct App {
    shared: Shared,
    root: Win,
    children: Vec<Child>,
    screenshot: Option<Screenshot>,
    serial: u64,
}

fn panel(ctx: &egui::Context, win: &mut Win, shared: &mut Shared) -> Vec<Action> {
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(theme::FACE))
        .show(ctx, |ui| win.ui(ui, shared))
        .inner
}

impl App {
    pub fn new(cc: &eframe::CreationContext, shared: Shared, root: Win, screenshot: Option<PathBuf>) -> Self {
        theme::install(&cc.egui_ctx);
        Self {
            shared,
            root,
            children: Vec::new(),
            screenshot: screenshot.map(|path| Screenshot { path, frames: 0, requested: false }),
            serial: 0,
        }
    }

    fn open(&mut self, kind: Kind) {
        let win = Win::create(kind, &mut self.shared);
        // One window per kind, like AutoHotkeyUxGui.Show: re-show the open one.
        if let Some(c) = self.children.iter_mut().find(|c| c.win.tag() == win.tag()) {
            c.focus = true;
            return;
        }
        self.serial += 1;
        let id = ViewportId::from_hash_of(("child", win.tag(), self.serial));
        self.children.push(Child { win, id, close: false, focus: false });
    }

    fn edit_file(&mut self, path: PathBuf) {
        let cmd = self.shared.cfg.get_str("Editor", "Command", "");
        if !cmd.is_empty() && crate::editor::find_executable(&cmd).is_some() {
            let _ = sys::spawn(&sys::expand_command(&cmd, &path), None);
        } else {
            self.open(Kind::Editor(Some(path)));
        }
    }

    /// Returns true when the window that produced the actions should close.
    fn apply(&mut self, actions: Vec<Action>, ctx: &egui::Context, viewport: ViewportId) -> bool {
        let mut close = false;
        for a in actions {
            match a {
                Action::Close => close = true,
                Action::Open(k) => self.open(k),
                Action::Resize(size) => ctx.send_viewport_cmd_to(viewport, ViewportCommand::InnerSize(size)),
                Action::SpawnSelf(args) => {
                    if let Ok(exe) = std::env::current_exe() {
                        let mut v = vec![exe.to_string_lossy().into_owned()];
                        v.extend(args);
                        let _ = sys::spawn(&v, None);
                    }
                }
                Action::EditFile(p) => self.edit_file(p),
            }
        }
        close
    }

    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(shot) = &mut self.screenshot else { return };
        ctx.request_repaint();
        shot.frames += 1;
        if shot.frames >= 120 && !shot.requested {
            shot.requested = true;
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let image = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = image {
            let [w, h] = img.size;
            let mut bytes = Vec::with_capacity(w * h * 4);
            for c in &img.pixels {
                bytes.extend_from_slice(&[c.r(), c.g(), c.b(), 255]);
            }
            if let Some(dir) = shot.path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            match image::RgbaImage::from_raw(w as u32, h as u32, bytes) {
                Some(buf) => {
                    if let Err(e) = buf.save(&shot.path) {
                        eprintln!("could not save screenshot: {e}");
                    }
                }
                None => eprintln!("screenshot had an unexpected size"),
            }
            ctx.send_viewport_cmd(ViewportCommand::Close);
        } else if shot.frames > 900 {
            eprintln!("no screenshot arrived");
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        theme::FACE.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_screenshot(ctx);

        let actions = panel(ctx, &mut self.root, &mut self.shared);
        if self.apply(actions, ctx, ViewportId::ROOT) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }

        let mut pending: Vec<(ViewportId, Vec<Action>)> = Vec::new();
        let Self { children, shared, .. } = self;
        for child in children.iter_mut() {
            let builder = child.win.builder();
            let shared = &mut *shared;
            let win = &mut child.win;
            let focus = std::mem::take(&mut child.focus);
            let mut close = false;
            let id = child.id;
            let acts = ctx.show_viewport_immediate(id, builder, |ctx, _class| {
                if focus {
                    ctx.send_viewport_cmd(ViewportCommand::Focus);
                }
                if ctx.input(|i| i.viewport().close_requested()) {
                    close = true;
                }
                panel(ctx, win, shared)
            });
            if close {
                child.close = true;
            }
            pending.push((id, acts));
        }
        // apply after drawing so new windows can be added to the list
        for (id, acts) in pending {
            if self.apply(acts, ctx, id) {
                if let Some(c) = self.children.iter_mut().find(|c| c.id == id) {
                    c.close = true;
                }
            }
        }
        self.children.retain(|c| !c.close);
    }
}
