//! Slint tutorial 04 (Rust) - actions and shortcuts.
//!
//! A button, a keyboard shortcut and (in the next steps) a menu entry should
//! run the same code. Slint calls a Rust **callback**; the callback calls one
//! method on the model:
//!
//!   Button      `clicked => { root.add-one(); }`
//!   FocusScope  `key-pressed(event) => { if Control + "i" ... }`
//!   model       `dispatch(Action::AddOne)`
//!
//! The `Action` enum is the vocabulary. The shortcut is *window-level*: it
//! still fires when the +1 button is disabled, exactly as a menu shortcut does.

mod harness;

slint::slint! {
    import { Button, CheckBox } from "std-widgets.slint";

    export component MainWindow inherits Window {
        title: "Tutorial 04 - Actions";
        width: 480px;
        height: 420px;
        background: #f5f6f8;

        in-out property <string> count: "0";
        in-out property <string> status: "Ready.";
        in-out property <bool> add-enabled: true;

        callback add-one();
        callback add-ten();
        callback reset();
        callback set-enabled(bool);
        callback quit-requested();

        // The focus scope makes the window itself listen for keys. Click
        // anywhere, or Tab into it: it takes the focus and stays there.
        FocusScope {
            init => { self.focus(); }

            key-pressed(event) => {
                if (event.modifiers.control && event.text == "i") {
                    root.add-one();
                    return accept;
                }
                if (event.modifiers.control && event.text == "0") {
                    root.reset();
                    return accept;
                }
                return reject;
            }

            VerticalLayout {
                padding: 20px;
                spacing: 10px;

                Text { text: "Actions"; font-size: 22px; }
                Text {
                    text: root.count;
                    font-size: 40px;
                    horizontal-alignment: center;
                }

                HorizontalLayout {
                    alignment: center;
                    spacing: 8px;

                    Button {
                        text: "+1";
                        enabled: root.add-enabled;
                        clicked => { root.add-one(); }
                    }
                    Button {
                        text: "+10";
                        enabled: root.add-enabled;
                        clicked => { root.add-ten(); }
                    }
                    Button {
                        text: "Reset";
                        clicked => { root.reset(); }
                    }
                }

                CheckBox {
                    text: "Add buttons enabled";
                    checked: root.add-enabled;
                    toggled => { root.set-enabled(self.checked); }
                }

                Rectangle { vertical-stretch: 1; }

                Text { text: root.status; color: #5f6670; }
                Text { text: "Ctrl+I adds one, Ctrl+0 resets."; color: #5f6670; }
                Button { text: "Quit"; clicked => { root.quit-requested(); } }
            }
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy)]
enum Action {
    AddOne,
    AddTen,
    Reset,
}

impl Action {
    fn name(self) -> &'static str {
        match self {
            Action::AddOne => "counter.add",
            Action::AddTen => "counter.add_ten",
            Action::Reset => "counter.reset",
        }
    }

    fn amount(self) -> i32 {
        match self {
            Action::AddOne => 1,
            Action::AddTen => 10,
            Action::Reset => 0,
        }
    }
}

struct Counter {
    value: i32,
    enabled: bool,
    last: String,
}

impl Counter {
    fn new() -> Self {
        Self {
            value: 0,
            enabled: true,
            last: String::from("none"),
        }
    }

    fn dispatch(&mut self, action: Action) {
        self.value = if matches!(action, Action::Reset) {
            0
        } else {
            self.value + action.amount()
        };
        self.last = action.name().to_string();
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.last = if enabled {
            "counter.enabled".into()
        } else {
            "counter.disabled".into()
        };
    }
}

fn show(ui: &MainWindow, counter: &Counter) {
    ui.set_count(counter.value.to_string().into());
    ui.set_add_enabled(counter.enabled);
    ui.set_status(format!("Last action: {}", counter.last).into());
}

fn wire(ui: &MainWindow, counter: &Rc<RefCell<Counter>>) {
    {
        let weak = ui.as_weak();
        let model = counter.clone();
        ui.on_add_one(move || {
            model.borrow_mut().dispatch(Action::AddOne);
            if let Some(ui) = weak.upgrade() {
                show(&ui, &model.borrow());
            }
        });
    }
    {
        let weak = ui.as_weak();
        let model = counter.clone();
        ui.on_add_ten(move || {
            model.borrow_mut().dispatch(Action::AddTen);
            if let Some(ui) = weak.upgrade() {
                show(&ui, &model.borrow());
            }
        });
    }
    {
        let weak = ui.as_weak();
        let model = counter.clone();
        ui.on_reset(move || {
            model.borrow_mut().dispatch(Action::Reset);
            if let Some(ui) = weak.upgrade() {
                show(&ui, &model.borrow());
            }
        });
    }

    let weak = ui.as_weak();
    let model = counter.clone();
    ui.on_set_enabled(move |enabled| {
        model.borrow_mut().set_enabled(enabled);
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
        let mut counter = Counter::new();
        harness::check(counter.value == 0, "the counter starts at 0");
        harness::check(counter.enabled, "the add buttons start enabled");

        counter.dispatch(Action::AddOne);
        harness::check(counter.value == 1, "AddOne adds one");
        harness::check(counter.last == "counter.add", "the action names itself");

        counter.dispatch(Action::AddTen);
        harness::check(counter.value == 11, "AddTen adds ten");

        counter.set_enabled(false);
        counter.dispatch(Action::AddOne);
        harness::check(
            counter.value == 12,
            "a window shortcut still fires when the button is disabled",
        );

        counter.dispatch(Action::Reset);
        harness::check(
            counter.value == 0 && counter.last == "counter.reset",
            "Reset wins over everything",
        );

        std::process::exit(harness::finish("04-actions"));
    }

    if let Err(error) = run() {
        eprintln!("04-actions: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let counter = Rc::new(RefCell::new(Counter::new()));
    wire(&ui, &counter);
    show(&ui, &counter.borrow());
    ui.run()
}
