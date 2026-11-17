//! Lab · orbit: dots on circular orbits, each at its Kepler speed ω ∝ r^−3/2, pulled off their paths
//! by the pointer and sprung back onto them when it goes.
use amenui::draw::Drawing;
use amenui::el::{El, Point, canvas};
use amenui::experiment::{Experiment, Params, Pointer, frame, ink, spec};
use amenui::field::hash;
use amenui::widgets::{Edit, Spec};

const W: f32 = 240.0;
const H: f32 = 180.0;
const DOTS: usize = 90;
static DEFS: [(&str, Spec); 4] = [("ω1", spec(0.0, 4.0, 0.05, 2, "rad/s")), ("G", spec(0.0, 40000.0, 500.0, 0, "")), ("k", spec(1.0, 200.0, 1.0, 0, "/s²")), ("c", spec(0.0, 30.0, 0.5, 1, "/s"))];

#[derive(Clone, Debug, PartialEq)]
pub struct Orbit { pub p: Params, pub ptr: Pointer, pub a: Vec<f32>, pub off: Vec<(f32, f32, f32, f32)>, pub t: f32 }
impl Default for Orbit {
    fn default() -> Self { Self { p: Params::new(&DEFS, &[1.2, 9000.0, 26.0, 5.0]), ptr: Pointer::new(W * 0.5, H * 0.5, 200.0, 26.0), a: (0..DOTS).map(|i| hash(i as u32) * std::f32::consts::TAU).collect(), off: vec![(0.0, 0.0, 0.0, 0.0); DOTS], t: 0.0 } }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Num(usize, Edit) }

/// Dot `i`'s orbit radius (0..1 of the half-height, in rings).
fn ring(i: usize) -> f32 { 0.25 + 0.22 * (i % 4) as f32 + 0.04 * hash(i as u32 + 99) }

impl Orbit {
    fn at(&self, i: usize) -> (f32, f32) { let r = ring(i) * H * 0.5; (W * 0.5 + r * 1.5 * self.a[i].cos() + self.off[i].0, H * 0.5 + r * self.a[i].sin() + self.off[i].1) }
}

impl Experiment for Orbit {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let ((mx, my), near) = (self.ptr.at(), self.ptr.near);
        let mut d = Drawing::new(W, H);
        for q in 0..4 { let r = (0.25 + 0.22 * q as f32 + 0.02) * H * 0.5; let pts: Vec<(f32, f32)> = (0..=72).map(|k| { let a = k as f32 / 72.0 * std::f32::consts::TAU; (W * 0.5 + r * 1.5 * a.cos(), H * 0.5 + r * a.sin()) }).collect(); d.polyline(&pts, 0.4, |_, _| ink(0.18, 0.0)); }
        for i in 0..DOTS { let (x, y) = self.at(i); d.dot(x, y, 2.2, ink(0.85, near * (1.0 - (x - mx).hypot(y - my) / 36.0))); }
        let piece = canvas(d).key(format!("{key}/canvas")).on_point(|p| Some(Msg::Point(p)));
        let formula = format!("ω(r) = {:.2}·r^−1.5,  δ″ = {:.0}·(m−p)/(|m−p|²+64)·{near:.2} − {:.0}δ − {:.1}δ′", self.p.get(0), self.p.get(1), self.p.get(2), self.p.get(3));
        frame(key, "orbit", "move to attract", piece, formula, &self.p, Msg::Num, "Kepler speeds · sprung offsets")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m { Msg::Point(p) => self.ptr.hear(p, now), Msg::Num(i, e) => self.p.edit(i, e) }
    }
    fn step(&mut self, now: f32) -> bool {
        let dt = (now - self.t).clamp(0.0, 0.05);
        self.t = now;
        self.ptr.step(now);
        let ((mx, my), near, (w1, g, k, c)) = (self.ptr.at(), self.ptr.near, (self.p.get(0), self.p.get(1), self.p.get(2), self.p.get(3)));
        for i in 0..DOTS {
            self.a[i] += w1 * ring(i).powf(-1.5) * 0.25 * dt;
            let (x, y) = self.at(i);
            let (dx, dy) = (mx - x, my - y);
            let pull = g * near / (dx * dx + dy * dy + 64.0);
            let o = &mut self.off[i];
            o.2 += (pull * dx - k * o.0 - c * o.2) * dt;
            o.3 += (pull * dy - k * o.1 - c * o.3) * dt;
            o.0 += o.2 * dt;
            o.1 += o.3 * dt;
        }
        true
    }
    fn moving(&self) -> bool { self.ptr.moving() }
}
