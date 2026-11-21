# Text on a form



A paragraph wrapped onto a sphere and kept legible. `form::Form` (a shape, how it is turned, a `mesh::Camera`, how much of the shape the text covers) gives every glyph of the text its place, turn and scale on the surface, its depth (`Pose::z`) and how it faces the viewer (`Pose::facing`). `El::form(FormFx)` then draws the glyphs far to near: the ones turned away are left out (`back`), the ones edge on fade (`edge`), a glyph whose ink is mostly covered by nearer glyphs is not drawn (`overlap`), and the far side is dimmer (`fog`). With `reveal_by(Reveal::Word)` words appear and disappear whole, never as half a word at the rim. The pointer is found on the surface with `Form::hit`, which answers in the text's own block, so the words it lights follow the sphere as it turns; a drag turns it.

The long lines are deliberate (`nowrap`): one line of the block goes once round the sphere, so the text keeps its size where the sphere faces you.

The code is `examples/form3d.rs`, compiled into the library as `amenui::recipes::form3d`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: a passage wrapped on a sphere (docs/recipes/form3d.md). `form::Form` gives each glyph its
//! place, turn, scale, depth and facing; `El::form` draws them far to near and keeps the sphere legible
//! (the far side left out, the rim faded so no word is drawn squeezed, glyphs mostly hidden by nearer
//! ones not drawn, the back dimmer); words come and go whole (`Reveal::Word`). The pointer is found on
//! the surface (`Form::hit`): the glyphs round it take the accent, fading smoothly from the glyph under
//! it to none at `REACH`, following the sphere as it turns; a drag turns it.
use amenui::el::{El, Paint, Point, Reveal, col, text};
use amenui::form::{Form, FormFx};
use amenui::mesh::Camera;
use amenui::text::Face;
use amenui::ui::Ui;

/// The passage, one line of the block per line: long lines, so the block goes round the sphere once.
pub const PASSAGE: &str = "the tide comes in over the flats twice a day and leaves the sand ribbed and shining, a map of where the water went\n\
and the gulls have learned the hour; they stand in a line facing the wind and wait for what the sea turns over\n\
far out a boat with a red hull keeps its place against the current, small and patient, its engine a low hum\n\
the light at this time of year is thin and comes in sideways, so every footprint holds a little shadow of its own\n\
someone has written a name in the wet sand with a stick, and the first wave of the evening is already reading it\n\
the wind smells of salt and diesel and cut grass from the dunes, where the paths are soft and the fences lean\n\
a dog runs the edge of the water and back, the edge and back, as if it had been asked to measure the shore\n\
when the tide turns there is a moment when nothing moves at all, and then everything begins to go the other way\n\
by dark the flats are gone and the sea is where it was, keeping only the sound, which carries further at night";

/// The sphere's size on the page, logical px.
pub const SIZE: f32 = 460.0;
/// How far round the pointer lights words, text px on the surface.
pub const REACH: f32 = 64.0;

/// How much of the accent a glyph `d` text px from the pointer's place takes: all of it within 0.15 ×
/// `REACH`, none from `REACH`, smoothly between, gently enough that two neighbours in a word never
/// differ by more than about a third (no hard edge falls in the middle of a word).
pub fn light(d: f32) -> f32 {
    let t = ((d - 0.15 * REACH) / (0.85 * REACH)).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}

pub struct Form3d { pub spin: [f32; 2], pub hover: Option<[f32; 2]>, pub last: Option<(f32, f32)> }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>) }

/// The form for a turn: a unit sphere, the camera far enough that the text keeps its size in front.
pub fn form(spin: [f32; 2]) -> Form {
    Form { spin, camera: Camera { yaw: 0.0, pitch: 0.0, distance: 4.3, ..Camera::default() }, cover: [1.0, 0.62], ..Form::default() }
}

pub fn view(s: &Form3d) -> El<Msg> {
    let f = form(s.spin);
    let hover = s.hover;
    // Where on the text block each glyph is, to light the ones near the pointer's place on the surface.
    let pose = move |g: amenui::el::GlyphAt| {
        let mut p = f.pose_of(g);
        if let Some([u, v]) = hover {
            let [x0, y0, x1, y1] = g.span;
            let (gx, gy) = (x0 + u * (x1 - x0), y0 + v * (y1 - y0));
            // Across the seam the block wraps round.
            let w = x1 - x0;
            let dx = ((g.x - gx).abs()).min(w - (g.x - gx).abs());
            let d = dx.hypot(g.y - gy);
            p.accent = p.accent.max(light(d));
        }
        p
    };
    col().key("page").pad(32.0, 28.0).gap(10.0)
        .child(text("text on a form", 26.0, Paint::INK).key("title"))
        .child(text("Move over the sphere to find a place on it; drag to turn it.", 14.0, Paint::DIM).key("hint"))
        .child(text(PASSAGE, 19.0, Paint::TEXT).key("passage").nowrap().width(SIZE).height(SIZE)
            .pose(pose).form(FormFx::default()).reveal_by(Reveal::Word).on_point(|p| Some(Msg::Point(p))))
}

pub fn update(s: &mut Form3d, m: Msg) {
    match m {
        Msg::Point(None) => { s.hover = None; s.last = None; }
        Msg::Point(Some(p)) => {
            if p.down {
                // Held: turn by how far it moved, the surface following the pointer.
                if let Some((x, y)) = s.last {
                    s.spin[0] -= (p.x - x) * 0.35;
                    s.spin[1] = (s.spin[1] - (p.y - y) * 0.35).clamp(-60.0, 60.0);
                }
                s.last = Some((p.x, p.y));
            } else { s.last = None; }
            s.hover = form(s.spin).hit(p.x, p.y, SIZE, SIZE).map(|h| h.uv);
        }
    }
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<Form3d, Msg> {
    Ui::new(Form3d { spin: [-18.0, 12.0], hover: None, last: None }, view, update, faces, w, h, scale)
}
```
