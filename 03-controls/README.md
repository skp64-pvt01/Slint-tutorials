# Step 03 — Controls and state

**Level 2 · Elementary · ~30 minutes**

Four controls drive one preview. The habit this step installs: **the model is
the source of truth, and one callback updates it**. A control reports a value;
Rust formats the result.

---

## What you will learn

| Control | API used here |
|---|---|
| `LineEdit` | `text`, `edited(text)` — fires on every keystroke |
| `CheckBox` | `checked`, `toggled` |
| `Slider` | `minimum`, `maximum`, `value`, `changed(value)` |
| `ComboBox` | `model`, `current-index`, `selected(value)` |

And the pattern:

```slint
callback changed(string, bool, float, int);

LineEdit {
    edited(text) => {
        root.name = text;
        root.changed(root.name, root.shout, root.size, root.theme);
    }
}
```

The callback carries the whole form. Rust never has to read a widget back and
the UI never formats a sentence.

---

## The program

`Form` is plain Rust:

```rust
struct Form { name: String, shout: bool, size: f32, theme: usize }

fn preview(&self) -> String {
    let mut who = if self.name.trim().is_empty() {
        "world".into()
    } else {
        self.name.trim().to_string()
    };
    if self.shout { who = who.to_uppercase(); }
    format!("Hello, {who}!")
}
```

The UI writes the preview:

```rust
ui.on_changed(move |name, shout, size, theme| {
    model.borrow_mut().update(name.as_str(), shout, size, theme);
    show(&ui, &model.borrow());
});
```

`show` sets `preview`, `summary` and `preview-size`:

```rust
fn show(ui: &MainWindow, form: &Form) {
    ui.set_preview(form.preview().into());
    ui.set_summary(form.summary().into());
    ui.set_preview_size(form.size);
}
```

`preview-size` is a `float`; the UI converts it to a length:

```slint
Text { font-size: root.preview-size * 1px; }
```

Slint never mixes numbers and lengths implicitly.

### The theme

`ComboBox { model: ["Light", "Dark"]; current-index: root.theme; }` maps 0/1
directly. `selected(value)` fires after the index changed, so the handler reads
`self.current-index` into the model. The summary, not the combo box, produces
the text `theme Dark, size 28`.

---

## Build, run, test

```bash
cargo build -p step-03-controls
cargo run  -p step-03-controls
cargo run  -p step-03-controls -- --test
```

```
03-controls: 6 checks, 0 failures
PASS
```

The checks call `Form::update` exactly as the callback does: empty name,
typing, shout, size/theme, and blank input falling back to `world`. They cover
the formatting rules without a display.

`cargo test -p step-03-controls` runs them through Cargo.

---

## Checkpoint

* Why does one callback serve four controls?
* When does `LineEdit.edited` fire, and how is that different from `accepted`?
* Where does the `Hello, …!` string come from — the `.slint` file or Rust?
* Why is `font-size: root.preview-size * 1px` needed?

---

## Exercises

1. Add a `Slider` for the preview's opacity (`0..1`, step `0.1`) and set
   `opacity` on the preview `Text`.
2. Add a second checkbox, "Greet twice", and make `preview()` repeat the line.
3. Show the number of characters typed in the summary.
4. Replace the combo box with two radio-style buttons (there is no radio
   widget in std-widgets; two `Button`s with `primary` toggled by the model).

---

## Troubleshooting

| Problem | Fix |
|---|---|
| Typing does not update the preview | Use `edited`, not `accepted` (`accepted` fires only on Enter). |
| `Cannot convert float to length` | Multiply by `1px`, as the preview does. |
| The slider jumps to whole numbers | `step` defaults to 1. Set `step: 0.5;` (or whatever you need). |
| The combo box selection is off by one | `selected(value)` is the text; `current-index` is the 0-based index. Use one, not both. |
| `BorrowMutError` in the callback | `show` borrows the model while `update` still holds a borrow. End the `borrow_mut()` statement first, as the code does. |

**Previous:** [Step 02 — Layout](../02-layout/README.md) ·
**Next:** [Step 04 — Actions and shortcuts](../04-actions/README.md)
