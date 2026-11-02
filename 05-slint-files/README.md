# Step 05 — Slint files and components

**Level 3 · Intermediate · ~30 minutes**

The interface moves out of `main.rs`. `build.rs` compiles `ui/main.slint`;
`slint::include_modules!()` pastes the generated Rust type into the crate. The
window is split into reusable components.

---

## What you will learn

| Piece | What it does |
|---|---|
| `build.rs` | `slint_build::compile("ui/main.slint")` before Rust compiles |
| `slint::include_modules!()` | generates `MainWindow` from the exported component |
| `import { ... } from "..."` | relative to the importing `.slint` file |
| `export component` | the public type, its properties and callbacks |
| `@children` (not used here) | where a wrapper places its content |

Steps 01–04 used `slint! { ... }`, which compiles a UI string inside
`main.rs`. That is fine for a demo; a real interface belongs in files that the
language server, the viewer and `git diff` can see.

---

## The files

```
05-slint-files/
├── build.rs
├── ui/
│   ├── main.slint
│   └── components/
│       ├── header.slint
│       └── status-bar.slint
└── src/main.rs
```

`ui/main.slint` imports the two components:

```slint
import { SectionHeader } from "components/header.slint";
import { StatusBar } from "components/status-bar.slint";
```

`components/header.slint` exports a component with only what its parent needs:

```slint
export component SectionHeader inherits Rectangle {
    in property <string> title;
    in property <string> subtitle;
    height: 56px;
    ...
}
```

The height is inside the component: the parent places it in a layout and does
not have to know how tall a header is.

### One string, two sides

The name is a one-way property; Rust reads it when the button is pressed:

```slint
LineEdit {
    text: root.name;
    edited(text) => { root.name = text; }
    accepted(text) => { root.name = text; root.greet(); }
}
```

```rust
ui.on_greet(move || {
    if let Some(ui) = weak.upgrade() {
        model.borrow_mut().name = ui.get_name().to_string();
        ui.set_greeting(model.borrow().greet().into());
    }
});
```

An alternative is a `name-changed(string)` callback on every edit. Step 07 uses
that shape for the editor, where every keystroke matters.

---

## Build, run, test

```bash
cargo build -p step-05-slint-files
cargo run  -p step-05-slint-files
cargo run  -p step-05-slint-files -- --test
```

```
05-slint-files: 4 checks, 0 failures
PASS
```

Try the live preview while editing the `.slint` files:

```bash
slint-viewer --auto-reload 05-slint-files/ui/main.slint
```

`cargo test -p step-05-slint-files` runs the checks through Cargo.

---

## Checkpoint

* What does `build.rs` produce, and what does `include_modules!` do with it?
* Why is `import` relative to the importing file, not to `main.rs`?
* A component exposes `in property` vs `in-out property`: when do you need the
  latter?

---

## Exercises

1. Add a `SectionHeader` property `accent: color` and use it for the title.
2. Create `ui/components/toolbar.slint` with two buttons and use it in
   `main.slint`.
3. Move the `main.slint` colors into an `export global Theme { ... }` and read
   them with `Theme.panel`.
4. Add a `@children` slot to `SectionHeader` and put a button inside it.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| `failed to load ui/main.slint` | `build.rs` runs from the package root. Use the path relative to `Cargo.toml`. |
| `unknown component MainWindow` | The root component must be `export component MainWindow`. |
| An import is not found | Paths are relative to the file that writes the `import`; `components/header.slint` is correct from `ui/main.slint`. |
| Changes to a component do not rebuild | `slint_build` tracks imports; if in doubt, `cargo clean -p step-05-slint-files`. |
| `slint-viewer` cannot resolve `components/...` | Run it from the step folder or pass `-I ui`. |

**Previous:** [Step 04 — Actions and shortcuts](../04-actions/README.md) ·
**Next:** [Step 06 — Menus](../06-menus/README.md)
