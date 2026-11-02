# Step 08 — Capstone: Notes

**Level 4 · Advanced · ~1 hour**

Everything the seven steps taught, in the shape of a real editor: menus, a
toolbar, shortcuts, open/save, an unsaved-changes guard, a status bar, a dirty
title and a Help overlay.

---

## What you will learn

| Piece | Where |
|---|---|
| one action dispatcher for menus + toolbar | `Action` / `apply_action` |
| unsaved-changes guard | `Notes::can_discard(confirmed)` + `may_replace` |
| close request | `ui.window().on_close_requested(...)` |
| data file next to the binary | `data/shortcuts.txt`, `NOTES_DATA_DIR` |
| real UI test | `#[cfg(test)] mod ui_tests` on `i-slint-backend-testing` |

---

## The window

```
MenuBar            File  Edit  View  Help
toolbar            New document | Open... | Save
editor             TextInput (multiline) in a ScrollView
status bar         message on the left, Ln/Col · characters on the right
Help overlay       shortcuts or About, loaded from data
```

The menu and the toolbar call the same callback:

```slint
callback action(string);

MenuItem { title: "Save"; shortcut: @keys(Control + S);
           activated => { root.action("file.save"); } }
Button  { text: "Save"; clicked => { root.action("file.save"); } }
```

`apply_action` matches once:

```rust
match action {
    Action::New  => if may_replace(notes) { ... },
    Action::Open => open(ui, notes),
    Action::Save => save(ui, notes, false),
    Action::Quit => if may_replace(notes) { ui.hide() },
    ...
}
```

`Edit > Undo / Redo / Select all` reach public Slint functions:

```slint
public function undo() { text-input.undo(); }
```

```rust
ui.invoke_undo();
```

so the menu still has one entry point, while the editing itself stays in the
widget.

### The unsaved-changes guard

The model has the pure predicate:

```rust
fn can_discard(&self, confirmed: bool) -> bool {
    !self.dirty || confirmed
}
```

The window supplies the answer from a dialog:

```rust
fn may_replace(notes: &Rc<RefCell<Notes>>) -> bool {
    let dirty = notes.borrow().dirty;
    if !dirty { return true; }
    let confirmed = confirm_discard();   // rfd, no borrow held
    notes.borrow().can_discard(confirmed)
}
```

New, Open and Quit call `may_replace` first. The window's close button is
handled too:

```rust
ui.window().on_close_requested(move || {
    if allowed { CloseRequestResponse::HideWindow }
    else { CloseRequestResponse::KeepWindowShown }
});
```

The dialog is opened outside any `RefCell` borrow, so a re-entrant callback
cannot panic.

### The data file

`data/shortcuts.txt` is the text behind **Help > Keyboard shortcuts**:

```rust
fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("NOTES_DATA_DIR") { return dir.into(); }
    match option_env!("NOTES_DATA_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data"),
    }
}
```

Step 09 compiles `NOTES_DATA_DIR=/usr/share/notes` into the binary and installs
the file there. A missing file falls back to a built-in string, so the app
never fails over a help page.

### Drag and drop

Slint 1.18's `DropArea` accepts its own `data-transfer` payload, but reading
external file paths from a desktop drop is backend-dependent. This course
leaves it out rather than shipping a feature that works on one compositor.
The exercise below suggests the experiment; Raven Tutorials' Rust course has
the equivalent feature for comparison.

---

## Run

```bash
cargo run -p step-08-capstone-notes
cargo run -p step-08-capstone-notes -- notes.txt
```

`Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+Shift+S`, `Ctrl+Q`; `F1` for shortcuts.
The title shows the file name and ` *` when the text is modified.

## Test

```bash
cargo run -p step-08-capstone-notes -- --test
```

```
08-capstone-notes: 18 checks, 0 failures
PASS
```

The model checks cover open, edit, dirty title, cursor line/column, the guard
with and without consent, save, re-read, New, unknown actions and a missing
file. Then the package adds one UI test:

```rust
#[test]
fn the_window_is_wired_to_the_model() {
    i_slint_backend_testing::init_no_event_loop();
    let ui = MainWindow::new().unwrap();
    let notes = Rc::new(RefCell::new(Notes::new()));
    wire(&ui, &notes);
    sync(&ui, &notes.borrow());
    ui.set_text("text to clear".into());

    let button = ElementHandle::find_by_accessible_label(&ui, "New document")
        .next().expect("the toolbar button");
    button.invoke_accessible_default_action();

    assert_eq!(ui.get_text().as_str(), "");
    assert_eq!(notes.borrow().status, "New document");
}
```

It builds the **real** component, runs the **real** `wire`, and clicks a real
button through accessibility. `build.rs` compiles the UI with debug info so the
element can be found. The testing backend is process-global, so this must stay
one `#[test]`.

```bash
cargo test -p step-08-capstone-notes   # the model check + the UI test
```

---

## Checkpoint

* Why is `may_replace` a free function instead of a method holding a borrow?
* What happens on window close when the document is dirty?
* Where does `NOTES_DATA_DIR` come from at compile time, and how can it be
  overridden at run time?
* Why is the UI test a single `#[test]`?

---

## Exercises

1. Add a **Recent files** submenu. Where does the list live — the model or the
   UI? How do you rebuild a `Menu` from Rust? (Hint: Slint menus are static;
   use a toolbar/list, or rebuild the component.)
2. Add **Edit > Find** with a second `LineEdit` overlay, and highlight the
   first match.
3. Add a "Wrap lines" toggle. `TextInput.wrap` is `word-wrap` / `no-wrap`.
4. Try `DropArea { dropped(event) => { ... } }` with a `data-transfer` from
   Rust. Check which backends deliver external files before keeping it.
5. Add a `--version` flag that prints the same text as Help > About.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| The New button in the UI test is not found | `build.rs` must use `with_debug_info(true)`, and the button text/accessibility label must match exactly (`"New document"`). |
| The UI test panics with "platform already initialized" | Another test or a previous run initialised the backend in the same process. Keep one `#[test]`. |
| A dialog opens during a test | A test called `apply_action` or a UI callback. Call `open_path`, `save_to`, `new_document`, `can_discard` instead. |
| Closing the window ignores the dirty flag | `on_close_requested` was not installed, or it always returns `HideWindow`. |
| The installed binary cannot find `shortcuts.txt` | It was built with another prefix. Set `NOTES_DATA_DIR` at run time, or configure with `-DCMAKE_INSTALL_PREFIX=/usr`. |
| The editor is empty after opening a file | `sync_text` was not called; setting the model alone does not update the UI. |

**Previous:** [Step 07 — Text editor](../07-text-editor/README.md) ·
**Next:** [Step 09 — Packaging](../09-packaging/README.md)
