use std::cell::Cell;
use std::rc::Rc;
use gtk::{EventControllerMotion, GestureClick, prelude::*};
use gtk::{Application, ApplicationWindow, glib, DrawingArea};

const APP_ID: &str = "org.gtk_rs.HelloWorld2";

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
    mouse.gesture.set_button(1); // Left click

    // Coordinates: (start_x, start_y, end_x, end_y)
    let coords = Rc::new((Cell::new(0.0), Cell::new(0.0), Cell::new(0.0), Cell::new(0.0)));
    
    // Track if the user is actively holding down the mouse button and dragging
    let is_dragging = Rc::new(Cell::new(false));

    // --- 1. HANDLE MOUSE PRESS (START DRAWING) ---
    let coords_press = coords.clone();
    let is_dragging_press = is_dragging.clone();
    mouse.gesture.connect_pressed(move |gesture, _n, x, y| {
        coords_press.0.set(x);
        coords_press.1.set(y);
        // Initialize end coordinates at start point so it doesn't jump from a previous line
        coords_press.2.set(x);
        coords_press.3.set(y);
        
        is_dragging_press.set(true);
        gesture.set_state(gtk::EventSequenceState::Claimed);
    });

    // --- 2. HANDLE MOUSE MOTION (LIVE UPDATE) ---
    let coords_motion = coords.clone();
    let is_dragging_motion = is_dragging.clone();
    let canvas_motion = canvas.clone();
    mouse.motion.connect_motion(move |_motion, x, y| {
        // Only update end positions if the user is holding down the mouse button
        if is_dragging_motion.get() {
            coords_motion.2.set(x);
            coords_motion.3.set(y);
            
            // Queue a redraw to update the line while moving
            canvas_motion.queue_draw();
        }
    });

    // --- 3. HANDLE MOUSE RELEASE (STOP DRAWING) ---
    let is_dragging_release = is_dragging.clone();
    mouse.gesture.connect_released(move |_gesture, _n, _x, _y| {
        is_dragging_release.set(false);
    });

    // --- 4. CANVAS CONFIG & DRAW FUNCTION ---
    canvas.set_content_width(200);
    canvas.set_content_height(200);
    
    let coords_draw = coords.clone();
    canvas.set_draw_func(move |_area, cr, _width, _height| {
        cr.set_source_rgb(0.0, 1.0, 0.0);
        cr.set_line_width(8.0);
        
        cr.move_to(coords_draw.0.get(), coords_draw.1.get());
        cr.line_to(coords_draw.2.get(), coords_draw.3.get());
        let _ = cr.stroke();
    });

    // --- 5. ATTACH CONTROLLERS TO CANVAS ---
    // You must attach BOTH controllers to the widget so GTK listens to their events
    canvas.add_controller(mouse.gesture);
    canvas.add_controller(mouse.motion);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&canvas)
        .build();

    window.present();
}