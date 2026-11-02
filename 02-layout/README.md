# Step 02 — Layout

**Level 1 · Beginner · ~25 minutes**

A page with three areas: a fixed header, a content area that takes the rest of
the window, and a fixed footer. Resize the window — only the content grows.

---

## What you will learn

| Idea | Effect |
|---|---|
| `VerticalLayout` / `HorizontalLayout` | arrange children in one direction |
| `spacing` | gap between children |
| `padding` | gap inside the layout |
| `horizontal-stretch: 1` | child takes an equal share of the free width |
| `vertical-stretch: 1` | child takes all the free height |
| `alignment` | `start` / `center` / `end` on a layout |

There is no `Slot.*` here: a child's size and stretch are properties on the
child, and the layout does the arithmetic.

---

## The shape

```slint
VerticalLayout {
    spacing: 0px;

    Rectangle { height: 56px; /* header */ }
    Rectangle { vertical-stretch: 1; /* content */ }
    Rectangle { height: 44px; /* footer */ }
}
```

A fixed `height` means "do not grow". `vertical-stretch: 1` means "take every
pixel the fixed children leave". That one pair explains most Slint layouts.

Inside the content:

```slint
HorizontalLayout {
    spacing: 8px;
    Button { text: "One";   horizontal-stretch: 1; }
    Button { text: "Two";   horizontal-stretch: 1; }
    Button { text: "Three"; horizontal-stretch: 1; }
}
```

Three equal shares. Remove `horizontal-stretch` from one button and it takes
its natural width while the other two share the rest.

The footer is a row with a label that grows and a button that does not:

```slint
Text { text: root.status; horizontal-stretch: 1; }
Button { text: "Clear"; }
```

Finally, a `Rectangle { vertical-stretch: 1; }` inside the content pushes the
last line to the bottom — a spacer is just an empty rectangle with stretch.

---

## Build, run, test

```bash
cargo build -p step-02-layout
cargo run  -p step-02-layout                # resize the window
cargo run  -p step-02-layout -- --test
```

```
02-layout: 5 checks, 0 failures
PASS
```

The headless checks cover the footer's model: a click names the button, the
last click wins, Clear resets it. Layout is checked by eye here — the next
steps keep the same split between visual layout and testable state.

`cargo test -p step-02-layout` runs the same check through Cargo.

---

## Checkpoint

* What is the difference between `spacing` and `padding`?
* Which property makes the content area fill the window?
* How do you push a line to the bottom of a layout without absolute positions?

---

## Exercises

1. Change the header to 72 px and add a second line of text under the title.
2. Give the first button `horizontal-stretch: 2` and the others `1`. Predict
   the widths before running.
3. Put the three buttons in a `VerticalLayout` and give each
   `vertical-stretch: 1`. What happens to the fixed content height?
4. Add a `Text` under the footer with `horizontal-alignment: center`.

---

## Troubleshooting

| Problem | Fix |
|---|---|
| The content area is zero height | It is not in a layout, or nothing has `vertical-stretch`. The root `VerticalLayout` must fill the window. |
| All children are the same width | That is the default equal stretch in a `HorizontalLayout` when the children can grow; set a fixed `width` or remove stretch to change it. |
| `padding` has no effect | `padding` belongs to a layout, not to a `Rectangle`. Wrap the children in a layout first. |
| The window grows instead of the content | A fixed-height child was replaced by one with stretch, or the window itself has no explicit height. |
| Text is clipped | The row is shorter than the font. Give the header/footer more height or set `font-size`. |

**Previous:** [Step 01 — A first window](../01-first-window/README.md) ·
**Next:** [Step 03 — Controls](../03-controls/README.md)
