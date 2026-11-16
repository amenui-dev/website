//! Lab · loupe: a block of text wrapped round a sphere. Each glyph's place in the block is a longitude
//! and latitude; the sphere turns (drag to spin, it coasts and slows), the glyph is put where its
//! point projects, squeezed by the slope, dimmed with depth and left out on the far side.
use lab::el::{El, Font, GlyphAt, Paint, Point, Pose, col, text};
use lab::experiment::{Experiment, Params, Pointer, frame, spec};
use lab::widgets::{Edit, Spec};

const WORDS: &str = "AROUND THE GLOBE EVERY LETTER KEEPS ITS OWN OUTLINE, TURNED AND SQUEEZED BY THE CURVE OF THE SURFACE IT LIES ON. DRAG TO SPIN IT. ";
static DEFS: [(&str, Spec); 3] = [("R", spec(30.0, 120.0, 1.0, 0, "px")), ("ω0", spec(0.0, 2.0, 0.05, 2, "rad/s")), ("μ", spec(0.1, 8.0, 0.1, 1, "/s"))];

#[derive(Clone, Debug, PartialEq)]
pub struct Loupe { pub p: Params, pub ptr: Pointer, pub yaw: f32, pub spin: f32, pub tilt: f32, pub t: f32, grab: Option<(f32, f32)> }
impl Default for Loupe {
    fn default() -> Self { Self { p: Params::new(&DEFS, &[66.0, 0.3, 1.6]), ptr: Pointer::new(0.0, 0.0, 200.0, 24.0), yaw: 0.0, spin: 0.3, tilt: 0.35, t: 0.0, grab: None } }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Num(usize, Edit) }

impl Experiment for Loupe {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let (r, yaw, tilt, (mx, my), near) = (self.p.get(0), self.yaw, self.tilt, self.ptr.at(), self.ptr.near);
        let words = WORDS.repeat(3);
        let globe = text(words, 9.0, Paint::INK).font(Font::Mono).key(format!("{key}/text")).pose(move |g: GlyphAt| {
            let s = g.span;
            let lon = ((g.x - s[0]) / (s[2] - s[0]).max(1.0) - 0.5) * std::f32::consts::TAU + yaw;
            let lat = ((g.y - s[1]) / (s[3] - s[1]).max(1.0) - 0.5) * 2.6;
            let (x, y0, z0) = (lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos());
            let (y, z) = (y0 * tilt.cos() - z0 * tilt.sin(), y0 * tilt.sin() + z0 * tilt.cos());
            let (sx, sy) = (g.w * 0.5 + r * x, g.h * 0.5 + r * y);
            let lit = near > 0.5 && (sx - mx).hypot(sy - my) < 16.0;
            Pose { dx: sx - g.x, dy: sy - g.y, sx: lon.cos().abs().max(0.2), sy: lat.cos().max(0.3), alpha: ((z + 0.05) * 5.0).clamp(0.0, 1.0), tone: 0.3 + 0.7 * z.max(0.0), accent: if lit { 1.0 } else { 0.0 }, ..Pose::default() }
        });
        let piece = col().key(format!("{key}/piece")).fill_width().height(2.0 * r + 24.0).clip().child(globe.fill_width().fill_height()).on_point(|p| Some(Msg::Point(p)));
        let formula = format!("θ += ω·dt,  ω = {:.2} → ω0 {:.2} at e^(−{:.1}t)   λ,φ → R·(cosφ sinλ, sinφ), R = {r:.0}", self.spin, self.p.get(1), self.p.get(2));
        frame(key, "loupe", "drag to spin", piece, formula, &self.p.caption(), Msg::Num, "text on a sphere · far side left out")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Point(p) => {
                match (p, self.grab) {
                    (Some(q), Some((x0, yaw0))) if q.down => { let y = yaw0 + (q.x - x0) / self.p.get(0); self.spin = (y - self.yaw) / (now - self.t).max(1.0 / 120.0); self.yaw = y; }
                    (Some(q), None) if q.down => self.grab = Some((q.x, self.yaw)),
                    _ => self.grab = None,
                }
                self.ptr.hear(p, now);
            }
            Msg::Num(i, e) => self.p.edit(i, e),
        }
    }
    fn step(&mut self, now: f32) -> bool {
        let dt = (now - self.t).clamp(0.0, 0.1);
        self.t = now;
        self.ptr.step(now);
        // Let go, it coasts and settles to the idle turn.
        if self.grab.is_none() { let w0 = self.p.get(1); self.spin = w0 + (self.spin - w0) * (-self.p.get(2) * dt).exp(); self.yaw += self.spin * dt; }
        true
    }
    fn moving(&self) -> bool { self.ptr.moving() || self.grab.is_some() || (self.spin - self.p.get(1)).abs() > 0.01 }
}
