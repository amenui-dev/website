//! Lab · lumen: a light over text. Every glyph keeps its whole outline; only its tone rises with the
//! light's falloff, so the dim text at rest is still whole letters. The light follows the pointer on a
//! spring and wanders slowly when nobody is there.
use lab::el::{El, Font, GlyphAt, Paint, Point, Pose, col, text};
use lab::experiment::{Experiment, Params, Pointer, frame, spec};
use lab::widgets::{Edit, Spec};

const WORDS: &str = "Light falls off with distance. Each glyph is drawn whole and only its tone changes: what the beam leaves is dim, never dissolved. Move across the page and read what it touches. The rest stays a quiet texture of letters at rest.";
static DEFS: [(&str, Spec); 3] = [("L0", spec(0.05, 1.0, 0.01, 2, "")), ("r", spec(8.0, 160.0, 1.0, 0, "px")), ("ω", spec(0.0, 2.0, 0.05, 2, "rad/s"))];

#[derive(Clone, Debug, PartialEq)]
pub struct Lumen { pub p: Params, pub ptr: Pointer, pub t: f32 }
impl Default for Lumen {
    fn default() -> Self { Self { p: Params::new(&DEFS, &[0.22, 46.0, 0.35]), ptr: Pointer::new(120.0, 60.0, 140.0, 20.0), t: 0.0 } }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Num(usize, Edit) }

impl Lumen {
    /// Where the light is: the pointer, or a slow figure of eight over the text.
    fn light(&self, w: f32, h: f32) -> (f32, f32) {
        let (a, n) = (self.t * self.p.get(2), self.ptr.near);
        let idle = (w * (0.5 + 0.38 * a.sin()), h * (0.5 + 0.35 * (2.0 * a).sin()));
        let (px, py) = self.ptr.at();
        (idle.0 + (px - idle.0) * n, idle.1 + (py - idle.1) * n)
    }
}

impl Experiment for Lumen {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let (l0, r, near) = (self.p.get(0), self.p.get(1), self.ptr.near);
        let me = self.clone();
        let body = text(WORDS, 12.0, Paint::INK).font(Font::Mono).key(format!("{key}/text")).pose(move |g: GlyphAt| {
            let (lx, ly) = me.light(g.w, g.h);
            let k = (-((g.x - lx).powi(2) + (g.y - ly).powi(2)) / (r * r)).exp();
            Pose { tone: l0 + (1.0 - l0) * k, accent: if near > 0.5 && k > 0.8 { 1.0 } else { 0.0 }, ..Pose::default() }
        });
        let piece = col().key(format!("{key}/piece")).fill_width().height(150.0).child(body).on_point(|p| Some(Msg::Point(p)));
        let formula = format!("L(d) = {l0:.2} + {:.2}·e^(−d²/{r:.0}²)", 1.0 - l0);
        frame(key, "lumen", "move to light", piece, formula, &self.p.caption(), Msg::Num, "tone by distance · outlines untouched")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m { Msg::Point(p) => self.ptr.hear(p, now), Msg::Num(i, e) => self.p.edit(i, e) }
    }
    fn step(&mut self, now: f32) -> bool { self.t = now; self.ptr.step(now); true }
    fn moving(&self) -> bool { true }
}
