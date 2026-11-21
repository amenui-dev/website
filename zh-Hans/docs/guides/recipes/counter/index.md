# 带重组和局部涟漪的计数器



The count is keyed text, so a new value recomposes only the changed digits (the old glyph's pixels fly to the new glyph's shape). The row is a `ripple_scope()` and the button sends its fronts to `Scope::Container(None)`: the shape beside it loosens, the label and digits only nudge (they are legible).

The code is `examples/counter.rs`, compiled into the library as `amenui::recipes::counter`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: a counter whose digits recompose and whose presses ripple only its row (docs/recipes/counter.md).
//! The count is keyed text, so a new value recomposes the changed digits; the row is a ripple scope and
//! the button sends its fronts there; the picture beside it loosens, the label only nudges.
use amenui::el::{El, Font, Paint, Scope, col, row, space, text};
use amenui::text::Face;
use amenui::ui::Ui;
use amenui::widgets::{Variant, button_as};

pub struct Counter { pub n: u32 }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Inc }

pub fn view(s: &Counter) -> El<Msg> {
    col().key("page").pad(32.0, 28.0).child(row().key("row").ripplr_scope().gap(24.0).pad(20.0, 16.0).border(1.0, Paint::FAINT)
        .child(button_as("inc", "Add one", Variant::Primary, Msg::Inc).ripplr(Scope::Container(None)))
        .child(text(s.n.to_string(), 40.0, Paint::INK).font(Font::Display).key("count"))
        .child(space(60.0, 40.0).radius(6.0).fill(Paint::INK).key("decor")))
}

pub fn update(s: &mut Counter, m: Msg) {
    match m { Msg::Inc => s.n += 1 }
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<Counter, Msg> {
    Ui::new(Counter { n: 9 }, view, update, faces, w, h, scale)
}
```

See it:

```sh
cargo run --release --bin render --page recipe-counter --events inc.json --at 300 --tree counter.json
```
