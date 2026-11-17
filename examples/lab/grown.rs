//! Lab · grown: a Gray–Scott reaction–diffusion grown inside a real outline. The shape is an ordinary
//! element, laid out but hidden; `on_outline` hands its true coverage over as a mask, and the pattern
//! fills it in maze stripes, each dot sized by how much is there. Three shapes take turns by themselves
//! (two letters, a toggle control, a rounded square); each change decodes the old pattern away and the
//! new shape in (`El::decode`).
use amenui::decode::{Decode, Plan};
use amenui::draw::Drawing;
use amenui::el::{El, Font, Paint, Point, canvas, col, space, stack, text};
use amenui::experiment::{Experiment, Params, Pointer, frame, ink, spec};
use amenui::field::{GrayScott, Grid, Scalar, hash};
use amenui::widgets::{Edit, Spec, switch_sized};
use std::rc::Rc;

const W: f32 = 240.0;
const H: f32 = 180.0;
const N: (usize, usize) = (120, 90);
/// How long a new shape's pattern takes to decode in.
const DECODE_S: f32 = 0.76;

/// The decode of a new shape's pattern over its outline `m` (stage px): the cells of the finest tier
/// that are inside it.
fn decode_plan(m: &Grid, seed: u32) -> Plan {
    let spec = Decode { tiers: vec![24.0, 12.0, 6.0], stagger_ms: 110.0, duration_ms: DECODE_S * 1000.0, seed, hold: None };
    let (cols, rows) = ((m.w / 6.0).ceil() as usize, (m.h / 6.0).ceil() as usize);
    let cells: Vec<(f32, f32)> = (0..rows).flat_map(|j| (0..cols).map(move |i| (i as f32 * 6.0, j as f32 * 6.0))).filter(|&(x, y)| m.at(x + 3.0, y + 3.0) >= 0.5).collect();
    spec.plan(&cells, 1.0)
}
static DEFS: [(&str, Spec); 4] = [("f", spec(0.01, 0.08, 0.001, 3, "")), ("k", spec(0.04, 0.07, 0.001, 3, "")), ("steps", spec(1.0, 40.0, 1.0, 0, "/frame")), ("T", spec(2.0, 30.0, 0.5, 1, "s"))];

#[derive(Clone, Debug, PartialEq)]
pub struct Grown {
    pub p: Params, pub ptr: Pointer, pub gs: GrayScott, pub mask: Option<Rc<Grid>>,
    /// The shape asked for, the shape whose mask is growing (and how many masks came: the drawing's key),
    /// when it came, the stage's size as the last mask measured it.
    pub shape: usize, pub grown: usize, pub epoch: u32, pub since: f32, pub size: (f32, f32), pub t: f32,
    /// When the last new shape came (its pattern decodes in from then; a new outline of the same shape does not).
    pub changed: f32,
}
impl Default for Grown {
    fn default() -> Self {
        let mut gs = GrayScott::new(N.0, N.1);
        (gs.f, gs.k) = (0.029, 0.057);
        Self { p: Params::new(&DEFS, &[0.029, 0.057, 24.0, 7.0]), ptr: Pointer::new(0.0, 0.0, 300.0, 30.0), gs, mask: None, shape: 0, grown: 0, epoch: 0, since: 0.0, size: (360.0, 270.0), t: 0.0, changed: 0.0 }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Outline(Rc<Grid>), Num(usize, Edit) }

impl Experiment for Grown {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        // Each shape about fills the stage (its size as the last outline measured it).
        let (sw, sh) = self.size;
        let shape: El<Msg> = match self.shape {
            0 => text("Aa", sh * 0.58, Paint::INK).font(Font::Face(0)).weight(sh * 0.035),
            1 => switch_sized(format!("{key}/toggle"), "", true, None, (sw * 0.36).min(sh * 0.5), Msg::Num(9, Edit::Cancel)),
            _ => space(sw * 0.62, sh * 0.72).radius(sh * 0.2).fill(Paint::INK),
        };
        let hidden = col().key(format!("{key}/shape/{}", self.shape)).fill_width().fill_height().center().child(shape).hidden_until_morphed(true).on_outline(N.0, N.1, |g| Some(Msg::Outline(g)));
        // A new shape's pattern decodes in (DESIGN.md §1.28's blocks, 24 → 12 → 6 px over its outline,
        // then the pattern) on the cell's own clock. The blocks are marks of the drawing, not
        // `El::decode`: a cell that leaves (the page changes) is simply gone, never decoded backwards
        // over the next page.
        let u = ((self.t - self.changed) / DECODE_S).clamp(0.0, 1.0);
        let plan = self.mask.as_ref().filter(|_| u < 1.0).map(|m| decode_plan(m, self.epoch));
        let content = plan.as_ref().is_none_or(|p| p.shows_content(u));
        let mut d = Drawing::new(W, H);
        let (v, (mx, my), near) = (self.gs.v_grid(W, H), self.ptr.at(), self.ptr.near);
        if let Some(m) = self.mask.as_ref().filter(|_| content) {
            let m = Grid { w: W, h: H, ..(**m).clone() };
            // The shape's ground: a faint lattice inside the outline, and the outline itself.
            d.dots(&m, 6.0, |x| if x >= 0.5 { 0.5 } else { 0.0 }, |_, _, _| Paint::tone(0.12));
            d.contours(&m, &[0.5], 0.5, |_, _, _| Paint::FAINT);
        }
        // The stripes: a dot per cell, sized by v; where the pointer is (and what it seeds) in the accent.
        let hot = |x: f32, y: f32| near * (1.0 - (x - mx).hypot(y - my) / 22.0);
        if content { d.dots(&v, W / N.0 as f32, |x| if x > 0.12 { (0.35 + 2.6 * x).min(1.2) } else { 0.0 }, |x, y, _| ink(0.8, hot(x, y))); }
        // The blocks over the pattern, as marks of the drawing (stage px to the drawing's): each a
        // stack of 2 px strokes, square but for its corners.
        let k = W / sw.max(1.0);
        for b in plan.iter().flat_map(|p| p.at(u)) {
            let paint = Paint { tone: 1.0, accent: 1.0, alpha: 1.0, hue: b.hue };
            for r in 0..(b.size / 2.0).round().max(1.0) as u32 {
                let y = (b.y + 1.0 + r as f32 * 2.0) * k;
                d.line(((b.x + 1.0) * k, y), ((b.x + b.size - 1.0) * k, y), 2.0 * k, paint);
            }
        }
        let drawing = canvas(d).key(format!("{key}/canvas")).on_point(|p| Some(Msg::Point(p)));
        let piece = stack([hidden, drawing]).fill_width().fill_height();
        let formula = format!("u′ = ∇²u − uv² + {:.3}(1−u),  v′ = ½∇²v + uv² − ({:.3}+{:.3})v   × {:.0}/frame", self.gs.f, self.gs.f, self.gs.k, self.p.get(2));
        frame(key, "grown", "drag to seed", piece, formula, &self.p, Msg::Num, "Gray–Scott in a true outline · 120 × 90")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Point(p) => {
                self.ptr.hear(p, now);
                if let Some(q) = p.filter(|q| q.down) { self.gs.seed_at((q.x / W * N.0 as f32) as usize, (q.y / H * N.1 as f32) as usize, 2); }
            }
            Msg::Outline(g) => {
                self.size = (g.w, g.h);
                self.gs.reset(&g, self.shape as u32 + 1);
                // More seeds than the reset's dozen, so the maze fills the shape within its turn.
                let inside: Vec<usize> = (0..N.0 * N.1).filter(|&k| self.gs.mask[k] >= 0.5).collect();
                for s in 0..36u32 { if let Some(&k) = inside.get((hash(s * 977 + self.epoch) * inside.len() as f32) as usize) { self.gs.seed_at(k % N.0, k / N.0, 1); } }
                if self.grown != self.shape || self.mask.is_none() { self.epoch += 1; self.changed = now; }
                self.grown = self.shape;
                self.mask = Some(g);
                self.since = now;
            }
            Msg::Num(i, e) => if i < 4 || i == amenui::experiment::TOGGLE { self.p.edit(i, e); (self.gs.f, self.gs.k) = (self.p.get(0), self.p.get(1)); },
        }
    }
    fn step(&mut self, now: f32) -> bool {
        self.t = now;
        self.ptr.step(now);
        // Every T seconds the next shape: its outline comes, the pattern starts again inside it.
        if self.mask.is_some() && self.grown == self.shape && now - self.since > self.p.get(3) { self.shape = (self.shape + 1) % 3; }
        self.gs.step(self.p.get(2) as usize);
        true
    }
    fn moving(&self) -> bool { self.ptr.moving() }
}
