//! Simple vector icons standing in for the Windows system icons the original
//! borrowed from imageres.dll / notepad.exe / spy.ico.

use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Painter, Rect, Shape, Stroke, StrokeKind};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Icon {
    NewScript,
    Compile,
    Help,
    Spy,
    Launch,
    Editor,
    Folder,
    Defaults,
    Info,
    Warning,
    /// A generic program icon, tinted by a seed (editor list entries).
    App(u32),
    /// An interpreter (version picker).
    Interpreter,
}

fn s(r: Rect, x: f32, y: f32) -> egui::Pos2 {
    pos2(r.left() + r.width() * x, r.top() + r.height() * y)
}

fn page(p: &Painter, r: Rect, fill: Color32, edge: Color32) -> Rect {
    let body = Rect::from_min_max(s(r, 0.18, 0.06), s(r, 0.82, 0.94));
    let fold = body.width() * 0.28;
    let pts = vec![
        body.left_top(),
        pos2(body.right() - fold, body.top()),
        pos2(body.right(), body.top() + fold),
        body.right_bottom(),
        body.left_bottom(),
    ];
    p.add(Shape::convex_polygon(pts.clone(), fill, Stroke::NONE));
    p.add(Shape::closed_line(pts, Stroke::new(1.0_f32, edge)));
    p.add(Shape::convex_polygon(
        vec![pos2(body.right() - fold, body.top()), pos2(body.right(), body.top() + fold), pos2(body.right() - fold, body.top() + fold)],
        Color32::from_rgb(0xDD, 0xE3, 0xEC),
        Stroke::new(1.0_f32, edge),
    ));
    body
}

pub fn draw(p: &Painter, r: Rect, icon: Icon) {
    match icon {
        Icon::NewScript => {
            let body = page(p, r, Color32::WHITE, Color32::from_rgb(0x7A, 0x8B, 0xA6));
            let c = pos2(body.center().x - 1.0, body.center().y + 2.0);
            let k = r.width() / 32.0;
            let col = Color32::from_rgb(0x4B, 0x5F, 0xB5);
            p.line_segment([pos2(c.x - 6.0 * k, c.y), pos2(c.x + 6.0 * k, c.y)], Stroke::new(2.6 * k, col));
            p.line_segment([pos2(c.x, c.y - 6.0 * k), pos2(c.x, c.y + 6.0 * k)], Stroke::new(2.6 * k, col));
        }
        Icon::Compile => {
            let k = r.width() / 32.0;
            let blocks = [
                (0.10, 0.48, 0.42, 0.42, Color32::from_rgb(0xD8, 0x3B, 0x2D)),
                (0.46, 0.52, 0.42, 0.40, Color32::from_rgb(0x2E, 0x6F, 0xD9)),
                (0.30, 0.16, 0.40, 0.38, Color32::from_rgb(0x39, 0xA0, 0x3F)),
                (0.56, 0.08, 0.32, 0.30, Color32::from_rgb(0xF2, 0xB7, 0x1B)),
            ];
            for (x, y, w, h, c) in blocks {
                let b = Rect::from_min_size(s(r, x, y), vec2(r.width() * w, r.height() * h));
                p.rect(b, CornerRadius::same(1), c, Stroke::new(1.0 * k, c.gamma_multiply(0.6)), StrokeKind::Inside);
                p.line_segment([b.left_top() + vec2(2.0 * k, 2.0 * k), pos2(b.right() - 2.0 * k, b.top() + 2.0 * k)], Stroke::new(1.0 * k, Color32::from_white_alpha(110)));
            }
        }
        Icon::Help => {
            let c = r.center();
            let rad = r.width() * 0.44;
            p.circle(c, rad, Color32::from_rgb(0x1E, 0x7B, 0xD6), Stroke::new(1.0_f32, Color32::from_rgb(0x15, 0x5E, 0xA8)));
            p.text(pos2(c.x, c.y + r.height() * 0.02), Align2::CENTER_CENTER, "?", FontId::new(r.height() * 0.62, FontFamily::Proportional), Color32::WHITE);
        }
        Icon::Info => {
            let c = r.center();
            p.circle_filled(c, r.width() * 0.45, Color32::from_rgb(0x1E, 0x7B, 0xD6));
            p.text(c, Align2::CENTER_CENTER, "i", FontId::new(r.height() * 0.7, FontFamily::Proportional), Color32::WHITE);
        }
        Icon::Warning => {
            let pts = vec![s(r, 0.5, 0.08), s(r, 0.95, 0.9), s(r, 0.05, 0.9)];
            p.add(Shape::convex_polygon(pts, Color32::from_rgb(0xF8, 0xC5, 0x1C), Stroke::new(1.0_f32, Color32::from_rgb(0xB8, 0x8A, 0x00))));
            p.text(s(r, 0.5, 0.58), Align2::CENTER_CENTER, "!", FontId::new(r.height() * 0.55, FontFamily::Proportional), Color32::BLACK);
        }
        Icon::Spy => {
            let k = r.width() / 32.0;
            let back = Rect::from_min_max(s(r, 0.18, 0.08), s(r, 0.96, 0.62));
            p.rect(back, CornerRadius::same(1), Color32::from_rgb(0xF4, 0xF6, 0xF9), Stroke::new(1.0 * k, Color32::from_rgb(0x55, 0x5E, 0x6B)), StrokeKind::Inside);
            p.rect_filled(Rect::from_min_size(back.min, vec2(back.width(), 4.0 * k)), CornerRadius::ZERO, Color32::from_rgb(0x55, 0x8B, 0xD3));
            let front = Rect::from_min_max(s(r, 0.04, 0.30), s(r, 0.80, 0.92));
            p.rect(front, CornerRadius::same(1), Color32::WHITE, Stroke::new(1.0 * k, Color32::from_rgb(0x55, 0x5E, 0x6B)), StrokeKind::Inside);
            p.rect_filled(Rect::from_min_size(front.min, vec2(front.width(), 4.0 * k)), CornerRadius::ZERO, Color32::from_rgb(0x55, 0x8B, 0xD3));
            // magnifier
            let c = pos2(front.center().x, front.center().y + 2.0 * k);
            p.circle(c, 5.5 * k, Color32::from_rgb(0xD5, 0xEA, 0xFF), Stroke::new(1.6 * k, Color32::from_rgb(0x30, 0x30, 0x30)));
            p.line_segment([c + vec2(4.0 * k, 4.0 * k), c + vec2(8.5 * k, 8.0 * k)], Stroke::new(2.4 * k, Color32::from_rgb(0x30, 0x30, 0x30)));
        }
        Icon::Launch => {
            let k = r.width() / 32.0;
            let frame = Rect::from_min_max(s(r, 0.04, 0.18), s(r, 0.96, 0.84));
            p.rect(frame, CornerRadius::same(1), Color32::WHITE, Stroke::new(1.2 * k, Color32::from_rgb(0x6B, 0x6B, 0x6B)), StrokeKind::Inside);
            let y = frame.center().y;
            let g = Color32::from_rgb(0x2E, 0x9E, 0x3A);
            p.line_segment([pos2(frame.left() + 6.0 * k, y), pos2(frame.right() - 9.0 * k, y)], Stroke::new(2.4 * k, g));
            p.add(Shape::convex_polygon(
                vec![pos2(frame.right() - 5.0 * k, y), pos2(frame.right() - 11.0 * k, y - 6.0 * k), pos2(frame.right() - 11.0 * k, y + 6.0 * k)],
                g,
                Stroke::NONE,
            ));
        }
        Icon::Editor => {
            let k = r.width() / 32.0;
            let body = Rect::from_min_max(s(r, 0.16, 0.14), s(r, 0.86, 0.96));
            p.rect(body, CornerRadius::same(2), Color32::from_rgb(0xEA, 0xF4, 0xFC), Stroke::new(1.0 * k, Color32::from_rgb(0x4A, 0x86, 0xC2)), StrokeKind::Inside);
            p.rect_filled(Rect::from_min_size(body.min, vec2(body.width(), 5.0 * k)), CornerRadius::same(2), Color32::from_rgb(0x3E, 0x9B, 0xE0));
            for i in 0..5 {
                let x = body.left() + (3.5 + i as f32 * 4.2) * k;
                p.circle_stroke(pos2(x, body.top()), 1.6 * k, Stroke::new(1.0 * k, Color32::from_rgb(0x55, 0x55, 0x55)));
            }
            for i in 0..5 {
                let y = body.top() + (9.0 + i as f32 * 3.6) * k;
                p.line_segment([pos2(body.left() + 3.0 * k, y), pos2(body.right() - 3.0 * k, y)], Stroke::new(1.0 * k, Color32::from_rgb(0x8C, 0xB6, 0xDB)));
            }
        }
        Icon::Folder => {
            let k = r.width() / 16.0;
            let back = Rect::from_min_max(s(r, 0.04, 0.14), s(r, 0.96, 0.86));
            p.rect_filled(back, CornerRadius::same(1), Color32::from_rgb(0xE8, 0xB3, 0x3C));
            let front = Rect::from_min_max(s(r, 0.04, 0.32), s(r, 0.96, 0.86));
            p.rect_filled(front, CornerRadius::same(1), Color32::from_rgb(0xF8, 0xD7, 0x75));
            let c = s(r, 0.58, 0.58);
            p.circle(c, 3.0 * k, Color32::from_rgb(0xDC, 0xEE, 0xFF), Stroke::new(1.0 * k, Color32::from_rgb(0x3A, 0x3A, 0x3A)));
            p.line_segment([c + vec2(2.2 * k, 2.2 * k), c + vec2(4.5 * k, 4.5 * k)], Stroke::new(1.4 * k, Color32::from_rgb(0x3A, 0x3A, 0x3A)));
        }
        Icon::Defaults => {
            let k = r.width() / 16.0;
            let c = s(r, 0.42, 0.45);
            let col = Color32::from_rgb(0x6A, 0x73, 0x80);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let d = vec2(a.cos(), a.sin());
                p.line_segment([c + d * 3.0 * k, c + d * 6.0 * k], Stroke::new(2.2 * k, col));
            }
            p.circle(c, 4.2 * k, col, Stroke::NONE);
            p.circle_filled(c, 1.8 * k, Color32::from_rgb(0xFD, 0xFD, 0xFD));
            let b = Rect::from_min_max(s(r, 0.52, 0.56), s(r, 1.0, 1.0));
            p.rect(b, CornerRadius::same(1), Color32::WHITE, Stroke::new(1.0 * k, Color32::from_rgb(0x3E, 0x7C, 0xC8)), StrokeKind::Inside);
            p.add(Shape::line(vec![b.left_center() + vec2(1.5 * k, 0.0), pos2(b.center().x - 0.5 * k, b.bottom() - 1.8 * k), pos2(b.right() - 1.4 * k, b.top() + 1.6 * k)], Stroke::new(1.1 * k, Color32::from_rgb(0x1F, 0x6F, 0xC8))));
        }
        Icon::App(seed) => {
            let palette = [
                Color32::from_rgb(0x1F, 0x8A, 0xD2),
                Color32::from_rgb(0x2E, 0x9E, 0x3A),
                Color32::from_rgb(0xC9, 0x4F, 0x2C),
                Color32::from_rgb(0x8A, 0x4F, 0xC9),
                Color32::from_rgb(0xD9, 0x9A, 0x16),
            ];
            let c = palette[(seed as usize) % palette.len()];
            let body = Rect::from_min_max(s(r, 0.12, 0.12), s(r, 0.88, 0.88));
            p.rect_filled(body, CornerRadius::same(5), c);
            let k = r.width() / 32.0;
            for i in 0..3 {
                let y = body.top() + (7.0 + i as f32 * 5.0) * k;
                let x2 = body.right() - (5.0 + (i % 2) as f32 * 5.0) * k;
                p.line_segment([pos2(body.left() + 5.0 * k, y), pos2(x2, y)], Stroke::new(2.0 * k, Color32::from_white_alpha(220)));
            }
        }
        Icon::Interpreter => {
            let body = Rect::from_min_max(s(r, 0.08, 0.08), s(r, 0.92, 0.92));
            p.rect_filled(body, CornerRadius::same(3), Color32::from_rgb(0x33, 0x8C, 0x3C));
            let k = r.width() / 16.0;
            let key = Rect::from_center_size(body.center(), vec2(8.0 * k, 6.0 * k));
            p.rect(key, CornerRadius::same(1), Color32::WHITE, Stroke::NONE, StrokeKind::Inside);
        }
    }
}

/// Window icon: a green rounded square with a white key cap (neutral placeholder).
pub fn window_icon() -> egui::IconData {
    let n = 64usize;
    let mut rgba = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let (fx, fy) = (x as f32, y as f32);
            let r = 12.0;
            let inside = {
                let cx = fx.clamp(r, n as f32 - 1.0 - r);
                let cy = fy.clamp(r, n as f32 - 1.0 - r);
                (fx - cx).powi(2) + (fy - cy).powi(2) <= r * r
            };
            let i = (y * n + x) * 4;
            if inside {
                let key = (18..46).contains(&x) && (22..42).contains(&y);
                let (cr, cg, cb) = if key { (255, 255, 255) } else { (0x33, 0x8C, 0x3C) };
                rgba[i..i + 4].copy_from_slice(&[cr, cg, cb, 255]);
            }
        }
    }
    egui::IconData { rgba, width: n as u32, height: n as u32 }
}
