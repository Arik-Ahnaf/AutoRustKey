//! "Window Spy for AHKv2" (WindowSpy.ahk). On Linux the window facts come from
//! xdotool/xprop (X11 / XWayland) or kdotool (KDE Plasma on Wayland).

use crate::app::{Action, Shared};
use crate::theme::{self, BASE, LINE, MARGIN_X, MARGIN_Y};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, Rect, Stroke, StrokeKind, Ui, Vec2};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default, Clone)]
pub struct SpyData {
    pub title: String,
    pub mouse: String,
    pub ctrl: String,
    pub pos: String,
    pub sb: String,
    pub vis: String,
    pub all: String,
    pub frozen: bool,
}

#[derive(Default)]
struct Control {
    follow_mouse: bool,
    suspended: bool,
    over_self: bool,
    stop: bool,
}

fn run(prog: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(prog).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn have(prog: &str) -> bool {
    crate::sys::which(prog).is_some()
}

struct WinFacts {
    id: String,
    title: String,
    class: String,
    pid: String,
    x: i64,
    y: i64,
    w: i64,
    h: i64,
}

fn parse_kv(text: &str) -> std::collections::HashMap<String, String> {
    text.lines().filter_map(|l| l.split_once('=')).map(|(k, v)| (k.trim().to_string(), v.trim().to_string())).collect()
}

fn x11_window(id: &str) -> WinFacts {
    let title = run("xdotool", &["getwindowname", id]).unwrap_or_default();
    let pid = run("xdotool", &["getwindowpid", id]).unwrap_or_default();
    let class = run("xprop", &["-id", id, "WM_CLASS"])
        .and_then(|s| s.rsplit('"').nth(1).map(|c| c.to_string()))
        .unwrap_or_default();
    let g = parse_kv(&run("xdotool", &["getwindowgeometry", "--shell", id]).unwrap_or_default());
    let n = |k: &str| g.get(k).and_then(|v| v.parse().ok()).unwrap_or(0);
    WinFacts { id: id.to_string(), title, class, pid, x: n("X"), y: n("Y"), w: n("WIDTH"), h: n("HEIGHT") }
}

fn kde_window(id: &str) -> WinFacts {
    let title = run("kdotool", &["getwindowname", id]).unwrap_or_default();
    let class = run("kdotool", &["getwindowclassname", id]).unwrap_or_default();
    let pid = run("kdotool", &["getwindowpid", id]).unwrap_or_default();
    let geo = run("kdotool", &["getwindowgeometry", id]).unwrap_or_default();
    let (mut x, mut y, mut w, mut h) = (0, 0, 0, 0);
    for line in geo.lines() {
        let l = line.trim();
        if let Some(v) = l.strip_prefix("Position:") {
            let mut it = v.trim().split(',').map(|s| s.trim().parse::<f64>().unwrap_or(0.0) as i64);
            x = it.next().unwrap_or(0);
            y = it.next().unwrap_or(0);
        } else if let Some(v) = l.strip_prefix("Geometry:") {
            let mut it = v.trim().split('x').map(|s| s.trim().parse::<f64>().unwrap_or(0.0) as i64);
            w = it.next().unwrap_or(0);
            h = it.next().unwrap_or(0);
        }
    }
    WinFacts { id: id.to_string(), title, class, pid, x, y, w, h }
}

fn process_name(pid: &str) -> String {
    std::fs::read_to_string(format!("/proc/{pid}/comm")).map(|s| s.trim().to_string()).unwrap_or_default()
}

fn pixel_color(x: i64, y: i64) -> Option<String> {
    let out = run("import", &["-window", "root", "-crop", &format!("1x1+{x}+{y}"), "-depth", "8", "txt:-"])?;
    let hex = out.split_whitespace().find(|t| t.starts_with('#') && t.len() >= 7)?;
    Some(hex[1..7].to_ascii_uppercase())
}

/// textMangle: first line only, at most 40 characters, " (...)" when cut.
pub fn text_mangle(x: &str) -> String {
    if let Some(i) = x.find('\n') {
        return format!("{} (...)", &x[..i]);
    }
    if x.chars().count() > 40 {
        return format!("{} (...)", x.chars().take(40).collect::<String>());
    }
    x.to_string()
}

fn gather(follow_mouse: bool) -> Option<SpyData> {
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let use_kde = wayland && have("kdotool");
    let use_x11 = !use_kde && have("xdotool");
    if !use_kde && !use_x11 {
        return Some(SpyData {
            title: if cfg!(windows) {
                "Window inspection uses xdotool (X11) or\nkdotool (KDE Plasma on Wayland).".into()
            } else {
                "Install xdotool and xorg-xprop (X11) or\nkdotool (KDE Plasma on Wayland) to inspect windows.".into()
            },
            ..Default::default()
        });
    }
    let (mx, my, mouse_win) = if use_kde {
        let s = run("kdotool", &["getmouselocation"]).unwrap_or_default();
        let mut x = 0;
        let mut y = 0;
        let mut w = String::new();
        for t in s.split_whitespace() {
            if let Some(v) = t.strip_prefix("x:") {
                x = v.parse::<f64>().unwrap_or(0.0) as i64;
            } else if let Some(v) = t.strip_prefix("y:") {
                y = v.parse::<f64>().unwrap_or(0.0) as i64;
            } else if let Some(v) = t.strip_prefix("window:") {
                w = v.to_string();
            }
        }
        (x, y, w)
    } else {
        let g = parse_kv(&run("xdotool", &["getmouselocation", "--shell"]).unwrap_or_default());
        (g.get("X").and_then(|v| v.parse().ok()).unwrap_or(0), g.get("Y").and_then(|v| v.parse().ok()).unwrap_or(0), g.get("WINDOW").cloned().unwrap_or_default())
    };
    let active = if use_kde { run("kdotool", &["getactivewindow"]) } else { run("xdotool", &["getactivewindow"]) }.unwrap_or_default();
    let cur = if follow_mouse && !mouse_win.is_empty() { mouse_win } else { active.clone() };
    let f = if use_kde { kde_window(&cur) } else { x11_window(&cur) };
    let title = format!(
        "{}\nahk_class {}\nahk_exe {}\nahk_pid {}\nahk_id {}",
        f.title,
        f.class,
        process_name(&f.pid),
        f.pid,
        f.id
    );
    let (wx, wy) = (mx - f.x, my - f.y);
    let color = if use_kde { None } else { pixel_color(mx, my) };
    let color_line = match color {
        Some(c) => format!("{c} (Red={} Green={} Blue={})", &c[0..2], &c[2..4], &c[4..6]),
        None => "(not available)".into(),
    };
    let mouse = format!("Screen:\t{mx}, {my}\nWindow:\t{wx}, {wy}\nClient:\t{wx}, {wy} (default)\nColor:\t{color_line}");
    let pos = format!("Screen:\tx: {}\ty: {}\tw: {}\th: {}\nClient:\tx: 0\ty: 0\tw: {}\th: {}", f.x, f.y, f.w, f.h, f.w, f.h);
    Some(SpyData { title, mouse, ctrl: String::new(), pos, sb: String::new(), vis: text_mangle(&f.title), all: f.title.clone(), frozen: false })
}

pub struct SpyWin {
    follow_mouse: bool,
    slow: bool,
    data: Arc<Mutex<SpyData>>,
    control: Arc<Mutex<Control>>,
}

impl SpyWin {
    pub fn new(_shared: &Shared) -> Self {
        let data = Arc::new(Mutex::new(SpyData::default()));
        let control = Arc::new(Mutex::new(Control { follow_mouse: true, ..Default::default() }));
        let (d, c) = (data.clone(), control.clone());
        std::thread::spawn(move || loop {
            let (follow, frozen, stop) = {
                let c = c.lock().unwrap();
                (c.follow_mouse, c.suspended || c.over_self, c.stop)
            };
            if stop {
                break;
            }
            if frozen {
                d.lock().unwrap().frozen = true;
            } else if let Some(new) = gather(follow) {
                *d.lock().unwrap() = new;
            }
            std::thread::sleep(Duration::from_millis(250));
        });
        Self { follow_mouse: true, slow: false, data, control }
    }

    pub fn title(&self) -> String {
        "Window Spy for AHKv2".into()
    }

    pub fn size(&self) -> Vec2 {
        vec2(320.0 + MARGIN_X * 2.0, Self::layout(320.0).last().map(|r| r.1 + LINE + MARGIN_Y).unwrap_or(600.0))
    }

    /// (label y, box y, box rows) for each field, then the freeze label y.
    fn layout(_w: f32) -> Vec<(f32, f32)> {
        let rows = [5.0, 4.0, 4.0, 2.0, 2.0, -1.0, 2.0, 2.0];
        let mut out = Vec::new();
        let mut y = MARGIN_Y;
        for r in rows {
            if r < 0.0 {
                out.push((y, y)); // checkbox row
                y += LINE + MARGIN_Y;
                continue;
            }
            let box_y = y + LINE + MARGIN_Y;
            out.push((y, box_y));
            y = box_y + r * LINE + 8.0 + MARGIN_Y;
        }
        out.push((y, y));
        out
    }

    fn text_box(ui: &Ui, rect: Rect, text: &str, arrows: bool) {
        let p = ui.painter();
        p.rect(rect.shrink(0.5), CornerRadius::same(3), Color32::from_rgb(0xFB, 0xFB, 0xFB), Stroke::new(1.0_f32, Color32::from_rgb(0xE3, 0xE3, 0xE3)), StrokeKind::Middle);
        p.line_segment([pos2(rect.left() + 2.0, rect.bottom() - 0.5), pos2(rect.right() - 2.0, rect.bottom() - 0.5)], Stroke::new(1.0_f32, Color32::from_rgb(0x86, 0x86, 0x86)));
        let clip = rect.shrink2(vec2(4.0, 2.0));
        let pc = p.with_clip_rect(Rect::from_min_max(clip.min, pos2(clip.right() - if arrows { 14.0 } else { 0.0 }, clip.bottom())));
        let x0 = rect.left() + 6.0;
        for (i, line) in text.lines().enumerate() {
            let y = rect.top() + 4.0 + i as f32 * LINE;
            let mut x = x0;
            for (j, part) in line.split('\t').enumerate() {
                if j > 0 {
                    let rel = x - x0;
                    x = x0 + ((rel / 50.0).floor() + 1.0) * 50.0;
                }
                let r = pc.text(pos2(x, y), Align2::LEFT_TOP, part, theme::font(BASE), theme::TEXT);
                x = r.right();
            }
        }
        if arrows {
            theme::scroll_arrows(ui, rect);
        } else {
            p.line_segment([pos2(rect.right() - 17.0, rect.top() + 2.0), pos2(rect.right() - 17.0, rect.bottom() - 2.0)], Stroke::new(1.0_f32, Color32::from_rgb(0xE8, 0xE8, 0xE8)));
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, _shared: &mut Shared) -> Vec<Action> {
        let ctx = ui.ctx().clone();
        let o = ui.max_rect().min;
        let w = (ui.max_rect().width() - MARGIN_X * 2.0).max(120.0);
        let lay = Self::layout(w);
        let data = self.data.lock().unwrap().clone();
        let mods = ctx.input(|i| i.modifiers);
        let hovered_self = ctx.input(|i| i.pointer.has_pointer());
        let focused = ctx.input(|i| i.viewport().focused.unwrap_or(false));
        {
            let mut c = self.control.lock().unwrap();
            c.follow_mouse = self.follow_mouse;
            c.suspended = mods.ctrl || mods.shift;
            c.over_self = if self.follow_mouse { hovered_self } else { focused };
        }
        let frozen = data.frozen || mods.ctrl || mods.shift || (self.follow_mouse && hovered_self);

        let labels = [
            "Window Title, Class and Process:",
            "Mouse Position:",
            if self.follow_mouse { "Control Under Mouse Position:" } else { "Focused Control:" },
            "Active Window Position:",
            "Status Bar Text:",
            "",
            "Visible Text:",
            "All Text:",
        ];
        let rows = [5.0, 4.0, 4.0, 2.0, 2.0, 0.0, 2.0, 2.0];
        let texts = [&data.title, &data.mouse, &data.ctrl, &data.pos, &data.sb, &data.sb, &data.vis, &data.all];
        for i in 0..8 {
            let (ly, by) = lay[i];
            if i == 5 {
                theme::checkbox(ui, o + vec2(MARGIN_X, ly), &mut self.slow, "Slow TitleMatchMode", true);
                continue;
            }
            theme::label(ui, o + vec2(MARGIN_X, ly), labels[i], BASE, theme::TEXT);
            let rect = Rect::from_min_size(o + vec2(MARGIN_X, by), vec2(w, rows[i] * LINE + 8.0));
            Self::text_box(ui, rect, texts[i], i < 5);
        }
        // "Follow Mouse" check box, right-aligned on the first row
        let fm_w = theme::text_width(ui, "Follow Mouse", BASE) + 19.0;
        let fm_x = o.x + MARGIN_X + w - fm_w;
        let tw = theme::text_width(ui, "Follow Mouse", BASE);
        // draw label then box on the right, like the original's Right option
        let rect = Rect::from_min_size(pos2(fm_x, o.y + MARGIN_Y), vec2(fm_w, LINE));
        let resp = ui.interact(rect, ui.id().with("follow"), egui::Sense::click());
        if resp.clicked() {
            self.follow_mouse = !self.follow_mouse;
        }
        let p = ui.painter();
        p.text(pos2(fm_x, rect.center().y), Align2::LEFT_CENTER, "Follow Mouse", theme::font(BASE), theme::TEXT);
        let b = Rect::from_min_size(pos2(fm_x + tw + 6.0, rect.top() + 1.5), vec2(13.0, 13.0));
        if self.follow_mouse {
            p.rect_filled(b, CornerRadius::same(3), theme::ACCENT);
            p.add(egui::Shape::line(
                vec![pos2(b.left() + 3.0, b.center().y + 0.2), pos2(b.left() + 5.4, b.bottom() - 3.4), pos2(b.right() - 2.8, b.top() + 3.6)],
                Stroke::new(1.4_f32, Color32::WHITE),
            ));
        } else {
            p.rect(b.shrink(0.5), CornerRadius::same(3), Color32::from_rgb(0xF7, 0xF7, 0xF7), Stroke::new(1.0_f32, Color32::from_rgb(0x85, 0x85, 0x85)), StrokeKind::Middle);
        }
        let fy = lay[8].0;
        theme::label(ui, o + vec2(MARGIN_X, fy), if frozen { "(Updates suspended)" } else { "(Hold Ctrl or Shift to suspend updates)" }, BASE, theme::TEXT);
        ctx.request_repaint_after(Duration::from_millis(250));
        Vec::new()
    }
}

impl Drop for SpyWin {
    fn drop(&mut self) {
        self.control.lock().unwrap().stop = true;
    }
}
