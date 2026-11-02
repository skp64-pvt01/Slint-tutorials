//! Slint tutorial 08 (Rust) - Notes, the starting point of an application.
//!
//! Everything the seven steps taught, in the usual shape of an editor:
//!
//!   ui/main.slint     the window: menus, toolbar, editor, status bar, overlay
//!   this file         the actions, the files, the unsaved-changes guard
//!
//!   cargo run                     a window
//!   cargo run -- notes.txt        open a file
//!   cargo run -- --test           no window: model checks only
//!
//! The model functions (`open_path`, `save_to`, `new_document`) take a path and
//! never show a dialog. Only the callbacks in `apply_action` call `rfd`, so a
//! test can drive everything without a display.
//!
//! Where to go from here: recent files, a preview pane, a toolbar editor for
//! fonts, drag and drop (see README.md for why it is not here).

mod harness;

slint::include_modules!();

use slint::{CloseRequestResponse, SharedString};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/* ---------------------------------------------------------------------------
 *  The model: no Slint type appears here
 * ------------------------------------------------------------------------- */

struct Notes {
    path: Option<PathBuf>,
    text: String,
    dirty: bool,
    status: String,
    cursor: usize,
    dark: bool,
}

impl Notes {
    fn new() -> Self {
        Self {
            path: None,
            text: String::new(),
            dirty: false,
            status: String::from("Ready"),
            cursor: 0,
            dark: false,
        }
    }

    fn file_name(&self) -> String {
        self.path
            .as_deref()
            .and_then(Path::file_name)
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from("Untitled"))
    }

    fn title(&self) -> String {
        format!(
            "{}{} - Notes",
            self.file_name(),
            if self.dirty { " *" } else { "" }
        )
    }

    fn new_document(&mut self) {
        self.path = None;
        self.text.clear();
        self.dirty = false;
        self.cursor = 0;
        self.status = String::from("New document");
    }

    fn open_path(&mut self, path: &str) -> bool {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                self.path = Some(PathBuf::from(path));
                self.text = text;
                self.dirty = false;
                self.cursor = 0;
                self.status = format!("Opened {path}");
                true
            }
            Err(error) => {
                self.status = format!("Cannot read {path}: {error}");
                false
            }
        }
    }

    fn save_to(&mut self, path: &str) -> bool {
        match std::fs::write(path, &self.text) {
            Ok(()) => {
                self.path = Some(PathBuf::from(path));
                self.dirty = false;
                self.status = format!("Saved {path}");
                true
            }
            Err(error) => {
                self.status = format!("Cannot write {path}: {error}");
                false
            }
        }
    }

    /// A user edit. Loading a file does not call this.
    fn set_text(&mut self, text: &str) {
        if self.text == text {
            return;
        }
        self.text = text.to_string();
        self.dirty = true;
        self.status = String::from("Modified");
    }

    fn set_cursor(&mut self, offset: usize) {
        self.cursor = offset.min(self.text.len());
    }

    /// The pure form of the unsaved-changes guard. The window computes
    /// `confirmed` from a dialog; a test passes `true` or `false`.
    fn can_discard(&self, confirmed: bool) -> bool {
        !self.dirty || confirmed
    }

    fn position_text(&self) -> String {
        let mut offset = self.cursor.min(self.text.len());
        while offset > 0 && !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        let before = &self.text[..offset];
        let line = before.matches('\n').count() + 1;
        let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        format!(
            "Ln {line}, Col {column} · {} characters",
            self.text.chars().count()
        )
    }
}

/* ---------------------------------------------------------------------------
 *  Data file (installed by step 09) and the static help texts
 * ------------------------------------------------------------------------- */

fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("NOTES_DATA_DIR") {
        return PathBuf::from(dir);
    }
    match option_env!("NOTES_DATA_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"),
    }
}

fn read_data(name: &str) -> Option<String> {
    std::fs::read_to_string(data_dir().join(name)).ok()
}

fn shortcuts_text() -> String {
    read_data("shortcuts.txt").unwrap_or_else(|| {
        String::from(
            "Ctrl+N  New\nCtrl+O  Open\nCtrl+S  Save\nCtrl+Shift+S  Save as\n\
             Ctrl+Z  Undo\nCtrl+Shift+Z  Redo\nCtrl+A  Select all\nCtrl+Q  Quit\nF1  Shortcuts",
        )
    })
}

fn about_text() -> String {
    format!(
        "Notes {}\n\nA small editor built with Slint 1.18.1 in the Slint tutorials.\n\
         Licensed GPL-3.0-only; Slint is also available under its royalty-free license.",
        env!("CARGO_PKG_VERSION")
    )
}

/* ---------------------------------------------------------------------------
 *  Actions
 * ------------------------------------------------------------------------- */

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    New,
    Open,
    Save,
    SaveAs,
    Quit,
    Undo,
    Redo,
    SelectAll,
    Dark,
    Light,
    Shortcuts,
    About,
}

impl Action {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "file.new" => Action::New,
            "file.open" => Action::Open,
            "file.save" => Action::Save,
            "file.save-as" => Action::SaveAs,
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

    fn status(self) -> String {
        match self {
            Action::Dark => String::from("Theme: dark"),
            Action::Light => String::from("Theme: light"),
            other => format!(
                "Menu: {}",
                match other {
                    Action::New => "new",
                    Action::Open => "open",
                    Action::Save => "save",
                    Action::SaveAs => "save-as",
                    Action::Quit => "quit",
                    Action::Undo => "undo",
                    Action::Redo => "redo",
                    Action::SelectAll => "select-all",
                    Action::Shortcuts => "shortcuts",
                    Action::About => "about",
                    Action::Dark | Action::Light => "theme",
                }
            ),
        }
    }
}

/* ---------------------------------------------------------------------------
 *  Dialogs: only the window calls these
 * ------------------------------------------------------------------------- */

fn confirm_discard() -> bool {
    rfd::MessageDialog::new()
        .set_title("Unsaved changes")
        .set_description("The document has changes that were not saved. Discard them?")
        .set_buttons(rfd::MessageButtons::OkCancel)
        .show()
        == rfd::MessageDialogResult::Ok
}

fn may_replace(notes: &Rc<RefCell<Notes>>) -> bool {
    let dirty = notes.borrow().dirty;
    if !dirty {
        return true;
    }
    let confirmed = confirm_discard();
    notes.borrow().can_discard(confirmed)
}

fn choose_open() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text", &["txt", "md"])
        .set_title("Open a text")
        .pick_file()
}

fn choose_save(default_name: String) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text", &["txt", "md"])
        .set_title("Save the text")
        .set_file_name(default_name)
        .save_file()
}

/* ---------------------------------------------------------------------------
 *  Refreshing the window
 * ------------------------------------------------------------------------- */

fn sync(ui: &MainWindow, notes: &Notes) {
    ui.set_doc_title(notes.title().into());
    ui.set_status(notes.status.clone().into());
    ui.set_position(notes.position_text().into());
    ui.set_dark(notes.dark);
}

fn sync_text(ui: &MainWindow, notes: &Notes) {
    ui.set_text(notes.text.clone().into());
    sync(ui, notes);
}

fn save(ui: &MainWindow, notes: &Rc<RefCell<Notes>>, save_as: bool) {
    let (known, default_name) = {
        let notes = notes.borrow();
        (notes.path.clone(), notes.file_name())
    };
    let chosen = if save_as || known.is_none() {
        let default_name = if default_name == "Untitled" {
            String::from("untitled.txt")
        } else {
            default_name
        };
        choose_save(default_name)
    } else {
        known
    };
    let Some(path) = chosen else {
        return;
    };
    let path = path.to_string_lossy().into_owned();
    notes.borrow_mut().save_to(&path);
    sync(ui, &notes.borrow());
}

fn open(ui: &MainWindow, notes: &Rc<RefCell<Notes>>) {
    if !may_replace(notes) {
        return;
    }
    let Some(path) = choose_open() else {
        return;
    };
    let path = path.to_string_lossy().into_owned();
    let opened = notes.borrow_mut().open_path(&path);
    if opened {
        sync_text(ui, &notes.borrow());
    } else {
        sync(ui, &notes.borrow());
    }
}

fn apply_action(ui: &MainWindow, notes: &Rc<RefCell<Notes>>, name: &str) {
    let Some(action) = Action::from_name(name) else {
        return;
    };
    match action {
        Action::New => {
            if may_replace(notes) {
                notes.borrow_mut().new_document();
                sync_text(ui, &notes.borrow());
            }
        }
        Action::Open => open(ui, notes),
        Action::Save => save(ui, notes, false),
        Action::SaveAs => save(ui, notes, true),
        Action::Quit => {
            if may_replace(notes) {
                let _ = ui.hide();
            }
        }
        Action::Undo => {
            ui.invoke_undo();
            notes.borrow_mut().status = Action::Undo.status();
            sync(ui, &notes.borrow());
        }
        Action::Redo => {
            ui.invoke_redo();
            notes.borrow_mut().status = Action::Redo.status();
            sync(ui, &notes.borrow());
        }
        Action::SelectAll => {
            ui.invoke_select_all();
            notes.borrow_mut().status = Action::SelectAll.status();
            sync(ui, &notes.borrow());
        }
        Action::Dark | Action::Light => {
            notes.borrow_mut().dark = action == Action::Dark;
            notes.borrow_mut().status = action.status();
            sync(ui, &notes.borrow());
        }
        Action::Shortcuts => {
            ui.set_help_title("Keyboard shortcuts".into());
            ui.set_help_text(shortcuts_text().into());
            ui.set_help_open(true);
        }
        Action::About => {
            ui.set_help_title("About Notes".into());
            ui.set_help_text(about_text().into());
            ui.set_help_open(true);
        }
    }
}

/* ---------------------------------------------------------------------------
 *  Wiring
 * ------------------------------------------------------------------------- */

fn wire(ui: &MainWindow, notes: &Rc<RefCell<Notes>>) {
    {
        let weak = ui.as_weak();
        let model = notes.clone();
        ui.on_action(move |name: SharedString| {
            if let Some(ui) = weak.upgrade() {
                apply_action(&ui, &model, name.as_str());
            }
        });
    }

    {
        let weak = ui.as_weak();
        let model = notes.clone();
        ui.on_edited(move |text| {
            model.borrow_mut().set_text(text.as_str());
            if let Some(ui) = weak.upgrade() {
                sync(&ui, &model.borrow());
            }
        });
    }

    {
        let weak = ui.as_weak();
        let model = notes.clone();
        ui.on_cursor_moved(move |offset| {
            model.borrow_mut().set_cursor(offset.max(0) as usize);
            if let Some(ui) = weak.upgrade() {
                ui.set_position(model.borrow().position_text().into());
            }
        });
    }

    {
        let weak = ui.as_weak();
        let model = notes.clone();
        ui.window().on_close_requested(move || {
            let allowed = match weak.upgrade() {
                Some(_) => may_replace(&model),
                None => true,
            };
            if allowed {
                CloseRequestResponse::HideWindow
            } else {
                CloseRequestResponse::KeepWindowShown
            }
        });
    }
}

/* ---------------------------------------------------------------------------
 *  Start
 * ------------------------------------------------------------------------- */

fn main() {
    if harness::is_test() {
        let path = "/tmp/slint_tutorial_08_rust.txt";
        let sample = "Notes\nsecond line\n";
        std::fs::write(path, sample).expect("write the sample");

        let mut notes = Notes::new();
        harness::check(
            notes.title() == "Untitled - Notes",
            "a new document has no file name",
        );
        harness::check(notes.open_path(path), "the sample opens");
        harness::check(notes.text == sample, "the model holds the file");
        harness::check(!notes.dirty, "opening does not modify");
        harness::check(
            notes.title() == "slint_tutorial_08_rust.txt - Notes",
            "the title names the file",
        );

        notes.set_text("changed text\n");
        harness::check(
            notes.dirty && notes.title().contains(" *"),
            "typing marks the title",
        );
        notes.set_cursor(13);
        harness::check(
            notes.position_text() == "Ln 2, Col 1 · 13 characters",
            "the cursor is a line and a column",
        );

        harness::check(
            !notes.can_discard(false),
            "dirty text cannot be discarded without consent",
        );
        harness::check(notes.can_discard(true), "the user can agree to discard");

        harness::check(notes.save_to(path), "the file saves");
        harness::check(!notes.dirty, "saving clears the mark");
        harness::check(
            std::fs::read_to_string(path).unwrap_or_default() == "changed text\n",
            "the editor text reached the disk",
        );

        notes.new_document();
        harness::check(
            notes.text.is_empty() && notes.path.is_none(),
            "New forgets text and path",
        );
        harness::check(notes.status == "New document", "New reports itself");

        harness::check(
            Action::from_name("file.save-as") == Some(Action::SaveAs),
            "Save as is an action",
        );
        harness::check(
            Action::from_name("nonsense").is_none(),
            "unknown names are ignored",
        );

        harness::check(
            !notes.open_path("/tmp/slint_tutorial_08_missing.txt"),
            "a missing file fails cleanly",
        );
        harness::check(
            notes.status.starts_with("Cannot read"),
            "the status explains the failure",
        );

        std::fs::remove_file(path).ok();
        std::process::exit(harness::finish("08-capstone-notes"));
    }

    if let Err(error) = run() {
        eprintln!("08-capstone-notes: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let notes = Rc::new(RefCell::new(Notes::new()));
    wire(&ui, &notes);

    let argument = std::env::args().skip(1).find(|arg| !arg.starts_with('-'));
    if let Some(path) = argument {
        let opened = notes.borrow_mut().open_path(&path);
        if opened {
            sync_text(&ui, &notes.borrow());
        }
    }
    sync(&ui, &notes.borrow());

    ui.run()
}

/* ---------------------------------------------------------------------------
 *  UI test: the real component, on Slint's testing backend, no display
 * ------------------------------------------------------------------------- */

#[cfg(test)]
mod ui_tests {
    use super::*;
    use i_slint_backend_testing::ElementHandle;

    #[test]
    fn the_window_is_wired_to_the_model() {
        i_slint_backend_testing::init_no_event_loop();
        let ui = MainWindow::new().expect("the component builds");
        let notes = Rc::new(RefCell::new(Notes::new()));
        wire(&ui, &notes);
        sync(&ui, &notes.borrow());

        assert_eq!(ui.get_doc_title().as_str(), "Untitled - Notes");

        // Put text in the input, then click the real toolbar button.
        ui.set_text("text to clear".into());
        let button = ElementHandle::find_by_accessible_label(&ui, "New document")
            .next()
            .expect("the New document button is reachable");
        button.invoke_accessible_default_action();

        assert_eq!(ui.get_text().as_str(), "", "the editor is cleared");
        assert_eq!(ui.get_status().as_str(), "New document");
        assert_eq!(notes.borrow().status, "New document");
        assert!(!notes.borrow().dirty);
    }
}
