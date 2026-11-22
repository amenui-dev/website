# Morph A into B



Both elements are in the tree in the same place (`stack`); the one not shown is `hidden_until_morphed`. `Cmd::Morph` loosens the source, flies its lattice tiles and tightens the target; asking for the reverse mid-flight turns the tiles around.

The code is `examples/morph.rs`, compiled into the library as `amenui::recipes::morph`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: morph A into B (docs/recipes/morph.md). Both elements are in the tree, in the same place; the
//! one not shown is `hidden_until_morphed`. `Cmd::Morph` loosens A, flies its tiles and tightens B.
use amenui::el::{El, Font, Paint, col, space, stack, text};
use amenui::morph::{Cmd, MorphSpec};
use amenui::text::Face;
use amenui::ui::Ui;
use amenui::widgets::{Variant, button_as};

pub struct Shapes { pub showing_b: bool, pub cmds: Vec<Cmd> }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Swap }

pub fn view(s: &Shapes) -> El<Msg> {
    let a = text("A", 120.0, Paint::INK).font(Font::Display).key("a").hidden_until_morphed(s.showing_b);
    let b = space(160.0, 160.0).radius(80.0).fill(Paint::INK).key("b").hidden_until_morphed(!s.showing_b);
    col().key("stage").pad(32.0, 28.0).gap(20.0)
        .child(stack([a, b]).width(240.0).height(220.0))
        .child(button_as("swap", "Morph", Variant::Primary, Msg::Swap))
}

pub fn update(s: &mut Shapes, m: Msg) {
    match m {
        Msg::Swap => {
            let (from, to) = if s.showing_b { ("b", "a") } else { ("a", "b") };
            s.cmds.push(Cmd::Morph { from: from.into(), to: to.into(), spec: MorphSpec::default() });
            s.showing_b = !s.showing_b;
        }
    }
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<Shapes, Msg> {
    Ui::new(Shapes { showing_b: false, cmds: vec![] }, view, update, faces, w, h, scale).on_commands(|s| std::mem::take(&mut s.cmds))
}
```

See it:

```sh
cargo run --release --bin render --page recipe-morph --events swap.json --at 900
```
