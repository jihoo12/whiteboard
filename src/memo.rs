use gtk::cairo::Context;

/// World-space dimensions of every memo card.
pub const MEMO_W: f64 = 160.0;
pub const MEMO_H: f64 = 120.0;
/// Corner radius (world units).
const RADIUS: f64 = 6.0;

/// A sticky-note memo anchored at world-space (x, y) — top-left corner.
#[derive(Clone)]
pub struct Memo {
    pub x:     f64,
    pub y:     f64,
    pub text:  String,
    pub color: MemoColor,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MemoColor { Yellow, Pink, Blue, Green }

impl MemoColor {
    /// (body_rgb, header_rgb)
    pub fn rgb(self) -> ((f64,f64,f64), (f64,f64,f64)) {
        match self {
            MemoColor::Yellow => ((0.99, 0.96, 0.60), (0.93, 0.84, 0.25)),
            MemoColor::Pink   => ((0.99, 0.75, 0.80), (0.95, 0.53, 0.62)),
            MemoColor::Blue   => ((0.68, 0.88, 0.99), (0.38, 0.68, 0.93)),
            MemoColor::Green  => ((0.76, 0.96, 0.76), (0.45, 0.82, 0.45)),
        }
    }

    pub fn cycle(self) -> MemoColor {
        match self {
            MemoColor::Yellow => MemoColor::Pink,
            MemoColor::Pink   => MemoColor::Blue,
            MemoColor::Blue   => MemoColor::Green,
            MemoColor::Green  => MemoColor::Yellow,
        }
    }
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
    for memo in memos {
        draw_memo(cr, memo);
    }
}

fn rounded_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    cr.move_to(x + r, y);
    cr.line_to(x + w - r, y);
    cr.arc(x+w-r, y+r,   r,  -std::f64::consts::FRAC_PI_2, 0.0);
    cr.line_to(x+w, y+h-r);
    cr.arc(x+w-r, y+h-r, r,  0.0,  std::f64::consts::FRAC_PI_2);
    cr.line_to(x+r, y+h);
    cr.arc(x+r,   y+h-r, r,  std::f64::consts::FRAC_PI_2,  std::f64::consts::PI);
    cr.line_to(x, y+r);
    cr.arc(x+r,   y+r,   r,  std::f64::consts::PI, 3.0*std::f64::consts::FRAC_PI_2);
    cr.close_path();
}

const HEADER_H: f64 = 24.0;
const FONT_SIZE: f64 = 11.0;
const PADDING: f64 = 8.0;

fn draw_memo(cr: &Context, memo: &Memo) {
    let (body, header) = memo.color.rgb();
    let (x, y) = (memo.x, memo.y);

    // Drop shadow.
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.18);
    rounded_rect(cr, x+3.0, y+4.0, MEMO_W, MEMO_H, RADIUS);
    let _ = cr.fill();

    // Card body.
    cr.set_source_rgb(body.0, body.1, body.2);
    rounded_rect(cr, x, y, MEMO_W, MEMO_H, RADIUS);
    let _ = cr.fill();

    // Header band — draw as full rect clipped to top of card then redraw
    // lower edge to merge into body.
    cr.set_source_rgb(header.0, header.1, header.2);
    cr.rectangle(x, y, MEMO_W, HEADER_H);
    let _ = cr.fill();

    // Re-draw top-rounded corners over the header (same header colour).
    cr.set_source_rgb(header.0, header.1, header.2);
    rounded_rect(cr, x, y, MEMO_W, HEADER_H + RADIUS, RADIUS);
    let _ = cr.fill();

    // Bottom of header straight edge (cover the rounded bottom of above).
    cr.rectangle(x, y + HEADER_H, MEMO_W, RADIUS);
    cr.set_source_rgb(body.0, body.1, body.2);
    let _ = cr.fill();

    // Subtle border.
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.10);
    cr.set_line_width(0.8);
    rounded_rect(cr, x, y, MEMO_W, MEMO_H, RADIUS);
    let _ = cr.stroke();

    // Body text.
    if !memo.text.is_empty() {
        cr.set_source_rgba(0.15, 0.10, 0.05, 0.9);
        cr.set_font_size(FONT_SIZE);
        let max_w = MEMO_W - PADDING * 2.0;
        let line_h = FONT_SIZE * 1.4;
        let mut ty = y + HEADER_H + PADDING + FONT_SIZE;
        for word_wrapped in wrap_text(cr, &memo.text, max_w) {
            if ty > y + MEMO_H - PADDING { break; }
            cr.move_to(x + PADDING, ty);
            let _ = cr.show_text(&word_wrapped);
            ty += line_h;
        }
    }
}

/// Naive word-wrap: split on spaces, accumulate words until line overflows.
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