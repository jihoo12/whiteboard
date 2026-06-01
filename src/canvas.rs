use std::rc::Rc;
use std::cell::{Cell, RefCell};

use gtk::{cairo, glib, prelude::*, DrawingArea, EventControllerMotion,
          EventControllerScroll, EventControllerScrollFlags, GestureClick,
          Entry, Popover};

use crate::types::{Stroke, draw_smooth_stroke};
use crate::viewport::Viewport;
use crate::memo::{Memo, MemoColor, draw_memos, MEMO_W, MEMO_H};

const RDP_EPSILON: f64 = 1.5;
const LINE_WIDTH:  f64 = 2.5;

// Stroke colour: a soft indigo-white.
const STROKE_R: f64 = 0.72;
const STROKE_G: f64 = 0.78;
const STROKE_B: f64 = 0.98;

// Background colour (matches CSS .canvas-area).
const BG_R: f64 = 0.074;
const BG_G: f64 = 0.074;
const BG_B: f64 = 0.102;

#[derive(Clone, Copy, PartialEq)]
pub enum Tool { Draw, Memo, Erase }

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

struct State {
    viewport:   Viewport,
    strokes:    Vec<Stroke>,
    memos:      Vec<Memo>,
    active:     Option<Stroke>,
    is_drawing: bool,
    pan_origin: Option<(f64, f64)>,
    cursor:     (f64, f64),
    next_color: MemoColor,
    hover_memo: Option<usize>,  // memo index under cursor when in Memo tool
}

impl State {
    fn new() -> Self {
        Self {
            viewport: Viewport::new(), strokes: Vec::new(), memos: Vec::new(),
            active: None, is_drawing: false, pan_origin: None,
            cursor: (0.0, 0.0), next_color: MemoColor::Yellow, hover_memo: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Cache
// ---------------------------------------------------------------------------

fn rebuild_cache(state: &State, w: i32, h: i32) -> cairo::ImageSurface {
    let surf = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h)
        .expect("cache surface");
    let cr = cairo::Context::new(&surf).expect("cache cr");

    // Background.
    cr.set_source_rgb(BG_R, BG_G, BG_B);
    cr.paint().unwrap();

    // Subtle dot-grid.
    draw_grid(&cr, state, w, h);

    // Strokes.
    cr.set_source_rgb(STROKE_R, STROKE_G, STROKE_B);
    cr.set_line_width(LINE_WIDTH / state.viewport.zoom);
    cr.set_line_cap(cairo::LineCap::Round);
    cr.set_line_join(cairo::LineJoin::Round);
    state.viewport.apply(&cr);
    for stroke in &state.strokes { draw_smooth_stroke(&cr, &stroke.points); }

    // Memos (world-space transform already applied).
    cr.identity_matrix();
    state.viewport.apply(&cr);
    draw_memos(&cr, &state.memos);

    surf
}

fn draw_grid(cr: &cairo::Context, state: &State, w: i32, h: i32) {
    let zoom = state.viewport.zoom;
    // World-space grid spacing: pick a nice multiple of 10 based on zoom.
    let base_spacing = 40.0_f64;
    let spacing = base_spacing * zoom;
    if spacing < 8.0 { return; }

    let alpha = ((spacing - 8.0) / 40.0).min(1.0) * 0.18;
    cr.set_source_rgba(0.55, 0.55, 0.75, alpha);

    let ox = state.viewport.pan_x % spacing;
    let oy = state.viewport.pan_y % spacing;

    let dot_r = 0.9_f64;
    let mut x = ox;
    while x < w as f64 {
        let mut y = oy;
        while y < h as f64 {
            cr.arc(x, y, dot_r, 0.0, std::f64::consts::TAU);
            let _ = cr.fill();
            y += spacing;
        }
        x += spacing;
    }
}

fn invalidate(cache: &Rc<RefCell<Option<cairo::ImageSurface>>>) {
    *cache.borrow_mut() = None;
}

// ---------------------------------------------------------------------------
// Popover text editor for a memo
// ---------------------------------------------------------------------------

fn show_memo_editor(
    canvas: &DrawingArea,
    memo_idx: usize,
    state: &Rc<RefCell<State>>,
    cache: &Rc<RefCell<Option<cairo::ImageSurface>>>,
) {
    let s = state.borrow();
    let memo = &s.memos[memo_idx];
    let (sx, sy) = s.viewport.to_screen(memo.x + MEMO_W/2.0, memo.y + MEMO_H/2.0);
    drop(s);

    let entry = Entry::new();
    {
        let text = state.borrow().memos[memo_idx].text.clone();
        entry.set_text(&text);
        entry.set_width_chars(22);
    }

    let popover = Popover::new();
    popover.set_child(Some(&entry));
    popover.set_parent(canvas);

    let rect = gtk::gdk::Rectangle::new(sx as i32, sy as i32, 1, 1);
    popover.set_pointing_to(Some(&rect));
    popover.set_position(gtk::PositionType::Bottom);

    {
        let state = state.clone();
        let cache = cache.clone();
        let canvas_c = canvas.clone();
        let entry_c = entry.clone();
        let popover_c = popover.clone();
        entry.connect_activate(move |_| {
            state.borrow_mut().memos[memo_idx].text = entry_c.text().to_string();
            invalidate(&cache);
            canvas_c.queue_draw();
            popover_c.popdown();
        });
    }
    {
        let state = state.clone();
        let cache = cache.clone();
        let canvas_c = canvas.clone();
        let entry_c = entry.clone();
        popover.connect_closed(move |_| {
            state.borrow_mut().memos[memo_idx].text = entry_c.text().to_string();
            invalidate(&cache);
            canvas_c.queue_draw();
        });
    }

    popover.popup();
    entry.grab_focus();
}

// ---------------------------------------------------------------------------
// Eraser: remove strokes/memos near a screen point
// ---------------------------------------------------------------------------

fn erase_at(state: &mut State, sx: f64, sy: f64) -> bool {
    let wp = state.viewport.to_world(sx, sy);
    let radius = 20.0 / state.viewport.zoom;
    let r2 = radius * radius;

    // Erase memos first (hit-test by bounding box).
    let before = state.memos.len();
    state.memos.retain(|m| !m.hit(wp.x, wp.y));
    if state.memos.len() < before { return true; }

    // Erase strokes whose any point is within radius.
    let before = state.strokes.len();
    state.strokes.retain(|s| {
        !s.points.iter().any(|p| {
            (p.x - wp.x).powi(2) + (p.y - wp.y).powi(2) < r2
        })
    });
    state.strokes.len() < before
}

// ---------------------------------------------------------------------------
// Public
// ---------------------------------------------------------------------------

pub fn setup_canvas(canvas: &DrawingArea, tool: Rc<Cell<Tool>>) {
    canvas.set_hexpand(true);
    canvas.set_vexpand(true);
    canvas.set_focusable(true);

    let state: Rc<RefCell<State>> = Rc::new(RefCell::new(State::new()));
    let cache: Rc<RefCell<Option<cairo::ImageSurface>>> = Rc::new(RefCell::new(None));

    // ----- Draw func -----
    {
        let state = state.clone();
        let cache = cache.clone();
        canvas.set_draw_func(move |area, cr, w, h| {
            { let mut c = cache.borrow_mut();
              if c.is_none() { *c = Some(rebuild_cache(&state.borrow(), w, h)); } }

            if let Some(surf) = cache.borrow().as_ref() {
                cr.set_source_surface(surf, 0.0, 0.0).unwrap();
                cr.paint().unwrap();
            }

            let s = state.borrow();

            // Live stroke while drawing.
            if s.is_drawing {
                if let Some(stroke) = &s.active {
                    cr.set_source_rgb(STROKE_R, STROKE_G, STROKE_B);
                    cr.set_line_width(LINE_WIDTH / s.viewport.zoom);
                    cr.set_line_cap(cairo::LineCap::Round);
                    cr.set_line_join(cairo::LineJoin::Round);
                    s.viewport.apply(cr);
                    draw_smooth_stroke(cr, &stroke.points);
                }
            }

            // Hover highlight over memo.
            if let Some(idx) = s.hover_memo {
                if let Some(m) = s.memos.get(idx) {
                    use crate::memo::{MEMO_W, MEMO_H};
                    let (sx, sy) = s.viewport.to_screen(m.x, m.y);
                    let sw = MEMO_W * s.viewport.zoom;
                    let sh = MEMO_H * s.viewport.zoom;
                    cr.identity_matrix();
                    cr.set_source_rgba(1.0, 1.0, 1.0, 0.18);
                    cr.set_line_width(2.0);
                    // Simple rounded rect at screen coords.
                    let r = 3.0 * s.viewport.zoom;
                    cr.move_to(sx + r, sy);
                    cr.line_to(sx + sw - r, sy);
                    cr.arc(sx+sw-r, sy+r,   r, -std::f64::consts::FRAC_PI_2, 0.0);
                    cr.line_to(sx+sw, sy+sh-r);
                    cr.arc(sx+sw-r, sy+sh-r, r, 0.0, std::f64::consts::FRAC_PI_2);
                    cr.line_to(sx+r, sy+sh);
                    cr.arc(sx+r, sy+sh-r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
                    cr.line_to(sx, sy+r);
                    cr.arc(sx+r, sy+r, r, std::f64::consts::PI, 3.0*std::f64::consts::FRAC_PI_2);
                    cr.close_path();
                    let _ = cr.stroke();
                }
            }

            // Origin crosshair — very subtle.
            let (ox, oy) = (s.viewport.pan_x, s.viewport.pan_y);
            cr.identity_matrix();
            cr.set_source_rgba(0.6, 0.6, 0.9, 0.06);
            cr.set_line_width(1.0);
            cr.move_to(ox, 0.0); cr.line_to(ox, area.height() as f64);
            cr.move_to(0.0, oy); cr.line_to(area.width() as f64, oy);
            let _ = cr.stroke();
        });
    }

    // ----- Left click -----
    let left = GestureClick::new();
    left.set_button(1);
    {
        let state = state.clone();
        let cache = cache.clone();
        let canvas_c = canvas.clone();
        let tool = tool.clone();
        left.connect_pressed(move |g, _, sx, sy| {
            g.set_state(gtk::EventSequenceState::Claimed);
            match tool.get() {
                Tool::Draw => {
                    let mut s = state.borrow_mut();
                    let wp = s.viewport.to_world(sx, sy);
                    s.active = Some(Stroke::new(wp.x, wp.y));
                    s.is_drawing = true;
                }
                Tool::Memo => {
                    let mut s = state.borrow_mut();
                    let wp = s.viewport.to_world(sx, sy);
                    let hit = s.memos.iter().position(|m| m.hit(wp.x, wp.y));
                    if hit.is_none() {
                        let color = s.next_color;
                        s.next_color = color.cycle();
                        s.memos.push(Memo::new(wp.x - MEMO_W/2.0, wp.y - MEMO_H/2.0, color));
                        invalidate(&cache);
                    }
                    drop(s);
                    let idx = hit.unwrap_or_else(|| state.borrow().memos.len() - 1);
                    show_memo_editor(&canvas_c, idx, &state, &cache);
                }
                Tool::Erase => {
                    let mut s = state.borrow_mut();
                    s.is_drawing = true; // use is_drawing as "erasing" flag
                    if erase_at(&mut s, sx, sy) {
                        invalidate(&cache);
                        canvas_c.queue_draw();
                    }
                }
            }
            canvas_c.queue_draw();
        });
    }
    {
        let state = state.clone();
        let cache = cache.clone();
        let canvas_c = canvas.clone();
        let tool = tool.clone();
        left.connect_released(move |_, _, _, _| {
            let mut s = state.borrow_mut();
            s.is_drawing = false;
            if tool.get() == Tool::Draw {
                if let Some(mut stroke) = s.active.take() {
                    stroke.simplify(RDP_EPSILON);
                    s.strokes.push(stroke);
                }
                drop(s);
                invalidate(&cache);
                canvas_c.queue_draw();
            }
        });
    }
    canvas.add_controller(left);

    // ----- Right click: pan -----
    let right = GestureClick::new();
    right.set_button(3);
    { let state = state.clone();
      right.connect_pressed(move |g, _, x, y| {
          state.borrow_mut().pan_origin = Some((x, y));
          g.set_state(gtk::EventSequenceState::Claimed);
      }); }
    { let state = state.clone();
      right.connect_released(move |_, _, _, _| {
          state.borrow_mut().pan_origin = None;
      }); }
    canvas.add_controller(right);

    // ----- Motion -----
    let motion = EventControllerMotion::new();
    {
        let state = state.clone();
        let cache = cache.clone();
        let canvas_c = canvas.clone();
        let tool = tool.clone();
        motion.connect_motion(move |_, sx, sy| {
            let mut s = state.borrow_mut();
            s.cursor = (sx, sy);

            if let Some((ox, oy)) = s.pan_origin {
                s.viewport.pan(sx - ox, sy - oy);
                s.pan_origin = Some((sx, sy));
                invalidate(&cache);
                canvas_c.queue_draw();
                return;
            }

            // Update hover highlight in Memo mode.
            if tool.get() == Tool::Memo {
                let wp = s.viewport.to_world(sx, sy);
                let new_hover = s.memos.iter().position(|m| m.hit(wp.x, wp.y));
                if new_hover != s.hover_memo {
                    s.hover_memo = new_hover;
                    canvas_c.queue_draw();
                }
            } else if s.hover_memo.is_some() {
                s.hover_memo = None;
                canvas_c.queue_draw();
            }

            if s.is_drawing {
                match tool.get() {
                    Tool::Draw => {
                        let wp = s.viewport.to_world(sx, sy);
                        if let Some(stroke) = s.active.as_mut() { stroke.push(wp.x, wp.y); }
                        canvas_c.queue_draw();
                    }
                    Tool::Erase => {
                        if erase_at(&mut s, sx, sy) {
                            invalidate(&cache);
                            canvas_c.queue_draw();
                        }
                    }
                    _ => {}
                }
            }
        });
    }
    canvas.add_controller(motion);

    // ----- Scroll: zoom -----
    let scroll = EventControllerScroll::new(
        EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::DISCRETE);
    {
        let state = state.clone();
        let cache = cache.clone();
        let canvas_c = canvas.clone();
        scroll.connect_scroll(move |_, _dx, dy| {
            let factor = if dy < 0.0 { 1.15 } else { 1.0 / 1.15 };
            let mut s = state.borrow_mut();
            let (cx, cy) = s.cursor;
            s.viewport.zoom_around(cx, cy, factor);
            drop(s);
            invalidate(&cache);
            canvas_c.queue_draw();
            glib::Propagation::Stop
        });
    }
    canvas.add_controller(scroll);
}