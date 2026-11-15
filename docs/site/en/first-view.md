# Your first view

A view is a function from state to an element tree; an update is a function from state and a message to a new state. Messages are plain values.

```rust
use lab::el::{El, Paint, col, text};
use lab::widgets::button;
#[derive(Clone)] enum Msg { Inc }
struct S { n: u32 }
fn view(s: &S) -> El<Msg> {
    col().pad(24.0, 24.0).gap(12.0)
        .child(text(format!("{}", s.n), 32.0, Paint::INK).key("count"))
        .child(button("inc", "Add one", Msg::Inc))
}
fn update(s: &mut S, m: Msg) { match m { Msg::Inc => s.n += 1 } }
```

Give a stable key to every element you will find again, animate or that holds state. Keyed text that changes recomposes: the pixels of the characters that changed fly into the new ones. Verify with `render`: draw a frame and read its tree (see [Testing](#docs-testing)).
