# DESIGN.md — Slint Tutorials

How this course is put together and why. It is written for the next developer
(or agent) who has to extend or teach it.

---

## 1. Goals and non-goals

**Goals**

* teach Slint and Rust together, from a first window to a small editor;
* one application, built up over eight steps, that a reader can run at every
  point;
* every step checkable with no display and no GPU;
* a reader can start at step 01 without GUI experience;
* packaging explained once, by example.

**Non-goals**

* a general-purpose Slint framework or component library;
* replacing Slint's documentation;
* a second language: Slint is Rust-first here, and there is one course;
* a package repository or CI matrix;
* pixel-perfect visual tests.

---

## 2. Shape

One Cargo workspace with eight member packages and one CMake project:

| Level | Steps | Theme |
|---|---|---|
| 1 — Beginner | 01, 02 | window, callbacks, layout |
| 2 — Elementary | 03, 04 | controls and state, actions and shortcuts |
| 3 — Intermediate | 05, 06 | `.slint` files and components, menus |
| 4 — Advanced | 07, 08, 09 | text editor and files, capstone, packaging |

The application grows into **Notes**: menus, shortcuts, open/save, a dirty
flag, an unsaved-changes guard and a status bar. Step 09 turns it into a DEB
and a TGZ.

| Step | Model checks (`--test`) | UI tests |
|---|---|---|
| 01 | 5 | — |
| 02 | 5 | — |
| 03 | 6 | — |
| 04 | 7 | — |
| 05 | 4 | — |
| 06 | 10 | — |
| 07 | 13 | — |
| 08 | 18 | 1 |

---

## 3. Anatomy of a step

```
0N-name/
├── Cargo.toml          package + [[bin]] with the step's name
├── src/main.rs         the program: UI (01–04) or include_modules! (05–08)
├── src/harness.rs      check counter, PASS summary
├── tests/check.rs      spawns the binary with `--test`
├── ui/ + build.rs      steps 05–08 only
└── README.md           goal, walkthrough, commands, checkpoint, exercises
```

Each step is a complete project: `src/harness.rs` is intentionally repeated
rather than moved to a shared crate, so a step folder can be copied out and
built alone.

---

## 4. Architecture

### UI and model

```
.slint component                     Rust
──────────────                       ────
property <string> status      ←──    Notes::status
callback action(string)       ──→    apply_action()
callback edited(string)       ──→    Notes::set_text()
property <string> text        ←──    Notes::text
```

Rules:

* the `.slint` file is layout only;
* Rust owns the counters, the form, the file path, the dirty flag and every
  formatted string;
* Rust sets properties after a model change (`sync`, `sync_text` in step 08);
* callbacks carry values (`edited(text)`, `changed(name, shout, size, theme)`)
  so Rust does not have to read the UI back.

### State ownership

`Rc<RefCell<Model>>` is enough for this course. Callbacks are `'static`; each
one owns a clone of the `Rc`. Borrows are short and never held across a modal
dialog or across another callback.

### Actions

Steps 04, 06 and 08 use the same idea:

```rust
enum Action { New, Open, Save, ... }
Action::from_name("file.save") -> Option<Action>
```

A button and a menu entry call one Slint callback with a name; Rust matches
once. In step 08 the action match also owns the dialogs and the guard.

### Files

`open_path` and `save_to` are methods on the model and use `std::fs` directly.
They take a path, never show a dialog, and return a `bool` plus a status text.
The window turns an `rfd` result into a path and then calls them.

---

## 5. Decisions

* **Inline `slint!` first, files at step 05.** Steps 01–04 keep the interface
  next to the callbacks, so the first program is one file. Step 05 introduces
  `build.rs`, `include_modules!`, imports and reusable components — the Slint
  equivalent of the Raven XML step.
* **The model is plain Rust.** `--test` does not create a component, so it
  needs no backend, no window and no fonts. It tests the same code the UI
  callbacks call. Slint's own UI coverage is one test on the testing backend.
* **The low-level `TextInput` in 07–08.** `TextEdit` is shorter, but it does
  not expose the cursor offset; `TextInput { single-line: false; }` does, and
  the status bar can show `Ln/Col`. The `ScrollView` follows the cursor with
  the same arithmetic as `TextEditBase`.
* **`rfd` for dialogs, `std::fs` for files.** Slint deliberately has no file
  dialog. `rfd` is the smallest crate that works on the three desktops. It is
  used only from UI callbacks.
* **No drag and drop.** Slint 1.18 has `DragArea`/`DropArea` for its own
  `data-transfer` payload, but reading external file paths from a drop is
  backend-dependent. The Raven course drops a file on the window; this step
  does not, and the README says so rather than shipping a fragile feature.
* **GPL-3.0-only.** Slint is dual-licensed; this course chooses the GPL option
  and the license text is copied from Slint's `LICENSES/GPL-3.0-only.txt`.
  The royalty-free alternative is noted in `NOTICE` and the README.

---

## 6. Testing

Three layers, all headless:

1. **`--test` in each binary.** Builds no component. Runs the model: counters,
   form strings, action dispatch, file I/O to `/tmp`, the dirty flag, the guard
   predicate. Prints `N checks, 0 failures` and `PASS`, exits 0/1.
2. **`tests/check.rs` in each package.** Spawns `env!("CARGO_BIN_EXE_...")`
   with `--test`; a non-zero exit fails the Cargo test. That is 8 tests.
3. **One UI test in step 08.** `i-slint-backend-testing`'s
   `init_no_event_loop()`, the real `MainWindow`, the real `wire`, then
   `ElementHandle::find_by_accessible_label(&ui, "New document")` and
   `invoke_accessible_default_action()`. It asserts the editor cleared and the
   model status changed. One `#[test]` because the backend is process-global.

`cargo test --workspace` therefore runs **9 tests**. The headless proof is:

```bash
env -u DISPLAY -u WAYLAND_DISPLAY cargo test --workspace
```

Tests write only to `/tmp/slint_tutorial_0N_*.txt` and remove the files.

---

## 7. File map

| Path | Contents |
|---|---|
| `Cargo.toml` | workspace members and shared dependency versions |
| `Cargo.lock` | locked Slint 1.18.1 and the rest |
| `0N-*/src/main.rs` | the step: model + wiring (+ inline UI in 01–04) |
| `0N-*/src/harness.rs` | check counter, `require`, `finish` |
| `0N-*/tests/check.rs` | spawns the step with `--test` |
| `05-…/build.rs` | `slint_build::compile("ui/main.slint")` |
| `05-…/ui/main.slint` | the window, importing components |
| `05-…/ui/components/*.slint` | reusable header / status bar |
| `08-…/build.rs` | compile with debug info for `ElementHandle` |
| `08-…/ui/main.slint` | menus, toolbar, editor, status bar, help overlay |
| `08-…/data/shortcuts.txt` | installed data; read by Help > Keyboard shortcuts |
| `09-packaging/CMakeLists.txt` | cargo + install + CPack |
| `09-packaging/notes.desktop` | the menu entry |
| `README.md`, `AGENTS.md`, `DESIGN.md` | course docs |
| `LICENSE`, `NOTICE` | GPL-3.0-only and provenance |

---

## 8. Limits and future work

* Tested on Linux. The code is portable, but the DEB and desktop entry are
  Linux-specific.
* `cursor-position-byte-offset` is an internal `TextInput` property; pinning
  Slint 1.18.1 keeps it stable. A future port may track the offset differently.
* The editor has no syntax highlighting, search, recent files or drag/drop.
  Those are named as exercises rather than shipped half-working.
* Step 08's UI test clicks one control; it does not simulate typing (the
  testing backend exposes no public keyboard injection in 1.18). Typing paths
  are covered by the model checks.
