# Slint Tutorials

A graded Rust course that takes you from an empty `fn main()` to a notes editor
with menus, keyboard shortcuts, native file dialogs, an unsaved-changes guard
and a status bar — built with [Slint](https://slint.dev/) 1.18.1.

Eight learning steps, each a Cargo package with its own README and a headless
`--test`, plus a ninth step that turns step 08 into a DEB and a TGZ. Every step
builds, runs and checks itself without a display, so the course works on a
machine with no GPU and no desktop session.

The course is inspired by [Raven Tutorials](https://github.com/skp64-pvt01/Raven-tutorials),
but it is standalone: Slint comes from crates.io, and there is no Raven
dependency, source or library here.

---

## Before you start

| Need | Why |
|---|---|
| **Rust 1.92 or newer** | `cargo` and `rustc`. Slint 1.18.1 needs 1.92. |
| **Slint 1.18.1** | Fetched automatically by Cargo; no native package to install. |
| **CMake 3.20+** | Only for step 09 (packaging). |
| **A desktop session** | Only to open a real window. `--test` and `cargo test` need neither a display nor a GPU. |

On Linux the default Slint backend pulls in winit and X11/Wayland; the
software renderer is used when no GPU is available. Nothing beyond the usual
C toolchain and font packages is required.

You should be comfortable with ownership, `Rc`/`RefCell` and closures. The
course explains the Slint side; it does not teach Rust from zero.

---

## Build and test the whole course

```bash
cargo build --workspace
cargo test --workspace
env -u DISPLAY -u WAYLAND_DISPLAY cargo test --workspace   # proves it is headless
```

Expected at the end: **9 tests pass** — one `tests/check.rs` per step, plus the
UI test in step 08 — and each binary prints its own check summary:

```
08-capstone-notes: 18 checks, 0 failures
PASS
```

Run one step:

```bash
cargo run -p step-03-controls               # a window
cargo run -p step-03-controls -- --test     # no window, no GPU
```

Step 09 is not a Cargo member: it is a CMake project that builds step 08 with
cargo and packages it with CPack. See
[09-packaging/README.md](09-packaging/README.md).

---

## The course

| Level | Step | You build | You learn |
|---|---|---|---|
| **1 — Beginner** | [01](01-first-window/README.md) | A window with a label and two buttons | `slint!`, properties, callbacks, Rust-owned state |
| | [02](02-layout/README.md) | A header/content/footer page | `VerticalLayout`, `HorizontalLayout`, stretch, spacing, padding |
| **2 — Elementary** | [03](03-controls/README.md) | A form that updates a preview | `LineEdit`, `CheckBox`, `Slider`, `ComboBox`, one update callback |
| | [04](04-actions/README.md) | A counter with shortcuts | An `Action` enum, one dispatcher, `FocusScope`, `@keys`-style keys |
| **3 — Intermediate** | [05](05-slint-files/README.md) | The same interface in `.slint` files | `build.rs`, `include_modules!`, imports, reusable components |
| | [06](06-menus/README.md) | A native menu bar | `MenuBar`, `Menu`, `MenuItem`, separators, menu shortcuts |
| **4 — Advanced** | [07](07-text-editor/README.md) | An editor that opens and saves | `TextInput` multiline, cursor status, `open_path`/`save_to`, dirty title |
| | [08](08-capstone-notes/README.md) | **Notes**, the starter project | Menus + toolbar + shortcuts, unsaved-changes guard, close request, UI test |
| | [09](09-packaging/README.md) | **A DEB and a TGZ of Notes** | CMake driving cargo, `install()`, CPack, DEB metadata, data files |

Estimated time: half a day for levels 1–2, half a day for level 3, an
afternoon for level 4 with the exercises.

---

## The two ways to run every step

```bash
cargo run -p step-05-slint-files            # opens a window
cargo run -p step-05-slint-files -- --test  # model checks, no window
```

`--test` never initialises a Slint backend and never creates a component. It
runs the same Rust model the UI callbacks call: counters, form values, action
dispatch, file reads and writes, the dirty flag. That is why it works on a
machine with no display and why it cannot be flaky.

The UI itself is checked where it matters: step 08 has one test on
`i-slint-backend-testing` that builds the real `MainWindow`, wires the real
callbacks and clicks the real **New document** button.

---

## Conventions used in the code

* **The `.slint` file is layout only.** State and behaviour live in a Rust
  struct (`Counter`, `Form`, `Notes`). The UI exposes properties and callbacks;
  Rust sets every display string.
* **One callback, one dispatcher.** Steps 04, 06 and 08 turn a name into an
  `Action` and match once. A button, a menu entry and a shortcut share it.
* **`--test` is part of the program.** Each `src/harness.rs` counts checks and
  prints `PASS`/`FAIL`. No test framework is needed for the binary.
* **Each step is standalone.** `src/harness.rs` is repeated in every package on
  purpose: a step folder can be copied out and still builds.
* **Files are plain Rust.** `std::fs::read_to_string` / `std::fs::write` live in
  the model. `rfd` supplies the dialogs, and only the window calls it.
* **GPL-3.0-only.** See [LICENSE](LICENSE) and [NOTICE](NOTICE).

---

## Packaging

```bash
cmake -S 09-packaging -B /tmp/notes -DCMAKE_INSTALL_PREFIX=/usr
cmake --build /tmp/notes
(cd /tmp/notes && cpack -G DEB)   # also produces TGZ; `cpack -G TGZ` on its own
```

Expected: `notes_1.0.0_amd64.deb` containing

```
/usr/bin/notes
/usr/share/notes/shortcuts.txt
/usr/share/applications/notes.desktop
```

The binary is built for `/usr`, so `NOTES_DATA_DIR` is compiled in as
`/usr/share/notes`. A staged or archive run can override it at run time:

```bash
NOTES_DATA_DIR=/tmp/stage/usr/share/notes /tmp/stage/usr/bin/notes --test
```

---

## Layout

```
Slint-tutorials/
├── Cargo.toml               workspace + shared dependency versions
├── Cargo.lock               locked Slint 1.18.1
├── 01-first-window/         one package per step:
│   ├── Cargo.toml
│   ├── src/main.rs          the step
│   ├── src/harness.rs       check counter and PASS summary
│   ├── tests/check.rs       spawns the binary with --test
│   └── README.md
├── 02-layout/ ... 07-text-editor/
├── 05-slint-files/          first step with build.rs + ui/ (through 08)
├── 08-capstone-notes/       menus, toolbar, editor, guard, data/, UI test
├── 09-packaging/            CMake + CPack (not a Cargo member)
├── README.md                this file
├── AGENTS.md                build/test hand-off and the traps
├── DESIGN.md                architecture and decisions
├── LICENSE                  GPL-3.0-only
└── NOTICE                   licensing and provenance
```

---

## License

The course is **GPL-3.0-only**, the Slint license option it uses. Slint is
also available under the [Slint Royalty-free license](https://slint.dev/pricing);
choosing that option for your own application is allowed by Slint's terms but
would change the license of the combined work. See [`NOTICE`](NOTICE).
