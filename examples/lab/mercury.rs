//! Lab · mercury: metaballs. Σ rᵢ²/dᵢ² over a few balls; the surface is where it reaches 1, drawn as
//! its contour with the inside filled in dots. One ball follows the pointer on a spring, the others
//! drift; near each other they merge, pulled apart they split.
use amenui::draw::Drawing;
use amenui::el::{El, Point, canvas};
use amenui::experiment::{Experiment, Params, Pointer, frame, ink, spec};
use amenui::field::{Balls, Grid, Scalar};
use amenui::widgets::{Edit, Spec};

const W: f32 = 240.0;
const H: f32 = 180.0;
static DEFS: [(&str, Spec); 4] = [("r0", spec(4.0, 40.0, 0.5, 1, "px")), ("r", spec(4.0, 40.0, 0.5, 1, "px")), ("k", spec(10.0, 600.0, 10.0, 0, "N/m")), ("c", spec(1.0, 60.0, 1.0, 0, "N·s/m"))];

#[derive(Clone, Debug, PartialEq)]
pub struct Mercury { pub p: Params, pub ptr: Pointer, pub t: f32 }
impl Default for Mercury {
    fn default() -> Self { Self { p: Params::new(&DEFS, &[22.0, 16.0, 120.0, 14.0]), ptr: Pointer::new(W * 0.5, H * 0.5, 120.0, 14.0), t: 0.0 } }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Num(usize, Edit) }

impl Mercury {
    fn balls(&self) -> Balls {
        let (t, r) = (self.t * 0.5, self.p.get(1));
        let mut b: Vec<(f32, f32, f32)> = (0..3).map(|i| { let a = t * (0.6 + 0.2 * i as f32) + i as f32 * 2.1; (W * 0.5 + 74.0 * a.cos(), H * 0.5 + 50.0 * (1.3 * a).sin(), r) }).collect();
        let (x, y) = self.ptr.at();
        b.push((x, y, self.p.get(0)));
        Balls(b)
    }
}

impl Experiment for Mercury {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let (balls, (mx, my), near) = (self.balls(), self.ptr.at(), self.ptr.near);
        let g = Grid::sample(73, 55, W, H, &balls);
        let hot = |x: f32, y: f32| near * (1.0 - (x - mx).hypot(y - my) / (self.p.get(0) * 2.2));
        let mut d = Drawing::new(W, H);
        d.dots(&balls, 4.0, |v| if v >= 1.0 { 0.6 + 0.5 * (v - 1.0).min(1.0) } else { 0.0 }, |x, y, _| ink(0.32, hot(x, y)));
        d.contours(&g, &[1.0], 1.0, |x, y, _| ink(0.9, hot(x, y)));
        d.contours(&g, &[0.6], 0.5, |_, _, _| ink(0.3, 0.0));
        let piece = canvas(d).key(format!("{key}/canvas")).on_point(|p| Some(Msg::Point(p)));
        let formula = format!("Σ r_i²/d_i² = 1   r₀ = {:.1}, r = {:.1}   m″ = {:.0}(p−m) − {:.0}m′   max = {:.2}", self.p.get(0), self.p.get(1), self.p.get(2), self.p.get(3), balls.at(mx, my).min(99.0));
        frame(key, "mercury", "move to pull", piece, formula, &self.p, Msg::Num, "metaballs · contour at 1, 0.6")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Point(p) => self.ptr.hear(p, now),
            Msg::Num(i, e) => { self.p.edit(i, e); for a in [&mut self.ptr.x, &mut self.ptr.y] { (a.spring.stiffness, a.spring.damping) = (self.p.get(2), self.p.get(3)); } }
        }
        if !self.ptr.over { let (vx, vy) = (self.ptr.x.velocity, self.ptr.y.velocity); self.ptr.x.to(W * 0.5, vx, now); self.ptr.y.to(H * 0.5, vy, now); }
    }
    fn step(&mut self, now: f32) -> bool { self.t = now; self.ptr.step(now); true }
    fn moving(&self) -> bool { self.ptr.moving() }
}
