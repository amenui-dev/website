//! Lab · silk: many marks carried by a field, the curl of drifting noise (no sources, so they never
//! bunch), plus what the pointer stirs in: its velocity is put into a flow grid that fades back.
//! Each mark is drawn as the short stroke it travelled. The count is a request to the frame budget.
use amenui::draw::Drawing;
use amenui::el::{El, Point, canvas};
use amenui::experiment::{Experiment, Params, Pointer, frame, ink, spec};
use amenui::field::{Flow, Noise, Swarm, curl};
use amenui::widgets::{Edit, Spec};

const W: f32 = 240.0;
const H: f32 = 180.0;
static DEFS: [(&str, Spec); 4] = [("N", spec(100.0, 20000.0, 100.0, 0, "")), ("v", spec(0.0, 60.0, 1.0, 0, "px/s")), ("s", spec(10.0, 200.0, 1.0, 0, "px")), ("τ", spec(0.05, 3.0, 0.05, 2, "s"))];

#[derive(Clone, Debug, PartialEq)]
pub struct Silk { pub p: Params, pub ptr: Pointer, pub swarm: Swarm, pub stir: Flow, pub t: f32 }
impl Default for Silk {
    fn default() -> Self { Self { p: Params::new(&DEFS, &[1400.0, 22.0, 60.0, 0.6]), ptr: Pointer::new(W * 0.5, H * 0.5, 300.0, 30.0), swarm: Swarm::new(0, W, H, 9), stir: Flow::new(32, 20, W, H), t: 0.0 } }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Num(usize, Edit) }

impl Experiment for Silk {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let mut d = Drawing::new(W, H);
        // Each mark is a short thread: a streak back along where the field carries it, as long as it is
        // fast (a few frames of travel), so the field reads as flowing lines rather than speckle.
        for (p, s) in self.swarm.p.iter().zip(&self.swarm.d) {
            let len = s.0.hypot(s.1);
            if !(1e-4..8.0).contains(&len) { continue; }
            let (u, v) = self.stir.at(p.0, p.1);
            let l = (len * 26.0).clamp(2.5, 16.0);
            let (tx, ty) = (s.0 / len * l, s.1 / len * l);
            let mid = (p.0 - tx * 0.5, p.1 - ty * 0.5);
            let hot = (u.abs() + v.abs()) / 30.0;
            d.line((p.0 - tx, p.1 - ty), mid, 0.35, ink(0.22, hot));
            d.line(mid, *p, 0.45, ink(0.6, hot));
        }
        let piece = canvas(d).key(format!("{key}/canvas")).on_point(|p| Some(Msg::Point(p)));
        let formula = format!("ṗ = {:.0}·curl n(p/{:.0}) + u(p),  u += v_ptr·e^(−d²/24²), u ×= e^(−dt/{:.2})   N = {}", self.p.get(1), self.p.get(2), self.p.get(3), self.swarm.p.len());
        frame(key, "silk", "move to stir", piece, formula, &self.p, Msg::Num, "curl-noise advection · budgeted marks")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m { Msg::Point(p) => self.ptr.hear(p, now), Msg::Num(i, e) => self.p.edit(i, e) }
    }
    fn step(&mut self, now: f32) -> bool {
        let dt = (now - self.t).clamp(0.0, 0.05);
        self.t = now;
        // As many marks as the frame budget grants this frame (the request is N).
        let n = amenui::budget::grant(self.p.get(0) as usize);
        if n != self.swarm.p.len() { self.swarm = Swarm::new(n, W, H, 9); }
        self.ptr.step(now);
        if self.ptr.over { let (x, y) = self.ptr.at(); self.stir.stir(x, y, self.ptr.vx * dt * 4.0, self.ptr.vy * dt * 4.0, 24.0); }
        self.stir.fade(dt, self.p.get(3));
        let (k, field, stir) = (self.p.get(1) * self.p.get(2), Noise { scale: self.p.get(2), speed: 0.05, t: now, seed: 3 }, &self.stir);
        self.swarm.advect(dt, |x, y| { let (cx, cy) = curl(&field, x, y, 1.0); let (u, v) = stir.at(x, y); (cx * k + u, cy * k + v) });
        true
    }
    fn moving(&self) -> bool { self.ptr.moving() }
}
