# Step 04 — Actions and shortcuts

**Level 2 · Elementary · ~30 minutes**

A button, a keyboard shortcut and (next step) a menu entry should run the same
code. This step names that code: an **action**.

---

## What you will learn

```rust
enum Action { AddOne, AddTen, Reset }
Action::from_name("counter.add") -> Option<Action>
Counter::dispatch(action)
```

* one dispatcher instead of one closure per widget;
* `FocusScope` receives keys the focused child rejects;
* `event.modifiers.control` and `event.text` identify `Ctrl+I` / `Ctrl+0`;
* a window-level shortcut still fires when the button is disabled.

---

## The program

| Action | Button | Shortcut | Effect |
|---|---|---|---|
| `AddOne` | **+1** | `Ctrl+I` | +1 |
| `AddTen` | **+10** | — | +10 |
| `Reset` | **Reset** | `Ctrl+0` | back to 0 |

`Counter::dispatch` is the only place that changes the value:

```rust
fn dispatch(&mut self, action: Action) {
    self.value = if matches!(action, Action::Reset) {
        0
    } else {
        self.value + action.amount()
    };
    self.last = action.name().to_string();
}
```

The buttons call the callbacks; the callbacks call `dispatch` and then `show`:

```rust
ui.on_add_one(move || {
    model.borrow_mut().dispatch(Action::AddOne);
    if let Some(ui) = weak.upgrade() { show(&ui, &model.borrow()); }
});
```

### The keys

```slint
FocusScope {
    init => { self.focus(); }

    key-pressed(event) => {
        if (event.modifiers.control && event.text == "i") {
            root.add-one();
            return accept;
        }
        if (event.modifiers.control && event.text == "0") {
            root.reset();
            return accept;
        }
        return reject;
    }

    // the page lives inside the scope
    VerticalLayout { ... }
}
```

`key-pressed` on a scope sees keys the focused child **rejected**; the scope
must contain the focused elements. `accept` stops delivery, `reject` lets it
continue. The scope takes the initial focus with `init`. In step 08 the menu
bar supplies shortcuts instead, so the editor keeps its focus.

### Disabled buttons, live shortcuts

The two add buttons are `enabled: root.add-enabled`. A disabled button takes
no clicks. The `Ctrl+I` shortcut is on the window's scope, not on the button,
so it still fires — the same behaviour a real menu shortcut has. The model does
not need to know which path called it.

---

## Build, run, test

```bash
cargo build -p step-04-actions
cargo run  -p step-04-actions
cargo run  -p step-04-actions -- --test
```

```
04-actions: 7 checks, 0 failures
PASS
```

The checks drive `dispatch` and `set_enabled` directly, including the
"shortcut ignores the disabled button" rule. Slint 1.18's testing backend has
no public keyboard injection, so the key handler itself is exercised by hand;
the model path it calls is what the test covers.

`cargo test -p step-04-actions` runs the same check.

---

## Checkpoint

* What is the difference between `accept` and `reject` in `key-pressed`?
* Why does the shortcut still work when the +1 button is disabled?
* Where would a menu entry plug into this design?

---

## Exercises

1. Add `Action::SubtractOne` with `Ctrl+D` and a **−1** button.
2. Register `Ctrl+Shift+0` as an alias for Reset. Does `event.text` change?
3. Disable the Reset button while the value is 0, and update `show` to set it.
4. Move the `match` from `Counter::dispatch` into a free function that also
   returns a status string, so the model stays data-only.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| The shortcut never fires | The `FocusScope` does not contain the focused element, or nothing has focus. Put the page inside the scope and give it `init => { self.focus(); }`. |
| `event.text == "i"` is false | Some layouts produce a different `text` with Control held. Print it with `debug("key", event.text);` to see the platform's value. |
| Ctrl+Shift+0 does not match | `event.text` is usually `"0"`; check `event.modifiers.shift` separately. |
| Two actions run at once | A handler returned `accept` *after* calling another action, or the callback fired twice. Return `accept` exactly once. |
| The counter changes but the label does not | `show` was not called after `dispatch`. |

**Previous:** [Step 03 — Controls](../03-controls/README.md) ·
**Next:** [Step 05 — Slint files](../05-slint-files/README.md)
