//! The decode transition's page (DESIGN.md §1.28): a card decoding at the timeline's progress, the same
//! plan exploded into its tiers in depth (one oblique plane per tier, each with a leader and a label), a timeline to drag,
//! the numbers, and a dialog that opens and closes by decoding.
use super::{Gallery, Msg};
use crate::decode::{Decode, Plan};
use crate::draw::Drawing;
use crate::el::{El, Font, Paint, Point, canvas, col, row, space, text};
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
    // Exploded: one plane per tier, apart in depth (an oblique view: each plane squashed to 0.42 of its
    // height and sheared, the planes stepping down and right), the blocks showing at this progress on
    // each, and the content at the bottom; a leader from each plane's right edge to its label.
    let n = d.spec.tiers.len().max(1);
    let (sw, ww, c) = d.spec.windows();
    let (k, sq, shear, step) = (0.84, 0.42, 0.55, 74.0);
    let ph = CARD.1 * k * sq;
    let (x0, y0) = (8.0, 10.0);
    let at = |layer: usize, x: f32, y: f32| { let ox = x0 + layer as f32 * 16.0; let oy = y0 + layer as f32 * step; (ox + x * k + (ph - y * k * sq) * shear, oy + y * k * sq) };
    let (ew, eh) = (CARD.0 * k + ph * shear + 16.0 * n as f32 + 350.0, y0 + n as f32 * step + ph + 12.0);
    let mut ex = Drawing::new(ew, eh);
    let mut labels: Vec<El<Msg>> = vec![];
    for t in 0..=n {
        // The plane's outline, then what is on it.
        let corners = [at(t, 0.0, 0.0), at(t, CARD.0, 0.0), at(t, CARD.0, CARD.1), at(t, 0.0, CARD.1), at(t, 0.0, 0.0)];
        ex.polyline(&corners, 0.8, |_, _| if t == n { Paint::DIM } else { Paint::FAINT });
        if t < n {
            for b in plan.at(d.u).filter(|b| b.tier as usize == t) {
                // A block is a bar across the plane: its top edge sheared, squashed to the plane.
                let (bx, by) = at(t, b.x + b.size * 0.06, b.y + b.size * 0.5);
                let (ex1, _) = at(t, b.x + b.size * 0.94, b.y + b.size * 0.5);
                ex.line((bx, by), (ex1, by), b.size * k * sq * 0.86, Paint::hue(b.hue));
            }
        } else if plan.shows_content(d.u) {
            for j in 0..3 { let y = 26.0 + j as f32 * 22.0; let (a1, b1) = (at(t, 16.0, y), at(t, if j == 0 { 90.0 } else { 230.0 }, y)); ex.line(a1, b1, 2.0, Paint::DIM); }
        }
        // The leader: from the plane's right edge, level, to the label beside it.
        let (rx, ry) = at(t, CARD.0, CARD.1 * 0.5);
        let lx = CARD.0 * k + ph * shear + x0 + 16.0 * n as f32 + 24.0;
        ex.line((rx + 4.0, ry), (lx - 6.0, ry), 0.6, Paint::DIM);
        ex.dot(rx + 4.0, ry, 2.4, Paint::INK);
        let label = if t < n {
            format!("tier {t} · {:.0} px · on {:.0}–{:.0}% · {} now", d.spec.tiers[t], t as f32 * sw * 100.0, (t as f32 * sw + ww) * 100.0, plan.at(d.u).filter(|b| b.tier as usize == t).count())
        } else { format!("content · from {:.0}%, last off {:.0}%", c * 100.0, (c + ww).min(1.0) * 100.0) };
        labels.push(text(label, 11.0, if t < n { Paint::TEXT } else { Paint::DIM }).font(Font::Mono).nowrap().key(format!("decode/label/{t}")).shift(lx, ry - 8.0));
    }
    let mut parts = vec![canvas(ex).key("decode/exploded").width(ew)];
    parts.extend(labels);
    let exploded = crate::el::layers(parts).key("decode/exploded-view").width(ew).height(eh);
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
        .child(row().key("decode/row").wrap().gap(28.0).child(col().gap(12.0).child(card).child(space(0.0, 4.0))).child(exploded))
        .child(timeline.caption_params("decode/cap", formula, &params, move |i, e| Msg::Decode(DMsg::Num(i, e))))
        .child(row().gap(12.0).child(button_as("decode/play", "Play", Variant::Default, m(DMsg::Play))).child(button_as("decode/open", "Open a dialog", Variant::Accent, m(DMsg::Open(true)))))
        .child(dialog("decode/dialog", d.open, "Decode", 380.0, 0.35, m(DMsg::Open(false)), body).decode(Decode { hold: None, ..d.spec.clone() }))
}
