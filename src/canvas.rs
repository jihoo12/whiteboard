use std::cell::Cell;
use std::rc::Rc;
use std::cell::RefCell;

use gtk::{cairo, prelude::*, DrawingArea, EventControllerMotion,
          EventControllerScroll, EventControllerScrollFlags, GestureClick};

use crate::types::{Stroke, draw_smooth_stroke};
use crate::viewport::Viewport;

const RDP_EPSILON: f64 = 1.5;
const LINE_WIDTH:  f64 = 3.0;
const LINE_R:      f64 = 0.0;
const LINE_G:      f64 = 0.8;
const LINE_B:      f64 = 0.2;

// ---------------------------------------------------------------------------
// Shared state
// ---------------------------------------------------------------------------

struct State {
    viewport:   Viewport,
    strokes:    Vec<Stroke>,
    active:     Option<Stroke>,
    is_drawing: bool,
    pan_origin: Option<(f64, f64)>,
    /// Last known screen-space pointer position (for zoom anchor).
    cursor:     (f64, f64),
}

impl State {
    fn new() -> Self {
        Self {
            viewport:   Viewport::new(),
            strokes:    Vec::new(),
            active:     None,
            is_drawing: false,
            pan_origin: None,
            cursor:     (0.0, 0.0),
        }
    }
}

// ---------------------------------------------------------------------------
// Cache helpers
// ---------------------------------------------------------------------------

fn rebuild_cache(state: &State, width: i32, height: i32) -> cairo::ImageSurface {
    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)
        .expect("cache surface");
    let cr = cairo::Context::new(&surface).expect("cache context");
    cr.set_source_rgb(LINE_R, LINE_G, LINE_B);
    cr.set_line_width(LINE_WIDTH / state.viewport.zoom);
    cr.set_line_cap(cairo::LineCap::Round);
    cr.set_line_join(cairo::LineJoin::Round);
    state.viewport.apply(&cr);
    for stroke in &state.strokes {
        draw_smooth_stroke(&cr, &stroke.points);
    }
    surface
}

fn invalidate_cache(cache: &Rc<RefCell<Option<cairo::ImageSurface>>>) {
    *cache.borrow_mut() = None;
}

// ---------------------------------------------------------------------------
// Public setup
// ---------------------------------------------------------------------------

pub fn setup_canvas(canvas: &DrawingArea) {
    canvas.set_hexpand(true);
    canvas.set_vexpand(true);
    canvas.set_focusable(true);

    let state: Rc<RefCell<State>> = Rc::new(RefCell::new(State::new()));
    let cache: Rc<RefCell<Option<cairo::ImageSurface>>> = Rc::new(RefCell::new(None));

    // ---- draw func ---------------------------------------------------------
    {
        let state = state.clone();
        let cache = cache.clone();
        canvas.set_draw_func(move |area, cr, w, h| {
            // Rebuild stale cache.
            {
                let mut c = cache.borrow_mut();
                if c.is_none() {
                    *c = Some(rebuild_cache(&state.borrow(), w, h));
                }
            }

            // Blit completed strokes (O(1)).
            if let Some(surf) = cache.borrow().as_ref() {
                cr.set_source_surface(surf, 0.0, 0.0).expect("set_source_surface");
                cr.paint().expect("paint");
            }

            let s = state.borrow();

            // Active stroke on top.
            if s.is_drawing {
                if let Some(stroke) = &s.active {
                    cr.set_source_rgb(LINE_R, LINE_G, LINE_B);
                    cr.set_line_width(LINE_WIDTH / s.viewport.zoom);
                    cr.set_line_cap(cairo::LineCap::Round);
                    cr.set_line_join(cairo::LineJoin::Round);
                    s.viewport.apply(cr);
                    draw_smooth_stroke(cr, &stroke.points);
                }
            }

            // Subtle origin crosshair so the user can find (0,0).
            let (ox, oy) = (s.viewport.pan_x, s.viewport.pan_y);
            let (cw, ch) = (area.width() as f64, area.height() as f64);
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.07);
            cr.set_line_width(1.0);
            cr.identity_matrix();
            cr.move_to(ox, 0.0);  cr.line_to(ox, ch);
            cr.move_to(0.0, oy);  cr.line_to(cw, oy);
            let _ = cr.stroke();
        });
    }

    // ---- left button: draw -------------------------------------------------
    let draw_gest = GestureClick::new();
    draw_gest.set_button(1);
    {
        let state = state.clone();
        let canvas_c = canvas.clone();
        draw_gest.connect_pressed(move |g, _, sx, sy| {
            let mut s = state.borrow_mut();
            let wp = s.viewport.to_world(sx, sy);
            s.active = Some(Stroke::new(wp.x, wp.y));
            s.is_drawing = true;
            g.set_state(gtk::EventSequenceState::Claimed);
            canvas_c.queue_draw();
        });
    }
    {
        let state = state.clone();
        let cache = cache.clone();
        let canvas_c = canvas.clone();
        draw_gest.connect_released(move |_, _, _, _| {
            let mut s = state.borrow_mut();
            s.is_drawing = false;
            if let Some(mut stroke) = s.active.take() {
                stroke.simplify(RDP_EPSILON);
                s.strokes.push(stroke);
            }
            invalidate_cache(&cache);
            canvas_c.queue_draw();
        });
    }
    canvas.add_controller(draw_gest);

    // ---- right button: pan -------------------------------------------------
    let pan_gest = GestureClick::new();
    pan_gest.set_button(3);
    {
        let state = state.clone();
        pan_gest.connect_pressed(move |g, _, x, y| {
            state.borrow_mut().pan_origin = Some((x, y));
            g.set_state(gtk::EventSequenceState::Claimed);
        });
    }
    {
        let state = state.clone();
        pan_gest.connect_released(move |_, _, _, _| {
            state.borrow_mut().pan_origin = None;
        });
    }
    canvas.add_controller(pan_gest);

    // ---- motion: draw points or pan ----------------------------------------
    let motion = EventControllerMotion::new();
    {
        let state = state.clone();
        let cache = cache.clone();
        let canvas_c = canvas.clone();
        motion.connect_motion(move |_, sx, sy| {
            let mut s = state.borrow_mut();
            s.cursor = (sx, sy);

            if let Some((ox, oy)) = s.pan_origin {
                s.viewport.pan(sx - ox, sy - oy);
                s.pan_origin = Some((sx, sy));
                invalidate_cache(&cache);
                canvas_c.queue_draw();
                return;
            }

            if s.is_drawing {
                let wp = s.viewport.to_world(sx, sy);
                if let Some(stroke) = s.active.as_mut() {
                    stroke.push(wp.x, wp.y);
                }
                canvas_c.queue_draw();
            }
        });
    }
    canvas.add_controller(motion);

    // ---- scroll: zoom around cursor ----------------------------------------
    let scroll = EventControllerScroll::new(
        EventControllerScrollFlags::VERTICAL | EventControllerScrollFlags::DISCRETE,
    );
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
            invalidate_cache(&cache);
            canvas_c.queue_draw();
            gtk::glib::Propagation::Stop
        });
    }
    canvas.add_controller(scroll);
}