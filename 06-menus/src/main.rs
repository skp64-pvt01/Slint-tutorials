//! Slint tutorial 06 (Rust) - a native menu bar.
//!
//! A `MenuBar` lives in the window, not in the layout, and `@keys(Control + N)`
//! gives an entry a shortcut that works while the menu is closed.
//!
//! Every entry calls `root.menu-action("file.new")`: one callback, one
//! dispatcher in Rust. The `Action` enum is that dispatcher's vocabulary.

mod harness;

slint::include_modules!();

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    New,
    Open,
    Save,
    Undo,
    Redo,
    SelectAll,
    Dark,
    Light,
    Shortcuts,
    About,
    Quit,
}

impl Action {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "file.new" => Action::New,
            "file.open" => Action::Open,
            "file.save" => Action::Save,
            "file.quit" => Action::Quit,
            "edit.undo" => Action::Undo,
            "edit.redo" => Action::Redo,
            "edit.select-all" => Action::SelectAll,
            "view.dark" => Action::Dark,
            "view.light" => Action::Light,
            "help.shortcuts" => Action::Shortcuts,
            "help.about" => Action::About,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Action::New => "file.new",
            Action::Open => "file.open",
            Action::Save => "file.save",
            Action::Undo => "edit.undo",
            Action::Redo => "edit.redo",
            Action::SelectAll => "edit.select-all",
            Action::Dark => "view.dark",
            Action::Light => "view.light",
            Action::Shortcuts => "help.shortcuts",
            Action::About => "help.about",
            Action::Quit => "file.quit",
        }
    }

    fn status(self) -> String {
        match self {
            Action::Dark => String::from("Theme: dark"),
            Action::Light => String::from("Theme: light"),
            Action::Shortcuts => String::from("Help: keyboard shortcuts"),
            Action::About => String::from("Help: about"),
            other => format!("Menu: {}", other.name().rsplit('.').next().unwrap_or("?")),
        }
    }
}

struct Menus {
    status: String,
    last: Option<Action>,
    dark: bool,
}

impl Menus {
    fn new() -> Self {
        Self {
            status: String::from("Menu: none"),
            last: None,
            dark: false,
        }
    }

    fn dispatch(&mut self, name: &str) -> Option<Action> {
        let action = Action::from_name(name)?;
        if action == Action::Dark {
            self.dark = true;
        } else if action == Action::Light {
            self.dark = false;
        }
        self.status = action.status();
        self.last = Some(action);
        Some(action)
    }
}

fn wire(ui: &MainWindow, menus: &Rc<RefCell<Menus>>) {
    let weak = ui.as_weak();
    let model = menus.clone();
    ui.on_menu_action(move |name| {
        let action = model.borrow_mut().dispatch(name.as_str());
        let (status, dark) = {
            let model = model.borrow();
            (model.status.clone(), model.dark)
        };
        if let Some(ui) = weak.upgrade() {
            ui.set_status(status.into());
            ui.set_dark(dark);
            if action == Some(Action::Quit) {
                let _ = ui.hide();
            }
        }
    });
}

fn main() {
    if harness::is_test() {
        let mut menus = Menus::new();
        harness::check(menus.status == "Menu: none", "the status starts neutral");
        harness::check(
            menus.dispatch("file.new") == Some(Action::New),
            "file.new is an action",
        );
        harness::check(menus.status == "Menu: new", "the status names the entry");
        harness::check(
            menus.dispatch("file.save") == Some(Action::Save),
            "file.save is an action",
        );
        harness::check(menus.status == "Menu: save", "the last entry wins");
        harness::check(
            menus.dispatch("nonsense").is_none(),
            "an unknown name is not an action",
        );
        harness::check(
            menus.status == "Menu: save",
            "an unknown name changes nothing",
        );
        harness::check(
            menus.dispatch("view.dark") == Some(Action::Dark) && menus.dark,
            "Dark sets the theme",
        );
        harness::check(
            menus.dispatch("file.quit") == Some(Action::Quit),
            "Quit is an action too",
        );
        harness::check(
            menus.last == Some(Action::Quit),
            "the model keeps the last action",
        );

        std::process::exit(harness::finish("06-menus"));
    }

    if let Err(error) = run() {
        eprintln!("06-menus: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let menus = Rc::new(RefCell::new(Menus::new()));
    wire(&ui, &menus);
    ui.run()
}
