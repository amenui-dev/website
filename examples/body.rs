//! Body (the first showcase scene, `#body`): a passage of text gathers itself into a slowly turning
//! luminous body made of its own glyphs, and goes back to being readable text. Written only against the
//! public API, as any application would: one text element whose glyphs are posed (`El::pose`), a
//! drawing for the motes, the gesture spring, and the frame budget.
//!
//! - Two rest states: the page (every glyph exactly where the layout put it: no pose at all) and the
//!   body. `gather` (0…1) is driven by a spring between them; a click, Space or Enter toggles.
//! - Glyph count is conserved: glyph i of n travels between its laid-out place and point i of a spiral
//!   on the body's surface (consecutive glyphs neighbours on the spiral, so the passage winds round
//!   the body in reading order), turned to the surface's tangent and scaled by depth.
//! - The body is a signed-distance form sampled to surface points: `rock` (a sphere displaced by
//!   noise) or `cloud` (a smooth union of balls), from a numeric seed.
//! - Light: one hue on a dark ground, bright where the surface faces the viewer and near, dim behind;
//!   rim glyphs shed a few motes from the frame budget (only in the body, never over readable text).
//! - Drag turns it with inertia; near the pointer the glyphs part and settle back to their flat places,
//!   so the original words can be read through the opening; leaving closes it. Idle: a slow turn.
use lab::draw::Drawing;
use lab::el::{El, GlyphAt, Key, Paint, Point, Pose, canvas, col, layers, row, space, text};
use lab::experiment::{Experiment, Params, TOGGLE, spec};
use lab::field::hash;
use lab::gesture::{Animated, Spring, Velocity};
use lab::widgets::{Edit, Spec};
use std::cell::RefCell;
use std::rc::Rc;

/// The numbers: gather (0…1; under ½ sends it to the page, else to the body), form (0 rock, 1 cloud),
/// seed, idle turn (rad/s), the opening's radius (px), the body's size (share of the text's height).
pub static DEFS: [(&str, Spec); 6] = [("gather", spec(0.0, 1.0, 0.01, 2, "")), ("form", spec(0.0, 1.0, 1.0, 0, "")), ("seed", spec(0.0, 99.0, 1.0, 0, "")),
    ("ω", spec(0.0, 1.5, 0.01, 2, "rad/s")), ("open", spec(20.0, 260.0, 5.0, 0, "px")), ("size", spec(0.3, 1.2, 0.01, 2, "·h"))];
/// The passage's size and line height (logical px), and the widest it is set.
pub const SIZE: f32 = 15.0;
pub const WIDTH: f32 = 780.0;

/// One point of the body's surface: where (unit body space), its outward normal, and the direction the
/// spiral runs there (the tangent a glyph's baseline follows).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfacePoint { pub p: [f32; 3], pub n: [f32; 3], pub t: [f32; 3] }

/// What the poses share across frames: the surface sampled for (form, seed, n glyphs), and the text's
/// box as last drawn (w, h logical px) so the motes know where the body is.
#[derive(Default)]
pub struct Cache { pub key: (u32, u32, usize), pub pts: Vec<SurfacePoint>, pub box_wh: (f32, f32) }

/// A mote shed from the rim: where (text-box px), its velocity, its age (s).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mote { pub x: f32, pub y: f32, pub vx: f32, pub vy: f32, pub age: f32 }
pub const MOTE_LIFE: f32 = 1.4;

#[derive(Clone)]
pub struct Body {
    pub p: Params,
    /// The spring between the page (0) and the body (1), and where it is going.
    pub gather: Animated,
    /// How far the body has turned (yaw about the vertical, pitch about the horizontal) and how fast.
    pub yaw: f32, pub pitch: f32, pub spin: f32, pub tip: f32,
    /// The pointer over the text box (local px) and the opening's amount (0…1, springing).
    pub ptr: Option<(f32, f32)>, pub open: Animated,
    grab: Option<(f32, f32, f32, f32, f32)>, vel: Velocity, dragged: bool,
    pub motes: Vec<Mote>, made: u32,
    pub reduced: bool,
    /// How many times the passage is set (1; more for measuring the frame time with many glyphs).
    pub copies: usize,
    pub t: f32,
    cache: Rc<RefCell<Cache>>,
}
impl Default for Body {
    fn default() -> Self {
        let s = Spring { stiffness: 38.0, damping: 12.0, rest_distance: 0.002, rest_speed: 0.01, ..Spring::default() };
        let o = Spring { stiffness: 120.0, damping: 20.0, rest_distance: 0.002, rest_speed: 0.01, ..Spring::default() };
        Self {
            p: Params::new(&DEFS, &[0.0, 0.0, 7.0, 0.18, 140.0, 0.72]),
            gather: Animated::new(0.0, s), yaw: 0.0, pitch: -0.25, spin: 0.0, tip: 0.0, ptr: None, open: Animated::new(0.0, o),
            grab: None, vel: Velocity::default(), dragged: false, motes: vec![], made: 0, reduced: false, copies: 1, t: 0.0, cache: Rc::default(),
        }
    }
}
impl PartialEq for Body { fn eq(&self, o: &Self) -> bool { self.p == o.p && self.gather == o.gather && self.yaw == o.yaw && self.pitch == o.pitch && self.motes == o.motes && self.ptr == o.ptr && self.open == o.open } }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Key(Key), Num(usize, Edit) }

/// The passage, from the language catalog (an original description of a tidal flat through a day).
pub fn passage() -> String { (1..=5).map(|i| lab::i18n::tr(&format!("body.p{i}"))).collect::<Vec<_>>().join("\n\n") }

// ── The form ─────────────────────────────────────────────────────────────────────────────────────

fn noise3(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let (xi, yi, zi) = (x.floor(), y.floor(), z.floor());
    let (fx, fy, fz) = (x - xi, y - yi, z - zi);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let h = |i: f32, j: f32, k: f32| hash(seed ^ (i as i32 as u32).wrapping_mul(73_856_093) ^ (j as i32 as u32).wrapping_mul(19_349_663) ^ (k as i32 as u32).wrapping_mul(83_492_791));
    let l = |a: f32, b: f32, t: f32| a + (b - a) * s(t);
    let c = |dz: f32| l(l(h(xi, yi, zi + dz), h(xi + 1.0, yi, zi + dz), fx), l(h(xi, yi + 1.0, zi + dz), h(xi + 1.0, yi + 1.0, zi + dz), fx), fy);
    l(c(0.0), c(1.0), fz) * 2.0 - 1.0
}
/// The signed distance to the body (unit space, about radius 1): `form` 0 a rock, 1 a cloud.
pub fn sdf(form: u32, seed: u32, p: [f32; 3]) -> f32 {
    let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    if form == 0 {
        // A sphere pushed in and out by two octaves of noise: a weathered stone.
        let q = [p[0] * 1.7 + 11.0, p[1] * 1.7 + 3.0, p[2] * 1.7 + 7.0];
        let d = 0.26 * noise3(q[0], q[1], q[2], seed) + 0.09 * noise3(q[0] * 2.3, q[1] * 2.3, q[2] * 2.3, seed ^ 0x55);
        len - 0.92 - d
    } else {
        // Six balls placed by the seed, joined smoothly: a cloud.
        let mut d = f32::MAX;
        for b in 0..6u32 {
            let r = |k: u32| hash(seed.wrapping_mul(131) ^ (b * 7 + k)) * 2.0 - 1.0;
            let c = [r(0) * 0.5, r(1) * 0.32, r(2) * 0.42];
            let rad = 0.34 + 0.2 * (r(3) * 0.5 + 0.5);
            let e = ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt() - rad;
            // Polynomial smooth minimum, k = 0.28.
            let k = 0.28;
            let h = (0.5 + 0.5 * (e - d) / k).clamp(0.0, 1.0);
            d = if d == f32::MAX { e } else { e + (d - e) * h - k * h * (1.0 - h) };
        }
        d
    }
}
fn norm(v: [f32; 3]) -> [f32; 3] { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6); [v[0] / l, v[1] / l, v[2] / l] }
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }

/// `n` points on the body's surface along one spiral from the top to the bottom, consecutive points
/// neighbours: a ray from far out along each spiral direction is marched in to the surface.
pub fn surface(form: u32, seed: u32, n: usize) -> Vec<SurfacePoint> {
    let n = n.max(2);
    let turns = (n as f32 * std::f32::consts::PI).sqrt();
    (0..n).map(|i| {
        let h = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
        let th = h.clamp(-1.0, 1.0).acos();
        let ph = -turns * th;
        let dir = [th.sin() * ph.cos(), h, th.sin() * ph.sin()];
        // March in from radius 2 along −dir to where the distance changes sign.
        let mut r = 2.0f32;
        for _ in 0..48 {
            let d = sdf(form, seed, [dir[0] * r, dir[1] * r, dir[2] * r]);
            if d.abs() < 1e-3 { break; }
            r = (r - d * 0.9).max(0.05);
        }
        let p = [dir[0] * r, dir[1] * r, dir[2] * r];
        let e = 1e-3;
        let g = |a: usize| { let mut q = p; q[a] += e; let mut w = p; w[a] -= e; sdf(form, seed, q) - sdf(form, seed, w) };
        let nrm = norm([g(0), g(1), g(2)]);
        // The spiral runs along increasing longitude: the tangent is up × normal, kept on the surface.
        // (on the side facing the viewer at rest it runs left to right, as the lines do).
        let t = norm(cross([0.0, 1.0, 0.0], nrm));
        let t = if t[0].is_finite() && (t[0].abs() + t[2].abs()) > 1e-3 { t } else { [1.0, 0.0, 0.0] };
        SurfacePoint { p, n: nrm, t }
    }).collect()
}

/// Where a surface point is on screen (text-box px), its depth scale, and how much it faces the
/// viewer, at this turn; `c` is the body's centre and `r` its radius (px).
fn project(sp: &SurfacePoint, yaw: f32, pitch: f32, c: (f32, f32), r: f32) -> ([f32; 2], f32, [f32; 3], [f32; 3]) {
    let rot = |v: [f32; 3]| {
        let (s, k) = yaw.sin_cos();
        let (x, z) = (v[0] * k + v[2] * s, -v[0] * s + v[2] * k);
        let (s2, k2) = pitch.sin_cos();
        [x, v[1] * k2 - z * s2, v[1] * s2 + z * k2]
    };
    let (p, n, t) = (rot(sp.p), rot(sp.n), rot(sp.t));
    // A camera 4 radii away: nearer is larger.
    let k = 4.0 / (4.0 - p[2]);
    ([c.0 + r * p[0] * k, c.1 - r * p[1] * k], k, n, t)
}

/// How far glyph `i` of `n` has gone at gather `g`: reading order leads, every glyph within the
/// spring's path; exactly 0 at 0 and 1 at 1.
pub fn progress(g: f32, i: usize, n: usize) -> f32 {
    let st = 0.6;
    let u = (g * (1.0 + st) - st * i as f32 / n.max(1) as f32).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

impl Body {
    /// At rest where the layout put every glyph: no pose at all.
    pub fn flat(&self) -> bool { self.gather.value == 0.0 && !self.gather.moving() }
    pub fn target(&self) -> f32 { self.gather.target }
    fn toggle(&mut self, now: f32) { let to = if self.gather.target >= 0.5 { 0.0 } else { 1.0 }; let v = self.gather.velocity; self.gather.to(to, v, now); }
    /// The body's centre (the text box's) and radius: dense enough that neighbouring glyphs nearly
    /// touch (about 7 px per √glyph), at most `size` of the text's height and half its width.
    fn geometry(w: f32, h: f32, size: f32, n: usize) -> ((f32, f32), f32) { ((w * 0.5, h * 0.5), (7.0 * (n as f32).sqrt()).min(h * size).min(w * 0.46).max(40.0)) }
    /// The body's pose for every glyph, sharing this state's surface cache.
    fn poser(&self) -> impl Fn(GlyphAt) -> Pose + 'static {
        let (g, yaw, pitch, size) = (self.gather.value.clamp(0.0, 1.2), self.yaw, self.pitch, self.p.get(5));
        let (form, seed) = (self.p.get(1) as u32, self.p.get(2) as u32);
        let (ptr, open, rad) = (self.ptr, self.open.value.clamp(0.0, 1.0), self.p.get(4));
        let cache = self.cache.clone();
        move |a: GlyphAt| {
            let mut c = cache.borrow_mut();
            c.box_wh = (a.w, a.h);
            if c.key != (form, seed, a.n) { c.pts = surface(form, seed, a.n); c.key = (form, seed, a.n); }
            let Some(sp) = c.pts.get(a.i) else { return Pose::default() };
            let mut e = progress(g.min(1.0), a.i, a.n);
            // The opening: near the pointer, glyphs go back to their own places on the page.
            // Fully flat within 0.45 of the radius, easing to untouched at the radius.
            if let Some((px, py)) = ptr {
                let d = ((a.x - px).powi(2) + (a.y - py).powi(2)).sqrt() / rad;
                let w = 1.0 - ((d - 0.45) / 0.55).clamp(0.0, 1.0);
                e *= 1.0 - open * w * w * (3.0 - 2.0 * w);
            }
            if e <= 0.0 { return Pose::default(); }
            let (cen, r) = Self::geometry(a.w, a.h, size, a.n);
            let ([mut sx, mut sy], k, n, t) = project(sp, yaw, pitch, cen, r);
            // The body's glyphs part round the opening: one that would land inside it is pushed out
            // to its rim, so the words come back to an open patch of page.
            if let Some((px, py)) = ptr && open > 0.0 {
                let (vx, vy) = (sx - px, sy - py);
                let d = vx.hypot(vy).max(1e-3);
                let rr = rad * 0.9 * open;
                if d < rr { let f = rr / d; sx = px + vx * f; sy = py + vy * f; }
            }
            // Facing: how much the surface turns toward the viewer; near is brighter, the back dim.
            let face = n[2];
            let rim = (1.0 - face.abs()).powi(3) * (face > -0.2) as i32 as f32;
            let bright = if face >= 0.0 { 0.7 + 0.4 * face.powf(0.6) + 0.4 * rim } else { 0.1 + 0.1 * (1.0 + face) };
            let depth = ((k - 0.8) / 0.45).clamp(0.0, 1.0);
            let lum = (bright * (0.7 + 0.3 * depth)).clamp(0.05, 1.0);
            // Turned to the spiral's tangent on screen, squeezed by its slope, scaled by depth.
            let ang = (-t[1]).atan2(t[0]);
            let squeeze = (t[0] * t[0] + t[1] * t[1]).sqrt().max(0.25);
            let lift = 1.0 + 0.5 * (e * (1.0 - e)) * 4.0 * 0.25;
            let (dx, dy) = (sx - a.x, sy - a.y);
            Pose {
                dx: dx * e, dy: dy * e,
                angle: ang * e,
                sx: 1.0 + (k * squeeze * 0.95 - 1.0) * e * lift.min(1.2),
                sy: 1.0 + (k * 0.95 - 1.0) * e,
                tone: 1.0,
                alpha: 1.0 + (lum - 1.0) * e,
                // One hue: the front nearly white with it, the rim and the back the hue itself.
                accent: e * if face >= 0.0 { 0.25 + 0.75 * (1.0 - face).powi(2) } else { 1.0 },
            }
        }
    }
}

impl Experiment for Body {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let passage = text(vec![passage(); self.copies.max(1)].join("\n\n"), SIZE, Paint::INK).max_width(WIDTH).fill_width().key(format!("{key}/text"));
        // At rest on the page, no pose: every glyph is exactly the plain layout's.
        let passage = if self.flat() { passage } else { passage.pose(self.poser()) };
        // The motes, over the text's box as last drawn (they never change the layout).
        let (w, h) = self.cache.borrow().box_wh;
        let mut parts = vec![passage];
        if w > 0.0 && h > 0.0 && !self.motes.is_empty() {
            let mut d = Drawing::new(w, h);
            for m in &self.motes {
                let f = 1.0 - m.age / MOTE_LIFE;
                d.dot(m.x, m.y, 0.8 + 1.0 * f, Paint::ACCENT.alpha(0.8 * f));
            }
            parts.push(canvas(d).key(format!("{key}/motes")).width(w).absolute(Some(0.0), Some(0.0), None, None).respond(lab::el::Response::Ignore));
        }
        let stage = layers(parts).key(format!("{key}/stage")).max_width(WIDTH).fill_width()
            .on_point(|p| Some(Msg::Point(p))).on_key(|k, _| matches!(k, Key::Char(' ') | Key::Enter | Key::Left | Key::Right).then_some(Msg::Key(k)))
            .role(lab::el::Role::Img).label(lab::i18n::tr("body.label"));
        let formula = format!("gather = {:.2} → {:.0}   form = {}  seed {:.0}   θ = {:.2}, φ = {:.2}, ω = {:+.2}   glyphs = {}   motes = {}",
            self.gather.value, self.gather.target, if self.p.get(1) < 0.5 { "rock" } else { "cloud" }, self.p.get(2), self.yaw, self.pitch, self.spin, self.cache.borrow().key.2, self.motes.len());
        let mut caption = col().key(format!("{key}/caption")).max_width(WIDTH).fill_width().gap(6.0)
            .child(row().fill_width().gap(10.0)
                .child(text("[ BODY ]", 11.0, Paint::TEXT).font(lab::el::Font::Mono).key(format!("{key}/name")))
                .child(text(lab::i18n::tr("body.hint"), 11.0, Paint::DIM).font(lab::el::Font::Mono).key(format!("{key}/hint")))
                .child(space(0.0, 0.0).grow(1.0))
                .child(text(if self.p.open { "[ − params ]" } else { "[ params ]" }, 11.0, Paint::DIM).font(lab::el::Font::Mono).key(format!("{key}/params"))
                    .focusable(true).role(lab::el::Role::Button).label("params").on_press(Msg::Num(TOGGLE, Edit::Cancel))))
            .child(col().fill_width().min_width(0.0).clip().child(text(formula, 11.0, Paint::FAINT).font(lab::el::Font::Mono).nowrap().recompose(false).key(format!("{key}/formula"))));
        if self.p.open {
            let mut nums = row().key(format!("{key}/nums")).wrap().gap(4.0);
            let vals: Vec<f32> = (0..DEFS.len()).map(|i| if i == 0 { self.gather.value.clamp(0.0, 1.0) } else { self.p.get(i) }).collect();
            for (i, (n, sp)) in DEFS.iter().enumerate() {
                let draft = self.p.draft.as_ref().filter(|d| d.0 == i).map(|d| d.1.as_str());
                nums = nums.child(lab::widgets::number(&format!("{key}/p{i}"), n, vals[i], *sp, draft, move |e| Msg::Num(i, e)).width(176.0).zoom(0.72));
            }
            caption = caption.child(nums);
        }
        // Room round the text for the body (it is centred on the text and may be taller than it).
        let room = col().key(format!("{key}/room")).fill_width().grow(1.0).min_height(0.0).center().child(stage);
        col().key(key.to_owned()).fill_width().fill_height().align(taffy::AlignItems::CENTER).gap(18.0).pad(24.0, 20.0).child(room).child(caption)
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Key(Key::Left) => { self.spin -= 0.6; }
            Msg::Key(Key::Right) => { self.spin += 0.6; }
            Msg::Key(_) => self.toggle(now),
            Msg::Num(0, Edit::Set(v)) => { let v0 = self.gather.velocity; self.gather.to(if v >= 0.5 { 1.0 } else { 0.0 }, v0, now); self.p.draft = None; }
            Msg::Num(i, e) => { self.p.edit(i, e); }
            Msg::Point(p) => {
                self.ptr = p.map(|q| (q.x, q.y));
                let over = p.is_some() && self.gather.target >= 0.5;
                let v = self.open.velocity;
                self.open.to(if over && self.grab.is_none() { 1.0 } else { 0.0 }, v, now);
                match (p, self.grab) {
                    (Some(q), Some((x0, y0, yaw0, pitch0, _))) if q.down => {
                        if (q.x - x0).hypot(q.y - y0) > 5.0 { self.dragged = true; }
                        if self.dragged && self.gather.target >= 0.5 {
                            let r = 260.0;
                            self.yaw = yaw0 + (q.x - x0) / r;
                            self.pitch = (pitch0 + (q.y - y0) / r).clamp(-1.0, 1.0);
                            self.vel.push(now, q.x, q.y);
                            self.open.to(0.0, self.open.velocity, now);
                        }
                    }
                    (Some(q), None) if q.down => { self.grab = Some((q.x, q.y, self.yaw, self.pitch, now)); self.dragged = false; self.vel.clear(); self.vel.push(now, q.x, q.y); }
                    (_, Some(_)) => {
                        // Let go: a press that did not move toggles; a drag keeps turning at its speed.
                        if self.dragged { let (vx, vy) = self.vel.get(); self.spin = (vx / 260.0).clamp(-8.0, 8.0); self.tip = (vy / 260.0).clamp(-4.0, 4.0); } else { self.toggle(now); }
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
        let mut changed = self.gather.step(now) | self.open.step(now);
        let body = self.gather.target >= 0.5;
        // The turn: a throw coasts down to the idle turn (none while motion is reduced or on the page).
        if self.grab.is_none() {
            let idle = if body && !self.reduced { self.p.get(3) } else { 0.0 };
            let k = (-1.4 * dt).exp();
            self.spin = idle + (self.spin - idle) * k;
            self.tip *= k;
            if (self.spin - idle).abs() < 1e-3 { self.spin = idle; }
            if self.tip.abs() < 1e-3 { self.tip = 0.0; }
            if self.spin != 0.0 || self.tip != 0.0 { self.yaw += self.spin * dt; self.pitch = (self.pitch + self.tip * dt).clamp(-1.0, 1.0); changed = true; }
        }
        // Motes: rim glyphs shed a few, from the frame budget, only while it is the body.
        for m in &mut self.motes { m.age += dt; m.x += m.vx * dt; m.y += m.vy * dt; m.vy -= 4.0 * dt; }
        let before = self.motes.len();
        self.motes.retain(|m| m.age < MOTE_LIFE);
        changed |= before != self.motes.len() || !self.motes.is_empty();
        // (while it turns: a body standing still sheds nothing, so it can come to rest)
        if body && self.gather.value > 0.97 && !self.reduced && dt > 0.0 && self.spin.abs() > 0.01 {
            let c = self.cache.borrow();
            let (w, h) = c.box_wh;
            let want = lab::budget::grant(2).min(90usize.saturating_sub(self.motes.len()));
            if !c.pts.is_empty() && want > 0 {
                let (cen, r) = Self::geometry(w, h, self.p.get(5), c.pts.len());
                let mut born = 0;
                for tries in 0..40u32 {
                    if born >= want { break; }
                    self.made = self.made.wrapping_add(1);
                    let i = (hash(self.made.wrapping_mul(2_654_435_761) ^ tries) * c.pts.len() as f32) as usize % c.pts.len();
                    let ([x, y], _, n, _) = project(&c.pts[i], self.yaw, self.pitch, cen, r);
                    // Only from the rim, facing sideways, and never inside the opening.
                    if n[2].abs() > 0.18 { continue; }
                    if let Some((px, py)) = self.ptr && self.open.value > 0.05 && (x - px).hypot(y - py) < self.p.get(4) * 1.6 { continue; }
                    let out = norm([n[0], -n[1], 0.0]);
                    let sp = 14.0 + 22.0 * hash(self.made ^ 0x9e37);
                    self.motes.push(Mote { x, y, vx: out[0] * sp, vy: out[1] * sp, age: 0.0 });
                    born += 1;
                }
            }
        }
        changed
    }
    fn moving(&self) -> bool {
        self.gather.moving() || self.open.moving() || !self.motes.is_empty() || self.grab.is_some()
            || self.spin != 0.0 || self.tip != 0.0 || (self.gather.target >= 0.5 && !self.reduced && self.p.get(3) > 0.0)
    }
}

/// A runtime of its own (the `render` tool, tests): the scene on the lab's dark ground.
#[cfg(not(target_arch = "wasm32"))]
pub fn ui(faces: Vec<lab::text::Face>, w: u32, h: u32, scale: f32) -> lab::ui::Ui<Body, Msg> {
    fn view(s: &Body) -> El<Msg> { col().fill_width().fill_height().fonts(lab::style::LAB.roles()).child(s.view("body")) }
    fn update(s: &mut Body, m: Msg) { let now = s.t; s.update(m, now); }
    lab::ui::Ui::new(Body::default(), view, update, faces, w, h, scale)
        .on_look(|_| look())
        .on_tick(|s, t| s.step(t))
        .on_wake(|s, t| s.moving().then_some(t))
}
/// The scene's look: the lab's theme (near-black ground, one blue), plain material, a bloom.
pub fn look() -> lab::style::Look {
    let mut l = lab::style::Look { theme: lab::style::LAB, material: lab::style::Material::Plain, ..lab::style::Look::default() };
    l.post.bloom = lab::post::Bloom { strength: 0.9, threshold: 0.3, radius: 12.0 };
    l
}
