use std::cell::Cell;
use std::rc::Rc;
use std::cell::RefCell;
use gtk::{EventControllerMotion, GestureClick, prelude::*};
use gtk::{Application, ApplicationWindow, glib, DrawingArea};

const APP_ID: &str = "org.gtk_rs.HelloWorld2";

// 1. Define a struct representing a single line
#[derive(Clone, Copy)]
struct Line {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
}

struct Mouse {
    gesture: GestureClick,
    motion: EventControllerMotion, 
}

impl Mouse {
    fn new() -> Self {
        Self { 
            gesture: GestureClick::new(), 
            motion: EventControllerMotion::new() 
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

    // Current active line being drawn
    let current_line = Rc::new((Cell::new(0.0), Cell::new(0.0), Cell::new(0.0), Cell::new(0.0)));
    
    // 2. Persistent storage for all completed lines. 
    // We use RefCell because Vec doesn't implement Copy, so Cell won't work here.
    let saved_lines = Rc::new(RefCell::new(Vec::<Line>::new()));
    
    let is_dragging = Rc::new(Cell::new(false));

    // --- HANDLE MOUSE PRESS ---
    let current_line_press = current_line.clone();
    let is_dragging_press = is_dragging.clone();
    mouse.gesture.connect_pressed(move |gesture, _n, x, y| {
        current_line_press.0.set(x);
        current_line_press.1.set(y);
        current_line_press.2.set(x);
        current_line_press.3.set(y);
        
        is_dragging_press.set(true);
        gesture.set_state(gtk::EventSequenceState::Claimed);
    });

    // --- HANDLE MOUSE MOTION ---
    let current_line_motion = current_line.clone();
    let is_dragging_motion = is_dragging.clone();
    let canvas_motion = canvas.clone();
    mouse.motion.connect_motion(move |_motion, x, y| {
        if is_dragging_motion.get() {
            current_line_motion.2.set(x);
            current_line_motion.3.set(y);
            canvas_motion.queue_draw();
        }
    });

    // --- HANDLE MOUSE RELEASE (SAVE THE LINE) ---
    let current_line_release = current_line.clone();
    let saved_lines_release = saved_lines.clone();
    let is_dragging_release = is_dragging.clone();
    mouse.gesture.connect_released(move |_gesture, _n, _x, _y| {
        if is_dragging_release.get() {
            is_dragging_release.set(false);
            
            // 3. Create a permanent Line from the current active coordinates
            let finished_line = Line {
                start_x: current_line_release.0.get(),
                start_y: current_line_release.1.get(),
                end_x: current_line_release.2.get(),
                end_y: current_line_release.3.get(),
            };
            
            // Push it into our saved list
            saved_lines_release.borrow_mut().push(finished_line);
        }
    });

    // --- CANVAS CONFIG & DRAW FUNCTION ---
    canvas.set_content_width(400);
    canvas.set_content_height(400);
    
    let current_line_draw = current_line.clone();
    let saved_lines_draw = saved_lines.clone();
    let is_dragging_draw = is_dragging.clone();
    
    canvas.set_draw_func(move |_area, cr, _width, _height| {
        cr.set_source_rgb(0.0, 1.0, 0.0);
        cr.set_line_width(8.0);
        
        // 4. Draw all previously saved lines first
        for line in saved_lines_draw.borrow().iter() {
            cr.move_to(line.start_x, line.start_y);
            cr.line_to(line.end_x, line.end_y);
            let _ = cr.stroke();
        }
        
        // 5. Draw the active preview line *only* if the user is currently dragging
        if is_dragging_draw.get() {
            cr.move_to(current_line_draw.0.get(), current_line_draw.1.get());
            cr.line_to(current_line_draw.2.get(), current_line_draw.3.get());
            let _ = cr.stroke();
        }
    });

    canvas.add_controller(mouse.gesture);
    canvas.add_controller(mouse.motion);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Persistent Drawing App")
        .child(&canvas)
        .build();

    window.present();
}