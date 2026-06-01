mod types;
use types::{Stroke, draw_smooth_stroke};

use std::cell::Cell;
use std::rc::Rc;
use std::cell::RefCell;
use gtk::{EventControllerMotion, GestureClick, cairo, prelude::*};
use gtk::{Application, ApplicationWindow, glib, DrawingArea};

const APP_ID: &str = "org.gtk_rs.CurveDrawing";

struct Mouse {
    gesture: GestureClick,
    motion: EventControllerMotion,
}

impl Mouse {
    fn new() -> Self {
        Self {
            gesture: GestureClick::new(),
            motion: EventControllerMotion::new(),
        }
    }
}

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let canvas = DrawingArea::new();
    let mouse = Mouse::new();
    mouse.gesture.set_button(1);

    // All completed strokes.
    let saved_strokes: Rc<RefCell<Vec<Stroke>>> = Rc::new(RefCell::new(Vec::new()));

    // The stroke currently being drawn (None when not dragging).
    let active_stroke: Rc<RefCell<Option<Stroke>>> = Rc::new(RefCell::new(None));

    let is_dragging = Rc::new(Cell::new(false));

    // --- MOUSE PRESS: start a new stroke ---
    let active_press = active_stroke.clone();
    let is_dragging_press = is_dragging.clone();
    mouse.gesture.connect_pressed(move |gesture, _n, x, y| {
        *active_press.borrow_mut() = Some(Stroke::new(x, y));
        is_dragging_press.set(true);
        gesture.set_state(gtk::EventSequenceState::Claimed);
    });

    // --- MOUSE MOTION: append points while dragging ---
    let active_motion = active_stroke.clone();
    let is_dragging_motion = is_dragging.clone();
    let canvas_motion = canvas.clone();
    mouse.motion.connect_motion(move |_motion, x, y| {
        if is_dragging_motion.get() {
            if let Some(stroke) = active_motion.borrow_mut().as_mut() {
                stroke.push(x, y);
            }
            canvas_motion.queue_draw();
        }
    });

    // --- MOUSE RELEASE: save the completed stroke ---
    let active_release = active_stroke.clone();
    let saved_release = saved_strokes.clone();
    let is_dragging_release = is_dragging.clone();
    mouse.gesture.connect_released(move |_gesture, _n, _x, _y| {
        if is_dragging_release.get() {
            is_dragging_release.set(false);
            if let Some(finished) = active_release.borrow_mut().take() {
                saved_release.borrow_mut().push(finished);
            }
        }
    });

    // --- CANVAS SETUP ---
    canvas.set_content_width(800);
    canvas.set_content_height(600);

    let active_draw = active_stroke.clone();
    let saved_draw = saved_strokes.clone();
    let is_dragging_draw = is_dragging.clone();

    canvas.set_draw_func(move |_area, cr, _width, _height| {
        cr.set_source_rgb(0.0, 0.8, 0.2);
        cr.set_line_width(3.0);
        cr.set_line_cap(cairo::LineCap::Round);
        cr.set_line_join(cairo::LineJoin::Round);

        // Draw all saved strokes.
        for stroke in saved_draw.borrow().iter() {
            draw_smooth_stroke(cr, &stroke.points);
        }

        // Draw the active (in-progress) stroke preview.
        if is_dragging_draw.get() {
            if let Some(stroke) = active_draw.borrow().as_ref() {
                draw_smooth_stroke(cr, &stroke.points);
            }
        }
    });

    canvas.add_controller(mouse.gesture);
    canvas.add_controller(mouse.motion);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Curve Drawing App")
        .child(&canvas)
        .build();

    window.present();
}