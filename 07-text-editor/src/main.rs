//! Slint tutorial 07 (Rust) - a text editor and files.
//!
//! The editor shows `TextInput { single-line: false; }` inside a `ScrollView`.
//! `TextEdit` would be shorter, but it does not expose the cursor offset; the
//! lower-level input does, so the status bar can say `Ln 2, Col 4`.
//!
//! Files are plain Rust: `std::fs::read_to_string` and `std::fs::write` in
//! `open_path` / `save_to`. The dialogs are only the way a user picks a path,
//! and `--test` calls the model functions directly, so no dialog ever opens
//! during a test.

mod harness;

slint::include_modules!();

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

struct Editor {
    path: Option<PathBuf>,
    text: String,
    dirty: bool,
    status: String,
}

impl Editor {
    fn new() -> Self {
        Self {
            path: None,
            text: String::new(),
            dirty: false,
            status: String::from("Ready"),
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
            "{}{} - Tutorial 07",
            self.file_name(),
            if self.dirty { " *" } else { "" }
        )
    }

    fn open_path(&mut self, path: &str) -> bool {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                self.path = Some(PathBuf::from(path));
                self.text = text;
                self.dirty = false;
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

    /// Called for a user edit, never for text loaded from a file.
    fn set_text(&mut self, text: &str) {
        if self.text == text {
            return;
        }
        self.text = text.to_string();
        self.dirty = true;
        self.status = String::from("Modified");
    }

    /// `offset` is a byte offset into the UTF-8 text. A test can pass 0; the
    /// UI passes the cursor offset reported by `TextInput`.
    fn position_text(&self, offset: usize) -> String {
        let mut offset = offset.min(self.text.len());
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

fn show(ui: &MainWindow, editor: &Editor) {
    ui.set_doc_title(editor.title().into());
    ui.set_status(editor.status.clone().into());
}

fn choose_open() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text", &["txt", "md"])
        .set_title("Open a text")
        .pick_file()
}

fn choose_save(default_name: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text", &["txt", "md"])
        .set_title("Save the text")
        .set_file_name(default_name)
        .save_file()
}

fn wire(ui: &MainWindow, editor: &Rc<RefCell<Editor>>) {
    let weak = ui.as_weak();
    let model = editor.clone();
    ui.on_edited(move |text| {
        model.borrow_mut().set_text(text.as_str());
        if let Some(ui) = weak.upgrade() {
            show(&ui, &model.borrow());
        }
    });

    let weak = ui.as_weak();
    let model = editor.clone();
    ui.on_cursor_moved(move |offset| {
        if let Some(ui) = weak.upgrade() {
            ui.set_position(model.borrow().position_text(offset.max(0) as usize).into());
        }
    });

    let weak = ui.as_weak();
    let model = editor.clone();
    ui.on_open_requested(move || {
        let Some(path) = choose_open() else {
            return;
        };
        let path = path.to_string_lossy().into_owned();
        let mut editor = model.borrow_mut();
        let opened = editor.open_path(&path);
        let text = editor.text.clone();
        let status = editor.status.clone();
        drop(editor);
        if let Some(ui) = weak.upgrade() {
            if opened {
                ui.set_text(text.into());
                ui.set_position(model.borrow().position_text(0).into());
            }
            ui.set_status(status.into());
            if opened {
                ui.set_doc_title(model.borrow().title().into());
            }
        }
    });

    let weak = ui.as_weak();
    let model = editor.clone();
    ui.on_save_requested(move || {
        let known = model.borrow().path.clone();
        let chosen = match known {
            Some(path) => Some(path),
            None => choose_save("untitled.txt"),
        };
        let Some(path) = chosen else {
            return;
        };
        let path = path.to_string_lossy().into_owned();
        model.borrow_mut().save_to(&path);
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
        let path = "/tmp/slint_tutorial_07_rust.txt";
        let sample = "first line\nsecond line\n";
        std::fs::write(path, sample).expect("write the sample");
        let mut editor = Editor::new();
        harness::check(editor.open_path(path), "the file opens");
        harness::check(editor.text == sample, "the editor holds the file");
        harness::check(!editor.dirty, "opening a file is not a modification");
        harness::check(
            editor.title() == "slint_tutorial_07_rust.txt - Tutorial 07",
            "the title names the file",
        );

        editor.set_text("typed line\n");
        harness::check(editor.dirty, "typing sets the dirty flag");
        harness::check(
            editor.title().contains(" *"),
            "the title marks a dirty file",
        );
        harness::check(
            editor.position_text(0) == "Ln 1, Col 1 · 11 characters",
            "the position counts characters",
        );
        harness::check(
            editor.position_text(11) == "Ln 2, Col 1 · 11 characters",
            "a newline starts a new line",
        );

        harness::check(editor.save_to(path), "the file saves");
        harness::check(!editor.dirty, "saving clears the dirty flag");
        harness::check(
            std::fs::read_to_string(path).unwrap_or_default() == "typed line\n",
            "the typed text reached the disk",
        );

        editor.set_text("");
        harness::check(
            !editor.open_path("/tmp/slint_tutorial_07_missing.txt"),
            "a missing file fails cleanly",
        );
        harness::check(
            editor.status.starts_with("Cannot read"),
            "the status explains the failure",
        );

        std::fs::remove_file(path).ok();
        std::process::exit(harness::finish("07-text-editor"));
    }

    if let Err(error) = run() {
        eprintln!("07-text-editor: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let editor = Rc::new(RefCell::new(Editor::new()));
    wire(&ui, &editor);
    ui.set_position(editor.borrow().position_text(0).into());
    ui.run()
}
