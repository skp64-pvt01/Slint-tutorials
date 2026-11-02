# Step 07 — Text editor and files

**Level 4 · Advanced · ~45 minutes**

A minimal editor that opens and saves a text file, marks unsaved changes with
a `*` in the title, and reports the cursor position and the character count.

---

## What you will learn

| Piece | How |
|---|---|
| multiline input | `TextInput { single-line: false; wrap: word-wrap; }` |
| scrolling | `ScrollView` around the input; content height follows the text |
| cursor | `cursor-position-changed` + `cursor-position-byte-offset` |
| file read/write | `std::fs::read_to_string` / `std::fs::write` in the model |
| file dialogs | `rfd::FileDialog` — only from the window, never from the model |
| dirty title | `Notes::title()` adds ` *`; Rust sets `doc-title` |

### Why not `TextEdit`?

The `TextEdit` widget is shorter, but in Slint 1.18 it does not expose the
cursor offset, so a `Ln/Col` status is impossible. The lower-level `TextInput`
does expose it. The cost is that the editor must handle its own scrolling; the
`cursor-position-changed` block is the same arithmetic `TextEditBase` uses.

`cursor-position-byte-offset` is an internal property, documented as
"only exposed for tests". It works in 1.18.1 and the step is pinned to that
release; if a future Slint removes it, track the offset in `key-pressed` or
show only the character count.

---

## The model

```rust
struct Editor {
    path: Option<PathBuf>,
    text: String,
    dirty: bool,
    status: String,
}
```

The file functions are plain Rust and take a path:

```rust
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

fn save_to(&mut self, path: &str) -> bool { /* std::fs::write */ }
```

Nothing here opens a dialog, so `--test` can call both with a `/tmp` path.

### Dirty and cursor

```rust
fn set_text(&mut self, text: &str) {
    if self.text == text { return; }
    self.text = text.to_string();
    self.dirty = true;
    self.status = "Modified".into();
}
```

Only the user's edit calls `set_text`; loading a file writes `text` directly
and clears `dirty`. The UI's `edited` callback is not fired by programmatic
property changes, which is exactly what makes this work.

```rust
fn position_text(&self) -> String {
    let before = &self.text[..cursor];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap().chars().count() + 1;
    format!("Ln {line}, Col {column} · {} characters", self.text.chars().count())
}
```

Bytes vs characters matters: the cursor offset is a UTF-8 byte offset, the
column is a character count. Clamp to a char boundary before slicing.

### The dialogs

```rust
fn choose_open() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text", &["txt", "md"])
        .pick_file()
}
```

The `on_open_requested` closure asks for a path, calls `open_path`, then sets
the UI text and title. `on_save_requested` saves to the known path or asks for
one. Tests never reach these closures.

---

## Build, run, test

```bash
cargo build -p step-07-text-editor
cargo run  -p step-07-text-editor
cargo run  -p step-07-text-editor -- --test
```

```
07-text-editor: 13 checks, 0 failures
PASS
```

The checks write a real file in `/tmp`, open it, edit it, check the `*`, save,
re-read the bytes, and check `Ln/Col` at the start and after a newline. They
also open a missing path and assert the error status.

`cargo test -p step-07-text-editor` runs them through Cargo.

---

## Checkpoint

* Why does opening a file not mark it dirty?
* Which function writes the file, and what does it return on failure?
* Why is the column computed in `chars()` while the cursor is a byte offset?
* Why must `rfd` stay out of the model?

---

## Exercises

1. Add a **Save as** button that always opens the save dialog.
2. Add a **New** button (clear the text, path and dirty flag).
3. Show the line count as well as the character count.
4. Refuse to overwrite a file whose content did not change (compare before
   writing).
5. Add a `Ctrl+S` shortcut with a `FocusScope` around the editor. Why does the
   `TextInput` need `capture-key-pressed` for `Ctrl+A`?

---

## Troubleshooting

| Problem | Fix |
|---|---|
| The editor does not scroll | `content-height: max(visible-height, text-input.preferred-height)` is missing, or the `TextInput` has a fixed height. |
| The cursor status is wrong after a multi-byte character | Slice at a char boundary; the code walks back with `is_char_boundary`. |
| Opening sets the dirty flag | `edited` fired on a programmatic change. Only user edits should call `set_text`. |
| A dialog opens during `cargo test` | The test called a UI callback instead of `open_path`/`save_to`. |
| `rfd` fails at startup on a headless machine | It is only called from a button; do not call it in `run()` or in a test. |
| The title never updates | `show()` sets `doc-title`, but it was not called after the model changed. |

**Previous:** [Step 06 — Menus](../06-menus/README.md) ·
**Next:** [Step 08 — Capstone: Notes](../08-capstone-notes/README.md)
