# 子树上的标注层



`on_gaze` turns the third pass on; `gaze_scope()` limits it to one subtree; `annotate(..)` gives an element a callout and, with `only_marked`, only those are framed.

The code is `examples/annotate.rs`, compiled into the library as `amenui::recipes::annotate`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: the annotation layer on one subtree (docs/recipes/annotate.md). `on_gaze` turns the layer on;
//! `gaze_scope` limits it to the panel; `annotate` gives an element its own callout.
use amenui::el::{El, Paint, col, row, space, text};
use amenui::style::{Gaze, GazeMode, gaze};
use amenui::text::Face;
use amenui::ui::Ui;

pub struct Panel;

pub fn view(_: &Panel) -> El<()> {
    let panel = col().key("panel").gaze_scope().pad(20.0, 18.0).gap(12.0).border(1.0, Paint::FAINT).width(360.0)
        .child(text("Power", 18.0, Paint::INK).key("panel/title").annotate("Title"))
        .child(row().gap(12.0)
            .child(space(80.0, 48.0).fill(Paint::INK).key("panel/switch").annotate("Main switch"))
            .child(space(120.0, 48.0).fill(Paint::DIM).key("panel/meter").annotate("Battery meter")));
    col().key("page").pad(32.0, 28.0).gap(16.0)
        .child(text("Text outside the panel is not annotated.", 14.0, Paint::DIM))
        .child(panel)
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<Panel, ()> {
    Ui::new(Panel, view, |_, _| {}, faces, w, h, scale)
        .on_gaze(|_| Gaze { mode: GazeMode::All, kinds: gaze::FRAME | gaze::INDEX | gaze::KIND | gaze::SIZE, only_marked: true, ..Gaze::default() })
}
```

See it:

```sh
cargo run --release --bin render --page recipe-annotate --at 1500
```
