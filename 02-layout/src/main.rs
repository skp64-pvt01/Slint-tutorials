//! Slint tutorial 02 (Rust) - layout.
//!
//! A Slint layout arranges its children; there is no XML and no `Slot.*`:
//!
//!   `VerticalLayout` / `HorizontalLayout`   the two layouts
//!   `spacing` / `padding`                   gaps, inside and between
//!   `horizontal-stretch` / `vertical-stretch`   share of the leftover space
//!   `alignment`                              where the children sit
//!
//! The window is three areas: a fixed header, a content area that takes the
//! rest, and a fixed footer. Resize the window: only the content grows.

mod harness;

slint::slint! {
    import { Button } from "std-widgets.slint";

    export component MainWindow inherits Window {
        title: "Tutorial 02 - Layout";
        width: 640px;
        height: 420px;
        background: #f5f6f8;

        in-out property <string> status: "Nothing pressed yet.";

        callback pressed(string);
        callback cleared();

        VerticalLayout {
            spacing: 0px;

            // Header: a fixed height, and a layout inside it.
            Rectangle {
                height: 56px;
                background: #ffffff;
                border-width: 1px;
                border-color: #d7dbe0;

                HorizontalLayout {
                    padding-left: 16px;
                    padding-right: 16px;
                    spacing: 12px;

                    Text {
                        text: "Layout";
                        font-size: 22px;
                        vertical-alignment: center;
                    }
                    Rectangle { horizontal-stretch: 1; }
                    Text {
                        text: "step 02";
                        color: #5f6670;
                        vertical-alignment: center;
                    }
                }
            }

            // Content: takes every pixel the header and footer leave.
            Rectangle {
                vertical-stretch: 1;
                background: #f5f6f8;

                VerticalLayout {
                    padding: 16px;
                    spacing: 10px;

                    Text {
                        text: "The three buttons below share the width: each has horizontal-stretch: 1.";
                        wrap: word-wrap;
                    }

                    HorizontalLayout {
                        spacing: 8px;

                        Button { text: "One"; horizontal-stretch: 1; clicked => { root.pressed("One"); } }
                        Button { text: "Two"; horizontal-stretch: 1; clicked => { root.pressed("Two"); } }
                        Button { text: "Three"; horizontal-stretch: 1; clicked => { root.pressed("Three"); } }
                    }

                    Rectangle { height: 1px; background: #d7dbe0; }

                    HorizontalLayout {
                        spacing: 8px;

                        Text {
                            text: "This label grows; the button keeps its natural size.";
                            horizontal-stretch: 1;
                            vertical-alignment: center;
                        }
                        Button { text: "Fixed"; clicked => { root.pressed("Fixed"); } }
                    }

                    // The spacer eats the leftover height, pushing what follows down.
                    Rectangle { vertical-stretch: 1; }

                    Text {
                        text: "A vertical-stretch: 1 spacer pushes this line to the bottom of the content area.";
                        color: #5f6670;
                        wrap: word-wrap;
                    }
                }
            }

            // Footer: fixed again.
            Rectangle {
                height: 44px;
                background: #ffffff;
                border-width: 1px;
                border-color: #d7dbe0;

                HorizontalLayout {
                    padding-left: 16px;
                    padding-right: 16px;
                    spacing: 8px;

                    Text {
                        text: root.status;
                        horizontal-stretch: 1;
                        vertical-alignment: center;
                        color: #1c1f23;
                    }
                    Button { text: "Clear"; clicked => { root.cleared(); } }
                }
            }
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;

/// The state behind the footer. The layout itself is in the `.slint`; a check
/// can only assert the data the program owns.
struct Page {
    status: String,
}

impl Page {
    fn new() -> Self {
        Self {
            status: String::from("Nothing pressed yet."),
        }
    }

    fn press(&mut self, name: &str) {
        self.status = format!("Pressed {name}");
    }

    fn clear(&mut self) {
        self.status = String::from("Nothing pressed yet.");
    }
}

fn wire(ui: &MainWindow, page: &Rc<RefCell<Page>>) {
    let weak = ui.as_weak();
    let model = page.clone();
    ui.on_pressed(move |name| {
        model.borrow_mut().press(name.as_str());
        if let Some(ui) = weak.upgrade() {
            ui.set_status(model.borrow().status.clone().into());
        }
    });

    let weak = ui.as_weak();
    let model = page.clone();
    ui.on_cleared(move || {
        model.borrow_mut().clear();
        if let Some(ui) = weak.upgrade() {
            ui.set_status(model.borrow().status.clone().into());
        }
    });
}

fn main() {
    if harness::is_test() {
        let mut page = Page::new();
        harness::check(
            page.status == "Nothing pressed yet.",
            "the footer starts neutral",
        );
        page.press("One");
        harness::check(page.status == "Pressed One", "a click names the button");
        page.press("Fixed");
        harness::check(page.status == "Pressed Fixed", "the last click wins");
        page.clear();
        harness::check(
            page.status == "Nothing pressed yet.",
            "Clear resets the footer",
        );
        harness::check(!page.status.is_empty(), "the footer always has text");

        std::process::exit(harness::finish("02-layout"));
    }

    if let Err(error) = run() {
        eprintln!("02-layout: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let page = Rc::new(RefCell::new(Page::new()));
    wire(&ui, &page);
    ui.run()
}
