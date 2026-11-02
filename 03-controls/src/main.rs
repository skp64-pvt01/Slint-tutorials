//! Slint tutorial 03 (Rust) - controls and state.
//!
//! Four controls drive one preview:
//!
//!   LineEdit   `edited(text)`      every keystroke
//!   CheckBox   `checked`, `toggled`
//!   Slider     `value`, `changed`
//!   ComboBox   `current-index`, `selected(value)`
//!
//! The rule that makes the program simple: **one model, one update**. Every
//! control reports the whole form; Rust writes the preview. A control never
//! computes a display string, and the model never keeps a second copy of the
//! widget state.

mod harness;

slint::slint! {
    import { Button, CheckBox, ComboBox, LineEdit, Slider } from "std-widgets.slint";

    export component MainWindow inherits Window {
        title: "Tutorial 03 - Controls";
        width: 520px;
        height: 480px;
        background: #f5f6f8;

        in-out property <string> name;
        in-out property <bool> shout: false;
        in-out property <float> size: 20;
        in-out property <int> theme: 0;

        in-out property <string> preview: "Hello, world!";
        in-out property <string> summary: "ready";
        in-out property <float> preview-size: 20;

        // One callback for all four controls: the model reads the whole form.
        callback changed(string, bool, float, int);
        callback quit-requested();

        VerticalLayout {
            padding: 20px;
            spacing: 10px;

            Text { text: "Controls"; font-size: 22px; }

            Text { text: "Your name"; color: #5f6670; }
            LineEdit {
                text: root.name;
                placeholder-text: "Type a name";
                edited(text) => {
                    root.name = text;
                    root.changed(root.name, root.shout, root.size, root.theme);
                }
            }

            CheckBox {
                text: "Shout it";
                checked: root.shout;
                toggled => {
                    root.shout = self.checked;
                    root.changed(root.name, root.shout, root.size, root.theme);
                }
            }

            Text { text: "Text size"; color: #5f6670; }
            Slider {
                minimum: 12;
                maximum: 32;
                value: root.size;
                changed(value) => {
                    root.size = value;
                    root.changed(root.name, root.shout, root.size, root.theme);
                }
            }

            Text { text: "Theme"; color: #5f6670; }
            ComboBox {
                model: ["Light", "Dark"];
                current-index: root.theme;
                selected(value) => {
                    root.theme = self.current-index;
                    root.changed(root.name, root.shout, root.size, root.theme);
                }
            }

            Rectangle { height: 1px; background: #d7dbe0; }

            Text {
                text: root.preview;
                font-size: root.preview-size * 1px;
                horizontal-alignment: center;
            }
            Text {
                text: root.summary;
                color: #5f6670;
                horizontal-alignment: center;
            }

            Rectangle { vertical-stretch: 1; }
            Button { text: "Quit"; clicked => { root.quit-requested(); } }
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;

/// The form, as Rust sees it. Nothing here knows a widget exists.
struct Form {
    name: String,
    shout: bool,
    size: f32,
    theme: usize,
}

impl Form {
    fn new() -> Self {
        Self {
            name: String::new(),
            shout: false,
            size: 20.0,
            theme: 0,
        }
    }

    fn update(&mut self, name: &str, shout: bool, size: f32, theme: i32) {
        self.name = name.to_string();
        self.shout = shout;
        self.size = size;
        self.theme = theme.max(0) as usize;
    }

    fn preview(&self) -> String {
        let mut who = if self.name.trim().is_empty() {
            String::from("world")
        } else {
            self.name.trim().to_string()
        };
        if self.shout {
            who = who.to_uppercase();
        }
        format!("Hello, {who}!")
    }

    fn summary(&self) -> String {
        let theme = if self.theme == 1 { "Dark" } else { "Light" };
        format!("theme {theme}, size {:.0}", self.size)
    }
}

fn show(ui: &MainWindow, form: &Form) {
    ui.set_preview(form.preview().into());
    ui.set_summary(form.summary().into());
    ui.set_preview_size(form.size);
}

fn wire(ui: &MainWindow, form: &Rc<RefCell<Form>>) {
    let weak = ui.as_weak();
    let model = form.clone();
    ui.on_changed(move |name, shout, size, theme| {
        model.borrow_mut().update(name.as_str(), shout, size, theme);
        if let Some(ui) = weak.upgrade() {
            show(&ui, &model.borrow());
        }
    });

    let weak = ui.as_weak();
    ui.on_quit_requested(move || {
        if let Some(ui) = weak.upgrade() {
            let _ = ui.hide();
        }
    });
}

fn main() {
    if harness::is_test() {
        let mut form = Form::new();
        harness::check(
            form.preview() == "Hello, world!",
            "an empty name greets the world",
        );

        form.update("Ada", false, 20.0, 0);
        harness::check(
            form.preview() == "Hello, Ada!",
            "the typed name reaches the preview",
        );
        harness::check(
            form.summary() == "theme Light, size 20",
            "the summary reports the form",
        );

        form.update("Ada", true, 20.0, 0);
        harness::check(
            form.preview() == "Hello, ADA!",
            "Shout it uppercases the name",
        );

        form.update("Ada", true, 28.0, 1);
        harness::check(
            form.summary() == "theme Dark, size 28",
            "the size and theme are live",
        );

        form.update("   ", false, 20.0, 0);
        harness::check(
            form.preview() == "Hello, world!",
            "blank input falls back to world",
        );

        std::process::exit(harness::finish("03-controls"));
    }

    if let Err(error) = run() {
        eprintln!("03-controls: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let form = Rc::new(RefCell::new(Form::new()));
    wire(&ui, &form);
    show(&ui, &form.borrow());
    ui.run()
}
