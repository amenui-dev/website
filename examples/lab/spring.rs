//! Lab · spring: a mass on a spring (the same `gesture::Spring` the sheets use, solved exactly).
//! Drag it along its track; let go and it springs back with the speed it was thrown at, its last two
//! seconds traced beneath.
use amenui::draw::Drawing;
use amenui::el::{El, Point, canvas};
use amenui::experiment::{Experiment, Params, frame, ink, spec};
use amenui::gesture::{Animated, Spring as Law, Velocity};
use amenui::widgets::{Edit, Spec};

const W: f32 = 240.0;
const H: f32 = 180.0;
const REST: f32 = 120.0;
static DEFS: [(&str, Spec); 3] = [("k", spec(10.0, 1200.0, 10.0, 0, "N/m")), ("c", spec(0.0, 80.0, 0.5, 1, "N·s/m")), ("m", spec(0.2, 5.0, 0.1, 1, "kg"))];

#[derive(Clone, Debug, PartialEq)]
pub struct Spring { pub p: Params, pub x: Animated, pub held: bool, pub v: Velocity, pub trace: Vec<(f32, f32)>, pub t: f32 }
impl Default for Spring {
    fn default() -> Self { Self { p: Params::new(&DEFS, &[180.0, 6.0, 1.0]), x: Animated::new(REST, Law { stiffness: 180.0, damping: 6.0, ..Law::default() }), held: false, v: Velocity::default(), trace: vec![], t: 0.0 } }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Num(usize, Edit) }

impl Experiment for Spring {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let (x, a) = (self.x.value, if self.held { 1.0 } else { 0.0 });
        let mut d = Drawing::new(W, H);
        const Y: f32 = 62.0;
        d.line((12.0, Y), (W - 12.0, Y), 0.5, ink(0.25, 0.0));
        d.line((REST, Y - 10.0), (REST, Y + 10.0), 0.5, ink(0.4, 0.0));
        let coil: Vec<(f32, f32)> = (0..=40).map(|i| { let f = i as f32 / 40.0; (12.0 + (x - 12.0) * f, Y + if i % 2 == 0 { -7.0 } else { 7.0 } * (i > 0 && i < 40) as i32 as f32) }).collect();
        d.polyline(&coil, 0.5, |_, _| ink(0.5, a));
        d.dot(x, Y, 9.0, ink(0.95, a));
        let t0 = self.t - 2.0;
        let pts: Vec<(f32, f32)> = self.trace.iter().map(|&(t, v)| (12.0 + (t - t0) / 2.0 * (W - 24.0), 136.0 + (v - REST) * 0.35)).collect();
        d.line((12.0, 136.0), (W - 12.0, 136.0), 0.3, ink(0.15, 0.0));
        d.polyline(&pts, 0.6, |_, _| ink(0.6, a));
        let piece = canvas(d).key(format!("{key}/canvas")).on_point(|p| Some(Msg::Point(p)));
        let (k, c, m) = (self.p.get(0), self.p.get(1), self.p.get(2));
        let formula = format!("{m:.1}·x″ = −{k:.0}(x − x₀) − {c:.1}·x′   ζ = {:.2}   x−x₀ = {:+.1}", self.x.spring.ratio(), x - REST);
        frame(key, "spring", "drag and let go", piece, formula, &self.p, Msg::Num, "damped oscillator · exact steps")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Point(Some(p)) if p.down => { self.held = true; self.v.push(now, p.x, 0.0); self.x.hold(p.x.clamp(12.0, W - 12.0)); }
            Msg::Point(_) if self.held => { self.held = false; let v = self.v.get().0; self.v.clear(); self.x.to(REST, v, now); }
            Msg::Point(_) => {}
            Msg::Num(i, e) => { self.p.edit(i, e); (self.x.spring.stiffness, self.x.spring.damping, self.x.spring.mass) = (self.p.get(0), self.p.get(1), self.p.get(2)); }
        }
    }
    fn step(&mut self, now: f32) -> bool {
        self.t = now;
        self.x.step(now);
        self.trace.push((now, self.x.value));
        self.trace.retain(|&(t, _)| now - t <= 2.0);
        true
    }
    fn moving(&self) -> bool { self.held || self.x.moving() || self.trace.windows(2).any(|w| w[0].1 != w[1].1) }
}
