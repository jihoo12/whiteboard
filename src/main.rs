mod types;
mod viewport;
mod memo;
mod canvas;

use std::rc::Rc;
use std::cell::Cell;

use gtk::{prelude::*, Application, ApplicationWindow, Box as GBox,
          Button, Label, Orientation, Separator, CssProvider};
use gtk::glib;
use gtk::gdk::Display;

use canvas::Tool;

const APP_ID: &str = "org.gtk_rs.CurveDrawing";

const CSS: &str = r#"
window {
    background-color: #1a1a1f;
}

.toolbar {
    background-color: #111116;
    border-bottom: 1px solid #2e2e38;
    padding: 6px 12px;
    min-height: 44px;
}

.tool-btn {
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    color: #9090a8;
    font-size: 12px;
    font-weight: 500;
    padding: 5px 14px;
    min-width: 72px;
    transition: all 120ms ease;
}

.tool-btn:hover {
    background-color: #2a2a35;
    border-color: #3a3a48;
    color: #d0d0e8;
}

.tool-btn.active {
    background-color: #25253a;
    border-color: #5a5aaa;
    color: #a8a8f8;
    box-shadow: 0 0 0 1px #5a5aaa inset;
}

.tool-btn label {
    font-size: 12px;
}

.toolbar-sep {
    background-color: #2e2e38;
    min-width: 1px;
    margin: 6px 4px;
}

.hint-label {
    color: #3e3e52;
    font-size: 11px;
    font-style: italic;
    letter-spacing: 0.02em;
}

.canvas-area {
    background-color: #13131a;
}

scrolledwindow {
    background-color: #13131a;
}

/* Remove ugly focus ring */
drawingarea:focus {
    outline: none;
}
"#;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    // Load CSS.
    let provider = CssProvider::new();
    provider.load_from_data(CSS);
    gtk::style_context_add_provider_for_display(
        &Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let tool: Rc<Cell<Tool>> = Rc::new(Cell::new(Tool::Draw));

    // --- Tool buttons ---
    let btn_draw  = make_tool_btn("✏", "Draw",  true);
    let btn_memo  = make_tool_btn("🗒", "Memo",  false);
    let btn_erase = make_tool_btn("⌫", "Erase", false);

    // Draw button.
    {
        let tool = tool.clone();
        let b_draw = btn_draw.clone();
        let b_memo = btn_memo.clone();
        let b_erase = btn_erase.clone();
        btn_draw.connect_clicked(move |_| {
            tool.set(Tool::Draw);
            set_active(&b_draw, &[&b_memo, &b_erase]);
        });
    }
    // Memo button.
    {
        let tool = tool.clone();
        let b_draw = btn_draw.clone();
        let b_memo = btn_memo.clone();
        let b_erase = btn_erase.clone();
        btn_memo.connect_clicked(move |_| {
            tool.set(Tool::Memo);
            set_active(&b_memo, &[&b_draw, &b_erase]);
        });
    }
    // Erase button — cycles strokes out (future), for now just sets tool.
    {
        let tool = tool.clone();
        let b_draw = btn_draw.clone();
        let b_memo = btn_memo.clone();
        let b_erase = btn_erase.clone();
        btn_erase.connect_clicked(move |_| {
            tool.set(Tool::Erase);
            set_active(&b_erase, &[&b_draw, &b_memo]);
        });
    }

    // --- Toolbar layout ---
    let toolbar = GBox::new(Orientation::Horizontal, 4);
    toolbar.add_css_class("toolbar");

    let tool_group = GBox::new(Orientation::Horizontal, 2);
    tool_group.append(&btn_draw);
    tool_group.append(&btn_memo);
    tool_group.append(&btn_erase);
    toolbar.append(&tool_group);

    // Spacer.
    let spacer = gtk::Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    toolbar.append(&spacer);

    // Hint label.
    let hint = Label::new(Some("drag: draw  ·  right-drag: pan  ·  scroll: zoom"));
    hint.add_css_class("hint-label");
    hint.set_halign(gtk::Align::End);
    toolbar.append(&hint);

    // --- Canvas ---
    let drawing_area = gtk::DrawingArea::new();
    drawing_area.add_css_class("canvas-area");
    canvas::setup_canvas(&drawing_area, tool);

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .child(&drawing_area)
        .build();
    scroll.set_vexpand(true);

    let vbox = GBox::new(Orientation::Vertical, 0);
    vbox.append(&toolbar);
    vbox.append(&scroll);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Canvas")
        .default_width(1280)
        .default_height(840)
        .child(&vbox)
        .build();

    window.present();
}

fn make_tool_btn(icon: &str, label: &str, active: bool) -> Button {
    let b = Button::new();
    b.add_css_class("tool-btn");
    if active { b.add_css_class("active"); }

    let inner = GBox::new(Orientation::Horizontal, 5);
    inner.set_halign(gtk::Align::Center);

    let icon_lbl = Label::new(Some(icon));
    let text_lbl = Label::new(Some(label));

    inner.append(&icon_lbl);
    inner.append(&text_lbl);
    b.set_child(Some(&inner));
    b
}

fn set_active(active: &Button, others: &[&Button]) {
    active.add_css_class("active");
    for b in others { b.remove_css_class("active"); }
}