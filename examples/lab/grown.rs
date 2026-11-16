//! Lab · grown: a Gray–Scott reaction–diffusion grown inside a real outline. The shape is an ordinary
//! element, laid out but hidden; `on_outline` hands its true coverage over as a mask, and the pattern
//! fills it like a maze. Three shapes take turns: two letters, a toggle control, a rounded square.
use lab::draw::Drawing;
use lab::el::{El, Font, Paint, Point, canvas, col, space, stack, text};
use lab::experiment::{Experiment, Params, Pointer, frame, ink, spec};
use lab::field::{GrayScott, Grid};
use lab::widgets::{Edit, Spec, switch_sized};
use std::rc::Rc;

const W: f32 = 240.0;
const H: f32 = 150.0;
const N: (usize, usize) = (120, 75);
static DEFS: [(&str, Spec); 4] = [("f", spec(0.01, 0.08, 0.001, 3, "")), ("k", spec(0.04, 0.07, 0.001, 3, "")), ("steps", spec(1.0, 40.0, 1.0, 0, "/frame")), ("T", spec(2.0, 30.0, 0.5, 1, "s"))];

#[derive(Clone, Debug, PartialEq)]
pub struct Grown { pub p: Params, pub ptr: Pointer, pub gs: GrayScott, pub mask: Option<Rc<Grid>>, pub shape: usize, pub since: f32, pub t: f32 }
impl Default for Grown {
    fn default() -> Self { Self { p: Params::new(&DEFS, &[0.037, 0.060, 24.0, 9.0]), ptr: Pointer::new(0.0, 0.0, 300.0, 30.0), gs: GrayScott::new(N.0, N.1), mask: None, shape: 0, since: 0.0, t: 0.0 } }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Outline(Rc<Grid>), Num(usize, Edit) }

impl Experiment for Grown {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let shape: El<Msg> = match self.shape {
            0 => text("Aa", 128.0, Paint::INK).font(Font::Face(0)),
            1 => switch_sized(format!("{key}/toggle"), "", true, None, 150.0, Msg::Num(9, Edit::Cancel)),
            _ => space(130.0, 112.0).radius(30.0).fill(Paint::INK),
        };
        let hidden = col().key(format!("{key}/shape/{}", self.shape)).fill_width().fill_height().center().child(shape).hidden_until_morphed(true).on_outline(N.0, N.1, |g| Some(Msg::Outline(g)));
        let mut d = Drawing::new(W, H);
        if let Some(m) = &self.mask { d.contours(&Grid { w: W, h: H, ..(**m).clone() }, &[0.5], 0.5, |_, _, _| Paint::FAINT); }
        let (v, (mx, my), near) = (self.gs.v_grid(W, H), self.ptr.at(), self.ptr.near);
        d.dots(&v, W / N.0 as f32, |x| if x > 0.1 { 1.25 * (x * 3.5).min(1.0) } else { 0.0 }, |x, y, _| ink(0.75, near * (1.0 - (x - mx).hypot(y - my) / 18.0)));
        let piece = stack([hidden, canvas(d).key(format!("{key}/canvas")).on_point(|p| Some(Msg::Point(p)))]).fill_width();
        let formula = format!("u′ = ∇²u − uv² + {:.3}(1−u),  v′ = ½∇²v + uv² − ({:.3}+{:.3})v   × {:.0}/frame", self.gs.f, self.gs.f, self.gs.k, self.p.get(2));
        frame(key, "grown", "drag to seed", piece, formula, &self.p.caption(), Msg::Num, "Gray–Scott in a true outline · 120 × 75")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Point(p) => {
                self.ptr.hear(p, now);
                if let Some(q) = p.filter(|q| q.down) { self.gs.seed_at((q.x / W * N.0 as f32) as usize, (q.y / H * N.1 as f32) as usize, 2); }
            }
            Msg::Outline(g) => { self.gs.reset(&g, self.shape as u32 + 1); self.mask = Some(g); self.since = now; }
            Msg::Num(i, e) => if i < 4 { self.p.edit(i, e); (self.gs.f, self.gs.k) = (self.p.get(0), self.p.get(1)); },
        }
    }
    fn step(&mut self, now: f32) -> bool {
        self.t = now;
        self.ptr.step(now);
        if self.mask.is_some() && now - self.since > self.p.get(3) { self.shape = (self.shape + 1) % 3; self.since = now; }
        self.gs.step(self.p.get(2) as usize);
        true
    }
    fn moving(&self) -> bool { true }
}
