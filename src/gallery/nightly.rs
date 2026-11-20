//! W-04 `nightly` (site-design M.4, V.2): night over flat water, and across the sky one line of type set on
//! a slow curve, the build's log. An aurora hangs from the line, each ray one stroke with a ramp
//! (`Drawing::ramp`) over the picture, whose sky (the gradient and the stars) is drawn in it, so every ray
//! mixes its stops over a picture. The curve drifts and the rays' lengths travel along it, paced at 30
//! frames a second; a press runs the next step of the build, a pulse along the line both ways.
//!
//! On a phone the piece has a layout of its own (V.2 j): the line from 6 % to 94 % of the window with as
//! many cells as fit at the desktop's pitch, cut on a word, the rays at the same 2.35 px, and the light's
//! callout to its left.
use super::works::{Fit, at, mono, rgb};
use super::{Gallery, Msg, NARROW};
use amenui::draw::Drawing;
use amenui::el::{El, Font, Paint, Pose, Response, canvas, col, text};

/// W-04's place in the day.
pub const NIGHTLY: usize = 3;
/// The horizon, a share of the picture's height down; on a phone 74 % down the window, so the title
/// block lies on the water under it (W.1 item 5).
const HORIZON: f32 = 0.80;
pub const PHONE_HORIZON: f32 = 0.74;
/// Where the line runs, a share of the width.
const FROM: f32 = 0.06;
const TO: f32 = 0.94;
/// The rays at 1440 wide and over (V.2 d: 540, 0.88 w / 539 apart), and the spacing kept under that
/// (2.35 px, W.1 item 6).
pub const RAYS: usize = 540;
const SPACING: f32 = 0.88 * 1440.0 / 539.0;
/// The line of type: 150 cells on a desktop, lowercase, the log repeated; its build number is the seed.
const CELLS: usize = 150;
fn log(build: u32) -> String { format!("nightly  ok  ok  ok  build {build:04}  ok  passing  ok  0 errors  0 warnings  ok  1 aurora  ok  ") }
/// The first build.
const FIRST_BUILD: u32 = 412;
/// The curve's control points (px of the 1440 × 900 picture, evenly from 6 % to 94 % across), between
/// 30 % and 62 % of the height; each drifts by ±26 px over about 24 s.
const KNOTS: [f32; 9] = [455.0, 425.0, 330.0, 318.0, 282.0, 402.0, 455.0, 548.0, 506.0];
const DRIFT: f32 = 26.0;
/// The knots' mean height, and how much of their swell a phone keeps.
const MEAN: f32 = 413.0;
const PHONE_FLAT: f32 = 0.4;
/// The rays' lengths, 40 to 360 px, from a noise that travels along the line at 12 px a second: the
/// short ones between the folds are the gaps (W.1 item 1).
const SHORT: f32 = 40.0;
const LONG: f32 = 360.0;
const TRAVEL: f32 = 12.0;
/// The colours (M.4, W.1 item 3): the ground (the sky's top), the type, the rays' middle, the stars and
/// notes. The ground is the night's own dark, so nothing pale shows where the picture is not yet drawn.
pub const GROUND: [f32; 3] = rgb(0x070b16);
pub const TYPE_C: [f32; 3] = rgb(0xe2fff0);
pub const MID: [f32; 3] = rgb(0x2fae8c);
pub const INK: [f32; 3] = rgb(0xe2e8f0);
/// The piece's own warm signal: its number in the title block and the pointer's box over the night.
pub const TAG: [f32; 3] = rgb(0xff6b2b);
/// The type is the theme's third hue. The rays are in colours of their own (`Drawing::palette`), so
/// they are the same in the index's window and band as on the page: root `#baffd8` → middle `#2fae8c` →
/// `#1f5f86` as they go (X.2 item 2), the tallest turning toward the accent.
const TYPE: Paint = Paint::hue(2);
pub const ROOT: [f32; 3] = rgb(0xbaffd8);
pub const DEEP: [f32; 3] = rgb(0x1f5f86);
const ACCENT: [f32; 3] = rgb(0xff6b2b);
/// A paint of the drawing's own colour `k`.
const fn own(k: u8) -> Paint { Paint { tone: 1.0, accent: 1.0, alpha: 1.0, hue: 3 + k } }
fn palette(d: &mut Drawing) { d.palette([ROOT, MID, DEEP, ACCENT]); }
/// A ray's opacity at its root and middle stop (at MID_AT of its length), and a pulse's brightening.
const ROOT_A: f32 = 0.97;
const MID_A: f32 = 0.6;
const MID_AT: f32 = 0.3;
/// A tipped ray turns toward the accent (up to 0.8) over its top third, at its neighbours' opacity, and
/// its last TIP_CLEAR px are gone (W.1 item 2).
const TIP_FROM: f32 = 2.0 / 3.0;
const TIP_CLEAR: f32 = 20.0;
const BRIGHT: f32 = 1.6;
const LONGER: f32 = 1.2;
/// A press: the pulse's speed along the line (px a second) and how long a ray stays lit.
const PULSE: f32 = 600.0;
const LIT: f32 = 0.5;
/// The new curtain after a build passes comes in by opacity over this (V.2 h).
const SWAP: f32 = 0.12;
/// The steps of the build, and the one it rests on.
const STEPS: u32 = 7;
const REST_STEP: u32 = 3;
/// The fold nearest the pointer leans toward it by at most this, on a spring.
const LEAN: f32 = 18.0;
/// The light on the horizon, a share of the width across.
const LIGHT: f32 = 0.70;
/// Frames for what moves by itself, a second.
const PACE: f32 = 30.0;

/// W-04's state: how far its motion has run (s, only while it is awake), when it last stepped, the build's
/// step and number (the seed), the last press (its place along the line and when), the flare after step
/// 7, when the last build passed, how many times the reflection has shifted, and the lean of the fold
/// nearest the pointer (its centre, its lean and the lean's speed, where it is going).
#[derive(Clone, Debug)]
pub struct Night {
    pub run: f32,
    pub stepped: f32,
    pub step: u32,
    pub build: u32,
    pub pulse: Option<(f32, f32)>,
    pub flare: f32,
    pub passed: f32,
    pub shift: u32,
    pub pointer: Option<(f32, f32)>,
    pub down: bool,
    pub fold: f32,
    pub lean: f32,
    pub lean_v: f32,
    pub aim: f32,
}
impl Default for Night {
    fn default() -> Self { Self { run: 0.0, stepped: -1.0, step: REST_STEP, build: FIRST_BUILD, pulse: None, flare: f32::NEG_INFINITY, passed: f32::NEG_INFINITY, shift: 0, pointer: None, down: false, fold: 0.0, lean: 0.0, lean_v: 0.0, aim: 0.0 } }
}

fn hash(a: u32, b: u32, c: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9e37_79b9) ^ b.wrapping_mul(0x85eb_ca6b) ^ c.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 15; h = h.wrapping_mul(0x2c1b_3c6d); h ^= h >> 12; h = h.wrapping_mul(0x297a_2d39); h ^= h >> 15;
    (h >> 8) as f32 / (1u32 << 24) as f32
}
/// One-dimensional value noise of three octaves, 0–1.
fn noise(x: f32, seed: u32) -> f32 {
    let (mut v, mut a, mut n) = (0.0, 1.0, 0.0);
    for o in 0..3u32 {
        let p = x * (1u32 << o) as f32;
        let (i, f) = (p.floor(), p - p.floor());
        let f = f * f * (3.0 - 2.0 * f);
        let g = |k: f32| hash(k as i32 as u32, seed, o);
        v += a * (g(i) + (g(i + 1.0) - g(i)) * f);
        n += a;
        a *= 0.5;
    }
    v / n
}

/// How the piece lies in a `w × h` window: across, at the window's own size (the line from 6 % to 94 %,
/// W.1 item 6); down, as its picture does (`fit`) on a desktop, and on a phone with heights scaled by the
/// window's height and the horizon 74 % down. Under 1440 wide the rays keep 2.35 px and the line as many
/// cells as fit (`cut`).
#[derive(Clone, Copy)]
pub struct Lay { phone: bool, cut: bool, w: f32, h: f32, s: f32, oy: f32 }
/// The layout of W-04's page at `w × h`.
pub fn lay_for(w: f32, h: f32) -> Lay { Lay::new(w, h, &Fit::new(w, h, super::works::PIECES[NIGHTLY].focus)) }
impl Lay {
    pub fn new(w: f32, h: f32, fit: &Fit) -> Self {
        let cut = w < 1440.0;
        if w < NARROW { Self { phone: true, cut, w, h, s: h / 900.0, oy: 0.0 } } else { Self { phone: false, cut, w, h, s: fit.s(), oy: fit.at(0.0, 0.0).1 } }
    }
    /// A point `u` (0–1) along the line, across.
    fn x(&self, u: f32) -> f32 { self.w * (FROM + (TO - FROM) * u) }
    /// A height of the 1440 × 900 picture, down the window.
    fn y(&self, y: f32) -> f32 { self.oy + y * self.s }
    pub fn hz(&self) -> f32 { if self.phone { (PHONE_HORIZON * self.h).floor() } else { self.y(HORIZON * 900.0).floor() } }
    /// The rays: how many, and where each is across.
    pub fn rays(&self) -> usize { if self.cut { (((self.x(1.0) - self.x(0.0)) / SPACING).floor() as usize + 1).min(RAYS) } else { RAYS } }
    fn ray(&self, i: usize) -> f32 { if self.cut { self.x(0.0) + i as f32 * SPACING } else { self.x(i as f32 / (RAYS - 1) as f32) } }
    /// Where along the noise a place across is (px of the picture's line, from its start).
    fn along(&self, x: f32) -> f32 { (x - self.x(0.0)) / self.s + FROM * 1440.0 }
}

/// The line on screen after `run` s: points every 4 px of the picture's line, a Catmull-Rom curve through
/// the knots, each moved by its own slow swell (none at 0, the rest pose); and how far along each is.
fn curve(l: &Lay, run: f32, build: u32) -> (Vec<(f32, f32)>, Vec<f32>) {
    let knot = |k: i32| {
        let k = k.clamp(0, KNOTS.len() as i32 - 1) as usize;
        let (p, w) = (20.0 + 8.0 * hash(k as u32, build, 1), 0.6 + 0.4 * hash(k as u32, build, 2));
        KNOTS[k] + DRIFT * w * (std::f32::consts::TAU * run / p).sin()
    };
    let n = (1440.0 * (TO - FROM) / 4.0).ceil() as usize;
    let last = KNOTS.len() as f32 - 2.0;
    let pts: Vec<(f32, f32)> = (0..=n).map(|j| {
        let u = j as f32 / n as f32;
        let q = u * (KNOTS.len() - 1) as f32;
        let k = q.floor().min(last);
        let t = q - k;
        let k = k as i32;
        let (p0, p1, p2, p3) = (knot(k - 1), knot(k), knot(k + 1), knot(k + 2));
        let y = 0.5 * (2.0 * p1 + (p2 - p0) * t + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t + (3.0 * p1 - p0 - 3.0 * p2 + p3) * t * t * t);
        // On a phone the line is a quarter as wide: its swells are flattened about its mean, so its
        // slopes stay near the desktop's and the glyphs on them stay apart.
        let y = if l.phone { MEAN + (y - MEAN) * PHONE_FLAT } else { y };
        (l.x(u), l.y(y))
    }).collect();
    let mut arc = vec![0.0];
    for w in pts.windows(2) { arc.push(arc[arc.len() - 1] + (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1)); }
    (pts, arc)
}
/// The line's centre down the window at rest (where a phone's band crops it).
pub fn centre(l: &Lay) -> f32 { let (p, _) = curve(l, 0.0, FIRST_BUILD); let (a, b) = p.iter().fold((f32::MAX, f32::MIN), |m, q| (m.0.min(q.1), m.1.max(q.1))); (a + b) * 0.5 }
/// The line's height at `x` on screen, and how far along it that is.
fn on_line(c: &(Vec<(f32, f32)>, Vec<f32>), x: f32) -> (f32, f32) {
    let (p, a) = c;
    let u = ((x - p[0].0) / (p[1].0 - p[0].0)).clamp(0.0, (p.len() - 1) as f32 - 1e-3);
    let (k, f) = (u.floor() as usize, u - u.floor());
    (p[k].1 + (p[k + 1].1 - p[k].1) * f, a[k] + (a[k + 1] - a[k]) * f)
}
/// A ray's length (px of the picture) at `x` along the picture's line after `run` s: the noise moving
/// along the line, shaped so most rays are short and a few folds are tall.
fn length(x: f32, run: f32, build: u32) -> f32 {
    let u = x - TRAVEL * run;
    let n = noise(u / 70.0, build) * 0.75 + noise(u / 260.0, build + 1) * 0.25;
    let k = ((n - 0.3) / 0.42).clamp(0.0, 1.0).powf(1.6);
    // Each ray a little shorter or longer than its neighbours, as the rays of a curtain are.
    let own = 0.72 + 0.28 * hash(x.to_bits(), build, 8);
    SHORT + (LONG - SHORT) * k * own
}

/// How lit a ray at `s` along the line is now (0–1): the press's pulse reached it under 500 ms ago, or
/// the flare after step 7 is under way.
fn lit(n: &Night, s: f32, now: f32) -> f32 {
    let on = |since: f32| if (0.0..LIT).contains(&since) { 1.0 - (since / LIT).powi(4) } else { 0.0 };
    let p = n.pulse.map_or(0.0, |(at, t)| on(now - t - (s - at).abs() / PULSE));
    p.max(on(now - n.flare))
}

/// The line's text as drawn at this layout: 150 cells from 1440 wide; under that as many as fit along
/// the line at rest at the pitch of 1440, cut after the last whole word.
pub fn line_text(l: &Lay, build: u32) -> String {
    let all: String = log(build).chars().cycle().take(CELLS).collect();
    if !l.cut { return all; }
    let wide = WIDE;
    let pitch = curve(&wide, 0.0, build).1.last().copied().unwrap_or(1.0) / CELLS as f32;
    let fit = (curve(l, 0.0, build).1.last().copied().unwrap_or(0.0) / pitch).floor() as usize;
    let cut: Vec<char> = all.chars().take(fit.min(CELLS)).collect();
    // A word cut by the end goes; so do the spaces before it.
    let end = if all.chars().nth(cut.len()).is_some_and(|c| c != ' ') { cut.iter().rposition(|&c| c == ' ').unwrap_or(0) } else { cut.len() };
    cut[..end].iter().collect::<String>().trim_end().to_owned()
}
/// The glyphs drawn on the line: its cells less the spaces (V.2 e).
pub fn glyphs(l: &Lay, build: u32) -> usize { line_text(l, build).chars().filter(|&c| c != ' ').count() }

/// The piece's picture (pixels, `w × h`), a pixel of it a pixel at 1440 × 900 or, for a phone's, at
/// 390 × 844: the sky from `#070b16` through `#0c1626` at 60 % to `#12233a` at the horizon, a hairline
/// of `#1f3043` on it, the water from `#0a1320` to `#070b14`, the stars (130 at 1440 × 900, as many a
/// pixel of sky on a phone; 1 or 2 px squares of `#e2e8f0` at 0.25 to 0.8), and the grain.
pub fn pixels(w: usize, h: usize) -> Vec<u8> {
    let horizon = if (w as f32) < NARROW { PHONE_HORIZON } else { HORIZON };
    let hz = (horizon * h as f32) as usize;
    let c = |v: u32| [((v >> 16) & 255) as f32, ((v >> 8) & 255) as f32, (v & 255) as f32];
    let mix = |a: [f32; 3], b: [f32; 3], t: f32| -> [f32; 3] { std::array::from_fn(|k| a[k] + (b[k] - a[k]) * t) };
    let (top, mid, low, line, sea, deep, star) = (c(0x070b16), c(0x0c1626), c(0x12233a), c(0x1f3043), c(0x0a1320), c(0x070b14), c(0xe2e8f0));
    let mut px = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        let v = (y as f32 + 0.5) / h as f32;
        let col = if y < hz { if v < 0.75 * horizon { mix(top, mid, v / (0.75 * horizon)) } else { mix(mid, low, (v - 0.75 * horizon) / (0.25 * horizon)) } } else if y == hz { line } else { mix(sea, deep, (v - horizon) / (1.0 - horizon)) };
        for x in 0..w {
            // The hairline is the horizon's own colour; the grain is on everything else.
            let n = if y == hz { 0.0 } else { (hash(x as u32, y as u32, 9) - 0.5) * 0.04 * 255.0 };
            px.extend(col.map(|v| (v + n).clamp(0.0, 255.0).round() as u8));
            px.push(255);
        }
    }
    let stars = (130.0 * (w * hz) as f32 / (1440.0 * 720.0)).round() as u32;
    for k in 0..stars {
        let (x, y) = ((hash(k, 1, 7) * (w - 1) as f32) as usize, (hash(k, 2, 7) * (hz as f32 - 12.0)) as usize);
        let (size, a) = (1 + (hash(k, 3, 7) < 0.3) as usize, 0.25 + 0.55 * hash(k, 4, 7));
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().take(size * size) {
            let o = ((y + dy) * w + x + dx) * 4;
            for j in 0..3 { px[o + j] = (px[o + j] as f32 + (star[j] - px[o + j] as f32) * a).round() as u8; }
        }
    }
    px
}

/// W-04 over its picture as it is now;
/// `view` is the part of the piece shown; `title` the title block's box and
/// `strip` the strip's, which the reflection keeps clear of.
#[allow(clippy::too_many_arguments)]
pub fn layer(s: &Gallery, l: &Lay, title: [f32; 4], strip: [f32; 4], view: (f32, f32, f32, f32)) -> El<Msg> {
    let n = &s.works.night;
    let now = s.clock;
    let w = l.w;
    let key = |p: &str| format!("work/nightly/{p}");
    let c = curve(l, n.run, n.build);
    // After a build passes the new curtain comes in over the old by opacity (V.2 h).
    let swap = ((now - n.passed) / SWAP).clamp(0.0, 1.0);
    let mut sky = col().key(key("sky")).width(w).height(l.h).absolute(Some(0.0), Some(0.0), None, None).respond(Response::Ignore).pass(true);
    if swap < 1.0 {
        let old = curve(l, n.run, n.build - 1);
        sky = sky.child(rays(l, n, &old, n.build - 1, now, view).key(key("rays/old")).opacity(1.0 - swap));
        sky = sky.child(rays(l, n, &c, n.build, now, view).key(key("rays")).opacity(swap));
    } else {
        sky = sky.child(rays(l, n, &c, n.build, now, view).key(key("rays")));
    }
    // The light on the horizon, two pixels wide, and the callout: up and to its right on a desktop, up
    // and to its left on a phone (clear of the title block above the horizon there).
    let hz = l.hz();
    let lx = (w * LIGHT).round();
    let call = if n.step == 0 { "KP 5   PASSING".to_owned() } else { format!("KP 5   STEP {}/{STEPS}", n.step) };
    let mut d = Drawing::new(w, hz);
    d.line((lx + 1.0, hz - 1.0), (lx + 1.0, hz - 1.0), 2.0, Paint::ACCENT);
    let leader = Paint::INK.alpha(0.6);
    let (cx, cy) = callout_at(l, &call);
    if l.phone {
        d.line((lx - 1.5, hz - 4.5), (lx - 25.5, hz - 28.5), 1.0, leader);
        d.line((lx - 25.5, hz - 28.5), (lx - 25.5, cy + 9.5), 1.0, leader);
        d.line((lx - 25.5, cy + 9.5), (lx - 44.0, cy + 9.5), 1.0, leader);
    } else {
        d.line((lx + 3.5, hz - 4.5), (lx + 29.5, hz - 29.5), 1.0, leader);
        d.line((lx + 29.5, hz - 29.5), (lx + 48.0, hz - 29.5), 1.0, leader);
    }
    sky.child(canvas(d).key(key("light")).width(w).absolute(Some(0.0), Some(0.0), None, None).pass(true))
        .child(line_of_type(l, &c, line_text(l, n.build), key("log")))
        .child(reflection(l, n.shift, title, strip, view))
        .child(at(mono(call, Paint::INK).key(key("callout")).recompose(false), cx, cy))
}
/// Where the callout's box goes (its top left).
pub fn callout_at(l: &Lay, call: &str) -> (f32, f32) {
    let (hz, lx) = (l.hz(), (l.w * LIGHT).round());
    if l.phone { ((lx - 50.0 - call.chars().count() as f32 * super::works::ADV).round(), hz - 92.0) } else { (lx + 54.0, hz - 38.0) }
}
/// The light's box.
pub fn light_at(l: &Lay) -> [f32; 4] { let (hz, lx) = (l.hz(), (l.w * LIGHT).round()); [lx, hz - 2.0, lx + 2.0, hz] }

/// The rays of build `build` on line `c`: each from its root straight up, the fold nearest the pointer
/// leaning to it. Those the view does not show across are not drawn, and none rises out of the picture
/// (it is gone at its tip, so this is not seen): only over a picture does a ramp mix its stops.
fn rays(l: &Lay, n: &Night, c: &(Vec<(f32, f32)>, Vec<f32>), build: u32, now: f32, view: (f32, f32, f32, f32)) -> El<Msg> {
    let mut d = Drawing::new(l.w, l.hz());
    palette(&mut d);
    let (root, mid, deep) = (own(0), own(1), own(2));
    let top = l.y(16.0);
    for i in 0..l.rays() {
        let rx = l.ray(i);
        let (ry, along) = on_line(c, rx);
        if rx < view.0 + LEAN + 1.0 || rx > view.2 - LEAN - 1.0 { continue; }
        let b = lit(n, along, now);
        let len = length(l.along(rx), n.run, build);
        let h = (len * l.s * (1.0 + (LONGER - 1.0) * b)).min(ry - top);
        let lean = n.lean * (-((rx - n.fold) / 60.0).powi(2)).exp();
        let up = 1.0 + (BRIGHT - 1.0) * b;
        let (ra, ma) = ((ROOT_A * up).min(1.0), (MID_A * up).min(1.0));
        let at = |f: f32| (rx + lean * f, ry - h * f);
        // The tallest turn toward the accent over their top third, up to 0.8, at the opacity every ray
        // has there (W.1 item 2): the ray in two strokes that meet at two thirds of its length, from
        // the middle stop into the accent.
        let tall = ((len - 330.0) / (LONG - 330.0)).clamp(0.0, 1.0).powi(3);
        if tall > 0.0 {
            let joint = ma * (1.0 - TIP_FROM) / (1.0 - MID_AT);
            let end = (1.0 - TIP_CLEAR / h.max(TIP_CLEAR * 3.0)).max(TIP_FROM + 0.05);
            let tip = Paint { tone: 1.0, accent: 0.8 * tall.powf(1.0 / 3.0), alpha: 0.0, hue: 6 };
            d.ramp(at(0.0), at(TIP_FROM), 1.0, root.alpha(ra), Some((mid.alpha(ma), MID_AT / TIP_FROM)), mid.alpha(joint));
            let fade = joint * (1.0 - end) / (1.0 - TIP_FROM);
            // The hue has turned by a third of the way up the top third; the opacity keeps falling as
            // every ray's does.
            let turn = 1.0 / 3.0;
            d.ramp(at(TIP_FROM), at(end), 1.0, mid.alpha(joint), Some((tip.alpha(joint + (fade - joint) * turn), turn)), tip.alpha(fade));
        } else {
            d.ramp(at(0.0), at(1.0), 1.0, root.alpha(ra), Some((mid.alpha(ma), MID_AT)), deep.alpha(0.0));
        }
    }
    canvas(d).width(l.w).absolute(Some(0.0), Some(0.0), None, None).pass(true)
}
/// The rays as built at rest (for the checks): each one's root, length on screen, and whether its tip
/// takes the accent.
pub fn rest_rays(l: &Lay) -> Vec<(f32, f32, f32, bool)> {
    let c = curve(l, 0.0, FIRST_BUILD);
    let top = l.y(16.0);
    (0..l.rays()).map(|i| { let rx = l.ray(i); let ry = on_line(&c, rx).0; let len = length(l.along(rx), 0.0, FIRST_BUILD); (rx, ry, (len * l.s).min(ry - top), len > 330.0) }).collect()
}

/// The line of type on the curve: one line laid out once, each glyph posed onto its place along the
/// curve (by its column: the face is monospaced), turned with it, its centre 7 px under the rays' roots.
/// On a desktop its cells share the line; on a phone they keep the desktop's pitch as the line drifts.
fn line_of_type(l: &Lay, c: &(Vec<(f32, f32)>, Vec<f32>), t: String, key: String) -> El<Msg> {
    let cells = t.chars().count().max(1);
    let adv = super::works::ADV;
    let places = place_cells(l, c, cells);
    text(t, 11.0, TYPE).font(Font::Face(amenui::text::MONO)).tracking(0.06).nowrap().key(key).recompose(false).pass(true)
        .pose(move |g| {
            let (x, y, angle) = places[((g.x / adv) as usize).min(cells - 1)];
            Pose { dx: x - g.x, dy: y - g.y, angle, ..Pose::default() }
        })
        .absolute(Some(0.0), Some(0.0), None, None)
}
/// Where each glyph's centre is at rest, and the line's turn there (for the checks).
pub fn places(l: &Lay) -> Vec<(f32, f32, f32)> { let c = curve(l, 0.0, FIRST_BUILD); place_cells(l, &c, line_text(l, FIRST_BUILD).chars().count()) }
fn place_cells(l: &Lay, c: &(Vec<(f32, f32)>, Vec<f32>), cells: usize) -> Vec<(f32, f32, f32)> {
    let (pts, arc) = c;
    let total = arc[arc.len() - 1];
    let step = if l.cut { total / (curve(l, 0.0, FIRST_BUILD).1.last().copied().unwrap_or(total) / pitch()) } else { total / CELLS as f32 };
    (0..cells).map(|c| {
        let s = (c as f32 + 0.5) * step;
        let j = arc.partition_point(|&a| a < s).clamp(1, arc.len() - 1);
        let (a, b) = (pts[j - 1], pts[j]);
        let f = ((s - arc[j - 1]) / (arc[j] - arc[j - 1]).max(1e-3)).clamp(0.0, 1.0);
        let (tx, ty) = (b.0 - a.0, b.1 - a.1);
        let l = tx.hypot(ty).max(1e-3);
        let (nx, ny) = (-ty / l, tx / l);
        (a.0 + tx * f + nx * 7.0, a.1 + ty * f + ny * 7.0, ty.atan2(tx))
    }).collect()
}
/// The layout at 1440 × 900.
const WIDE: Lay = Lay { phone: false, cut: false, w: 1440.0, h: 900.0, s: 1.0, oy: 0.0 };
/// The desktop's pitch of the line's cells (px along it at rest at 1440 × 900).
fn pitch() -> f32 {
    curve(&WIDE, 0.0, FIRST_BUILD).1.last().copied().unwrap_or(1.0) / CELLS as f32
}

/// Where the reflection runs across (V.2 f): from 24 px right of the title block (in the rows beside it)
/// to 94 % of the width.
pub fn reflection_span(l: &Lay, title: [f32; 4], y: f32) -> (f32, f32) {
    let beside = y > title[1] - 24.0 && y < title[3] + 24.0;
    (if beside { (title[2] + 24.0).max(l.x(0.0)) } else { l.x(0.0) }, l.w * TO)
}
/// The curtain's broken reflection under the horizon: rows of short dashes where the rest pose's rays
/// stand tallest, fainter further down; each press shifts the rows once. None by the strip.
fn reflection(l: &Lay, shift: u32, title: [f32; 4], strip: [f32; 4], view: (f32, f32, f32, f32)) -> El<Msg> {
    let hz = l.hz();
    let mut d = Drawing::new(l.w, l.h);
    palette(&mut d);
    let k = l.s;
    for row in 0..20u32 {
        let y = (hz + (30.0 + row as f32 * 4.0) * k).floor() + 0.5;
        let (x0, x1) = reflection_span(l, title, y);
        let fade = 1.0 - row as f32 / 20.0;
        let mut x = x0 + hash(row, shift, 4) * 6.0 * k;
        while x < x1 {
            let a = l.along(x);
            let v = (length(a, 0.0, FIRST_BUILD) - SHORT) / (LONG - SHORT);
            let len = ((2.0 + 5.0 * hash(a as u32, row, 5)) * k).min(x1 - x);
            let seg = [x, y - 0.5, x + len, y + 0.5];
            // None by the strip, and none across the view's edge (a mark in its own colours lies wholly
            // over the picture as shown).
            let out = (seg[0] < strip[2] && strip[0] < seg[2] && seg[1] < strip[3] && strip[1] < seg[3]) || seg[0] < view.0 + 1.0 || seg[2] > view.2 - 1.0;
            if hash(a as u32, row + 40 * shift, 6) < (0.2 + 0.8 * v) * fade * 1.4 && !out && len > 0.5 {
                d.line((seg[0], y), (seg[2], y), 1.0, Paint { tone: 1.0, accent: 1.0, alpha: 0.2 + 0.3 * v * fade, hue: 4 });
            }
            x += len + (3.0 + 6.0 * hash(a as u32, row, 7)) * k;
        }
    }
    canvas(d).key("work/nightly/reflection").width(l.w).absolute(Some(0.0), Some(0.0), None, None).pass(true)
}

/// The small line: the counts as built at this layout (V.2 e, j).
pub fn micro(l: &Lay, build: u32) -> String { format!("W-04 · 23:10 · {} GLYPHS, {} RAYS · PRESS TO RUN THE NEXT STEP", glyphs(l, build), l.rays()) }

/// A press at `x` in the piece: the next step of the build, a pulse from the nearest point of the line
/// both ways; the reflection shifts its rows once. After step 7 the curtain flares, then the build passes:
/// the callout says `PASSING`, the number rises by one and the new curtain comes in. With reduced motion
/// the step is counted and the reflection shifts, with no pulse and no flare (V.2 h).
pub fn press(s: &mut Gallery, x: f32, l: &Lay) {
    let (now, still) = (s.clock, s.body.reduced);
    let n = &mut s.works.night;
    let c = curve(l, n.run, n.build);
    let along = on_line(&c, x.clamp(c.0[0].0, c.0[c.0.len() - 1].0)).1;
    n.shift += 1;
    n.step += 1;
    if still {
        if n.step == STEPS { pass(n, now); }
        return;
    }
    n.pulse = Some((along, now));
    // The flare comes once the pulse has run the whole line.
    if n.step == STEPS { n.flare = now + along.max(c.1[c.1.len() - 1] - along) / PULSE; }
}
/// The build passes.
fn pass(n: &mut Night, now: f32) { n.step = 0; n.build += 1; n.passed = now; }

/// The pointer over the piece (its own space), or gone.
pub fn point(s: &mut Gallery, p: Option<(f32, f32)>) { s.works.night.pointer = p; }

/// A tick while W-04 runs: its motion steps at 30 frames a second while it is awake (not held, resting
/// or still), and the fold nearest the pointer follows it on a spring (at once with reduced motion). Whether the view changed.
pub fn tick(s: &mut Gallery, t: f32, dt: f32, l: &Lay, awake: bool) -> bool {
    let still = s.body.reduced;
    let n = &mut s.works.night;
    let mut changed = t - n.passed <= SWAP + 0.02;
    if n.step == STEPS && t >= n.flare + LIT { pass(n, t); changed = true; }
    if awake {
        let since = if n.stepped < 0.0 { 1.0 / PACE } else { t - n.stepped };
        if since >= 1.0 / PACE - 1e-3 { n.run += since.min(0.1); n.stepped = t; changed = true; }
    } else { n.stepped = -1.0; }
    // The fold: the tallest ray within 150 px of the pointer; it leans toward the pointer.
    let target = match n.pointer {
        Some((px, _)) => {
            let (run, build) = (n.run, n.build);
            let best = (0..l.rays()).map(|i| l.ray(i)).filter(|&x| (x - px).abs() < 150.0).max_by(|a, b| length(l.along(*a), run, build).total_cmp(&length(l.along(*b), run, build)));
            if let Some(x) = best { n.fold = x; }
            if best.is_some() { (px - n.fold).clamp(-LEAN, LEAN) } else { 0.0 }
        }
        None => 0.0,
    };
    n.aim = target;
    // With reduced motion the fold takes its lean at once.
    if still { if n.lean != target { n.lean = target; n.lean_v = 0.0; changed = true; } }
    else if (target - n.lean).abs() > 0.05 || n.lean_v.abs() > 0.05 {
        // A spring a little under critical, about 0.4 s to settle.
        let (wn, z) = (10.0, 0.8);
        let a = wn * wn * (target - n.lean) - 2.0 * z * wn * n.lean_v;
        n.lean_v += a * dt;
        n.lean += n.lean_v * dt;
        changed = true;
    } else if n.lean != target { n.lean = target; n.lean_v = 0.0; changed = true; }
    changed
}
/// When W-04 next wants a frame while it runs.
pub fn wake(s: &Gallery, t: f32, awake: bool) -> Option<f32> {
    let n = &s.works.night;
    let spring = (n.aim - n.lean).abs() > 0.05 || n.lean_v.abs() > 0.05;
    if spring || t - n.passed < SWAP { return Some(t); }
    if n.step == STEPS { return Some(if awake { n.stepped + 1.0 / PACE } else { n.flare + LIT }); }
    awake.then_some(n.stepped + 1.0 / PACE)
}
