//! Body (the landing scene, `#body`): a passage of text gathers itself into a slowly turning luminous
//! body made of its own glyphs, and goes back to being readable text. Written only against the public
//! API: one text element whose glyphs are posed (`El::pose`), drawings for the body's volume, its
//! reflection, the dust and the instrument markings, springs for the motion, the frame budget for
//! every small mark.
//!
//! - Two rest states: the page (every glyph exactly where the layout put it: no pose at all) and the
//!   body. Arriving, the page is shown, readable, and then gathers by itself. A click on the body,
//!   Space or Enter toggles; the spring carries `gather` between 0 and 1.
//! - The passage's glyphs are the body's skin and their count is conserved: glyph i of n travels
//!   between its laid-out place and point i of a spiral on the surface. The body's volume is a second
//!   population of small marks (the far shell and the inside), born while it gathers and gone when it
//!   releases; it is not text and is not conserved.
//! - The form is a signed-distance field sampled to points: `rock` (a sphere with large low-frequency
//!   relief and facets) or `cloud` (smooth lobes), from a seed.
//! - One key light from the upper left: the lit side white-hot, the middle in the form's hue, the far
//!   side its deep shade, a rim on the silhouette; depth fog; the finishing pass's bloom; a pool of
//!   reflected marks on the ground beneath it, a faint spill of light, dust drifting at three depths.
//! - The pointer: parallax over the whole picture; dragging turns the body with inertia; near it, the
//!   glyphs snap back to their places on the page so the words can be read through a clean opening.
//! - The few controls are markings in the picture with leader lines: form, seed, gather / release,
//!   language, and the way to the Lab and the components.
use amenui::draw::Drawing;
use amenui::el::{El, Font, GlyphAt, Key, Paint, Point, Pose, canvas, col, layers, text};
use amenui::experiment::Experiment;
use amenui::field::hash;
use amenui::gesture::{Animated, Spring, Velocity};
use amenui::widgets::Edit;
use std::cell::RefCell;
use std::rc::Rc;

/// The passage's size, and the widest it is set (logical px).
pub const SIZE: f32 = 15.0;
pub const WIDTH: f32 = 760.0;
/// How long the page is shown on arriving before it gathers by itself (s).
pub const DWELL: f32 = 2.4;
/// Marks in the body's volume, asked of the frame budget (the far shell and the inside).
pub const VOLUME: usize = 11_000;
pub const MOTE_LIFE: f32 = 1.6;

/// One point of a form: where (unit body space), its outward normal, the spiral's tangent there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfacePoint { pub p: [f32; 3], pub n: [f32; 3], pub t: [f32; 3] }
/// A mark of the volume: where (unit space), its normal-ish direction, when it is born (gather 0..1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VolumePoint { pub p: [f32; 3], pub n: [f32; 3], pub born: f32, pub size: f32 }

/// What the poses and the drawings share across frames: the skin sampled for (form, seed, n), the
/// volume for (form, seed), and the text's box as last drawn.
#[derive(Default)]
pub struct Cache { pub key: (u32, u32, usize), pub pts: Vec<SurfacePoint>, pub vkey: (u32, u32, usize), pub vol: Vec<VolumePoint>, pub box_wh: (f32, f32) }

/// A mote shed from the rim while the body turns: where (window px), velocity, age (s).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mote { pub x: f32, pub y: f32, pub vx: f32, pub vy: f32, pub age: f32 }

#[derive(Clone)]
pub struct Body {
    /// Form (0 rock, 1 cloud) and seed.
    pub form: u32, pub seed: u32,
    /// The idle turn (rad/s), the opening's radius (px).
    pub omega: f32, pub radius: f32,
    /// The spring between the page (0) and the body (1).
    pub gather: Animated,
    pub yaw: f32, pub pitch: f32, pub spin: f32, pub tip: f32,
    /// The pointer over the picture (window px), the opening (0..1, a short spring), the parallax
    /// (the pointer's offset from the centre, -0.5..0.5, followed softly).
    pub ptr: Option<(f32, f32)>, pub open: Animated, pub par: (Animated, Animated),
    grab: Option<(f32, f32, f32, f32)>, vel: Velocity, dragged: bool,
    pub motes: Vec<Mote>, made: u32,
    /// The window (logical px), when it arrived, whether it still gathers by itself.
    pub size: (f32, f32), pub entered: f32, pub auto: bool,
    pub reduced: bool,
    /// How many times the passage is set (1; more only to measure many glyphs).
    pub copies: usize,
    /// What the host should do: change the language, go to a page (`lab`, `gallery`).
    pub asked: Option<Ask>,
    pub t: f32,
    cache: Rc<RefCell<Cache>>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ask { Language, Go(&'static str) }
impl Default for Body {
    fn default() -> Self {
        let g = Spring { stiffness: 30.0, damping: 11.0, rest_distance: 0.002, rest_speed: 0.01, ..Spring::default() };
        let o = Spring { stiffness: 520.0, damping: 46.0, rest_distance: 0.004, rest_speed: 0.02, ..Spring::default() };
        let p = Spring { stiffness: 14.0, damping: 7.5, rest_distance: 0.0005, rest_speed: 0.002, ..Spring::default() };
        Self {
            form: 0, seed: 7, omega: 0.16, radius: 120.0, gather: Animated::new(0.0, g), yaw: -0.4, pitch: -0.22, spin: 0.0, tip: 0.0,
            ptr: None, open: Animated::new(0.0, o), par: (Animated::new(0.0, p), Animated::new(0.0, p)),
            grab: None, vel: Velocity::default(), dragged: false, motes: vec![], made: 0,
            size: (1600.0, 1000.0), entered: 0.0, auto: true, reduced: false, copies: 1, asked: None, t: 0.0, cache: Rc::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Key(Key), Form, Seed(i32), Gather, Language, Go(&'static str), Num(usize, Edit) }

/// The scene's words: English whatever the interface language (the page and its markings show no Chinese).
fn tr(key: &str) -> String { amenui::i18n::Locale::En.tr(key) }
/// The passage, from the English catalog (an original description of a tidal flat through a day).
pub fn passage() -> String { (1..=5).map(|i| tr(&format!("body.p{i}"))).collect::<Vec<_>>().join("\n\n") }

// ── The forms ────────────────────────────────────────────────────────────────────────────────────

fn noise3(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let (xi, yi, zi) = (x.floor(), y.floor(), z.floor());
    let (fx, fy, fz) = (x - xi, y - yi, z - zi);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let h = |i: f32, j: f32, k: f32| hash(seed ^ (i as i32 as u32).wrapping_mul(73_856_093) ^ (j as i32 as u32).wrapping_mul(19_349_663) ^ (k as i32 as u32).wrapping_mul(83_492_791));
    let l = |a: f32, b: f32, t: f32| a + (b - a) * s(t);
    let c = |dz: f32| l(l(h(xi, yi, zi + dz), h(xi + 1.0, yi, zi + dz), fx), l(h(xi, yi + 1.0, zi + dz), h(xi + 1.0, yi + 1.0, zi + dz), fx), fy);
    l(c(0.0), c(1.0), fz) * 2.0 - 1.0
}
/// The signed distance to the form (unit space, about radius 1): 0 a rock, 1 a cloud.
pub fn sdf(form: u32, seed: u32, p: [f32; 3]) -> f32 {
    let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    if form == 0 {
        // Large low-frequency relief, a middle octave, and a few flat facets cut across it (planes).
        let q = [p[0] * 1.15 + 11.0, p[1] * 1.15 + 3.0, p[2] * 1.15 + 7.0];
        let d = 0.30 * noise3(q[0], q[1], q[2], seed) + 0.10 * noise3(q[0] * 2.4, q[1] * 2.4, q[2] * 2.4, seed ^ 0x55);
        let mut r = len - 0.86 - d;
        for f in 0..5u32 {
            let a = |k: u32| hash(seed.wrapping_mul(977) ^ (f * 5 + k)) * 2.0 - 1.0;
            let nrm = { let v = [a(0), a(1) * 0.7, a(2)]; let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-3); [v[0] / l, v[1] / l, v[2] / l] };
            let plane = p[0] * nrm[0] + p[1] * nrm[1] + p[2] * nrm[2] - (0.62 + 0.12 * (a(3) * 0.5 + 0.5));
            r = r.max(plane);
        }
        r
    } else {
        // Seven balls placed by the seed, joined smoothly: lobes.
        let mut d = f32::MAX;
        for b in 0..7u32 {
            let r = |k: u32| hash(seed.wrapping_mul(131) ^ (b * 7 + k)) * 2.0 - 1.0;
            let c = [r(0) * 0.55, r(1) * 0.35, r(2) * 0.45];
            let rad = 0.3 + 0.22 * (r(3) * 0.5 + 0.5);
            let e = ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt() - rad;
            let k = 0.18;
            let h = (0.5 + 0.5 * (e - d) / k).clamp(0.0, 1.0);
            d = if d == f32::MAX { e } else { e + (d - e) * h - k * h * (1.0 - h) };
        }
        d
    }
}
fn norm(v: [f32; 3]) -> [f32; 3] { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6); [v[0] / l, v[1] / l, v[2] / l] }
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }

/// Where the form's surface is along `dir`, marched in from radius 2, with its normal.
fn on_surface(form: u32, seed: u32, dir: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let mut r = 2.0f32;
    for _ in 0..56 {
        let d = sdf(form, seed, [dir[0] * r, dir[1] * r, dir[2] * r]);
        if d.abs() < 1e-3 { break; }
        r = (r - d * 0.85).max(0.05);
    }
    let p = [dir[0] * r, dir[1] * r, dir[2] * r];
    let e = 1e-3;
    let g = |a: usize| { let mut q = p; q[a] += e; let mut w = p; w[a] -= e; sdf(form, seed, q) - sdf(form, seed, w) };
    (p, norm([g(0), g(1), g(2)]))
}

/// `n` points on the surface along one spiral from the top to the bottom, consecutive points
/// neighbours, so the passage winds round the body in reading order.
pub fn surface(form: u32, seed: u32, n: usize) -> Vec<SurfacePoint> {
    let n = n.max(2);
    let turns = (n as f32 * std::f32::consts::PI).sqrt();
    (0..n).map(|i| {
        let h = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
        let th = h.clamp(-1.0, 1.0).acos();
        let ph = -turns * th;
        let (p, nrm) = on_surface(form, seed, [th.sin() * ph.cos(), h, th.sin() * ph.sin()]);
        // The spiral runs along increasing longitude; facing the viewer at rest, left to right.
        let t = norm(cross([0.0, 1.0, 0.0], nrm));
        let t = if t[0].is_finite() && (t[0].abs() + t[2].abs()) > 1e-3 { t } else { [1.0, 0.0, 0.0] };
        SurfacePoint { p, n: nrm, t }
    }).collect()
}

/// The volume: `n` marks, two thirds on the shell (seen through the skin on the far side), a third
/// inside it, each born at its own moment of the gathering.
pub fn volume(form: u32, seed: u32, n: usize) -> Vec<VolumePoint> {
    (0..n as u32).map(|i| {
        let u = hash(i.wrapping_mul(2_654_435_761) ^ seed);
        let v = hash(i.wrapping_mul(40_503) ^ seed ^ 0x51);
        let z = 1.0 - 2.0 * u;
        let a = v * std::f32::consts::TAU;
        let s = (1.0 - z * z).max(0.0).sqrt();
        let (p, nrm) = on_surface(form, seed, [s * a.cos(), z, s * a.sin()]);
        let inside = hash(i ^ 0x7777) < 0.34;
        let depth = if inside { 0.3 + 0.62 * hash(i ^ 0x1234).powf(0.6) } else { 0.99 };
        VolumePoint { p: [p[0] * depth, p[1] * depth, p[2] * depth], n: nrm, born: 0.25 + 0.7 * hash(i ^ 0xabc), size: 0.55 + 0.8 * hash(i ^ 0x99) }
    }).collect()
}

/// The view of the body: its turn and the camera's breathing and parallax.
#[derive(Clone, Copy)]
struct View { yaw: f32, pitch: f32, c: (f32, f32), r: f32 }
impl View {
    fn rot(&self, v: [f32; 3]) -> [f32; 3] {
        let (s, k) = self.yaw.sin_cos();
        let (x, z) = (v[0] * k + v[2] * s, -v[0] * s + v[2] * k);
        let (s2, k2) = self.pitch.sin_cos();
        [x, v[1] * k2 - z * s2, v[1] * s2 + z * k2]
    }
    /// Screen place (window px), perspective scale; a camera 4 radii away.
    fn project(&self, p: [f32; 3]) -> ([f32; 2], f32) { let k = 4.0 / (4.0 - p[2]); ([self.c.0 + self.r * p[0] * k, self.c.1 - self.r * p[1] * k], k) }
}
/// The key light, from the upper left and in front.
const LIGHT: [f32; 3] = [-0.55, 0.62, 0.56];

/// How lit a mark is (0 shade … 1 core) from its normal and depth, and its rim.
fn light(n: [f32; 3]) -> (f32, f32) {
    let lam = dot3(n, norm(LIGHT)).max(0.0);
    let rim = (1.0 - n[2].abs()).powf(2.5) * (n[2] > -0.3) as i32 as f32;
    (lam, rim)
}
/// The paint a lit mark takes: core white with a little hue, then the form's hue, then its shade.
fn lit_paint(lam: f32, rim: f32, fog: f32) -> Paint {
    // On the shaded side the rim is the deep hue made bright: the silhouette always reads.
    if lam < 0.2 && rim > 0.45 { return Paint { tone: 1.0, accent: 0.75, alpha: (0.45 + 0.5 * rim) * fog, hue: 1 }; }
    let l = (lam + rim * 0.6).min(1.0);
    if l > 0.62 { Paint { tone: 1.0, accent: 0.15 + 0.6 * (1.0 - l), alpha: (0.55 + 0.45 * l) * fog, hue: 0 } }
    else if l > 0.2 { Paint { tone: 1.0, accent: 1.0, alpha: (0.35 + 0.6 * l) * fog, hue: 0 } }
    else { Paint { tone: 1.0, accent: 1.0, alpha: (0.22 + 0.5 * l) * fog, hue: 1 } }
}

/// How far glyph `i` of `n` has gone at gather `g`: reading order leads; exactly 0 at 0 and 1 at 1.
pub fn progress(g: f32, i: usize, n: usize) -> f32 {
    let st = 0.55;
    let u = (g * (1.0 + st) - st * i as f32 / n.max(1) as f32).clamp(0.0, 1.0);
    u * u * u * (u * (u * 6.0 - 15.0) + 10.0)
}

impl Body {
    /// At rest where the layout put every glyph: no pose at all.
    pub fn flat(&self) -> bool { self.gather.value == 0.0 && !self.gather.moving() }
    pub fn target(&self) -> f32 { self.gather.target }
    /// The host's window (logical px).
    pub fn resize(&mut self, w: f32, h: f32) { self.size = (w.max(1.0), h.max(1.0)); }
    /// The pointer's place for the parallax (-0.5..0.5 from the window's centre).
    pub fn look_at(&mut self, x: f32, y: f32) {
        let now = self.t;
        let (vx, vy) = (self.par.0.velocity, self.par.1.velocity);
        self.par.0.to(x.clamp(-0.5, 0.5), vx, now);
        self.par.1.to(y.clamp(-0.5, 0.5), vy, now);
    }
    fn toggle(&mut self, now: f32) { self.auto = false; let to = if self.gather.target >= 0.5 { 0.0 } else { 1.0 }; let v = self.gather.velocity; self.gather.to(to, v, now); }
    /// The camera now: the turn, a slow breathing (a little zoom and tilt), the parallax.
    fn view(&self) -> View {
        let (w, h) = self.size;
        let breathe = if self.reduced || self.omega <= 0.0 { 0.0 } else { (self.t * 0.33).sin() };
        let (px, py) = (self.par.0.value, self.par.1.value);
        let r = (h * 0.33).min(w * 0.3) * (1.0 + 0.018 * breathe);
        View { yaw: self.yaw + px * 0.22, pitch: self.pitch + py * 0.14 + 0.03 * breathe, c: (w * 0.5 - px * 26.0, h * 0.47 - py * 16.0), r }
    }
    /// Where the text's box starts in the window (it is centred).
    fn text_origin(&self) -> (f32, f32) { let (bw, bh) = self.cache.borrow().box_wh; ((self.size.0 - bw) * 0.5, (self.size.1 - bh) * 0.5) }
    fn hole(&self) -> Option<(f32, f32, f32)> { let o = self.open.value.clamp(0.0, 1.0); self.ptr.filter(|_| o > 0.01).map(|(x, y)| (x, y, self.radius * o)) }

    /// The skin's pose for every glyph.
    fn poser(&self) -> impl Fn(GlyphAt) -> Pose + 'static {
        let (g, v, (form, seed)) = (self.gather.value.clamp(0.0, 1.0), self.view(), (self.form, self.seed));
        let (size, hole, cache) = (self.size, self.hole(), self.cache.clone());
        move |a: GlyphAt| {
            let mut c = cache.borrow_mut();
            c.box_wh = (a.w, a.h);
            if c.key != (form, seed, a.n) { c.pts = surface(form, seed, a.n); c.key = (form, seed, a.n); }
            let Some(sp) = c.pts.get(a.i) else { return Pose::default() };
            let o = ((size.0 - a.w) * 0.5, (size.1 - a.h) * 0.5);
            let (fx, fy) = (a.x + o.0, a.y + o.1);
            // The opening: a glyph whose place on the page is inside it is on the page, exactly.
            if let Some((hx, hy, hr)) = hole && (fx - hx).hypot(fy - hy) < hr { return Pose::default(); }
            let e = progress(g, a.i, a.n);
            if e <= 0.0 { return Pose::default(); }
            let (p, n, t) = (v.rot(sp.p), v.rot(sp.n), v.rot(sp.t));
            let ([sx, sy], k) = v.project(p);
            // On the body, but where the opening is: there, the page shows, not the body.
            let hidden = hole.is_some_and(|(hx, hy, hr)| (sx - hx).hypot(sy - hy) < hr + 6.0);
            let (lam, rim) = light(n);
            let fog = 0.4 + 0.6 * ((p[2] + 1.0) * 0.5).clamp(0.0, 1.0);
            let paint = lit_paint(lam, rim, fog);
            // Turned to the spiral's tangent, squeezed by its slope, larger in front (up to twice).
            let ang = (-t[1]).atan2(t[0]);
            let squeeze = (t[0] * t[0] + t[1] * t[1]).sqrt().max(0.3);
            let big = k * (0.7 + 1.25 * n[2].max(0.0));
            let lift = 1.0 + 0.6 * e * (1.0 - e);
            Pose {
                dx: (sx - fx) * e, dy: (sy - fy) * e, angle: ang * e,
                sx: 1.0 + (big * squeeze * lift - 1.0) * e, sy: 1.0 + (big * lift - 1.0) * e,
                tone: 1.0, alpha: if hidden { 0.01 } else { 1.0 + (paint.alpha - 1.0) * e },
                accent: paint.accent * e, hue: if e > 0.5 { paint.hue as f32 } else { -1.0 },
            }
        }
    }

    /// Behind the text: the ground's light spill, the reflection pool, the volume, the far dust.
    fn back(&self) -> Drawing {
        let (w, h) = self.size;
        let (g, v) = (self.gather.value.clamp(0.0, 1.0), self.view());
        let mut d = Drawing::new(w, h);
        let (px, py) = (self.par.0.value, self.par.1.value);
        dust(&mut d, w, h, self.t, (px, py), 0, 140);
        if g <= 0.0 { return d; }
        let mut c = self.cache.borrow_mut();
        let want = amenui::budget::grant(VOLUME);
        if c.vkey != (self.form, self.seed, want) { c.vol = volume(self.form, self.seed, want); c.vkey = (self.form, self.seed, want); }
        let ground = v.c.1 + v.r * 1.0;
        // A faint spill of light on the haze beneath it.
        for i in 0..220u32 {
            let (a, b) = (hash(i * 3 + 1) * 2.0 - 1.0, hash(i * 3 + 2) * 2.0 - 1.0);
            let r = 1.0 - (a * a + b * b).min(1.0);
            d.dot(v.c.0 + a * v.r * 0.95, ground - v.r * 0.02 + b * v.r * 0.09, 22.0 + 30.0 * hash(i * 3 + 3), Paint { tone: 1.0, accent: 1.0, alpha: 0.012 * g * r, hue: 0 });
        }
        let hole = self.hole();
        for (i, q) in c.vol.iter().enumerate() {
            let born = ((g - q.born) / 0.12).clamp(0.0, 1.0);
            if born <= 0.0 { continue; }
            let (p, n) = (v.rot(q.p), v.rot(q.n));
            let ([x, y], k) = v.project(p);
            if hole.is_some_and(|(hx, hy, hr)| (x - hx).hypot(y - hy) < hr + 6.0) { continue; }
            let (lam, rim) = light(n);
            let inside = (q.p[0] * q.p[0] + q.p[1] * q.p[1] + q.p[2] * q.p[2]) < 0.7;
            let fog = 0.3 + 0.7 * ((p[2] + 1.0) * 0.5).clamp(0.0, 1.0);
            let mut paint = if inside { Paint { tone: 1.0, accent: 1.0, alpha: 0.32 * fog, hue: if p[2] > 0.0 { 0 } else { 1 } } } else { lit_paint(lam * 0.85, rim, fog) };
            paint.alpha *= born * if p[2] > 0.2 && !inside { 0.55 } else { 1.0 };
            d.dot(x, y, q.size * k * (0.8 + 0.6 * lam), paint);
            // Its reflection in the pool on the ground: squashed, dim, every third.
            if i % 2 == 0 && y < ground {
                let ry = ground + (ground - y) * 0.22;
                let fade = (1.0 - (ry - ground) / (v.r * 0.42)).clamp(0.0, 1.0);
                if fade > 0.0 { d.dot(x + (hash(i as u32) - 0.5) * 4.0, ry, q.size * 0.75, Paint { alpha: (paint.alpha * 0.55 * fade).min(0.3), ..paint }); }
            }
        }
        d
    }
    /// Over the text: the near dust, the motes, the markings' leader lines.
    fn front(&self, labels: &[Marking]) -> Drawing {
        let (w, h) = self.size;
        let mut d = Drawing::new(w, h);
        dust(&mut d, w, h, self.t, (self.par.0.value, self.par.1.value), 1, 70);
        for m in &self.motes { let f = 1.0 - m.age / MOTE_LIFE; d.dot(m.x, m.y, 0.6 + 0.9 * f, Paint { tone: 1.0, accent: 0.6, alpha: 0.7 * f, hue: 0 }); }
        for l in labels {
            if l.alpha <= 0.0 { continue; }
            let p = Paint { tone: 0.45, accent: 0.0, alpha: l.alpha * 0.8, hue: 0 };
            d.line(l.from, l.elbow, 0.5, p);
            d.line(l.elbow, l.to, 0.5, p);
            d.dot(l.from.0, l.from.1, 1.4, Paint { tone: 1.0, ..p });
        }
        d
    }
    /// The instrument markings round the body: where each points on the body, where its text goes.
    fn markings(&self) -> Vec<Marking> {
        let (v, g) = (self.view(), self.gather.value.clamp(0.0, 1.0));
        // (On a narrow window there is no room beside the body: the corners' markings only.)
        let a = if self.size.0 < 700.0 { 0.0 } else { ((g - 0.85) / 0.15).clamp(0.0, 1.0) };
        let (cx, cy, r) = (v.c.0, v.c.1, v.r);
        // Each leader starts on the body's real silhouette: the projected skin point farthest along its
        // direction (left; up and right; down and right).
        let edge = |dx: f32, dy: f32, fallback: (f32, f32)| -> (f32, f32) {
            let c = self.cache.borrow();
            c.pts.iter().map(|sp| v.project(v.rot(sp.p)).0).max_by(|a, b| (a[0] * dx + a[1] * dy).total_cmp(&(b[0] * dx + b[1] * dy))).map_or(fallback, |p| (p[0], p[1]))
        };
        let (fl, fs, fg) = (edge(-1.0, 0.25, (cx - r * 0.93, cy - r * 0.28)), edge(0.75, -0.66, (cx + r * 0.72, cy - r * 0.62)), edge(0.8, 0.6, (cx + r * 0.8, cy + r * 0.5)));
        let form = if self.form == 0 { tr("body.rock") } else { tr("body.cloud") };
        vec![
            Marking { key: "form", text: format!("{}  {}", tr("body.form"), form), from: fl, elbow: (fl.0 - r * 0.22, fl.1 - r * 0.18), to: (fl.0 - r * 0.55, fl.1 - r * 0.18), left: true, alpha: a, msg: Msg::Form },
            Marking { key: "seed", text: format!("{}  {:02}", tr("body.seed"), self.seed), from: fs, elbow: (fs.0 + r * 0.22, fs.1 - r * 0.2), to: (fs.0 + r * 0.6, fs.1 - r * 0.2), left: false, alpha: a, msg: Msg::Seed(1) },
            Marking { key: "gather", text: tr(if self.gather.target >= 0.5 { "body.release" } else { "body.gather" }), from: fg, elbow: (fg.0 + r * 0.22, fg.1 + r * 0.16), to: (fg.0 + r * 0.6, fg.1 + r * 0.16), left: false, alpha: a, msg: Msg::Gather },
        ]
    }
}
/// A marking: its text, the point on the body it names, the bend and the end of its leader.
pub struct Marking { pub key: &'static str, pub text: String, pub from: (f32, f32), pub elbow: (f32, f32), pub to: (f32, f32), pub left: bool, pub alpha: f32, pub msg: Msg }

/// Dust at depth `layer` (0 far, 1 near): slow drift, parallax by depth, a few marks from the budget.
fn dust(d: &mut Drawing, w: f32, h: f32, t: f32, par: (f32, f32), layer: u32, n: usize) {
    let n = amenui::budget::grant(n);
    let depth = if layer == 0 { 0.35 } else { 1.0 };
    for i in 0..n as u32 {
        let s = i * 7 + layer * 10_007;
        let z = depth * (0.5 + 0.5 * hash(s + 3));
        let x = (hash(s) * w + t * (3.0 + 9.0 * z) - par.0 * 70.0 * z).rem_euclid(w);
        let y = (hash(s + 1) * h - t * (1.0 + 2.0 * z) * hash(s + 2) - par.1 * 44.0 * z).rem_euclid(h);
        let tw = 0.6 + 0.4 * (t * (0.4 + hash(s + 5)) + hash(s + 6) * 6.3).sin();
        d.dot(x, y, 0.5 + 1.1 * z, Paint { tone: 0.8, accent: 0.4, alpha: (0.06 + 0.22 * z) * tw, hue: 0 });
    }
}

impl Body {
    /// The scene filling a `w × h` window.
    pub fn view_in(&self, key: &str) -> El<Msg> {
        let (w, h) = self.size;
        let passage = text(vec![passage(); self.copies.max(1)].join("\n\n"), SIZE, Paint::INK).max_width(WIDTH).fill_width().key(format!("{key}/text"));
        // At rest on the page no pose at all: every glyph is exactly the plain layout's.
        let passage = if self.flat() { passage } else { passage.pose(self.poser()) };
        let marks = self.markings();
        let mut parts = vec![
            canvas(self.back()).key(format!("{key}/back")).width(w).absolute(Some(0.0), Some(0.0), None, None).respond(amenui::el::Response::Ignore),
            col().key(format!("{key}/page")).width(w).height(h).center().child(col().key(format!("{key}/column")).width(WIDTH.min(w - 48.0)).child(passage)),
            canvas(self.front(&marks)).key(format!("{key}/front")).width(w).absolute(Some(0.0), Some(0.0), None, None).respond(amenui::el::Response::Ignore),
        ];
        let mark = |k: String, s: String, x: f32, y: f32, alpha: f32, m: Msg| text(s, 10.0, Paint { tone: 0.6, accent: 0.0, alpha, hue: 0 }).font(Font::Mono).nowrap().key(k)
            .absolute(Some(x), Some(y), None, None).focusable(alpha > 0.5).role(amenui::el::Role::Button).cursor(amenui::el::Cursor::Pointer).on_press(m);
        for m in &marks {
            if m.alpha <= 0.0 { continue; }
            let x = if m.left { m.to.0 - 8.0 * m.text.chars().count() as f32 } else { m.to.0 + 4.0 };
            let mut e = mark(format!("{key}/mark/{}", m.key), m.text.clone(), x, m.to.1 - 14.0, m.alpha, m.msg.clone());
            if m.key == "seed" { e = e.on_key(|k, _| match k { Key::Up | Key::Right => Some(Msg::Seed(1)), Key::Down | Key::Left => Some(Msg::Seed(-1)), _ => None }); }
            parts.push(e);
        }
        // The corners: the language, and the way out to the Lab and the components.
        let lang = if amenui::i18n::current() == amenui::i18n::Locale::En { "EN" } else { "ZH" };
        parts.push(mark(format!("{key}/mark/lang"), format!("{}  {lang}", tr("body.language")), w - 112.0, 22.0, 0.7, Msg::Language));
        parts.push(mark(format!("{key}/mark/lab"), tr("body.to-lab"), 26.0, h - 34.0, 0.55, Msg::Go("lab")));
        parts.push(mark(format!("{key}/mark/gallery"), tr("body.to-gallery"), 26.0 + 90.0, h - 34.0, 0.55, Msg::Go("gallery")));
        layers(parts).key(key.to_owned()).width(w).height(h).clip()
            .on_point(|p| Some(Msg::Point(p)))
            .on_key(|k, _| matches!(k, Key::Char(' ') | Key::Enter | Key::Left | Key::Right).then_some(Msg::Key(k)))
            .role(amenui::el::Role::Img).label(tr("body.label"))
    }
}

impl Experiment for Body {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> { self.view_in(key) }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Key(Key::Left) => { self.spin -= 0.6; }
            Msg::Key(Key::Right) => { self.spin += 0.6; }
            Msg::Key(_) | Msg::Gather => self.toggle(now),
            Msg::Form => { self.form = 1 - self.form; }
            Msg::Seed(d) => { self.seed = (self.seed as i32 + d).rem_euclid(100) as u32; }
            Msg::Language => self.asked = Some(Ask::Language),
            Msg::Go(p) => self.asked = Some(Ask::Go(p)),
            Msg::Num(i, Edit::Set(v)) => match i {
                0 => { let v0 = self.gather.velocity; self.auto = false; self.gather.to(if v >= 0.5 { 1.0 } else { 0.0 }, v0, now); }
                1 => self.form = (v >= 0.5) as u32,
                2 => self.seed = v.clamp(0.0, 99.0) as u32,
                3 => self.omega = v.clamp(0.0, 1.5),
                _ => self.radius = v.clamp(20.0, 300.0),
            },
            Msg::Num(..) => {}
            Msg::Point(p) => {
                self.ptr = p.map(|q| (q.x, q.y));
                if let Some(q) = p { self.look_at(q.x / self.size.0 - 0.5, q.y / self.size.1 - 0.5); }
                let over = p.is_some() && self.gather.target >= 0.5 && self.gather.value > 0.9;
                let v = self.open.velocity;
                self.open.to(if over && !self.dragged { 1.0 } else { 0.0 }, v, now);
                match (p, self.grab) {
                    (Some(q), Some((x0, y0, yaw0, pitch0))) if q.down => {
                        if (q.x - x0).hypot(q.y - y0) > 5.0 { self.dragged = true; }
                        if self.dragged && self.gather.target >= 0.5 {
                            let r = 300.0;
                            self.yaw = yaw0 + (q.x - x0) / r;
                            self.pitch = (pitch0 + (q.y - y0) / r).clamp(-1.0, 1.0);
                            self.vel.push(now, q.x, q.y);
                            self.open.to(0.0, self.open.velocity, now);
                        }
                    }
                    (Some(q), None) if q.down => { self.grab = Some((q.x, q.y, self.yaw, self.pitch)); self.dragged = false; self.vel.clear(); self.vel.push(now, q.x, q.y); }
                    (_, Some(_)) => {
                        // Let go: a press that did not move toggles; a drag keeps turning at its speed.
                        if self.dragged { let (vx, vy) = self.vel.get(); self.spin = (vx / 300.0).clamp(-8.0, 8.0); self.tip = (vy / 300.0).clamp(-4.0, 4.0); } else { self.toggle(now); }
                        self.grab = None; self.dragged = false; self.vel.clear();
                    }
                    _ => {}
                }
            }
        }
    }
    fn step(&mut self, now: f32) -> bool {
        let dt = (now - self.t).clamp(0.0, 0.05);
        self.t = now;
        // Arriving: the page is read first, then it gathers by itself.
        // (With reduced motion it stays the page until asked.)
        if self.auto && !self.reduced && now - self.entered >= DWELL && self.gather.target < 0.5 { self.auto = false; let v = self.gather.velocity; self.gather.to(1.0, v, now); }
        let mut changed = self.gather.step(now) | self.open.step(now) | self.par.0.step(now) | self.par.1.step(now);
        let body = self.gather.target >= 0.5;
        if self.grab.is_none() {
            let idle = if body && !self.reduced { self.omega } else { 0.0 };
            let k = (-1.3 * dt).exp();
            self.spin = idle + (self.spin - idle) * k;
            self.tip *= k;
            if (self.spin - idle).abs() < 1e-3 { self.spin = idle; }
            if self.tip.abs() < 1e-3 { self.tip = 0.0; }
            if self.spin != 0.0 || self.tip != 0.0 { self.yaw += self.spin * dt; self.pitch = (self.pitch + self.tip * dt).clamp(-1.0, 1.0); changed = true; }
        }
        for m in &mut self.motes { m.age += dt; m.x += m.vx * dt; m.y += m.vy * dt; m.vy -= 3.0 * dt; }
        let before = self.motes.len();
        self.motes.retain(|m| m.age < MOTE_LIFE);
        changed |= before != self.motes.len() || !self.motes.is_empty();
        // Rim glyphs shed a few motes while it turns (from the budget; never on the page).
        if body && self.gather.value > 0.97 && !self.reduced && dt > 0.0 && self.spin.abs() > 0.01 {
            let v = self.view();
            let c = self.cache.borrow();
            let want = amenui::budget::grant(2).min(70usize.saturating_sub(self.motes.len()));
            let mut born = 0;
            for tries in 0..30u32 {
                if born >= want || c.pts.is_empty() { break; }
                self.made = self.made.wrapping_add(1);
                let i = (hash(self.made.wrapping_mul(2_654_435_761) ^ tries) * c.pts.len() as f32) as usize % c.pts.len();
                let (p, n) = (v.rot(c.pts[i].p), v.rot(c.pts[i].n));
                if n[2].abs() > 0.16 { continue; }
                let ([x, y], _) = v.project(p);
                let out = norm([n[0], -n[1], 0.0]);
                let sp = 10.0 + 18.0 * hash(self.made ^ 0x9e37);
                self.motes.push(Mote { x, y, vx: out[0] * sp, vy: out[1] * sp, age: 0.0 });
                born += 1;
            }
        }
        // While it is the body with its idle motion (the turn, the camera's breathing), each frame changes.
        changed || (body && !self.reduced && self.omega > 0.0) || (self.auto && !self.reduced)
    }
    fn moving(&self) -> bool {
        self.gather.moving() || self.open.moving() || self.par.0.moving() || self.par.1.moving() || !self.motes.is_empty() || self.grab.is_some()
            || self.spin != 0.0 || self.tip != 0.0 || (self.gather.target >= 0.5 && !self.reduced && self.omega > 0.0) || (self.auto && !self.reduced)
    }
}
impl Body {
    /// The text box's origin in the window, for hosts and tests.
    pub fn text_at(&self) -> (f32, f32) { self.text_origin() }
}

/// A runtime of its own (the `render` tool, tests): the scene alone in its look.
#[cfg(not(target_arch = "wasm32"))]
pub fn ui(faces: Vec<amenui::text::Face>, w: u32, h: u32, scale: f32) -> amenui::ui::Ui<Body, Msg> {
    fn view(s: &Body) -> El<Msg> { col().fill_width().fill_height().fonts(amenui::style::LAB.roles()).child(s.view_in("body")) }
    fn update(s: &mut Body, m: Msg) {
        let now = s.t;
        s.update(m, now);
        if s.asked.take() == Some(Ask::Language) {
            let l = if amenui::i18n::current() == amenui::i18n::Locale::En { amenui::i18n::Locale::ZhHans } else { amenui::i18n::Locale::En };
            amenui::i18n::set_current(l);
        }
    }
    let mut b = Body::default();
    b.resize(w as f32 / scale, h as f32 / scale);
    amenui::ui::Ui::new(b, view, update, faces, w, h, scale)
        .on_resize(|s, w, h| s.resize(w, h))
        .on_look(|s| look(s.form))
        .on_tick(|s, t| s.step(t))
        .on_wake(|s, t| s.moving().then_some(t))
}
/// The scene's look for a form: a dark ground with a horizon, plain material, a bloom; the palette
/// is the form's: rock bone-white, ember, deep violet; cloud pale cyan, cyan, indigo; and one other
/// colour (hue 2) only where the pointer touches.
pub fn look(form: u32) -> amenui::style::Look {
    let lab_t = amenui::style::LAB;
    let theme = if form == 0 {
        amenui::style::Theme { ground: [0.018, 0.016, 0.026], ground2: [0.060, 0.040, 0.046], ink: [0.97, 0.93, 0.86], accent: [1.0, 0.47, 0.16], second: [0.42, 0.22, 0.70], third: [0.55, 0.88, 1.0], ..lab_t }
    } else {
        amenui::style::Theme { ground: [0.012, 0.018, 0.030], ground2: [0.030, 0.046, 0.075], ink: [0.90, 0.97, 1.0], accent: [0.36, 0.78, 1.0], second: [0.26, 0.24, 0.66], third: [1.0, 0.74, 0.42], ..lab_t }
    };
    let mut l = amenui::style::Look { theme, material: amenui::style::Material::Plain, ..amenui::style::Look::default() };
    l.post.bloom = amenui::post::Bloom { strength: 1.4, threshold: 0.35, radius: 10.0 };
    l.post.vignette = amenui::post::Vignette { strength: 0.45, ..amenui::post::Vignette::default() };
    l
}
