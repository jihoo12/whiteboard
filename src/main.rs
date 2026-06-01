mod types;
mod viewport;
mod canvas;

use gtk::{prelude::*, Application, ApplicationWindow, ScrolledWindow};
use gtk::glib;

const APP_ID: &str = "org.gtk_rs.CurveDrawing";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let drawing_area = gtk::DrawingArea::new();
    canvas::setup_canvas(&drawing_area);

    // Wrap in a ScrolledWindow so the OS-level window is resizable while the
    // canvas itself expands to fill all available space.
    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .child(&drawing_area)
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Infinite Canvas")
        .default_width(1024)
        .default_height(768)
        .child(&scroll)
        .build();

    window.present();
}