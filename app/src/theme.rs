//! The Windows 11 "light" look of the original dialogs, drawn with egui:
//! #F0F0F0 dialog face, Segoe-UI-sized text, Explorer-style tile lists,
//! rounded push buttons, blue check boxes and radio buttons, underlined links.

use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Response, Sense,
    Stroke, StrokeKind, Ui,
};

pub const FACE: Color32 = Color32::from_rgb(0xF0, 0xF0, 0xF0);
pub const TEXT: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
pub const GRAY_TEXT: Color32 = Color32::from_rgb(0x6D, 0x6D, 0x6D);
pub const DISABLED_TEXT: Color32 = Color32::from_rgb(0xA0, 0xA0, 0xA0);
pub const LINK: Color32 = Color32::from_rgb(0x00, 0x66, 0xCC);
pub const ACCENT: Color32 = Color32::from_rgb(0x00, 0x5F, 0xB8);
pub const HOVER_FILL: Color32 = Color32::from_rgb(0xE5, 0xF3, 0xFF);
pub const SEL_FILL: Color32 = Color32::from_rgb(0xCC, 0xE8, 0xFF);
pub const SEL_BORDER: Color32 = Color32::from_rgb(0x99, 0xD1, 0xFF);
pub const EDGE: Color32 = Color32::from_rgb(0xD0, 0xD0, 0xD0);

/// Base text size: 9 pt Segoe UI = 12 px at 96 DPI.
pub const BASE: f32 = 12.0;
/// Line advance for the base font (Segoe UI 9 pt draws ~16 px rows).
pub const LINE: f32 = 16.0;
pub const MARGIN_X: f32 = 10.0;
pub const MARGIN_Y: f32 = 6.0;

pub fn font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

/// Load the closest system font to Segoe UI and set the dialog colours.
pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let candidates = [
        r"C:\Windows\Fonts\segoeui.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/TTF/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/TTF/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert("system-ui".into(), egui::FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "system-ui".into());
            break;
        }
    }
    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(egui::TextStyle::Body, font(BASE));
    style.text_styles.insert(egui::TextStyle::Button, font(BASE));
    style.text_styles.insert(egui::TextStyle::Small, font(10.0));
    style.text_styles.insert(egui::TextStyle::Monospace, FontId::new(BASE, FontFamily::Monospace));
    style.spacing.item_spacing = vec2(0.0, 0.0);
    style.spacing.button_padding = vec2(6.0, 2.0);
    let v = &mut style.visuals;
    *v = egui::Visuals::light();
    v.panel_fill = FACE;
    v.window_fill = FACE;
    v.override_text_color = Some(TEXT);
    v.extreme_bg_color = Color32::WHITE;
    v.selection.bg_fill = Color32::from_rgb(0x00, 0x78, 0xD7);
    v.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
    v.text_cursor.stroke = Stroke::new(1.0_f32, TEXT);
    let r = CornerRadius::same(3);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = r;
        w.expansion = 0.0;
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, EDGE);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0xC8, 0xC8, 0xC8));
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0xA8, 0xA8, 0xA8));
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    v.widgets.inactive.bg_fill = Color32::from_rgb(0xFD, 0xFD, 0xFD);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0xFD, 0xFD, 0xFD);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0xE0, 0xEE, 0xF9);
    v.widgets.hovered.bg_fill = Color32::from_rgb(0xE0, 0xEE, 0xF9);
    v.widgets.active.weak_bg_fill = Color32::from_rgb(0xCC, 0xE4, 0xF7);
    v.popup_shadow = egui::epaint::Shadow { offset: [2, 3], blur: 8, spread: 0, color: Color32::from_black_alpha(40) };
    v.window_shadow = v.popup_shadow;
    v.menu_corner_radius = CornerRadius::same(6);
    ctx.set_style(style);
}

pub fn text_width(ui: &Ui, text: &str, size: f32) -> f32 {
    ui.fonts(|f| f.layout_no_wrap(text.to_owned(), font(size), TEXT).size().x)
}

/// Plain text at a position (top-left).
pub fn label(ui: &Ui, pos: Pos2, text: &str, size: f32, color: Color32) -> Rect {
    ui.painter().text(pos, Align2::LEFT_TOP, text, font(size), color)
}

/// Text with a single '&' marking the mnemonic underline, like Win32 button captions.
fn split_mnemonic(text: &str) -> (String, Option<usize>) {
    let mut out = String::new();
    let mut idx = None;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' {
            if chars.peek() == Some(&'&') {
                chars.next();
                out.push('&');
            } else if idx.is_none() {
                idx = Some(out.chars().count());
            }
        } else {
            out.push(c);
        }
    }
    (out, idx)
}

/// Push button. `default` draws the accent border of the dialog's default button.
pub fn button(ui: &mut Ui, rect: Rect, caption: &str, default: bool, enabled: bool) -> Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let resp = ui.interact(rect, ui.id().with(("btn", caption, rect.min.x as i32, rect.min.y as i32)), sense);
    let p = ui.painter();
    let (fill, border) = if !enabled {
        (Color32::from_rgb(0xF5, 0xF5, 0xF5), Color32::from_rgb(0xE0, 0xE0, 0xE0))
    } else if resp.is_pointer_button_down_on() {
        (Color32::from_rgb(0xCC, 0xE4, 0xF7), Color32::from_rgb(0x00, 0x54, 0x99))
    } else if resp.hovered() {
        (Color32::from_rgb(0xE0, 0xEE, 0xF9), Color32::from_rgb(0x00, 0x78, 0xD4))
    } else if default {
        (Color32::from_rgb(0xFD, 0xFD, 0xFD), ACCENT)
    } else {
        (Color32::from_rgb(0xFD, 0xFD, 0xFD), Color32::from_rgb(0xD0, 0xD0, 0xD0))
    };
    let r = rect.shrink(0.5);
    p.rect(r, CornerRadius::same(4), fill, Stroke::new(1.0_f32, border), StrokeKind::Middle);
    if default && enabled {
        p.rect_stroke(r.shrink(1.0), CornerRadius::same(3), Stroke::new(1.0_f32, border), StrokeKind::Middle);
    }
    if !caption.is_empty() {
        let (txt, _) = split_mnemonic(caption);
        let color = if enabled { TEXT } else { DISABLED_TEXT };
        p.text(rect.center(), Align2::CENTER_CENTER, txt, font(BASE), color);
    }
    if enabled {
        resp.on_hover_cursor(egui::CursorIcon::Default)
    } else {
        resp
    }
}

/// Square icon-only button (the 28 px browse / defaults buttons).
pub fn icon_button(ui: &mut Ui, rect: Rect, id: &str, draw: impl FnOnce(&egui::Painter, Rect)) -> Response {
    let resp = ui.interact(rect, ui.id().with(("ibtn", id)), Sense::click());
    let p = ui.painter();
    let (fill, border) = if resp.is_pointer_button_down_on() {
        (Color32::from_rgb(0xCC, 0xE4, 0xF7), Color32::from_rgb(0x00, 0x54, 0x99))
    } else if resp.hovered() {
        (Color32::from_rgb(0xE0, 0xEE, 0xF9), Color32::from_rgb(0x00, 0x78, 0xD4))
    } else {
        (Color32::from_rgb(0xFD, 0xFD, 0xFD), Color32::from_rgb(0xD0, 0xD0, 0xD0))
    };
    p.rect(rect.shrink(0.5), CornerRadius::same(4), fill, Stroke::new(1.0_f32, border), StrokeKind::Middle);
    draw(p, Rect::from_center_size(rect.center(), vec2(16.0, 16.0)));
    resp
}

/// Windows 11 check box; returns true when toggled.
pub fn checkbox(ui: &mut Ui, pos: Pos2, value: &mut bool, text: &str, enabled: bool) -> Response {
    let tw = text_width(ui, text, BASE);
    let rect = Rect::from_min_size(pos, vec2(13.0 + 6.0 + tw + 2.0, LINE));
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let mut resp = ui.interact(rect, ui.id().with(("chk", text, pos.x as i32, pos.y as i32)), sense);
    if resp.clicked() {
        *value = !*value;
        resp.mark_changed();
    }
    let p = ui.painter();
    let b = Rect::from_min_size(pos2(pos.x, pos.y + (LINE - 13.0) / 2.0), vec2(13.0, 13.0));
    if *value {
        let fill = if !enabled { DISABLED_TEXT } else if resp.hovered() { Color32::from_rgb(0x19, 0x6E, 0xBF) } else { ACCENT };
        p.rect_filled(b, CornerRadius::same(3), fill);
        let pts = vec![
            pos2(b.left() + 3.0, b.center().y + 0.2),
            pos2(b.left() + 5.4, b.bottom() - 3.4),
            pos2(b.right() - 2.8, b.top() + 3.6),
        ];
        p.add(egui::Shape::line(pts, Stroke::new(1.4_f32, Color32::WHITE)));
    } else {
        let fill = if resp.hovered() && enabled { Color32::from_rgb(0xEA, 0xEA, 0xEA) } else { Color32::from_rgb(0xF7, 0xF7, 0xF7) };
        let border = if enabled { Color32::from_rgb(0x85, 0x85, 0x85) } else { Color32::from_rgb(0xC0, 0xC0, 0xC0) };
        p.rect(b.shrink(0.5), CornerRadius::same(3), fill, Stroke::new(1.0_f32, border), StrokeKind::Middle);
    }
    p.text(
        pos2(b.right() + 6.0, pos.y + LINE / 2.0),
        Align2::LEFT_CENTER,
        text,
        font(BASE),
        if enabled { TEXT } else { DISABLED_TEXT },
    );
    resp
}

/// Windows 11 radio button; returns clicked.
pub fn radio(ui: &mut Ui, pos: Pos2, selected: bool, text: &str) -> Response {
    let tw = text_width(ui, text, BASE);
    let rect = Rect::from_min_size(pos, vec2(13.0 + 6.0 + tw + 2.0, LINE));
    let resp = ui.interact(rect, ui.id().with(("radio", text)), Sense::click());
    let p = ui.painter();
    let c = pos2(pos.x + 6.5, pos.y + LINE / 2.0);
    if selected {
        p.circle_filled(c, 6.5, if resp.hovered() { Color32::from_rgb(0x19, 0x6E, 0xBF) } else { ACCENT });
        p.circle_filled(c, if resp.hovered() { 3.4 } else { 3.0 }, Color32::WHITE);
    } else {
        p.circle(c, 6.0, Color32::from_rgb(0xF7, 0xF7, 0xF7), Stroke::new(1.0_f32, Color32::from_rgb(0x85, 0x85, 0x85)));
    }
    p.text(pos2(pos.x + 19.0, c.y), Align2::LEFT_CENTER, text, font(BASE), TEXT);
    resp
}

/// Underlined hyperlink text; returns clicked.
pub fn link(ui: &mut Ui, pos: Pos2, text: &str) -> Response {
    let galley = ui.fonts(|f| f.layout_no_wrap(text.to_owned(), font(BASE), LINK));
    let rect = Rect::from_min_size(pos, galley.size());
    let resp = ui.interact(rect, ui.id().with(("link", text)), Sense::click());
    let p = ui.painter();
    p.galley(pos, galley, LINK);
    let y = rect.bottom() - 1.5;
    p.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], Stroke::new(1.0_f32, LINK));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A read-only "edit" box like the disabled-looking path fields.
pub fn readonly_field(ui: &Ui, rect: Rect, text: &str) {
    let p = ui.painter();
    p.rect(rect.shrink(0.5), CornerRadius::same(3), Color32::from_rgb(0xE8, 0xE8, 0xE8), Stroke::new(1.0_f32, Color32::from_rgb(0xD5, 0xD5, 0xD5)), StrokeKind::Middle);
    let clip = rect.shrink2(vec2(5.0, 1.0));
    p.with_clip_rect(clip).text(pos2(rect.left() + 6.0, rect.center().y), Align2::LEFT_CENTER, text, font(BASE), TEXT);
}

/// A single-line text box styled like a Win11 edit control.
pub fn text_edit(ui: &mut Ui, rect: Rect, text: &mut String, hint: &str, id: &str) -> Response {
    let resp = ui
        .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0_f32, EDGE);
            ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(0xB0, 0xB0, 0xB0));
            ui.visuals_mut().selection.stroke = Stroke::new(1.0_f32, ACCENT);
            ui.visuals_mut().selection.bg_fill = Color32::from_rgb(0x99, 0xC9, 0xEF);
            ui.add_sized(
                rect.size(),
                egui::TextEdit::singleline(text)
                    .id(egui::Id::new(("edit", id)))
                    .hint_text(egui::RichText::new(hint).color(GRAY_TEXT))
                    .margin(vec2(5.0, 3.0))
                    .vertical_align(egui::Align::Center),
            )
        })
        .inner;
    if resp.has_focus() {
        let p = ui.painter();
        p.line_segment(
            [pos2(rect.left() + 1.0, rect.bottom() - 1.0), pos2(rect.right() - 1.0, rect.bottom() - 1.0)],
            Stroke::new(2.0_f32, ACCENT),
        );
    }
    resp
}

/// A multi-line text box.
pub fn text_area(ui: &mut Ui, rect: Rect, text: &mut String, id: &str, readonly: bool) -> Response {
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0_f32, EDGE);
        ui.visuals_mut().widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, EDGE);
        ui.visuals_mut().selection.stroke = Stroke::new(1.0_f32, ACCENT);
        ui.visuals_mut().selection.bg_fill = Color32::from_rgb(0x99, 0xC9, 0xEF);
        let mut te = egui::TextEdit::multiline(text)
            .id(egui::Id::new(("area", id)))
            .margin(vec2(5.0, 3.0))
            .desired_rows(1)
            .lock_focus(false);
        if readonly {
            te = te.interactive(true).code_editor().font(font(BASE));
        }
        let r = ui.add_sized(rect.size(), te);
        r
    })
    .inner
}

/// Draw a Win11-style thin vertical scroll arrow column (decorative, like the
/// read-only multi-line edits in Window Spy).
pub fn scroll_arrows(ui: &Ui, rect: Rect) {
    let p = ui.painter();
    let x = rect.right() - 9.0;
    let up = pos2(x, rect.top() + 8.0);
    let dn = pos2(x, rect.bottom() - 8.0);
    let c = Color32::from_rgb(0x8A, 0x8A, 0x8A);
    p.add(egui::Shape::convex_polygon(vec![pos2(up.x - 3.5, up.y + 2.5), pos2(up.x + 3.5, up.y + 2.5), pos2(up.x, up.y - 2.0)], c, Stroke::NONE));
    p.add(egui::Shape::convex_polygon(vec![pos2(dn.x - 3.5, dn.y - 2.5), pos2(dn.x + 3.5, dn.y - 2.5), pos2(dn.x, dn.y + 2.0)], c, Stroke::NONE));
}

/// One row of an Explorer-themed tile list.
pub struct Tile<'a> {
    pub name: &'a str,
    pub desc: Option<&'a str>,
    pub icon: Option<crate::icons::Icon>,
}

pub struct TileListResult {
    pub clicked: Option<usize>,
    pub double_clicked: Option<usize>,
    pub right_clicked: Option<(usize, Pos2)>,
    pub hovered: Option<usize>,
}

/// Tile-view list ("list menu" in the original). `selected` shows the
/// selection highlight; `focus` draws the keyboard focus rectangle.
pub fn tile_list(
    ui: &mut Ui,
    id: &str,
    origin: Pos2,
    width: f32,
    tile_h: f32,
    tiles: &[Tile],
    selected: Option<usize>,
    focus: Option<usize>,
    show_focus: bool,
) -> TileListResult {
    let mut out = TileListResult { clicked: None, double_clicked: None, right_clicked: None, hovered: None };
    for (i, t) in tiles.iter().enumerate() {
        let rect = Rect::from_min_size(pos2(origin.x, origin.y + i as f32 * tile_h), vec2(width, tile_h));
        let resp = ui.interact(rect, ui.id().with((id, "tile", i)), Sense::click());
        let p = ui.painter();
        let is_sel = selected == Some(i);
        let hov = resp.hovered();
        if hov {
            out.hovered = Some(i);
        }
        let inner = rect.shrink(1.0);
        if is_sel {
            let fill = if hov { Color32::from_rgb(0xC4, 0xE0, 0xF9) } else { SEL_FILL };
            p.rect(inner, CornerRadius::ZERO, fill, Stroke::new(1.0_f32, if show_focus && focus == Some(i) { Color32::from_rgb(0x26, 0x26, 0x26) } else { SEL_BORDER }), StrokeKind::Inside);
        } else if hov {
            p.rect(inner, CornerRadius::ZERO, HOVER_FILL, Stroke::new(1.0_f32, HOVER_FILL), StrokeKind::Inside);
        } else if show_focus && focus == Some(i) {
            p.rect_stroke(inner, CornerRadius::ZERO, Stroke::new(1.0_f32, Color32::from_rgb(0x26, 0x26, 0x26)), StrokeKind::Inside);
        }
        let mut x = rect.left() + 4.0;
        if let Some(icon) = t.icon {
            let ir = Rect::from_min_size(pos2(x, rect.center().y - 16.0), vec2(32.0, 32.0));
            crate::icons::draw(p, ir, icon);
            x += 32.0 + 6.0;
        } else {
            x += 1.0;
        }
        match t.desc {
            Some(d) if !d.is_empty() => {
                let top = rect.center().y - LINE;
                p.text(pos2(x, top), Align2::LEFT_TOP, t.name, font(BASE), TEXT);
                p.with_clip_rect(rect).text(pos2(x, top + LINE), Align2::LEFT_TOP, d, font(BASE), GRAY_TEXT);
            }
            _ => {
                p.text(pos2(x, rect.center().y), Align2::LEFT_CENTER, t.name, font(BASE), TEXT);
            }
        }
        if resp.double_clicked() {
            out.double_clicked = Some(i);
        } else if resp.clicked() {
            out.clicked = Some(i);
        }
        if resp.secondary_clicked() {
            let pos = resp.interact_pointer_pos().unwrap_or(rect.center());
            out.right_clicked = Some((i, pos));
        }
    }
    out
}

/// Popup menu item.
#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub enabled: bool,
    pub checked: bool,
    pub radio: bool,
    pub separator: bool,
    pub submenu: Vec<MenuItem>,
    pub action: u32,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, action: u32) -> Self {
        Self { label: label.into(), enabled: true, checked: false, radio: false, separator: false, submenu: vec![], action }
    }
    pub fn header(label: impl Into<String>) -> Self {
        Self { enabled: false, ..Self::new(label, 0) }
    }
    pub fn separator() -> Self {
        Self { separator: true, enabled: false, ..Self::new("", 0) }
    }
    pub fn checked(mut self, c: bool) -> Self {
        self.checked = c;
        self
    }
    pub fn radio(mut self) -> Self {
        self.radio = true;
        self
    }
}

/// A Windows-style context menu that stays open until an item is chosen or
/// the user clicks elsewhere. Returns Some(action) when chosen, Some(0) when dismissed.
pub struct PopupMenu {
    pub pos: Pos2,
    pub items: Vec<MenuItem>,
    pub open_sub: Option<usize>,
    opened_frame: u64,
}

impl PopupMenu {
    pub fn new(ctx: &egui::Context, pos: Pos2, items: Vec<MenuItem>) -> Self {
        Self { pos, items, open_sub: None, opened_frame: ctx.cumulative_pass_nr() }
    }

    fn draw_items(ui: &mut Ui, items: &[MenuItem], open_sub: &mut Option<usize>, chosen: &mut Option<u32>) -> Option<(usize, Pos2)> {
        let width = items
            .iter()
            .map(|i| text_width(ui, &i.label.replace('&', ""), BASE))
            .fold(120.0_f32, f32::max)
            + 70.0;
        let mut sub_anchor = None;
        for (idx, item) in items.iter().enumerate() {
            if item.separator {
                let (r, _) = ui.allocate_exact_size(vec2(width, 9.0), Sense::hover());
                ui.painter().line_segment([pos2(r.left() + 4.0, r.center().y), pos2(r.right() - 4.0, r.center().y)], Stroke::new(1.0_f32, Color32::from_rgb(0xD7, 0xD7, 0xD7)));
                continue;
            }
            let (r, resp) = ui.allocate_exact_size(vec2(width, 28.0), if item.enabled { Sense::click() } else { Sense::hover() });
            let p = ui.painter();
            if item.enabled && (resp.hovered() || *open_sub == Some(idx)) {
                p.rect_filled(r.shrink2(vec2(4.0, 1.0)), CornerRadius::same(4), Color32::from_rgb(0xF0, 0xF0, 0xF0));
            }
            if item.checked {
                let c = pos2(r.left() + 20.0, r.center().y);
                if item.radio {
                    p.circle_filled(c, 3.0, TEXT);
                } else {
                    p.add(egui::Shape::line(vec![pos2(c.x - 4.5, c.y), pos2(c.x - 1.5, c.y + 3.0), pos2(c.x + 4.5, c.y - 3.5)], Stroke::new(1.2_f32, TEXT)));
                }
            }
            let (txt, _) = split_mnemonic(&item.label);
            p.text(pos2(r.left() + 36.0, r.center().y), Align2::LEFT_CENTER, txt, font(BASE), if item.enabled { TEXT } else { DISABLED_TEXT });
            if !item.submenu.is_empty() {
                let a = pos2(r.right() - 14.0, r.center().y);
                p.add(egui::Shape::line(vec![pos2(a.x - 2.0, a.y - 4.0), pos2(a.x + 2.0, a.y), pos2(a.x - 2.0, a.y + 4.0)], Stroke::new(1.0_f32, TEXT)));
                if resp.hovered() || resp.clicked() {
                    *open_sub = Some(idx);
                }
                if *open_sub == Some(idx) {
                    sub_anchor = Some((idx, r.right_top()));
                }
            } else if resp.hovered() && item.enabled {
                *open_sub = None;
            }
            if resp.clicked() && item.submenu.is_empty() {
                *chosen = Some(item.action);
            }
        }
        sub_anchor
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<u32> {
        let mut chosen = None;
        let frame = egui::Frame::NONE
            .fill(Color32::from_rgb(0xF9, 0xF9, 0xF9))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0xD0, 0xD0, 0xD0)))
            .corner_radius(CornerRadius::same(7))
            .shadow(egui::epaint::Shadow { offset: [2, 4], blur: 10, spread: 0, color: Color32::from_black_alpha(45) })
            .inner_margin(egui::Margin::symmetric(0, 4));
        let items = self.items.clone();
        let mut open_sub = self.open_sub;
        let mut rects = Vec::new();
        let area = egui::Area::new(egui::Id::new("popup-menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(self.pos)
            .constrain(true)
            .show(ctx, |ui| frame.show(ui, |ui| Self::draw_items(ui, &items, &mut open_sub, &mut chosen)).inner);
        rects.push(area.response.rect);
        if let Some((idx, anchor)) = area.inner {
            let sub = items[idx].submenu.clone();
            let mut dummy = None;
            let a2 = egui::Area::new(egui::Id::new("popup-submenu"))
                .order(egui::Order::Foreground)
                .fixed_pos(anchor + vec2(-2.0, -4.0))
                .constrain(true)
                .show(ctx, |ui| frame.show(ui, |ui| Self::draw_items(ui, &sub, &mut dummy, &mut chosen)));
            rects.push(a2.response.rect);
        }
        self.open_sub = open_sub;
        if chosen.is_some() {
            return chosen;
        }
        let just_opened = ctx.cumulative_pass_nr() <= self.opened_frame + 1;
        let clicked_outside = ctx.input(|i| {
            (i.pointer.any_pressed())
                && i.pointer.interact_pos().map(|p| !rects.iter().any(|r| r.contains(p))).unwrap_or(true)
        });
        let esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if (!just_opened && clicked_outside) || esc {
            return Some(0);
        }
        None
    }
}

/// A simple modal message box ("OK", or "Yes"/"No").
pub struct MessageBox {
    pub title: String,
    pub text: String,
    pub yes_no: bool,
    pub icon: MsgIcon,
    pub tag: u32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MsgIcon {
    None,
    Info,
    Warning,
}

#[derive(PartialEq)]
pub enum MsgResult {
    Ok,
    Yes,
    No,
}

impl MessageBox {
    pub fn new(title: &str, text: &str, icon: MsgIcon) -> Self {
        Self { title: title.into(), text: text.into(), yes_no: false, icon, tag: 0 }
    }
    pub fn yes_no(mut self, tag: u32) -> Self {
        self.yes_no = true;
        self.tag = tag;
        self
    }

    pub fn show(&self, ctx: &egui::Context) -> Option<MsgResult> {
        let mut result = None;
        let modal = egui::Modal::new(egui::Id::new(("msgbox", &self.title, &self.text)))
            .backdrop_color(Color32::from_black_alpha(30))
            .frame(
                egui::Frame::NONE
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0xB0, 0xB0, 0xB0)))
                    .corner_radius(CornerRadius::same(6))
                    .shadow(egui::epaint::Shadow { offset: [2, 6], blur: 16, spread: 0, color: Color32::from_black_alpha(60) }),
            );
        let r = modal.show(ctx, |ui| {
            let max_w = (ctx.screen_rect().width() - 40.0).clamp(220.0, 420.0);
            let galley = ui.fonts(|f| f.layout(self.text.clone(), font(BASE), TEXT, max_w - 70.0));
            let icon_w = if self.icon == MsgIcon::None { 0.0 } else { 42.0 };
            let body_h = galley.size().y.max(32.0) + 40.0;
            let w = (galley.size().x + icon_w + 50.0).max(240.0);
            let (rect, _) = ui.allocate_exact_size(vec2(w, body_h + 42.0), Sense::hover());
            let p = ui.painter();
            p.text(rect.left_top() + vec2(12.0, 8.0), Align2::LEFT_TOP, &self.title, font(BASE), GRAY_TEXT);
            let body = Rect::from_min_size(rect.left_top() + vec2(0.0, 26.0), vec2(w, body_h - 10.0));
            if self.icon != MsgIcon::None {
                let c = pos2(body.left() + 30.0, body.top() + 22.0);
                crate::icons::draw(p, Rect::from_center_size(c, vec2(32.0, 32.0)), if self.icon == MsgIcon::Info { crate::icons::Icon::Info } else { crate::icons::Icon::Warning });
            }
            p.galley(pos2(body.left() + 18.0 + icon_w, body.top() + 14.0), galley, TEXT);
            let footer = Rect::from_min_max(pos2(rect.left(), rect.bottom() - 46.0), rect.max);
            p.rect_filled(footer, CornerRadius { nw: 0, ne: 0, sw: 6, se: 6 }, FACE);
            let bw = 75.0;
            if self.yes_no {
                let no = Rect::from_min_size(pos2(footer.right() - 12.0 - bw, footer.top() + 11.0), vec2(bw, 23.0));
                let yes = no.translate(vec2(-(bw + 8.0), 0.0));
                if button(ui, yes, "&Yes", true, true).clicked() {
                    result = Some(MsgResult::Yes);
                }
                if button(ui, no, "&No", false, true).clicked() {
                    result = Some(MsgResult::No);
                }
            } else {
                let ok = Rect::from_min_size(pos2(footer.right() - 12.0 - bw, footer.top() + 11.0), vec2(bw, 23.0));
                if button(ui, ok, "OK", true, true).clicked() {
                    result = Some(MsgResult::Ok);
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                result = Some(if self.yes_no { MsgResult::Yes } else { MsgResult::Ok });
            }
        });
        if result.is_none() && r.should_close() {
            result = Some(if self.yes_no { MsgResult::No } else { MsgResult::Ok });
        }
        result
    }
}
