//! Lab · flow: a line of text on a path. The baseline is a travelling wave; the pointer pulls a bump
//! into it. Each glyph sits on the curve, turned to its tangent, with its own true outline.
use lab::el::{El, Font, GlyphAt, Paint, Point, Pose, col, text};
use lab::experiment::{Experiment, Params, Pointer, frame, spec};
use lab::widgets::{Edit, Spec};

const LINE: &str = "text that rides a wave, letter by letter, along its own baseline";
const MID: f32 = 70.0;
static DEFS: [(&str, Spec); 4] = [("A", spec(0.0, 40.0, 0.5, 1, "px")), ("λ", spec(20.0, 400.0, 1.0, 0, "px")), ("ω", spec(0.0, 8.0, 0.1, 1, "rad/s")), ("σ", spec(6.0, 160.0, 1.0, 0, "px"))];

#[derive(Clone, Debug, PartialEq)]
pub struct Flow { pub p: Params, pub ptr: Pointer, pub t: f32 }
impl Default for Flow {
    fn default() -> Self { Self { p: Params::new(&DEFS, &[9.0, 150.0, 1.6, 40.0]), ptr: Pointer::new(150.0, MID, 110.0, 14.0), t: 0.0 } }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Num(usize, Edit) }

impl Experiment for Flow {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let (a, l, w, sg) = (self.p.get(0), self.p.get(1), self.p.get(2), self.p.get(3));
        let ((mx, my), near, t) = (self.ptr.at(), self.ptr.near, self.t);
        let k = std::f32::consts::TAU / l;
        // The baseline's height at x: the wave plus the pointer's pull.
        let y = move |x: f32| a * (k * x - w * t).sin() + (my - MID) * near * (-((x - mx) / sg).powi(2)).exp();
        let line = text(LINE, 13.0, Paint::INK).font(Font::Mono).nowrap().key(format!("{key}/line")).pose(move |g: GlyphAt| {
            let slope = (y(g.x + 0.5) - y(g.x - 0.5)) / 1.0;
            let pulled = near > 0.5 && (g.x - mx).abs() < sg;
            Pose { dy: y(g.x), angle: slope.atan(), tone: 0.8, accent: if pulled { 1.0 } else { 0.0 }, ..Pose::default() }
        });
        let piece = col().key(format!("{key}/piece")).fill_width().height(2.0 * MID).clip().justify(taffy::JustifyContent::CENTER).child(line).on_point(|p| Some(Msg::Point(p)));
        let formula = format!("y(x) = {a:.1}·sin(2π·x/{l:.0} − {w:.1}·{t:.1}) + {:.1}·e^(−(x−{mx:.0})²/{sg:.0}²)", (my - MID) * near);
        frame(key, "flow", "move to bend", piece, formula, &self.p.caption(), Msg::Num, "text on a path · glyph turned to the tangent")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m { Msg::Point(p) => self.ptr.hear(p, now), Msg::Num(i, e) => self.p.edit(i, e) }
    }
    fn step(&mut self, now: f32) -> bool { self.t = now; self.ptr.step(now); true }
    fn moving(&self) -> bool { true }
}
