//! Slint tutorial 01 (Rust) - a first window.
//!
//! Three ideas only:
//!   1. `slint!` describes the window in Slint, right here in `main.rs`.
//!   2. `MainWindow::new()` builds it; a callback reaches Rust.
//!   3. State lives in Rust. The UI shows it; it never owns it.
//!
//! The `--test` run does not create a window: it exercises the same `Counter`
//! the callbacks use, so it needs no display and no GPU.

mod harness;

slint::slint! {
    import { Button } from "std-widgets.slint";

    export component MainWindow inherits Window {
        title: "Tutorial 01 - First window";
        width: 440px;
        height: 240px;

        in-out property <string> message: "Press the button";
        in-out property <int> clicks: 0;

        callback clicked();
        callback quit-requested();

        VerticalLayout {
            alignment: center;
            spacing: 12px;
            padding: 24px;

            Text {
                text: "Hello from Rust";
                font-size: 24px;
                horizontal-alignment: center;
            }
            Text {
                text: root.message;
                horizontal-alignment: center;
            }
            HorizontalLayout {
                alignment: center;
                spacing: 8px;

                Button {
                    text: "Click me";
                    clicked => { root.clicked(); }
                }
                Button {
                    text: "Quit";
                    clicked => { root.quit-requested(); }
                }
            }
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;

/// The whole program state. Plain Rust: no Slint type appears here, so the
/// checks can run without a backend.
struct Counter {
    clicks: u32,
    running: bool,
}

impl Counter {
    fn new() -> Self {
        Self {
            clicks: 0,
            running: true,
        }
    }

    /// Counts one click and returns the label the window shows.
    fn click(&mut self) -> String {
        self.clicks += 1;
        format!(
            "Clicked {} time{}",
            self.clicks,
            if self.clicks == 1 { "" } else { "s" }
        )
    }

    fn quit(&mut self) {
        self.running = false;
    }
}

/// Connects the UI callbacks to the model. The closures own an `Rc` clone, so
/// the model outlives `main`'s locals and both sides see the same value.
fn wire(ui: &MainWindow, counter: &Rc<RefCell<Counter>>) {
    let weak = ui.as_weak();
    let clicks = counter.clone();
    ui.on_clicked(move || {
        let (message, count) = {
            let mut counter = clicks.borrow_mut();
            (counter.click(), counter.clicks as i32)
        };
        if let Some(ui) = weak.upgrade() {
            ui.set_message(message.into());
            ui.set_clicks(count);
        }
    });

    let weak = ui.as_weak();
    let quit = counter.clone();
    ui.on_quit_requested(move || {
        quit.borrow_mut().quit();
        if let Some(ui) = weak.upgrade() {
            let _ = ui.hide();
        }
    });
}

fn main() {
    if harness::is_test() {
        let mut app = Counter::new();
        harness::check(app.running, "the window starts running");
        harness::check(
            app.click() == "Clicked 1 time",
            "the first click says 1 time",
        );
        harness::check(
            app.click() == "Clicked 2 times",
            "the second click says 2 times",
        );
        harness::check(app.clicks == 2, "the model counted both clicks");
        app.quit();
        harness::check(!app.running, "Quit stops the window");

        std::process::exit(harness::finish("01-first-window"));
    }

    if let Err(error) = run() {
        eprintln!("01-first-window: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let counter = Rc::new(RefCell::new(Counter::new()));
    wire(&ui, &counter);
    ui.run()
}
