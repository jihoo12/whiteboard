mod types;
mod viewport;
mod memo;
mod canvas;

use std::rc::Rc;
use std::cell::Cell;

use gtk::{prelude::*, Application, ApplicationWindow, Box as GBox,
          Button, Orientation, ScrolledWindow, Separator};
use gtk::glib;

use canvas::Tool;

const APP_ID: &str = "org.gtk_rs.CurveDrawing";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let tool: Rc<Cell<Tool>> = Rc::new(Cell::new(Tool::Draw));

    // Toolbar buttons.
    let btn_draw = Button::with_label("✏ Draw");
    let btn_memo = Button::with_label("📝 Memo");
    btn_draw.add_css_class("suggested-action"); // highlight active tool

    {
        let tool = tool.clone();
        let btn_draw_c = btn_draw.clone();
        let btn_memo_c = btn_memo.clone();
        btn_draw.connect_clicked(move |b| {
            tool.set(Tool::Draw);
            b.add_css_class("suggested-action");
            btn_memo_c.remove_css_class("suggested-action");
        });
    }
    {
        let tool = tool.clone();
        let btn_draw_c = btn_draw.clone();
        btn_memo.connect_clicked(move |b| {
            tool.set(Tool::Memo);
            b.add_css_class("suggested-action");
            btn_draw_c.remove_css_class("suggested-action");
        });
    }

    let toolbar = GBox::new(Orientation::Horizontal, 6);
    toolbar.set_margin_top(6);
    toolbar.set_margin_bottom(6);
    toolbar.set_margin_start(10);
    toolbar.set_margin_end(10);
    toolbar.append(&btn_draw);
    toolbar.append(&Separator::new(Orientation::Vertical));
    toolbar.append(&btn_memo);

    // Hint label.
    let hint = gtk::Label::new(Some(
        "Left-drag: draw  |  Right-drag: pan  |  Scroll: zoom  |  Memo tool: click to place, click again to edit"
    ));
    hint.add_css_class("dim-label");
    hint.set_margin_end(10);
    hint.set_hexpand(true);
    hint.set_halign(gtk::Align::End);
    toolbar.append(&hint);

    let drawing_area = gtk::DrawingArea::new();
    canvas::setup_canvas(&drawing_area, tool);

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .child(&drawing_area)
        .build();

    let vbox = GBox::new(Orientation::Vertical, 0);
    vbox.append(&toolbar);
    vbox.append(&Separator::new(Orientation::Horizontal));
    vbox.append(&scroll);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Infinite Canvas")
        .default_width(1200)
        .default_height(800)
        .child(&vbox)
        .build();

    window.present();
}