//! Lab · contours: iso-lines of a field (drifting noise plus a bump under the pointer) by marching
//! squares, drawn as strokes. The lines the bump bends are the accent.
use amenui::draw::Drawing;
use amenui::el::{El, Point, canvas};
use amenui::experiment::{Experiment, Params, Pointer, frame, ink, spec};
use amenui::field::{Bump, Grid, Noise, Scalar, Sum};
use amenui::widgets::{Edit, Spec};

const W: f32 = 240.0;
const H: f32 = 180.0;
static DEFS: [(&str, Spec); 4] = [("s", spec(12.0, 120.0, 1.0, 0, "px")), ("a", spec(0.0, 2.0, 0.05, 2, "")), ("r", spec(6.0, 90.0, 1.0, 0, "px")), ("n", spec(2.0, 16.0, 1.0, 0, ""))];

#[derive(Clone, Debug, PartialEq)]
pub struct Contours { pub p: Params, pub ptr: Pointer, pub t: f32 }
impl Default for Contours {
    fn default() -> Self { Self { p: Params::new(&DEFS, &[52.0, 0.7, 30.0, 9.0]), ptr: Pointer::new(W * 0.5, H * 0.5, 90.0, 16.0), t: 0.0 } }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Num(usize, Edit) }

impl Experiment for Contours {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let (s, a, r, n) = (self.p.get(0), self.p.get(1), self.p.get(2), self.p.get(3));
        let (mx, my) = self.ptr.at();
        let bump = Bump { x: mx, y: my, r, a: a * self.ptr.near };
        let noise = Noise { scale: s, speed: 0.06, t: self.t, seed: 1 };
        let g = Grid::sample(61, 46, W, H, &Sum(noise, bump));
        let levels: Vec<f32> = (1..=n as usize).map(|i| i as f32 / (n + 1.0) * (1.0 + a * 0.5)).collect();
        let mut d = Drawing::new(W, H);
        // The accent follows the contours the pointer's bump bends.
        d.contours(&g, &levels, 0.6, |x, y, _| ink(0.55, self.ptr.near * bump.at(x, y) / (a * 0.18).max(1e-3)));
        let piece = canvas(d).key(format!("{key}/canvas")).on_point(|p| Some(Msg::Point(p)));
        let formula = format!("f(x,y) = n(x/{s:.0}, y/{s:.0}, {:.2}) + {:.2}·e^(−|p−m|²/{r:.0}²)   {n:.0} levels", self.t * 0.06, a * self.ptr.near);
        frame(key, "contours", "move to bend", piece, formula, &self.p, Msg::Num, "marching squares · 61 × 46 nodes")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m { Msg::Point(p) => self.ptr.hear(p, now), Msg::Num(i, e) => self.p.edit(i, e) }
    }
    fn step(&mut self, now: f32) -> bool { self.t = now; self.ptr.step(now); true }
    fn moving(&self) -> bool { self.ptr.moving() }
}
