//! The field map's page (DESIGN.md §1.29): a place with decoration, a picture and a paragraph, a
//! floating window over it whose trail loosens what it passes, the formula and the map's twelve numbers
//! under it, and a button that stirs the place with `Cmd::Stir`.
use super::pages::ratio;
use super::{Gallery, IMAGE_SLOT, Msg};
use crate::demo::SKY;
use crate::el::{El, Font, Paint, Response, Scope, col, layers, picture, row, space, text};
use crate::fieldmap::MAP_PARAMS;
use crate::morph::{Cmd, StirAt};
use crate::widgets::{CaptionParam, Edit, FloatMsg, Floating, Spec, Variant, button_as, floating};

/// The place under the window (logical px).
pub const AREA: (f32, f32) = (760.0, 420.0);

#[derive(Clone, Debug, PartialEq)]
pub struct StirrPage { pub float: Floating, pub draft: Option<(usize, String)> }
impl Default for StirrPage {
    fn default() -> Self {
        let mut float = Floating::default();
        (float.size, float.area) = ((180.0, 108.0), AREA);
        float.place(0);
        Self { float, draft: None }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SMsg { Float(FloatMsg), Stir, Num(usize, Edit) }

pub fn update(s: &mut Gallery, m: SMsg) {
    let now = s.clock;
    match m {
        SMsg::Float(f) => s.stirr.float.update(f, now),
        SMsg::Stir => s.cmds.push(Cmd::Stir { at: StirAt::Key("stirr/place".into()), v: [1.0, 0.35], energy: 1.0, radius: 220.0, scope: Scope::Container(Some("stirr/place".into())) }),
        SMsg::Num(i, e) => match e {
            Edit::Draft(t) => s.stirr.draft = Some((i, t)),
            Edit::Cancel => s.stirr.draft = None,
            Edit::Set(v) => { s.map.set(i, v); s.stirr.draft = None; }
        },
    }
}
/// The window's spring, while it glides to a corner.
pub fn tick(s: &mut Gallery, t: f32) -> bool { s.stirr.float.step(t) }
pub fn wake(s: &Gallery, t: f32) -> Option<f32> { s.stirr.float.moving().then_some(t) }

pub fn page(s: &Gallery) -> El<Msg> {
    let p = &s.stirr;
    let fx = &s.map;
    let deco = |i: usize, w: f32, h: f32, r: f32, hue: u8| space(w, h).radius(r).fill(Paint::hue(hue)).key(format!("stirr/deco/{i}")).respond(Response::Loosen);
    let shapes = row().gap(22.0).key("stirr/shapes")
        .child(deco(0, 64.0, 64.0, 32.0, 0)).child(deco(1, 96.0, 40.0, 4.0, 1)).child(deco(2, 40.0, 40.0, 20.0, 2))
        .child(deco(3, 120.0, 12.0, 6.0, 3)).child(deco(4, 56.0, 56.0, 0.0, 4)).child(deco(5, 30.0, 70.0, 15.0, 5));
    let words = "A floating window leaves a trail behind it. Decoration beside its path loosens and drifts with the flow, then settles back; the text you read stays legible, and the window never loosens itself.";
    let lower = row().gap(28.0).key("stirr/lower")
        .child(col().width(260.0).child(picture(SKY, ratio(SKY)).key("stirr/picture").pic_fx(IMAGE_SLOT).respond(Response::Loosen)))
        .child(col().gap(12.0).width(380.0)
            .child(text(words, 14.0, Paint::TEXT).key("stirr/text"))
            .child(text("LOOSEN · LEGIBLE", 12.0, Paint::DIM).font(Font::Mono).key("stirr/legible").respond(Response::Loosen)));
    let place = col().key("stirr/place").pad(28.0, 28.0).gap(36.0).width(AREA.0).height(AREA.1).child(shapes).child(lower);
    let label = col().pad(12.0, 10.0).gap(6.0).key("stirr/window")
        .child(text("DRAG ME", 12.0, Paint::INK).font(Font::Mono).key("stirr/window/title"))
        .child(text("Throw it to a corner.", 11.0, Paint::DIM).key("stirr/window/hint"));
    let stage = layers([place, floating("stirr/float", &p.float, label, |m| Msg::Stirr(SMsg::Float(m)))])
        .key("stirr/stage").width(AREA.0).height(AREA.1).radius(8.0).border(1.0, Paint::FAINT).clip();
    let params: Vec<CaptionParam> = MAP_PARAMS.iter().enumerate().map(|(i, &(n, min, max, step, decimals, unit))| {
        CaptionParam::new(n, fx.get(i), Spec { min, max, step, decimals, unit }).draft(p.draft.as_ref().filter(|x| x.0 == i).map(|x| x.1.as_str()))
    }).collect();
    let tau = fx.energy_life;
    let formula = format!("e ← max(0, e·exp(−dt/{tau:.2}) − 0.15·dt)  ·  trail (1 − (d/R)²)², R = {:.0} + ½·short side  ·  k = max(own, e)  ·  at rest after ≤ {:.2} s",
        fx.stirr_radius, crate::fieldmap::settle_time(1.0, tau));
    col().key("stirr").gap(18.0).fill_width().min_width(0.0)
        .child(text("Field map", 30.0, Paint::INK).font(Font::Display).heading(1).key("stirr/heading"))
        .child(stage.caption_params("stirr/cap", formula, &params, |i, e| Msg::Stirr(SMsg::Num(i, e))))
        .child(row().gap(12.0).child(button_as("stirr/stir", "Stir", Variant::Default, Msg::Stirr(SMsg::Stir))))
}
