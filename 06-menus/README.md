# Step 06 — Menus

**Level 3 · Intermediate · ~30 minutes**

A native menu bar with **File / Edit / View / Help**, shortcuts that work while
the menu is closed, and one status line that names the last entry.

---

## What you will learn

| Piece | Meaning |
|---|---|
| `MenuBar` | the bar; a direct child of `Window`, outside the page layout |
| `Menu { title: "File"; }` | a top-level menu |
| `MenuItem` | an entry with `title`, optional `shortcut`, `activated` |
| `MenuSeparator {}` | a line |
| `@keys(Control + N)` | the platform's shortcut syntax and label |
| `@keys(Control + Shift + S)` | chords with more than one modifier |

Slint 1.18 renders the menu bar inside the window on Linux/Windows and uses the
native bar where the platform provides one. It is not a `Rectangle` you draw.

---

## One callback, one dispatcher

Every entry calls the same callback:

```slint
MenuItem {
    title: "Save";
    shortcut: @keys(Control + S);
    activated => { root.action("file.save"); }
}
```

```rust
callback action(string);
```

Rust turns the name into an `Action`:

```rust
enum Action { New, Open, Save, Undo, Redo, SelectAll, Dark, Light, Shortcuts, About, Quit }

fn dispatch(&mut self, name: &str) -> Option<Action> {
    let action = Action::from_name(name)?;
    if action == Action::Dark { self.dark = true; }
    if action == Action::Light { self.dark = false; }
    self.status = action.status();
    self.last = Some(action);
    Some(action)
}
```

`Some`/`None` makes an unknown name harmless. The same `Action` will later be
reached from a toolbar button, without touching the menu.

### Shortcuts

`@keys(Control + N)` both registers the shortcut and shows it in the menu. It
works with the menu closed, and it belongs to the window, not to the page. Two
notes:

* a digit is a string: `@keys(Control + "0")`, not `Control + 0`;
* Slint 1.18 `MenuItem` has no checkable state. If you need a checked entry,
  keep the state in Rust and change the title, or use a toolbar toggle. The
  Raven course uses checkable View entries; this course uses a theme switch
  instead.

---

## Build, run, test

```bash
cargo build -p step-06-menus
cargo run  -p step-06-menus
cargo run  -p step-06-menus -- --test
```

```
06-menus: 10 checks, 0 failures
PASS
```

The checks exercise the dispatcher: the first action, the last action, an
unknown name, Dark, Quit and the stored `last`. The native menu itself and its
shortcuts are a window feature; open the app and try **Ctrl+N**, **Ctrl+S**,
**Ctrl+Q** to see them.

`cargo test -p step-06-menus` runs the checks through Cargo.

---

## Checkpoint

* Why must `MenuBar` be a direct child of `Window`?
* What does `activated` run — a function name, or whatever the expression says?
* How does an unknown `action("...")` name stay harmless?
* Where does the menu shortcut live: the menu item or the page?

---

## Exercises

1. Add **Edit > Duplicate line** with `Ctrl+D`, even if it only updates the
   status for now.
2. Add a **Help > About** entry and show the version in the status bar.
3. Add a toolbar button that calls `root.action("file.save")`; confirm it is
   the same dispatcher.
4. Replace the Dark/Light pair with a single "Toggle theme" entry and test the
   new action.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| No menu bar appears | Put `MenuBar` directly under `Window`, before the page `Rectangle`. A `MenuBar` inside a layout is not a menu bar. |
| `@keys(Control + 0)` fails to compile | Digits are strings: `@keys(Control + "0")`. |
| Ctrl+N does nothing | Another focused element consumed the key, or the entry is in a submenu that is not built. Native menu shortcuts are window-level; test with the app focused. |
| The menu is drawn but entries are empty | `MenuItem` is missing `title`, or the `Menu` has no children. |
| I want a check mark | Not available on `MenuItem` in 1.18. Keep the state in Rust and change the title, or use a `CheckBox` in a toolbar. |

**Previous:** [Step 05 — Slint files](../05-slint-files/README.md) ·
**Next:** [Step 07 — Text editor](../07-text-editor/README.md)
