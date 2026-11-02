# Step 01 — A first window

**Level 1 · Beginner · ~20 minutes**

The smallest Slint program that is still a program: a window, a label, a button
that changes it, and a Quit button. The interface sits in `main.rs`; Rust owns
the counter.

---

## What you will learn

| Idea | In this step |
|---|---|
| `slint::slint!` | compiles the UI written inline in `main.rs` |
| `Window` | `title`, `width`, `height` |
| `VerticalLayout` | stacks the label and the row of buttons |
| `in-out property` | `message` and `clicks`, written by Rust |
| `callback` | `clicked()` and `quit-requested()`, answered by Rust |
| `MainWindow::new()` / `ui.run()` | build the component and enter the loop |

Run it:

```bash
cargo run -p step-01-first-window
```

Type nothing, click **Click me** twice: the label says `Clicked 2 times`.
**Quit** hides the window, so `run()` returns and the process exits.

---

## The program

### The UI

```slint
export component MainWindow inherits Window {
    title: "Tutorial 01 - First window";
    in-out property <string> message: "Press the button";
    in-out property <int> clicks: 0;

    callback clicked();
    callback quit-requested();

    VerticalLayout {
        Text { text: root.message; }
        HorizontalLayout {
            Button { text: "Click me"; clicked => { root.clicked(); } }
            Button { text: "Quit"; clicked => { root.quit-requested(); } }
        }
    }
}
```

`export` is what makes the component visible to `include_modules!` and to the
inline `slint!` macro. `root` is the component itself, so `root.clicked()`
reaches the callback declared at the top.

### The model

```rust
struct Counter { clicks: u32, running: bool }

impl Counter {
    fn click(&mut self) -> String {
        self.clicks += 1;
        format!("Clicked {} time{}", self.clicks,
                if self.clicks == 1 { "" } else { "s" })
    }
}
```

No Slint type appears in `Counter`. That is what lets `--test` run without a
backend, a window or a font.

### The wiring

```rust
let weak = ui.as_weak();
let clicks = counter.clone();
ui.on_clicked(move || {
    let (message, count) = {
        let mut counter = clicks.borrow_mut();
        (counter.click(), counter.clicks as i32)
    };
    if let Some(ui) = weak.upgrade() {
        ui.set_message(message.into());
        ui.set_clicks(count);
    }
});
```

`as_weak()` avoids a cycle: the component owns the callback, and a strong
`Rc` back to the component would keep both alive forever. `Rc<RefCell<_>>`
lets `main` and the closure share the counter.

`Message` is a `String`; Slint wants a `SharedString`, hence `.into()`.

---

## Build

```bash
cargo build -p step-01-first-window
```

## Run

```bash
cargo run -p step-01-first-window
```

## Test

```bash
cargo run -p step-01-first-window -- --test
```

```
01-first-window: 5 checks, 0 failures
PASS
```

`--test` does **not** call `MainWindow::new()`. It runs `Counter` directly:
two clicks, the singular/plural message, the count, and Quit. That is the whole
model the window uses, and it needs no display.

`cargo test -p step-01-first-window` runs the same binary through
`tests/check.rs`.

---

## Checkpoint

* What does `export` do, and where does `MainWindow` come from?
* Why `as_weak()` instead of cloning the `Rc` into the closure?
* Which part of the program runs in `--test`?

---

## Exercises

1. Add a third button that resets the counter to zero.
2. Change the label to `"No clicks yet"` before the first click (a third
   property, or a `Counter` method that returns the initial text).
3. Add a `Text` that shows `clicks` directly with string interpolation:
   `"Clicks: \{root.clicks}"`.
4. Make the window 520 px wide and put the two buttons at opposite ends.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| `cannot find macro slint in the crate root` | Use `slint::slint!`, or add `use slint::slint;`. The inline macro is on the `slint` crate. |
| `no method named on_clicked` | The callback must be declared in the component: `callback clicked();`. Hyphens become underscores: `quit-requested` → `on_quit_requested`. |
| The label does not change | Rust sets `ui.set_message(...)` *after* the model changes; setting the model alone does not update the UI. |
| `BorrowMutError` | A `borrow()` was held while calling something that borrows again. Copy the value out, then drop the borrow. |
| The window does not appear | No display. Run with `--test`, or use a desktop session. |

**Next:** [Step 02 — Layout](../02-layout/README.md)
