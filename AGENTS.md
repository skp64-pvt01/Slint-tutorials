# AGENTS.md — Slint Tutorials

One Rust course on [Slint](https://slint.dev/) 1.18.1, modelled on the
pedagogy of Raven Tutorials: eight learning steps and one packaging step, each
a complete project that runs and checks itself without a display.

The course is standalone. Slint is a normal crates.io dependency; there is no
Raven dependency, source or library in this repository.

---

## What is here

| Path | What it is |
|---|---|
| `Cargo.toml` | the workspace and the shared Slint versions |
| `01-first-window` … `04-actions` | inline `slint!` UI, no `build.rs` |
| `05-slint-files` … `08-capstone-notes` | `build.rs` + `ui/*.slint` + `include_modules!` |
| `08-capstone-notes` | menus, toolbar, editor, unsaved-changes guard, data file, UI test |
| `09-packaging` | CMake project: cargo + install + CPack (DEB, TGZ); **not** a Cargo member |
| `README.md` | the course overview and the commands |
| `DESIGN.md` | architecture, step shape, decisions, testing |
| `LICENSE` | GPL-3.0-only, copied from Slint's `LICENSES/GPL-3.0-only.txt` |
| `NOTICE` | provenance and the Slint royalty-free alternative |

Every step has `src/main.rs`, `src/harness.rs`, `tests/check.rs` and a
`README.md`. Steps 05–08 also have `build.rs` and `ui/`.

## Prerequisites

* Rust 1.92 or newer (Slint 1.18.1's minimum).
* No system Slint package: Cargo fetches `slint` and `slint-build` from
  crates.io.
* A desktop session only for a real window. `cargo test` and `--test` are
  headless.
* CMake 3.20+ and `dpkg-deb`/`dpkg-shlibdeps` for step 09.

## Build and test

```bash
cargo build --workspace
cargo test --workspace
env -u DISPLAY -u WAYLAND_DISPLAY cargo test --workspace
```

Expected: nine tests — eight `tests/check.rs` spawns plus the UI test in step
08. Each spawned binary prints its own `N checks, 0 failures` and `PASS`.

Run one step and its check:

```bash
cargo run -p step-03-controls
cargo run -p step-03-controls -- --test
cargo test -p step-08-capstone-notes
```

Packaging (the build folder may live in `/tmp`; cargo's folder can be moved):

```bash
cmake -S 09-packaging -B /tmp/notes -DCMAKE_INSTALL_PREFIX=/usr \
  -DNOTES_CARGO_TARGET_DIR=/tmp/notes-cargo
cmake --build /tmp/notes
(cd /tmp/notes && cpack -G DEB)
(cd /tmp/notes && cpack -G TGZ)
```

## Rules

1. **The `.slint` is layout only.** No file I/O, no formatting, no business
   logic in a Slint callback. The callback calls a Rust method.
2. **Rust sets every display string.** The UI may concatenate two strings for
   layout, but titles, statuses and counters are formatted in Rust, so the
   model tests see exactly what the user sees.
3. **One action vocabulary.** A name (`"file.save"`) becomes an `Action` in
   Rust; a button, a menu entry and a shortcut all call the same dispatcher.
4. **Model functions never open a dialog.** `open_path`, `save_to`,
   `new_document` and `can_discard` take values, not windows. `rfd` is called
   only from UI callbacks, so `--test` cannot block on a modal dialog.
5. **A step is standalone.** `src/harness.rs` is copied per package on purpose;
   do not move it into a shared crate.
6. **Every behaviour gets a check** in the step's `--test` path. Pixel checks
   are not part of the course; the one UI test asserts state after a real click.
7. **Tests write only to `/tmp`.** The sample files are removed afterwards.
8. **Do not commit `target/` or `build/`.** Both are in `.gitignore`.

## Pitfalls

| Symptom | Cause and fix |
|---|---|
| `Cannot specify both 'width' and 'min-width'` | A `Window` sets its size one way. Use `width`/`height` **or** the `min-*` pair; step 08 dropped the minimums. |
| `Cannot convert float to length` | `length` and `float` are different types. Multiply by `1px`: `font-size: root.size * 1px`. |
| The UI test cannot find an element by label | `ElementHandle` needs debug info. Step 08's `build.rs` uses `compile_with_config(..., CompilerConfiguration::new().with_debug_info(true))`; the other steps do not need it. |
| `ElementHandle` panics after another test | The testing backend can be initialised once per process. Keep the UI test a single `#[test]` and call `init_no_event_loop()` once. |
| A headless test tries to open a window | `--test` must not call `MainWindow::new()`. It runs the Rust model; only the `#[cfg(test)]` UI test builds a component, on the testing backend. |
| An `rfd` dialog opens during `cargo test` | A model function called a dialog, or a test drove a UI callback that does. Keep dialogs in `apply_action` / toolbar callbacks; call `open_path` and `save_to` in tests. |
| `edited` says it has 0 arguments | The builtin `TextInput` has `edited` with no arguments (bind `text`); the `TextEdit` **widget** has `edited(string)`. Steps 07–08 use `TextInput` and `edited => { root.edited(self.text); }`. |
| Keyboard shortcuts stop after clicking the editor | A `FocusScope` sees only keys the focused child rejects; `capture-key-pressed` sees them first. In step 08 the menu shortcuts are native (`MenuBar`), so this does not apply; step 04 uses a scope with no text input. |
| A menu bar does not appear | `MenuBar` must be a direct child of `Window`, outside the page layout. Right-click menus are `ContextMenuArea`, not a hand-rolled `if` overlay. |
| Path commands draw nothing | `Path` coordinates are in the `Path`'s own width × height canvas, not window coordinates. Keep the commands and the `Path` size in step; this course does not use `Path`, but the Workbench does. |
| A numeric field cannot hold `2.5` | Slint's `SpinBox` is integer-only. Use a `LineEdit` and parse/clamp in Rust (this course does not need a number field). |
| `cursor-position-byte-offset` is not a documented property | It is an internal `TextInput` property, exposed for tests. Steps 07–08 use it to compute `Ln/Col`. If a future Slint removes it, track the offset in `key-pressed` or show only the character count. |
| The window opens on a machine with no display | Expected: use `--test`, or `cargo test`. `SLINT_BACKEND=software` helps on thin clients but does not create a display. |

## Adding a step

1. Copy the nearest step's `Cargo.toml`, `src/harness.rs` and
   `tests/check.rs`; rename the package and the `[[bin]]`.
2. Add the package to the workspace `members` in `Cargo.toml`.
3. Keep the model in `main.rs`; add checks to `--test` for every new branch.
4. If the step needs a `.slint` file, add `build.rs` and `ui/`; otherwise keep
   the inline `slint!`.
5. Update the step README and the table in the top `README.md`.
6. Run the three commands above, including the `env -u DISPLAY` one.

## Definition of done

* `cargo build --workspace` with **zero warnings**.
* `cargo test --workspace` passes, headless, nine tests.
* Every `--test` prints `PASS`, exit 0.
* `09-packaging` configures, builds and produces a DEB.
* README / AGENTS / DESIGN updated together.
* Nothing built or temporary is staged.
