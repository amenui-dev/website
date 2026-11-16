//! The decode transition's page (DESIGN.md §1.28): a card decoding at the timeline's progress, the same
//! plan exploded into its tiers in depth (a 3D view, one layer per tier, labelled), a timeline to drag,
//! the numbers, and a dialog that opens and closes by decoding.
use super::{Gallery, Msg};
use crate::decode::{Decode, Plan};
use crate::draw::Drawing;
use crate::el::{El, Font, Paint, Point, canvas, col, row, scene3d, space, text};
use crate::mesh::{Camera, Light, Mat4, Mesh3d, MeshId, MeshInstances, Projection, Scene3d};
use crate::widgets::{CaptionParam, Edit, Spec, Variant, button_as, dialog};

/// The card's size (logical px).
const CARD: (f32, f32) = (260.0, 150.0);

#[derive(Clone, Debug, PartialEq)]
pub struct DecodePage { pub spec: Decode, pub u: f32, pub playing: Option<f32>, pub open: bool, pub draft: Option<(usize, String)> }
impl Default for DecodePage { fn default() -> Self { Self { spec: Decode::default(), u: 0.55, playing: None, open: false, draft: None } } }

#[derive(Clone, Debug, PartialEq)]
pub enum DMsg { Scrub(Option<Point>), Play, Open(bool), Num(usize, Edit) }

const fn spec(min: f32, max: f32, step: f32, decimals: usize, unit: &'static str) -> Spec { Spec { min, max, step, decimals, unit } }
const NAMES: [(&str, Spec); 8] = [("u", spec(0.0, 1.0, 0.01, 2, "")), ("tier0", spec(2.0, 64.0, 1.0, 0, "px")), ("tier1", spec(2.0, 64.0, 1.0, 0, "px")), ("tier2", spec(2.0, 64.0, 1.0, 0, "px")), ("tier3", spec(2.0, 64.0, 1.0, 0, "px")),
    ("stagger", spec(0.0, 400.0, 5.0, 0, "ms")), ("duration", spec(100.0, 4000.0, 10.0, 0, "ms")), ("seed", spec(0.0, 999.0, 1.0, 0, ""))];

impl DecodePage {
    fn values(&self) -> [f32; 8] {
        let t = |i: usize| self.spec.tiers.get(i).copied().unwrap_or(0.0);
        [self.u, t(0), t(1), t(2), t(3), self.spec.stagger_ms, self.spec.duration_ms, self.spec.seed as f32]
    }
    /// The plan over the card's whole box (the card is filled, so its ink is its box).
    fn plan(&self) -> Plan {
        let f = self.spec.tiers.last().copied().unwrap_or(4.0);
        let cells: Vec<(f32, f32)> = (0..(CARD.1 / f) as usize).flat_map(|j| (0..(CARD.0 / f) as usize).map(move |i| (i as f32 * f, j as f32 * f))).collect();
        self.spec.plan(&cells, 1.0)
    }
}

pub fn update(s: &mut Gallery, m: DMsg) {
    let (d, now) = (&mut s.decode, s.clock);
    match m {
        DMsg::Scrub(Some(p)) if p.down => { d.u = (p.x / 600.0).clamp(0.0, 1.0); d.playing = None; }
        DMsg::Scrub(_) => {}
        DMsg::Play => d.playing = Some(now),
        DMsg::Open(o) => d.open = o,
        DMsg::Num(i, e) => match e {
            Edit::Draft(t) => d.draft = Some((i, t)),
            Edit::Cancel => d.draft = None,
            Edit::Set(v) => {
                let v = NAMES[i].1.clamp(v);
                match i { 0 => d.u = v, 1..=4 => { if d.spec.tiers.len() < 4 { d.spec.tiers.resize(4, 4.0); } d.spec.tiers[i - 1] = v; } 5 => d.spec.stagger_ms = v, 6 => d.spec.duration_ms = v, _ => d.spec.seed = v as u32 }
                d.draft = None;
            }
        },
    }
}
/// While it plays, the timeline runs from 0 to 1.
pub fn tick(s: &mut Gallery, t: f32) -> bool {
    let d = &mut s.decode;
    let Some(t0) = d.playing else { return false };
    d.u = ((t - t0) / (d.spec.duration_ms / 1000.0)).clamp(0.0, 1.0);
    if d.u >= 1.0 { d.playing = None; }
    true
}
pub fn wake(s: &Gallery, t: f32) -> Option<f32> { s.decode.playing.map(|_| t) }

pub fn page(s: &Gallery) -> El<Msg> {
    let d = &s.decode;
    let plan = d.plan();
    let m = |x: DMsg| Msg::Decode(x);
    // The card, held at the timeline's progress.
    let card = col().key("decode/card").width(CARD.0).height(CARD.1).pad(16.0, 14.0).gap(8.0).radius(6.0).fill(Paint::tone(0.12)).border(1.0, Paint::FAINT)
        .child(text("DECODE", 14.0, Paint::INK).font(Font::Mono))
        .child(text("Blocks over the true outline resolve 32 → 16 → 8 → 4, then the content.", 11.0, Paint::DIM).font(Font::Mono))
        .decode(Decode { hold: Some(d.u), ..d.spec.clone() });
    // Exploded: one layer per tier, apart in depth, the blocks showing at this progress.
    let n = d.spec.tiers.len().max(1);
    let mut items = vec![];
    for t in 0..n { for hue in 0..2u8 {
        let tf: Vec<Mat4> = plan.at(d.u).filter(|b| b.tier as usize == t && b.hue == hue)
            .map(|b| Mat4::translate((b.x + b.size * 0.5) / CARD.1 - CARD.0 / CARD.1 * 0.5, 0.5 - (b.y + b.size * 0.5) / CARD.1, t as f32 * 0.7).mul(&Mat4::scale(b.size / CARD.1 * 0.92, b.size / CARD.1 * 0.92, 0.01))).collect();
        if !tf.is_empty() { items.push(MeshInstances { mesh: MeshId::CUBE, transforms: tf, paint: Paint::hue(hue), id: (t * 2 + hue as usize + 1) as u32 }); }
    } }
    let camera = Camera { projection: Projection::Perspective { fov: 30.0 }, yaw: -58.0, pitch: 20.0, distance: 6.4, target: [0.0, 0.0, 1.0], ..Camera::default() };
    let exploded = scene3d(Scene3d { items, camera, light: Light::from_angles(-30.0, 50.0, 0.6), spec: Mesh3d { tones: 2.0, ..Mesh3d::default() }, picture: None }).key("decode/exploded").width(420.0).height(260.0);
    let (sw, ww, c) = d.spec.windows();
    let labels = (0..n).fold(col().key("decode/labels").gap(10.0), |c2, t| c2.child(text(format!("tier {t} · {:.0} px · on {:.0}–{:.0}%  · {} now", d.spec.tiers[t], t as f32 * sw * 100.0, (t as f32 * sw + ww) * 100.0, plan.at(d.u).filter(|b| b.tier as usize == t).count()), 11.0, Paint::DIM).font(Font::Mono).key(format!("decode/label/{t}"))))
        .child(text(format!("content from {:.0}%, last off {:.0}%", c * 100.0, (c + ww).min(1.0) * 100.0), 11.0, Paint::DIM).font(Font::Mono).key("decode/label/content"));
    // The timeline: the tiers' windows, the content's, and where it is now; drag to scrub.
    let mut tl = Drawing::new(600.0, 40.0);
    for t in 0..n { let y = 6.0 + t as f32 * 6.0; tl.line((t as f32 * sw * 600.0, y), ((t as f32 * sw + ww) * 600.0, y), 2.0, Paint::DIM); }
    tl.line((c * 600.0, 32.0), ((c + ww) * 600.0, 32.0), 2.0, Paint::FAINT);
    tl.line((d.u * 600.0, 0.0), (d.u * 600.0, 40.0), 1.5, Paint::ACCENT);
    let timeline = canvas(tl).key("decode/timeline").width(600.0).on_point(|p| Some(Msg::Decode(DMsg::Scrub(p))));
    let params: Vec<CaptionParam> = NAMES.iter().zip(d.values()).enumerate().map(|(i, ((n, sp), v))| CaptionParam::new(*n, v, *sp).draft(d.draft.as_ref().filter(|x| x.0 == i).map(|x| x.1.as_str()))).collect();
    let formula = format!("tier i on over [i·{:.0}, i·{:.0} + {:.0}] ms, content at {:.0} ms, finest off by {:.0} ms   u = {:.2}", d.spec.stagger_ms, d.spec.stagger_ms, ww * d.spec.duration_ms, c * d.spec.duration_ms, d.spec.duration_ms, d.u);
    let body = col().gap(10.0).child(text("This dialog appeared by decoding; closing it plays the same backwards.", 13.0, Paint::TEXT).key("decode/dialog/text"))
        .child(button_as("decode/close", &crate::i18n::tr("widget.dialog.close"), Variant::Default, m(DMsg::Open(false))));
    col().key("decode").gap(18.0).fill_width().min_width(0.0)
        .child(text("Decode", 30.0, Paint::INK).font(Font::Display).heading(1).key("decode/title"))
        .child(row().key("decode/row").wrap().gap(28.0).child(col().gap(12.0).child(card).child(space(0.0, 4.0))).child(exploded).child(labels))
        .child(timeline.caption_params("decode/cap", formula, &params, move |i, e| Msg::Decode(DMsg::Num(i, e))))
        .child(row().gap(12.0).child(button_as("decode/play", "Play", Variant::Default, m(DMsg::Play))).child(button_as("decode/open", "Open a dialog", Variant::Accent, m(DMsg::Open(true)))))
        .child(dialog("decode/dialog", d.open, "Decode", 380.0, 0.35, m(DMsg::Open(false)), body).decode(Decode { hold: None, ..d.spec.clone() }))
}
