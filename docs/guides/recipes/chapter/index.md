# A chapter: one scene with actors that flow in



A film is scenes that share a few actors. Give each actor a key of its scene and a flow id of the film; to cut, push `Cmd::Commit` (keep the frame as it is, then play the target look's transition — here a preset applied in `on_look`) and `Cmd::Flow` (the actors travel from where they are to where the new scene puts them), then change the scene. Everything else changes by the transition. A second cut during the first goes on from the frame on screen and from where the actors are: no queue. `src/gallery/film.rs` is the same thing at twelve scenes, with a 2.39:1 stage, a grid and a scripted autoplay.

The code is `examples/chapter.rs`, compiled into the library as `amenui::recipes::chapter`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: a chapter — one scene with actors that flow in (docs/recipes/chapter.md). Two scenes share
//! two actors (the sentence and the number) by flow id; going to the other scene is a commit (the target
//! preset's transition plays from the press) and a flow (the actors travel to their new places), so the
//! cut is a match cut. Keys differ per scene, flow ids do not.
use amenui::el::{El, Font, Paint, col, row, text};
use amenui::morph::{Cmd, FlowSpec};
use amenui::preset::{Preset, presets};
use amenui::style::Look;
use amenui::text::Face;
use amenui::ui::Ui;
use amenui::widgets::{Variant, button_as};

pub struct Film { pub scene: usize, pub n: u32, pub presets: Vec<Preset>, pub cmds: Vec<Cmd> }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Cut, Count }

/// Each scene: its preset (by hash) and where the actors stand.
const SCENES: [&str; 2] = ["normcore", "dotmatrix"];

pub fn view(s: &Film) -> El<Msg> {
    let c = s.scene;
    // The actors: a key of this scene, a flow id of the film.
    let sentence = text("Loosen, then close", if c == 0 { 40.0 } else { 64.0 }, Paint::INK).font(Font::Display).key(format!("s{c}/sentence")).flow("sentence");
    let number = text(format!("{:02}", s.n), if c == 0 { 40.0 } else { 28.0 }, Paint::INK).key(format!("s{c}/number")).flow("number");
    let cut = button_as("cut", "Next scene", Variant::Primary, Msg::Cut);
    let count = button_as("count", "Add one", Variant::Quiet, Msg::Count);
    match c {
        0 => col().key("scene0").pad(48.0, 40.0).gap(24.0).child(row().gap(40.0).child(sentence).child(number)).child(row().gap(12.0).child(cut).child(count)),
        _ => col().key("scene1").pad(48.0, 40.0).gap(24.0).child(number).child(sentence).child(row().gap(12.0).child(count).child(cut)),
    }
}

pub fn update(s: &mut Film, m: Msg) {
    match m {
        Msg::Cut => {
            // The frame as it is, then the new scene: the target preset's transition, and the actors fly.
            s.cmds.push(Cmd::Commit { x: 400.0, y: 300.0, scope: None });
            s.cmds.push(Cmd::Flow(FlowSpec { flow_ms: 640.0, ..FlowSpec::default() }));
            s.scene = 1 - s.scene;
        }
        Msg::Count => s.n += 1,
    }
}

fn look(s: &Film) -> Look {
    let p = s.presets.iter().find(|p| p.hash == SCENES[s.scene]).unwrap_or(&s.presets[0]);
    p.apply(&Look::default())
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<Film, Msg> {
    let s = Film { scene: 0, n: 12, presets: presets(), cmds: vec![] };
    Ui::new(s, view, update, faces, w, h, scale).on_commands(|s| std::mem::take(&mut s.cmds)).on_look(look)
}
```

See it:

```sh
cargo run --release --bin render -- --page recipe-chapter --events cut.json --at 1500 --out cut.png
```

`cut.json`: `[{"at_ms": 300, "kind": "press", "target": "cut"}, {"at_ms": 330, "kind": "release", "target": "cut"}]`
