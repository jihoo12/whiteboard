use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib, DrawingArea};

const APP_ID: &str = "org.gtk_rs.HelloWorld2";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create a button with label and margins
    let canvas = DrawingArea::new();
    canvas.set_content_width(200);
    canvas.set_content_height(200);
    canvas.set_draw_func(|_area, cr, width, height| {
        let center_x = width as f64 / 2.0;
        let center_y = height as f64 /2.0;
        let radius = (width.min(height) as f64)/2.0;
        cr.set_source_rgb(0.0, 1.0, 0.0);
        cr.arc(center_x, center_y, radius, 0.0, 6.28);
        let _ = cr.fill();
    });

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&canvas)
        .build();

    // Present window
    window.present();
}