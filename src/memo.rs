use gtk::cairo::{Context, FontSlant, FontWeight};

/// World-space dimensions of every memo card.
pub const MEMO_W: f64 = 180.0;
pub const MEMO_H: f64 = 150.0;

/// Corner radius (world units).
const RADIUS: f64 = 3.0;

/// A sticky-note memo anchored at world-space (x, y) — top-left corner.
#[derive(Clone)]
pub struct Memo {
    pub x:     f64,
    pub y:     f64,
    pub text:  String,
    pub color: MemoColor,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MemoColor { Yellow, Pink, Blue, Green, Lavender }

impl MemoColor {
    /// Returns (paper, dark_edge, line, text, fold_dark, fold_light)
    pub fn palette(self) -> Palette {
        match self {
            MemoColor::Yellow  => Palette {
                paper:       (0.98, 0.94, 0.62),
                dark_edge:   (0.76, 0.70, 0.20),
                line:        (0.84, 0.80, 0.42),
                text:        (0.20, 0.14, 0.02),
                fold_dark:   (0.72, 0.67, 0.20),
                fold_light:  (0.99, 0.97, 0.80),
            },
            MemoColor::Pink    => Palette {
                paper:       (0.99, 0.80, 0.84),
                dark_edge:   (0.84, 0.46, 0.58),
                line:        (0.92, 0.68, 0.74),
                text:        (0.28, 0.06, 0.10),
                fold_dark:   (0.80, 0.44, 0.56),
                fold_light:  (1.00, 0.92, 0.94),
            },
            MemoColor::Blue    => Palette {
                paper:       (0.72, 0.88, 0.98),
                dark_edge:   (0.30, 0.58, 0.84),
                line:        (0.56, 0.76, 0.90),
                text:        (0.04, 0.12, 0.26),
                fold_dark:   (0.28, 0.56, 0.80),
                fold_light:  (0.88, 0.95, 1.00),
            },
            MemoColor::Green   => Palette {
                paper:       (0.78, 0.96, 0.76),
                dark_edge:   (0.34, 0.72, 0.34),
                line:        (0.60, 0.86, 0.60),
                text:        (0.04, 0.18, 0.04),
                fold_dark:   (0.32, 0.68, 0.32),
                fold_light:  (0.90, 0.99, 0.90),
            },
            MemoColor::Lavender => Palette {
                paper:       (0.88, 0.82, 0.98),
                dark_edge:   (0.58, 0.44, 0.84),
                line:        (0.76, 0.68, 0.92),
                text:        (0.14, 0.06, 0.28),
                fold_dark:   (0.56, 0.42, 0.80),
                fold_light:  (0.96, 0.94, 1.00),
            },
        }
    }

    pub fn cycle(self) -> MemoColor {
        match self {
            MemoColor::Yellow   => MemoColor::Pink,
            MemoColor::Pink     => MemoColor::Blue,
            MemoColor::Blue     => MemoColor::Green,
            MemoColor::Green    => MemoColor::Lavender,
            MemoColor::Lavender => MemoColor::Yellow,
        }
    }
}

pub struct Palette {
    pub paper:      (f64, f64, f64),
    pub dark_edge:  (f64, f64, f64),
    pub line:       (f64, f64, f64),
    pub text:       (f64, f64, f64),
    pub fold_dark:  (f64, f64, f64),
    pub fold_light: (f64, f64, f64),
}

impl Memo {
    pub fn new(x: f64, y: f64, color: MemoColor) -> Self {
        Self { x, y, text: String::new(), color }
    }

    /// Returns true if world-space point (px, py) is inside this memo.
    pub fn hit(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + MEMO_W &&
        py >= self.y && py <= self.y + MEMO_H
    }
}

/// Draw all memos onto `cr` (world-space transform already applied).
pub fn draw_memos(cr: &Context, memos: &[Memo]) {
    for memo in memos { draw_memo(cr, memo); }
}

// ── Geometry helpers ────────────────────────────────────────────────────────

fn rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    use std::f64::consts::{FRAC_PI_2, PI};
    cr.move_to(x + r, y);
    cr.line_to(x + w - r, y);
    cr.arc(x+w-r, y+r,   r, -FRAC_PI_2, 0.0);
    cr.line_to(x+w, y+h-r);
    cr.arc(x+w-r, y+h-r, r,  0.0,  FRAC_PI_2);
    cr.line_to(x+r, y+h);
    cr.arc(x+r,   y+h-r, r,  FRAC_PI_2,  PI);
    cr.line_to(x, y+r);
    cr.arc(x+r,   y+r,   r,  PI, 3.0*FRAC_PI_2);
    cr.close_path();
}

// ── Main draw routine ────────────────────────────────────────────────────────

const FOLD:       f64 = 18.0;   // size of the dog-ear fold triangle
const TOP_PAD:    f64 = 14.0;   // space above first ruled line
const LINE_STEP:  f64 = 16.0;   // ruled-line spacing
const SIDE_PAD:   f64 = 12.0;
const FONT_SIZE:  f64 = 11.5;
const FIRST_LINE: f64 = TOP_PAD + LINE_STEP; // y of first text baseline rel to card top

fn draw_memo(cr: &Context, memo: &Memo) {
    let pal = memo.color.palette();
    let (x, y) = (memo.x, memo.y);
    let (w, h) = (MEMO_W, MEMO_H);

    // ── 1. Layered soft shadow ───────────────────────────────────────────
    for (dx, dy, alpha) in [(5.0_f64, 7.0_f64, 0.07_f64),
                             (3.0,    4.5,  0.09),
                             (1.5,    2.0,  0.10)] {
        cr.set_source_rgba(0.0, 0.0, 0.0, alpha);
        rounded_rect(cr, x+dx, y+dy, w, h, RADIUS);
        let _ = cr.fill();
    }

    // ── 2. Card body (clip to it for all subsequent drawing) ────────────
    rounded_rect(cr, x, y, w, h, RADIUS);
    let _ = cr.fill_preserve(); // will be overwritten, just sets path
    cr.set_source_rgb(pal.paper.0, pal.paper.1, pal.paper.2);
    rounded_rect(cr, x, y, w, h, RADIUS);
    let _ = cr.fill();

    // ── 3. Ruled lines ───────────────────────────────────────────────────
    cr.set_source_rgba(pal.line.0, pal.line.1, pal.line.2, 0.70);
    cr.set_line_width(0.6);
    let lines_start = y + FIRST_LINE + LINE_STEP * 0.3; // first visible line
    let mut ly = lines_start;
    while ly < y + h - 6.0 {
        cr.move_to(x + SIDE_PAD, ly);
        cr.line_to(x + w - SIDE_PAD, ly);
        let _ = cr.stroke();
        ly += LINE_STEP;
    }

    // ── 4. Left margin rule (red-ish, like real paper) ───────────────────
    let margin_x = x + SIDE_PAD + 14.0;
    cr.set_source_rgba(
        pal.dark_edge.0, pal.dark_edge.1, pal.dark_edge.2, 0.30);
    cr.set_line_width(0.7);
    cr.move_to(margin_x, y + TOP_PAD);
    cr.line_to(margin_x, y + h - 6.0);
    let _ = cr.stroke();

    // ── 5. Top strip (subtle tint above first line) ──────────────────────
    // A very lightly darkened band at top gives a header feel without a block.
    cr.set_source_rgba(pal.dark_edge.0, pal.dark_edge.1, pal.dark_edge.2, 0.12);
    cr.rectangle(x, y, w, TOP_PAD + LINE_STEP * 0.5);
    let _ = cr.fill();

    // ── 6. Dog-ear fold (top-right corner) ──────────────────────────────
    // Cover the corner with paper colour first so rounded corner is clean.
    cr.set_source_rgb(pal.paper.0, pal.paper.1, pal.paper.2);
    cr.move_to(x + w - FOLD, y);
    cr.line_to(x + w, y);
    cr.line_to(x + w, y + FOLD);
    cr.close_path();
    let _ = cr.fill();

    // Fold triangle (back face = darker).
    cr.set_source_rgb(pal.fold_dark.0, pal.fold_dark.1, pal.fold_dark.2);
    cr.move_to(x + w - FOLD, y);
    cr.line_to(x + w, y + FOLD);
    cr.line_to(x + w - FOLD, y + FOLD);
    cr.close_path();
    let _ = cr.fill();

    // Fold highlight crease.
    cr.set_source_rgba(pal.fold_light.0, pal.fold_light.1, pal.fold_light.2, 0.6);
    cr.set_line_width(0.8);
    cr.move_to(x + w - FOLD, y);
    cr.line_to(x + w, y + FOLD);
    let _ = cr.stroke();

    // Fold shadow cast on card.
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.10);
    cr.set_line_width(0.5);
    cr.move_to(x + w - FOLD - 1.0, y + 1.0);
    cr.line_to(x + w - 1.0, y + FOLD + 1.0);
    let _ = cr.stroke();

    // ── 7. Outer border ──────────────────────────────────────────────────
    cr.set_source_rgba(pal.dark_edge.0, pal.dark_edge.1, pal.dark_edge.2, 0.28);
    cr.set_line_width(0.8);
    rounded_rect(cr, x, y, w, h, RADIUS);
    let _ = cr.stroke();

    // ── 8. Body text ─────────────────────────────────────────────────────
    if !memo.text.is_empty() {
        cr.set_source_rgba(pal.text.0, pal.text.1, pal.text.2, 0.88);
        cr.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
        cr.set_font_size(FONT_SIZE);

        let text_left = margin_x + 5.0;
        let max_w = w - (text_left - x) - SIDE_PAD - FOLD * 0.4;
        let line_h = LINE_STEP;
        let mut ty = y + FIRST_LINE + LINE_STEP * 0.9; // sit on first ruled line

        for line in wrap_text(cr, &memo.text, max_w) {
            if ty > y + h - 6.0 { break; }
            cr.move_to(text_left, ty);
            let _ = cr.show_text(&line);
            ty += line_h;
        }
    } else {
        // Placeholder hint.
        cr.set_source_rgba(pal.dark_edge.0, pal.dark_edge.1, pal.dark_edge.2, 0.30);
        cr.select_font_face("Sans", FontSlant::Italic, FontWeight::Normal);
        cr.set_font_size(FONT_SIZE - 1.0);
        let text_left = margin_x + 5.0;
        cr.move_to(text_left, y + FIRST_LINE + LINE_STEP * 0.9);
        let _ = cr.show_text("click to edit…");
    }

    // ── 9. Tiny pin at top-centre ─────────────────────────────────────────
    draw_pin(cr, x + w / 2.0, y, &pal);
}

fn draw_pin(cr: &Context, cx: f64, top_y: f64, pal: &Palette) {
    let pin_y = top_y - 1.5;
    // Pin head.
    cr.set_source_rgba(pal.dark_edge.0 * 0.7, pal.dark_edge.1 * 0.7, pal.dark_edge.2 * 0.7, 0.85);
    cr.arc(cx, pin_y, 3.5, 0.0, std::f64::consts::TAU);
    let _ = cr.fill();
    // Shine on pin head.
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.45);
    cr.arc(cx - 1.0, pin_y - 1.0, 1.2, 0.0, std::f64::consts::TAU);
    let _ = cr.fill();
    // Pin needle.
    cr.set_source_rgba(0.4, 0.4, 0.4, 0.6);
    cr.set_line_width(0.8);
    cr.move_to(cx, pin_y + 3.5);
    cr.line_to(cx, pin_y + 9.0);
    let _ = cr.stroke();
}

// ── Word-wrap ────────────────────────────────────────────────────────────────

fn wrap_text(cr: &Context, text: &str, max_w: f64) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in text.split('\n') {
        let mut current = String::new();
        for word in raw_line.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{current} {word}")
            };
            let ext = cr.text_extents(&candidate).unwrap();
            if ext.width() > max_w && !current.is_empty() {
                lines.push(current.clone());
                current = word.to_string();
            } else {
                current = candidate;
            }
        }
        lines.push(current);
    }
    lines
}