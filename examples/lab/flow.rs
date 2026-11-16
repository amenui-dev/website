//! Lab · flow: a line of text on a path. The baseline is a travelling wave; the pointer pulls a bump
//! into it. Each glyph sits on the curve, turned to its tangent, with its own true outline.
use lab::el::{El, Font, GlyphAt, Paint, Point, Pose, col, text};
use lab::experiment::{Experiment, Params, Pointer, frame, spec};
use lab::widgets::{Edit, Spec};

const LINES: [&str; 4] = ["text that rides a wave, letter by letter,", "along its own baseline: every glyph is", "turned to the tangent of the curve at", "its centre, and keeps its whole outline."];
const MID: f32 = 120.0;
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
                // Four lines spread down the stage, each riding the wave a little behind the one above; near the
        // pointer the baseline is pulled toward it.
        let n = LINES.len() as f32;
        let block = text(LINES.join("\n"), 15.0, Paint::INK).font(Font::Mono).nowrap().fill_width().fill_height().key(format!("{key}/line")).pose(move |g: GlyphAt| {
            let gap = ((g.span[3] - g.span[1]) / (n - 1.0)).max(1.0);
            let j = ((g.y - g.span[1]) / gap).round();
            let base = g.h * (j + 0.5) / n - g.y;
            let y = |x: f32| a * (k * x - w * t + j * 1.1).sin();
            let pull = |x: f32| (my - (g.y + base)) * near * (-((x - mx) / sg).powi(2)).exp() * 0.8;
            let slope = (y(g.x + 0.5) + pull(g.x + 0.5)) - (y(g.x - 0.5) + pull(g.x - 0.5));
            let pulled = near > 0.5 && (g.x - mx).abs() < sg * 0.8 && (my - (g.y + base)).abs() < g.h / n;
            Pose { dy: base + y(g.x) + pull(g.x), angle: slope.atan(), tone: 0.8, accent: if pulled { 1.0 } else { 0.0 }, ..Pose::default() }
        });
        let piece = col().key(format!("{key}/piece")).fill_width().fill_height().clip().child(block).on_point(|p| Some(Msg::Point(p)));
        let formula = format!("y(x) = {a:.1}·sin(2π·x/{l:.0} − {w:.1}·{t:.1} + 1.1j) + 0.8·(m_y − y)·{near:.2}·e^(−(x−{mx:.0})²/{sg:.0}²)");
        frame(key, "flow", "move to bend", piece, formula, &self.p, Msg::Num, "text on a path · turned to the tangent")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m { Msg::Point(p) => self.ptr.hear(p, now), Msg::Num(i, e) => self.p.edit(i, e) }
    }
    fn step(&mut self, now: f32) -> bool { self.t = now; self.ptr.step(now); true }
    fn moving(&self) -> bool { self.ptr.moving() }
}
