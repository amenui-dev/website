# 自定义预设



A style preset is plain data (`amenui::preset::Preset`): theme, material and its numbers, faces, annotation, the interaction character (press feedback, fronts, default responses, recomposition) and a transition. Start from one of the twelve and change what you want, or read one from JSON (`Preset::from_json`; the website's styles page exports any preset). `Preset::apply` gives the look; when a commit applies it, its transition plays between the frame as it was and the new one — here rows that reveal from the top. A second press in the middle goes on from what is on the screen. The Crack button commits a third look through the Crack transition (`StepKind::Crack`, numbers `preset::CRACK`): the page splits along long pieces and the cracks widen until only the new page is left.

A preset also carries `post`, the finishing pass (`amenui::post::PostSpec`: bloom → chromatic offset → scanlines → grain → vignette → curvature, then a dither and a palette, each off by default), and `marks` (`depth_scale`, `depth_dim`) for 3D elements. They are numbers like the rest: `p.set_number("post.scanlines.strength", 0.2)`, and they travel in the `look:v3:` recipe.

The code is `examples/preset.rs`, compiled into the library as `amenui::recipes::preset`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: a custom preset (docs/recipes/preset.md). A preset is data: start from one of the twelve, change
//! its numbers (or read it from JSON), and give the runtime the look it makes. Its transition plays when
//! a commit applies it — here a button that switches between the plain look and the custom one, and one
//! that cracks the page open onto a third.
use amenui::el::{El, Paint, col, row, text};
use amenui::preset::{Preset, Step, StepKind, Transition, presets};
use amenui::style::Look;
use amenui::text::Face;
use amenui::ui::Ui;
use amenui::widgets::{Variant, button_as};

pub struct App { pub custom: Preset, pub on: bool, pub cracked: Preset, pub crack: bool }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Switch, Crack }

/// Bauhaus's look, arriving by rows of 18 px that reveal top to bottom, softly, in 600 ms.
pub fn custom() -> Preset {
    let mut p = presets().into_iter().find(|p| p.hash == "bauhaus").expect("a preset");
    p.name = "Rows".into();
    p.transition = Transition::of(600.0, &[Step { kind: StepKind::Reveal, from: 0.0, to: 1.0, a: 2.0, b: 0.0, c: 18.0, d: 24.0, e: 0.0 }]);
    p.interaction.press_ms = 240.0;
    p
}

/// Tech-Manual's look (dark ink on light paper), arriving through cracks: the old page splits into
/// long pieces leaning to 70°, and the cracks widen until only the new page is left, in 800 ms.
pub fn cracked() -> Preset {
    let mut p = presets().into_iter().find(|p| p.hash == "tech-manual").expect("a preset");
    p.name = "Crack".into();
    let [a, b, c, d, e] = StepKind::CRACK;
    p.transition = Transition::of(800.0, &[Step { kind: StepKind::Crack, from: 0.0, to: 1.0, a, b, c, d, e }]);
    p
}

pub fn view(s: &App) -> El<Msg> {
    col().key("page").pad(32.0, 28.0).gap(16.0)
        .child(text(if s.crack { "Cracked open" } else if s.on { "A style of its own" } else { "As it was" }, 28.0, Paint::INK).key("title"))
        .child(text("Press the button: the new look arrives through its own transition.", 15.0, Paint::TEXT))
        // A commit: the frame as it was stays, and the preset's transition reveals the new one.
        .child(row().gap(12.0)
            .child(button_as("switch", "Switch", Variant::Primary, Msg::Switch).commit())
            .child(button_as("crack", "Crack", Variant::Default, Msg::Crack).commit()))
}

pub fn update(s: &mut App, m: Msg) {
    match m { Msg::Switch => { s.on = !s.on; s.crack = false } Msg::Crack => s.crack = !s.crack }
}

fn look(s: &App) -> Look {
    if s.crack { s.cracked.apply(&Look::default()) } else if s.on { s.custom.apply(&Look::default()) } else { Look::default() }
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<App, Msg> {
    Ui::new(App { custom: custom(), on: false, cracked: cracked(), crack: false }, view, update, faces, w, h, scale).on_look(look)
}
```

See it (mid-transition at 400 ms):

```sh
cargo run --release --bin render -- --page recipe-preset --events switch.json --at 400 --out rows.png
```

`switch.json`: `[{"at_ms": 100, "kind": "press", "target": "switch"}, {"at_ms": 130, "kind": "release", "target": "switch"}]`
