//! Body (the home page, `/`): a slowly turning body whose surface is the text of this file, set in
//! lines round the form like latitude rings, lit by a lamp that follows the pointer. Written only
//! against amenui's public API: text elements whose glyphs are posed (`El::pose`), drawings for the
//! dots behind and over it, springs for the motion.
//!
//! - Two rest states: the source listing (01) and the body (04), joined by one spring (`gather`).
//! - The text is the first 2,700 characters of this file, ending at a line end (1,400 on a small body).
//! - The listing fills a stage rectangle the host gives (`set_stage`): 14 px lines, as many of the
//!   text's lines as fit above the caption, each cut at the last whole word that fits. The web page
//!   draws the same listing in HTML while the module loads, at the same pixels.
//! - Glyph i of the text sits in one cell of a spiral on the surface; spaces take a cell. The glyph
//!   size is the one at which the text fills the surface.
//! - Glyphs are one size in body space; only perspective changes their size on screen.
//! - The lamp is a point light; one continuous ramp from dim accent to ink, no bands: opacity keeps the
//!   text legible everywhere, the colour carries the light.
//! - Dragging turns the body; a click sends a ring round. Two readouts name what is under the lamp
//!   and how far the body has turned.
use amenui::draw::Drawing;
use amenui::el::{El, Font, GlyphAt, Key, Paint, Point, Pose, Reveal, canvas, layers, text_lh};
use amenui::experiment::Experiment;
use amenui::field::hash;
use amenui::form::FormFx;
use amenui::gesture::{Animated, Spring, Velocity};
use amenui::widgets::Edit;
use std::cell::RefCell;
use std::rc::Rc;

/// This file: the text on the body.
pub const SOURCE: &str = include_str!("body.rs");
/// The listing's size (Departure Mono's native 11 px) and that face's advance at it (350 / 550 em).
pub const SIZE: f32 = 11.0;
pub const ADVANCE: f32 = 7.0;
/// Marks in the body's volume (the far shell and the inside), asked of the frame budget.
pub const VOLUME: usize = 1_500;
pub const MOTE_LIFE: f32 = 1.6;
/// How much of this file the body is made of (characters, spaces included, cut at a line end), and
/// on a small body (the phone).
pub const CHARS: usize = 2_700;
pub const CHARS_SMALL: usize = 1_400;
/// The listing's line height and its line numbers' gutter (three digits and a space) in px. The line
/// height never changes: a shorter stage shows fewer lines.
pub const LINE: f32 = 14.0;
/// The gap between the listing's last line and the caption (px).
pub const CAPTION_GAP: f32 = 16.0;
pub const GUTTER: f32 = 28.0;
/// The short path: after the first frame the listing waits this long (s), then gathers.
pub const INTRO_DELAY: f32 = 0.5;
/// On the short path the front across the listing starts this long after the first frame (s), from
/// the caret, at this speed (px/s).
pub const FRONT_DELAY: f32 = 0.15;
pub const FRONT_SPEED: f32 = 1_400.0;
/// The widest the home page lays out (px); a wider window centres it.
pub const FRAME: f32 = 1_600.0;
/// With reduced motion a change of state is an opacity change this long (s): out, then in. The first
/// one (the listing giving way to the body) is a little longer.
pub const FADE: f32 = 0.15;
pub const FADE_INTRO: f32 = 0.2;
pub const REST_LIGHT: [f32; 3] = [-0.9, 0.9, 1.6];
/// The surface round the point under the pointer rises (within this distance, body units) and is lit
/// more; a click's ring crosses the surface at this speed (units/s), this wide, lifting what it passes.
pub const BULGE: f32 = 0.35;
pub const RING_SPEED: f32 = 1.6;
pub const RING_WIDTH: f32 = 0.12;
/// A flick slows with this time constant (s); the idle turn comes back this long after the last
/// input (s), easing in over a second; an arrow key turns the body this far (rad).
pub const FLICK_TAU: f32 = 1.2;
pub const IDLE_AFTER: f32 = 2.0;
pub const KEY_TURN: f32 = 0.6;
/// The specimen of states 02 and 03 (px; on a phone narrower, and without the Pitch row), and the
/// materials its segmented control offers.
pub const SPECIMEN: (f32, f32) = (360.0, 268.0);
pub const SPECIMEN_PHONE_H: f32 = 204.0;
pub const SPEC_MATERIALS: [(&str, amenui::style::Material); 4] = {
    use amenui::style::Material::{Dot, Lino, Near, Plain};
    [("plain", Plain), ("near", Near), ("dot", Dot), ("lino", Lino)]
};
/// The intro's morphs (loosen, flight, tighten in ms) and the short one of a change at rest or a skip.
pub const MORPH_INTRO: (f32, f32, f32) = (220.0, 900.0, 260.0);
pub const MORPH_SHORT: (f32, f32, f32) = (160.0, 560.0, 220.0);
/// The intro's version: the page remembers it once the landing state is reached or the intro skipped.
pub const INTRO_VERSION: u32 = 1;
/// How the text is drawn on the form: nothing turned away, glyphs fading out from facing 0.5 (gone at
/// 0.3), a glyph more than a third covered by nearer ones left out, the far side dimmer.
pub const FORM_FX: FormFx = FormFx { back: 0.0, edge: 0.5, overlap: 0.35, fog: 0.6 };

/// One cell of the surface: where (unit body space), its outward normal, the line's tangent there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfacePoint { pub p: [f32; 3], pub n: [f32; 3], pub t: [f32; 3], pub k: f32 }
/// A mark of the volume: where (unit space), its normal-ish direction, its size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VolumePoint { pub p: [f32; 3], pub n: [f32; 3], pub size: f32 }

/// The text on the body: the listing (the whole lines of `SOURCE` in its first `chars` characters)
/// and, for each of its glyphs (characters that are not white space), the cell it sits in. A run of
/// white space (indentation, a line end) takes one cell; `cells` is how many the text takes. `mid`
/// gives each glyph the middle glyph of its word.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stream { pub listing: String, pub lines: usize, pub cell: Vec<usize>, pub cells: usize, pub mid: Vec<usize> }
pub fn stream(chars: usize) -> Stream {
    let (mut listing, mut cell, mut at, mut lines) = (String::new(), vec![], 0usize, 0usize);
    for line in SOURCE.lines() {
        let line = line.trim_end();
        if listing.chars().count() + (lines > 0) as usize + line.chars().count() > chars { break; }
        let mut add = vec![];
        let mut k = at;
        let mut gap = lines > 0;
        for c in line.chars() {
            if c.is_whitespace() { gap = true; continue; }
            if gap && k > 0 { k += 1; }
            gap = false;
            add.push(k);
            k += 1;
        }
        if lines > 0 { listing.push('\n'); }
        listing.push_str(line);
        cell.extend(add);
        at = k;
        lines += 1;
    }
    // Words: runs of glyphs in consecutive cells.
    let mut mid = vec![0; cell.len()];
    let mut a = 0;
    for i in 1..=cell.len() {
        if i == cell.len() || cell[i] != cell[i - 1] + 1 { for m in &mut mid[a..i] { *m = (a + i - 1) / 2; } a = i; }
    }
    Stream { listing, lines, cell, cells: at, mid }
}


/// A line cut to `cols` cells: whole if it fits, otherwise at the end of the last whole word that
/// fits (never inside a word), without trailing space.
pub fn cut(line: &str, cols: usize) -> &str {
    let line = line.trim_end();
    let Some((end, _)) = line.char_indices().nth(cols) else { return line };
    let head = &line[..end];
    if line[end..].starts_with(char::is_whitespace) { return head.trim_end(); }
    match head.rfind(char::is_whitespace) { Some(b) => head[..b].trim_end(), None => "" }
}

/// The listing as a stage shows it: its first `rows` lines, each cut to `cols` cells (`text`), the
/// stream's glyphs it shows (`map`) and those it leaves out (`rest_map`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shown { pub text: String, pub rows: usize, pub map: Vec<usize>, pub rest_map: Vec<usize> }
pub fn shown(s: &Stream, rows: usize, cols: usize) -> Shown {
    let mut out = Shown::default();
    let mut g = 0;
    for (l, line) in s.listing.split('\n').enumerate() {
        let keep = if l < rows { cut(line, cols).len() } else { 0 };
        if l < rows {
            if l > 0 { out.text.push('\n'); }
            out.text.push_str(&line[..keep]);
            out.rows += 1;
        }
        for (b, c) in line.char_indices() {
            if c.is_whitespace() { continue; }
            if b < keep { out.map.push(g); } else { out.rest_map.push(g); }
            g += 1;
        }
    }
    out
}
/// A number with thousands separated by commas (2,694).
pub fn thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() { if i > 0 && (s.len() - i).is_multiple_of(3) { out.push(','); } out.push(c); }
    out
}
/// What the poses and the drawings share across frames: the cells for (form, seed, glyph size), the
/// volume for (form, seed, count), the text for the cell count.
#[derive(Default)]
pub struct Cache { pub key: (u32, u32, u32), pub pts: Vec<SurfacePoint>, pub radius: f32, pub em: f32, pub vkey: (u32, u32, usize), pub vol: Vec<VolumePoint>, pub text: Stream }

/// A mote shed from the rim while the body turns: where (window px), velocity, age (s).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mote { pub x: f32, pub y: f32, pub vx: f32, pub vy: f32, pub age: f32 }

/// How the window is laid out round the text block: wide (≥ 1080 px), middle, narrow (< 720 px).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width { Wide, Middle, Narrow }

#[derive(Clone)]
pub struct Body {
    /// Form (0 rock, 1 cloud) and seed.
    pub form: u32, pub seed: u32,
    /// The idle turn (rad/s).
    pub omega: f32,
    /// The spring between the listing (0) and the body (1).
    pub gather: Animated,
    /// The turn, and its free rates (rad/s: a flick's or an arrow key's, slowing by `FLICK_TAU`); the
    /// idle turn's share (0..1) is added on top, and the time of the last input it waits for.
    pub yaw: f32, pub pitch: f32, pub spin: f32, pub tip: f32, pub idle: f32, pub last_input: f32,
    /// The pointer over the picture (window px) and the lamp following it (view space).
    pub ptr: Option<(f32, f32)>, pub lamp: [Animated; 3],
    /// The glyph of the text drawn under the pointer (its number among the text's glyphs).
    pub under: Option<usize>,
    /// The pointer is over the body; the point under it (body space) and the bulge's strength; the
    /// rings sent by clicks (body-space origin, start).
    pub over: bool, pub touch_at: Option<[f32; 3]>, pub bulge: Animated, pub rings: Vec<([f32; 3], f32)>,
    grab: Option<Grab>, vel: Velocity, dragged: bool,
    pub motes: Vec<Mote>, made: u32,
    /// The window (logical px), and the stage (x0, y0, x1, y1): the rectangle the host's page lays
    /// out for the scene, which the listing fills and the body and its readouts stay in.
    pub size: (f32, f32), pub stage: Option<[f32; 4]>,
    /// The optical centre of the page's text block (window px, vertical), which the body's centre is
    /// level with; none for where the site's grid puts it.
    pub text_centre: Option<f32>,
    pub entered: f32,
    pub reduced: bool,
    /// The short path is to play: waiting for the first frame (`intro_pending`), then at a time.
    pub intro_pending: bool, pub intro_at: Option<f32>,
    /// The front across the listing on the short path: when it is to start, the place it starts from
    /// (window px, for the host to send as a front, `take_front`), and when it started.
    pub front_at: Option<f32>, pub front: Option<(f32, f32)>, pub fronted: Option<f32>,
    /// The state (1 source, 2 interface, 3 material, 4 body), the one before it, and when the morph
    /// between them lands (the body's drawings wait for it).
    pub at: u8, pub was: u8, pub lands: f32,
    /// The specimen's controls.
    pub spec: Specimen,
    /// The intro (the full walk): when it started and the steps still to come; a person took over.
    pub walk: Option<(f32, Vec<(f32, Step)>)>, pub taken: bool,
    /// The first frame plays the full walk (else the short path); the walk has been seen (the page
    /// remembers it); steps at a time outside the walk; the gather's value to jump to once a morph lands.
    pub full: bool, pub seen: bool, pub later: Vec<(f32, Step)>, pub settle: Option<(f32, f32)>,
    /// What the scene asks the runtime to do (morphs, presses, focus, fronts), for the host to send.
    pub cmds: Vec<amenui::morph::Cmd>,
    /// With reduced motion, a change of state under way: (start, target, length).
    pub fade: Option<(f32, f32, f32)>,
    /// When the state last changed (the caption's dimming counts from it).
    pub changed: f32,
    pub t: f32,
    cache: Rc<RefCell<Cache>>,
}
/// A press on the body: where (window px), the turn then, and the point under it (body space) with its
/// distance from the centre (the arcball's radius), none when it was off the body.
#[derive(Clone, Copy, Debug)]
struct Grab { x: f32, y: f32, yaw: f32, pitch: f32, on: Option<([f32; 3], f32)> }
/// The lamp's spring: it trails the hand a little.
const LAMP: Spring = Spring { stiffness: 90.0, damping: 18.0, rest_distance: 0.001, rest_speed: 0.005, mass: 1.0 };
/// The gather spring: about 0.9 s from the listing to the body.
const GATHER: Spring = Spring { stiffness: 30.0, damping: 11.0, rest_distance: 0.002, rest_speed: 0.01, mass: 1.0 };
impl Default for Body {
    fn default() -> Self {
        let lamp = |v: f32| Animated::new(v, Spring { ..LAMP });
        Self {
            form: 0, seed: DEFAULT_SEED, omega: 0.16, gather: Animated::new(1.0, Spring { ..GATHER }), yaw: -0.4, pitch: -0.22, spin: 0.0, tip: 0.0, idle: 1.0, last_input: f32::NEG_INFINITY,
            ptr: None, under: None, over: false, touch_at: None, bulge: Animated::new(0.0, Spring { ..LAMP }), rings: vec![], lamp: [lamp(REST_LIGHT[0]), lamp(REST_LIGHT[1]), lamp(REST_LIGHT[2])],
            grab: None, vel: Velocity::default(), dragged: false, motes: vec![], made: 0,
            at: 4, was: 4, lands: 0.0, spec: Specimen::default(), walk: None, taken: false, cmds: vec![], full: false, seen: false, later: vec![], settle: None,
            size: (1440.0, 900.0), stage: None, text_centre: None, entered: 0.0, reduced: false, intro_pending: false, intro_at: None, front_at: None, front: None, fronted: None, fade: None, changed: 0.0,
            t: 0.0, cache: Rc::default(),
        }
    }
}
/// How much larger than its mean radius the body's silhouette is on screen (perspective, relief).
pub const SILHOUETTE: f32 = 1.12;
/// The seed whose largest facet faces the viewer at the resting turn (`best_seed`, checked by eye).
pub const DEFAULT_SEED: u32 = 26;

/// `Show(n)` state n (the rail's states); `Seed(d)` steps the seed; `Spec` a specimen control; `Play`
/// the walk from the current state to the body (from the source when at the body); `Skip` to the body.
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Key(Key), Form, Seed(i32), Gather, Show(u8), Num(usize, Edit), Spec(SpecMsg), Play, Skip }
/// The specimen's controls: Loosen, Pitch, Material.
#[derive(Clone, Debug, PartialEq)]
pub enum SpecMsg { Loosen, Pitch(f32), Material(usize) }
/// The specimen's state: its looseness held at 1, the lattice pitch (px), the material (of `SPEC_MATERIALS`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Specimen { pub loosen: bool, pub pitch: f32, pub material: usize }
impl Default for Specimen { fn default() -> Self { Self { loosen: false, pitch: 5.0, material: 0 } } }
/// One step of the intro.
#[derive(Clone, Debug, PartialEq)]
pub enum Step { State(u8), Focus(&'static str), Press(&'static str), Lamp, Idle, End }

/// The key the scene is drawn under (its morphs and the operator's presses name its parts by it).
pub const KEY: &str = "body";
/// The scene's words: English whatever the interface language.
fn tr(key: &str) -> String { crate::install_catalogs(); amenui::i18n::Locale::En.tr(key) }


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
/// The rock's five facets: (normal, offset) of each cutting plane.
pub fn facets(seed: u32) -> [([f32; 3], f32); 5] {
    std::array::from_fn(|f| {
        let f = f as u32;
        let a = |k: u32| hash(seed.wrapping_mul(977) ^ (f * 5 + k)) * 2.0 - 1.0;
        (norm([a(0), a(1) * 0.7, a(2)]), 0.62 + 0.12 * (a(3) * 0.5 + 0.5))
    })
}
/// The signed distance to the form (unit space, about radius 0.86): 0 a rock, 1 a cloud.
pub fn sdf(form: u32, seed: u32, p: [f32; 3]) -> f32 {
    let len = dot3(p, p).sqrt();
    if form == 0 {
        // Low-frequency relief, a middle octave, and five flat facets cut across it (planes).
        let q = [p[0] * 1.15 + 11.0, p[1] * 1.15 + 3.0, p[2] * 1.15 + 7.0];
        let d = 0.18 * noise3(q[0], q[1], q[2], seed) + 0.10 * noise3(q[0] * 2.4, q[1] * 2.4, q[2] * 2.4, seed ^ 0x55);
        let mut r = len - 0.86 - d;
        for (nrm, off) in facets(seed) { r = r.max(dot3(p, nrm) - off); }
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
fn norm(v: [f32; 3]) -> [f32; 3] { let l = dot3(v, v).sqrt().max(1e-6); [v[0] / l, v[1] / l, v[2] / l] }
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn gradient(form: u32, seed: u32, p: [f32; 3]) -> [f32; 3] {
    let e = 1e-3;
    let g = |a: usize| { let mut q = p; q[a] += e; let mut w = p; w[a] -= e; sdf(form, seed, q) - sdf(form, seed, w) };
    norm([g(0), g(1), g(2)])
}

/// Where the form's surface is along `dir`, marched in from radius 2, with its normal.
fn on_surface(form: u32, seed: u32, dir: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let mut r = 2.0f32;
    for _ in 0..56 {
        let d = sdf(form, seed, [dir[0] * r, dir[1] * r, dir[2] * r]);
        if d.abs() < 1e-3 { break; }
        r = (r - d * 0.85).max(0.05);
    }
    let p = [dir[0] * r, dir[1] * r, dir[2] * r];
    (p, gradient(form, seed, p))
}
/// The form's mean radius (unit space), from a few directions.
pub fn mean_radius(form: u32, seed: u32) -> f32 {
    let n = 48;
    (0..n).map(|i| {
        let z = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
        let a = i as f32 * 2.399_963;
        let s = (1.0 - z * z).sqrt();
        let (p, _) = on_surface(form, seed, [s * a.cos(), z, s * a.sin()]);
        dot3(p, p).sqrt()
    }).sum::<f32>() / n as f32
}

/// How many cells the spiral of `cells` has for a step and a pitch (no surface needed: the spiral is
/// laid on the sphere of directions).
pub fn spiral_count(step: f32, pitch: f32) -> usize {
    let tau = std::f32::consts::TAU;
    let (mut th, mut n) = (pitch * 0.5, 0);
    while th < std::f32::consts::PI - pitch * 0.25 { n += 1; th += step / (1.0 + (tau / pitch * th.sin()).powi(2)).sqrt(); }
    n
}
/// The largest glyph size (units of the mean radius) whose cells (0.62 × 1.45 em) hold `cells` cells:
/// the text fills the surface.
pub fn em_to_fill(cells: usize) -> f32 {
    let (mut lo, mut hi) = (0.02f32, 0.25f32);
    for _ in 0..24 { let m = (lo + hi) * 0.5; if spiral_count(0.62 * m, 1.45 * m) >= cells { lo = m; } else { hi = m; } }
    lo
}

/// The cells: one spiral from the top to the bottom whose turns are `pitch` apart and whose cells are
/// `step` apart along it (both in units of the mean radius), so lines of text run round the body like
/// latitude rings. Each cell's tangent runs along the line.
pub fn cells(form: u32, seed: u32, step: f32, pitch: f32) -> Vec<SurfacePoint> {
    let tau = std::f32::consts::TAU;
    let mut th = pitch * 0.5;
    let mut out = vec![];
    while th < std::f32::consts::PI - pitch * 0.25 {
        let ph = -tau * th / pitch;
        let dir = [th.sin() * ph.cos(), th.cos(), th.sin() * ph.sin()];
        let (p, nrm) = on_surface(form, seed, dir);
        // Along increasing longitude: facing the viewer at rest, left to right.
        let t = norm(cross([0.0, 1.0, 0.0], nrm));
        let t = if t[0].is_finite() && (t[0].abs() + t[2].abs()) > 1e-3 { t } else { [1.0, 0.0, 0.0] };
        out.push(SurfacePoint { p, n: nrm, t, k: 1.0 });
        // The next cell one step further along the spiral: ds² = dθ² + sin²θ dφ², dφ = τ/pitch dθ.
        th += step / (1.0 + (tau / pitch * th.sin()).powi(2)).sqrt();
    }
    // The line's real direction and spacing on the form (a facet bends and draws the cells together):
    // the tangent from the neighbours, and how much smaller than planned the spacing is there.
    let mean = mean_radius(form, seed);
    for i in 1..out.len().saturating_sub(1) {
        let d = sub(out[i + 1].p, out[i - 1].p);
        let n = out[i].n;
        let along = sub(d, { let k = dot3(d, n); [n[0] * k, n[1] * k, n[2] * k] });
        if dot3(along, along) > 1e-10 { out[i].t = norm(along); }
        let gap = dot3(d, d).sqrt() * 0.5;
        let r = dot3(out[i].p, out[i].p).sqrt() / mean;
        out[i].k = (gap / (step * mean)).min(r).clamp(0.3, 1.0);
    }
    out
}

/// The volume: `n` marks, two thirds on the shell (seen through on the far side), a third inside.
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
        VolumePoint { p: [p[0] * depth, p[1] * depth, p[2] * depth], n: nrm, size: 0.55 + 0.8 * hash(i ^ 0x99) }
    }).collect()
}

/// The view of the body: its turn, where its centre is and its scale (px per unit).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View { pub yaw: f32, pub pitch: f32, pub c: (f32, f32), pub r: f32 }
impl View {
    pub fn rot(&self, v: [f32; 3]) -> [f32; 3] {
        let (s, k) = self.yaw.sin_cos();
        let (x, z) = (v[0] * k + v[2] * s, -v[0] * s + v[2] * k);
        let (s2, k2) = self.pitch.sin_cos();
        [x, v[1] * k2 - z * s2, v[1] * s2 + z * k2]
    }
    /// From view space back to body space.
    pub fn unrot(&self, v: [f32; 3]) -> [f32; 3] {
        let (s2, k2) = self.pitch.sin_cos();
        let (y, z) = (v[1] * k2 + v[2] * s2, -v[1] * s2 + v[2] * k2);
        let (s, k) = self.yaw.sin_cos();
        [v[0] * k - z * s, y, v[0] * s + z * k]
    }
    /// Screen place (window px), perspective scale; a camera 4 units away.
    pub fn project(&self, p: [f32; 3]) -> ([f32; 2], f32) { let k = 4.0 / (4.0 - p[2]); ([self.c.0 + self.r * p[0] * k, self.c.1 - self.r * p[1] * k], k) }
}

/// How lit a surface point is (0..1) under a point light at `l` (all in view space).
pub fn lit(l: [f32; 3], p: [f32; 3], n: [f32; 3]) -> f32 {
    let to = sub(l, p);
    let d2 = dot3(to, to);
    let lam = dot3(n, norm(to)).max(0.0);
    let wrap = 0.5 * lam + 0.5 * lam * lam;
    (0.28 + 2.0 * wrap / (1.0 + 0.6 * d2)).clamp(0.0, 1.0)
}
/// The paint of a glyph lit `l`: the accent up to 0.5, then towards the ink (85 % ink at 1). The
/// opacity has a high floor and a short range: it keeps the text legible, the colour shows the light.
pub fn lit_paint(l: f32) -> Paint {
    let accent = if l <= 0.5 { 1.0 } else { 1.0 - 0.85 * (l - 0.5) / 0.5 };
    Paint { tone: 1.0, accent, alpha: 0.70 + 0.30 * l, hue: 0 }
}

/// How a cell's glyph looks on the body: its lit paint, how it faces the viewer (the camera is 4
/// units in front), its opacity as drawn (the paint's, faded by `FORM_FX` for its facing), and how much
/// the line's tangent (`run`) and the line's height across it (`gap`) are foreshortened on screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellLook { pub paint: Paint, pub alpha: f32, pub run: f32, pub gap: f32, pub facing: f32 }
pub fn cell_look(v: &View, light: [f32; 3], sp: &SurfacePoint, boost: f32) -> CellLook {
    let (p, n, t) = (v.rot(sp.p), v.rot(sp.n), v.rot(sp.t));
    let paint = lit_paint((lit(light, p, n) + boost).min(1.0));
    let run = (t[0] * t[0] + t[1] * t[1]).sqrt();
    // Across the line: the part of the projected bitangent square to the projected line.
    let b = cross(n, t);
    let gap = ((b[0] * t[1] - b[1] * t[0]) / run.max(1e-4)).abs();
    let facing = dot3(n, norm(sub([0.0, 0.0, 4.0], p)));
    let alpha = paint.alpha * FORM_FX.facing_alpha(facing);
    CellLook { paint, alpha, run, gap, facing }
}

/// What the hand does to the surface: the bulge round the point under the pointer (body space, with
/// its strength) and the rings of clicks (body-space origin, radius now).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Touch { pub at: Option<[f32; 3]>, pub k: f32, pub rings: Vec<([f32; 3], f32)> }
impl Touch {
    /// How far a surface point (body space) is pushed out along its normal, and how much more lit it is.
    pub fn push(&self, p: [f32; 3]) -> (f32, f32) {
        let (mut out, mut boost) = (0.0, 0.0);
        if let Some(h) = self.at {
            let g = dot3(sub(p, h), sub(p, h)).sqrt();
            if g < BULGE { let f = 1.0 - g / BULGE; out += 0.07 * f * f * self.k; boost += 0.25 * f * self.k; }
        }
        for &(o, r) in &self.rings {
            let x = (dot3(sub(p, o), sub(p, o)).sqrt() - r) / RING_WIDTH;
            if x.abs() < 1.0 { out += 0.05 * 0.5 * (1.0 + (std::f32::consts::PI * x).cos()); }
        }
        (out, boost)
    }
    /// The surface point pushed out.
    pub fn moved(&self, sp: &SurfacePoint) -> (SurfacePoint, f32) {
        let (out, boost) = self.push(sp.p);
        (SurfacePoint { p: [sp.p[0] + sp.n[0] * out, sp.p[1] + sp.n[1] * out, sp.p[2] + sp.n[2] * out], ..*sp }, boost)
    }
}

/// How the word of glyph `gi` faces the viewer: as its middle cell does.
fn word_facing(v: &View, c: &Cache, gi: usize) -> f32 {
    let m = c.text.mid.get(gi).copied().unwrap_or(gi);
    c.text.cell.get(m).and_then(|&k| c.pts.get(k)).map_or(0.0, |sp| { let (p, n) = (v.rot(sp.p), v.rot(sp.n)); dot3(n, norm(sub([0.0, 0.0, 4.0], p))) })
}

/// How far glyph `i` of `n` has gone at gather `g`: reading order leads; exactly 0 at 0 and 1 at 1.
pub fn progress(g: f32, i: usize, n: usize) -> f32 {
    let st = 0.55;
    let u = (g * (1.0 + st) - st * i as f32 / n.max(1) as f32).clamp(0.0, 1.0);
    u * u * u * (u * (u * 6.0 - 15.0) + 10.0)
}

/// The glyph size on the body in units of the mean radius, and whether it is the small-window case
/// (fewer, larger glyphs: below about 116 px of radius the normal size would be under 11 px on screen).
pub fn em_for(radius_px: f32) -> (f32, bool) { if em_px(radius_px, 1.0 / 14.0) * 1.33 < SIZE { (1.0 / 10.0, true) } else { (1.0 / 14.0, false) } }
/// A glyph's size on screen at the body's centre plane (px) for a silhouette radius and a size in
/// units of the mean radius.
pub fn em_px(radius_px: f32, em: f32) -> f32 { radius_px / SILHOUETTE * em }

impl Body {
    /// At rest in the listing.
    pub fn flat(&self) -> bool { self.gather.value == 0.0 && !self.gather.moving() && self.fade.is_none() }
    /// The state it is going to: 0 the listing, 1 the body.
    pub fn target(&self) -> f32 { self.fade.map_or(self.gather.target, |f| f.1) }
    /// The host's window (logical px).
    pub fn resize(&mut self, w: f32, h: f32) { self.size = (w.max(1.0), h.max(1.0)); }
    /// The stage the host laid out (window px), or none for the default one.
    pub fn set_stage(&mut self, r: Option<[f32; 4]>) { self.stage = r.filter(|r| r[2] > r[0] + 1.0 && r[3] > r[1] + 1.0); }
    /// The optical centre of the page's text block (window px, vertical), or none for the site's grid.
    pub fn set_text_centre(&mut self, y: Option<f32>) { self.text_centre = y; }
    /// The pointer's place as a fraction of the window from its centre (-0.5..0.5): the lamp goes there.
    pub fn look_at(&mut self, x: f32, y: f32) { let (w, h) = self.size; self.point_lamp(Some(((x + 0.5) * w, (y + 0.5) * h))); }
    /// To the listing (0) or the body (1): by the spring, or with reduced motion by an opacity change.
    pub fn go(&mut self, to: f32, now: f32, fade: f32) {
        self.intro_pending = false;
        self.intro_at = None;
        let st = if to >= 0.5 { 4 } else { 1 };
        if st != self.at { (self.was, self.at) = (self.at, st); }
        if (self.target() - to).abs() < 1e-6 { return; }
        self.changed = now;
        if self.reduced { self.fade = Some((now, to, fade)); } else { let v = self.gather.velocity; self.gather.to(to, v, now); }
    }
    fn set_now(&mut self, to: f32) {
        self.gather = Animated::new(to, Spring { ..GATHER });
        (self.fade, self.intro_pending, self.intro_at) = (None, false, None);
        let st = if to >= 0.5 { 4 } else { 1 };
        (self.at, self.was, self.lands, self.walk) = (st, st, 0.0, None);
    }
    /// Open a state from the page's address: `source`, `interface`, `material`, `body`,
    /// `body/cloud/08`, at rest; empty is the body with the default form and seed, reached by the
    /// short path (`walk` 1) or the full walk (`walk` 2) from the source, or at rest (0).
    pub fn open(&mut self, state: &str, walk: u8) {
        let mut parts = state.trim_start_matches('#').split('/');
        match parts.next().unwrap_or("") {
            "source" => self.set_now(0.0),
            s @ ("interface" | "material") => {
                self.set_now(0.0);
                (self.at, self.was) = (if s == "interface" { 2 } else { 3 }, 1);
                self.spec.material = if s == "material" { 2 } else { 0 };
            }
            "body" => {
                if let Some(f) = parts.next() { self.form = (f == "cloud") as u32; }
                if let Some(n) = parts.next().and_then(|n| n.parse::<u32>().ok()) { self.seed = n.min(99); }
                self.set_now(1.0);
            }
            _ => {
                (self.form, self.seed) = (0, DEFAULT_SEED);
                self.set_now(if walk > 0 { 0.0 } else { 1.0 });
                self.intro_pending = walk > 0;
                self.full = walk == 2;
            }
        }
        self.changed = self.t;
    }
    /// The state for the page's address (`open` reads it back): `source`, `interface`, `material`,
    /// `body/cloud/08`, or empty for the body with the default form and seed.
    pub fn state(&self) -> String {
        if self.walk.is_some() || self.intro_pending || self.intro_at.is_some() { return String::new(); }
        match self.at {
            1 => return "source".into(),
            2 => return "interface".into(),
            3 => return "material".into(),
            _ => {}
        }
        if self.form == 0 && self.seed == DEFAULT_SEED { return String::new(); }
        format!("body/{}/{:02}", if self.form == 0 { "rock" } else { "cloud" }, self.seed)
    }
    /// The state on the stage now: during a reduced-motion change, the one before it until half way.
    pub fn shown_state(&self) -> u8 { match self.fade { Some((t0, _, d)) if self.t - t0 < d * 0.5 => self.was, _ => self.at } }
    /// The walk: 0 none, 1 playing, 2 stopped by a person (`continue ›`).
    pub fn walking(&self) -> u8 { if self.walk.is_some() { 1 } else if self.taken { 2 } else { 0 } }
    /// The morph between the text and the specimen: the lattice is the specimen's pitch.
    fn morph(&mut self, from: &str, to: &str, m: (f32, f32, f32), now: f32) {
        let count_max = if self.width_class() == Width::Narrow { 8_000.0 } else { 24_000.0 };
        let spec = amenui::morph::MorphSpec { loosen_ms: m.0, duration_ms: m.1, tighten_ms: m.2, count_max, lattice: self.spec.pitch, ..Default::default() };
        self.cmds.push(amenui::morph::Cmd::Morph { from: format!("{KEY}/{from}"), to: format!("{KEY}/{to}"), spec });
        self.lands = now + (m.0 + m.1 + m.2) / 1000.0;
    }
    /// To state `to` (1 to 4): 01 ↔ 04 by the gather spring, to and from the specimen by a morph, 02 ↔
    /// 03 by a commit from the segmented control; with reduced motion an opacity change.
    pub fn go_to(&mut self, to: u8, now: f32, m: (f32, f32, f32)) {
        let to = to.clamp(1, 4);
        if (to == 1 || to == 4) && !matches!(self.at, 2 | 3) { return self.go(if to == 4 { 1.0 } else { 0.0 }, now, FADE); }
        (self.intro_pending, self.intro_at) = (false, None);
        let from = self.at;
        if to == from { return; }
        (self.was, self.at, self.changed) = (from, to, now);
        let card = |s: u8| s == 2 || s == 3;
        if self.reduced {
            // An opacity change; a material shows at once.
            self.fade = Some((now, if to == 4 { 1.0 } else { 0.0 }, FADE));
            if to == 3 && self.spec.material == 0 { self.spec.material = 2; }
            if to == 2 { self.spec.material = 0; }
            return;
        }
        match (card(from), card(to)) {
            (false, true) => {
                self.morph("text", "specimen", m, now);
                // The text leaves as it is drawn, then rests in the listing, unseen.
                self.settle = Some((self.lands, 0.0));
                if to == 3 { self.later.push((self.lands - 0.2, Step::Press("material:2"))); }
            }
            (true, false) => {
                self.gather = Animated::new(if to == 4 { 1.0 } else { 0.0 }, Spring { ..GATHER });
                // In the walk the body is held still while it lands, the lamp behind it (only the
                // silhouette's dots show) until the walk turns it on.
                if self.walk.is_some() && to == 4 {
                    self.lamp = [-0.2, 0.3, -2.5].map(|v| Animated::new(v, Spring { ..LAMP }));
                    (self.spin, self.tip, self.idle, self.last_input) = (0.0, 0.0, 0.0, f32::INFINITY);
                }
                if from == 3 { self.spec.material = 0; }
                self.morph("specimen", "text", m, now);
            }
            _ => self.cmds.push(amenui::morph::Cmd::Press { key: format!("{KEY}/specimen/material:{}", if to == 3 { 2 } else { 0 }) }),
        }
    }
    /// The full walk from the current state to the body (from the source when at the body).
    pub fn play(&mut self, now: f32) { self.walk_from(now, false) }
    /// The walk; `first`: from the first frame, on its own clock (0.7 s of the source first).
    fn walk_from(&mut self, now: f32, first: bool) {
        if self.reduced { return self.go_to(4, now, MORPH_SHORT); }
        let phone = self.width_class() == Width::Narrow;
        let mut steps: Vec<(f32, Step)> = vec![(0.70, Step::State(2)), (2.10, Step::Focus("material")), (2.50, Step::Press("material:1")), (3.10, Step::Press("material:2"))];
        if phone {
            steps.extend([(3.70, Step::Press("material:0")), (3.80, Step::Focus("gather")), (4.10, Step::Press("gather")), (5.30, Step::Lamp), (5.90, Step::Idle)]);
        } else {
            steps.extend([(3.70, Step::Press("material:3")), (4.30, Step::Press("material:0")), (4.40, Step::Focus("gather")), (4.70, Step::Press("gather")), (5.90, Step::Lamp), (6.50, Step::Idle)]);
        }
        steps.push((steps.last().map_or(0.0, |s| s.0), Step::End));
        // From where the walk is: the steps before the current state's beat are done.
        let skip = match self.at { _ if first => 0.0, 2 => 2.10, 3 => if phone { 3.80 } else { 4.40 }, _ => 0.55 };
        let mut t0 = now - skip;
        if self.at == 4 { self.go(0.0, now, FADE); t0 = now + 1.0 - 0.55; }
        steps.retain(|s| s.0 >= skip);
        (self.walk, self.taken) = (Some((t0, steps)), false);
    }
    /// A person acted during the walk: it stops where it is (`continue ›` plays on).
    pub fn heard(&mut self) -> bool {
        if self.walk.is_none() { return false; }
        (self.walk, self.taken) = (None, true);
        self.later.clear();
        true
    }
    /// `skip ›`, Esc or a scroll: straight to the body by the short morph (or the gather from the
    /// source), and the walk counts as seen.
    pub fn skip(&mut self, now: f32) {
        (self.walk, self.taken) = (None, false);
        self.later.clear();
        self.go_to(4, now, MORPH_SHORT);
        self.seen = true;
        self.idle = 1.0;
        self.last_input = f32::NEG_INFINITY;
    }
    fn do_step(&mut self, step: Step, now: f32) {
        match step {
            Step::State(n) => self.go_to(n, now, MORPH_INTRO),
            Step::Focus(k) => self.cmds.push(amenui::morph::Cmd::Focus { key: format!("{KEY}/specimen/{k}") }),
            Step::Press(k) => self.cmds.push(amenui::morph::Cmd::Press { key: format!("{KEY}/specimen/{k}") }),
            // The lamp comes on from behind the form, to its rest place or the pointer.
            Step::Lamp => self.point_lamp(self.ptr),
            Step::Idle => { self.idle = 0.0; self.last_input = now - IDLE_AFTER; }
            Step::End => { self.walk = None; self.seen = true; }
        }
    }
    /// How opaque everything is during a reduced-motion change of state (1 otherwise).
    pub fn dim(&self) -> f32 { self.fade.map_or(1.0, |(t0, _, d)| (1.0 - 2.0 * ((self.t - t0) / d).clamp(0.0, 1.0)).abs()) }
    /// The home page's frame: at most 1,600 px wide, centred (its left edge and its width).
    pub fn frame(&self) -> (f32, f32) { let fw = self.size.0.min(FRAME); ((self.size.0 - fw) * 0.5, fw) }
    pub fn width_class(&self) -> Width { let w = self.size.0; if w >= 1080.0 { Width::Wide } else if w >= 720.0 { Width::Middle } else { Width::Narrow } }
    /// Where the site's page puts its text block by its grid (for the default stage).
    pub fn text_box(&self) -> [f32; 4] {
        let (w, h) = self.size;
        match self.width_class() {
            Width::Wide => { let (fx, fw) = self.frame(); let col = (fw - 64.0 - 11.0 * 24.0) / 12.0; [fx + 32.0, h * 0.5 - 190.0, fx + 32.0 + (5.0 * col + 96.0).min(520.0), h * 0.5 + 190.0] }
            Width::Middle => { let col = (w - 48.0 - 11.0 * 24.0) / 12.0; [24.0, h * 0.5 - 200.0, 24.0 + (7.0 * col + 144.0).min(440.0), h * 0.5 + 200.0] }
            Width::Narrow => { let y = Self::narrow_stage_bottom(h) + 16.0 + 88.0 + 16.0; [16.0, y, w - 16.0, h - 16.0] }
        }
    }
    fn narrow_stage_bottom(h: f32) -> f32 { 64.0 + (h * 0.38).clamp(210.0, 17.0 * LINE) }
    /// The stage: the host's, or where the site's page puts it. Wide and middle: from 48 px right of
    /// the text block to the window's margin, 72 px from the top to 88 px above the bottom (the status
    /// line, the rail row and a gap). Narrow: inside 16 px margins from 64 px down.
    pub fn stage(&self) -> [f32; 4] {
        if let Some(s) = self.stage { return s; }
        let (w, h) = self.size;
        match self.width_class() {
            Width::Wide => { let (fx, fw) = self.frame(); [(self.text_box()[2] + 48.0).round(), 72.0, fx + fw - 32.0, h - 88.0] }
            Width::Middle => [(self.text_box()[2] + 48.0).round(), 72.0, w - 24.0, h - 88.0],
            Width::Narrow => [16.0, 64.0, w - 16.0, Self::narrow_stage_bottom(h)],
        }
    }
    /// The listing's lines and their width in cells: whole 7 px cells, and as many of the text's lines
    /// (whole, 14 px) as fit in the stage with the caption 16 px under the last.
    pub fn grid(&self) -> (usize, usize) {
        let s = self.stage();
        self.ensure();
        let lines = self.cache.borrow().text.lines.max(1);
        let rows = (((s[3] - s[1] - CAPTION_GAP - LINE) / LINE).floor().max(1.0) as usize).min(lines);
        let cols = ((s[2] - s[0] - GUTTER) / ADVANCE).floor().max(1.0) as usize;
        (rows, cols)
    }
    /// The caption's top (window px): 16 px under the listing's last line, in every state.
    pub fn caption_y(&self) -> f32 { self.stage()[1] + self.grid().0 as f32 * LINE + CAPTION_GAP }
    /// The caption's box (window px), as wide as its longest sentence.
    pub fn caption_box(&self) -> [f32; 4] { let (x, y) = (self.stage()[0], self.caption_y()); [x, y, x + GUTTER + ADVANCE * 44.0, y + LINE] }
    /// The optical centre of the text block, vertical (window px): the host's, or the middle of where
    /// the site's grid puts it.
    pub fn text_centre(&self) -> f32 { self.text_centre.unwrap_or_else(|| { let b = self.text_box(); (b[1] + b[3]) * 0.5 }) }
    /// Where the body is in the stage: centre (window px) and visible radius (px). Wide: centred
    /// across the stage, level with the text block's optical centre. Middle: the same height; the body
    /// may run off the right edge by a third of its radius. Narrow: centred in the stage above the
    /// caption. The readouts' room (a quarter of the radius each side) stays inside the stage, and the
    /// body's circle (1.05 of its radius) stays in the stage and off the caption.
    pub fn placement(&self) -> (f32, f32, f32) {
        let (w, h) = self.size;
        let s = self.stage();
        let cap = self.caption_box();
        let sw = s[2] - s[0];
        // The largest radius whose circle round (cx, cy) keeps off the caption.
        let off_caption = |cx: f32, cy: f32| { let (dx, dy) = ((cap[0] - cx).max(cx - cap[2]).max(0.0), (cap[1] - cy).max(cy - cap[3]).max(0.0)); dx.hypot(dy) / 1.05 };
        let (cx, cy, r) = match self.width_class() {
            Width::Wide => {
                let (cx, cy) = ((s[0] + s[2]) * 0.5, self.text_centre());
                let r = (0.34 * h).min(0.23 * self.frame().1).min(sw * 0.5 / 1.25).min((cy - s[1]) / 1.05).min((s[3] - cy) / 1.05).min(off_caption(cx, cy));
                (cx, cy, r)
            }
            Width::Middle => {
                let cy = self.text_centre();
                let mut r = (0.26 * w).min((w - s[0]) / (1.25 + 2.0 / 3.0)).min((cy - s[1]) / 1.05).min((s[3] - cy) / 1.05);
                // The centre moves with the radius: shrink until the circle is off the caption.
                for _ in 0..8 { r = r.min(off_caption(s[0] + 1.25 * r, cy)); }
                (s[0] + 1.25 * r, cy, r)
            }
            Width::Narrow => {
                let half = (cap[1] - 8.0 - s[1]) * 0.5;
                ((s[0] + s[2]) * 0.5, s[1] + half, (0.36 * w).min(half / 1.05))
            }
        };
        (cx, cy, r.max(24.0))
    }
    /// The camera now.
    pub fn view(&self) -> View {
        let (cx, cy, r) = self.placement();
        let mean = self.cache.borrow().radius;
        let mean = if mean > 0.0 { mean } else { 0.86 };
        // Perspective makes the near side larger than the mean radius: the silhouette is about `r`.
        View { yaw: self.yaw, pitch: self.pitch, c: (cx, cy), r: r / (mean * SILHOUETTE) }
    }
    /// The lamp now (view space).
    pub fn light(&self) -> [f32; 3] { [self.lamp[0].value, self.lamp[1].value, self.lamp[2].value] }
    /// Fill the cache for the current form, seed and glyph size.
    fn ensure(&self) {
        let small = self.small();
        let key = (self.form, self.seed, small as u32);
        let mut c = self.cache.borrow_mut();
        if c.key == key && !c.pts.is_empty() { return; }
        c.text = stream(if small { CHARS_SMALL } else { CHARS });
        c.em = em_to_fill(c.text.cells);
        c.pts = cells(self.form, self.seed, 0.62 * c.em, 1.45 * c.em);
        c.radius = mean_radius(self.form, self.seed);
        c.key = key;
    }
    /// The small-window case (fewer, larger glyphs). It depends on the stage, not on the body's radius,
    /// so the text (and with it the listing and the caption under it) is settled before the body is placed.
    pub fn small(&self) -> bool {
        let s = self.stage();
        let r = match self.width_class() { Width::Narrow => (0.36 * self.size.0).min((s[3] - s[1]) * 0.5), _ => (0.34 * self.size.1).min(0.23 * self.frame().1) };
        em_for(r).1
    }
    /// How many cells of the body the text takes (its glyphs and the spaces between words).
    pub fn glyphs(&self) -> usize { self.ensure(); self.cache.borrow().text.cells }
    /// The text on the body (whole lines of this file).
    pub fn listing(&self) -> String { self.ensure(); self.cache.borrow().text.listing.clone() }
    /// The listing as the stage shows it now.
    pub fn shown(&self) -> Shown { self.ensure(); let (rows, cols) = self.grid(); shown(&self.cache.borrow().text, rows, cols) }

    /// The pose of every glyph of the text: from its place in the listing to its cell on the body, with
    /// its depth and how it faces the viewer for `El::form`. `shown` says which glyphs the listing
    /// shows; the rest wait unseen where they would have been.
    fn poser(&self, origin: (f32, f32), shown: Rc<Vec<bool>>) -> impl Fn(GlyphAt) -> Pose + 'static {
        let (g, v, l, cache, dim, touch) = (self.gather.value.clamp(0.0, 1.0), self.view(), self.light(), self.cache.clone(), self.dim(), self.touch());
        let (_, _, rpx) = self.placement();
        let em_px = em_px(rpx, self.cache.borrow().em);
        move |a: GlyphAt| {
            let c = cache.borrow();
            let (fx, fy) = (a.x + origin.0, a.y + origin.1);
            let n = c.text.cell.len();
            let gi = if a.n == n { a.i } else { a.i * n / a.n.max(1) };
            let flat_alpha = if shown.get(gi).copied().unwrap_or(false) { dim } else { 0.0 };
            let Some(sp) = c.text.cell.get(gi).and_then(|&k| c.pts.get(k)) else { return Pose { alpha: flat_alpha, ..Pose::default() } };
            let e = progress(g, gi, n);
            if e <= 0.0 { return Pose { alpha: flat_alpha, ..Pose::default() }; }
            // Lifted along its normal by the hand (the bulge, a ring), and lit more near the pointer.
            let (moved, boost) = touch.moved(sp);
            let sp = &moved;
            let (p, t) = (v.rot(sp.p), v.rot(sp.t));
            let ([sx, sy], k) = v.project(p);
            let CellLook { paint, run, gap, .. } = cell_look(&v, l, sp, boost);
            // A word faces the viewer as its middle does, so it comes and goes whole by its middle
            // (by its end, a long word would leave a hole in the line).
            let facing = word_facing(&v, &c, gi);
            // Turned to the line's tangent and foreshortened as the surface is seen there; one size on
            // the body (cells are set onto the form along their direction, so where the form is nearer
            // the centre, on a facet, they are closer and the glyphs as much smaller).
            let ang = (-t[1]).atan2(t[0]);
            let s = em_px * k / SIZE * sp.k;
            // Never wider than the cell's real spacing on screen: where the line crosses a facet's edge
            // or runs towards the limb the cells are drawn together more than the tangent says. (The
            // nearer neighbour counts.)
            let cell = c.text.cell[gi];
            let near = |j: usize| c.pts.get(j).map(|q| { let o = v.project(v.rot(q.p)).0; (o[0] - sx).hypot(o[1] - sy) });
            let room = cell.checked_sub(1).and_then(near).unwrap_or(f32::INFINITY).min(near(cell + 1).unwrap_or(f32::INFINITY));
            Pose {
                dx: (sx - fx) * e, dy: (sy - fy) * e, angle: ang * e,
                sx: 1.0 + ((s * run).min(room * 0.97 / ADVANCE) - 1.0) * e, sy: 1.0 + (s * gap - 1.0) * e,
                alpha: flat_alpha + (paint.alpha * dim - flat_alpha) * e,
                accent: paint.accent * e, hue: if e > 0.5 { 0.0 } else { -1.0 },
                // Depth toward the viewer (px) and facing, for `El::form`: flat in the listing.
                z: p[2] * v.r * e, facing: 1.0 + (facing - 1.0) * e,
                ..Pose::default()
            }
        }
    }

    /// Behind the text: the ground's light spill, the reflection, the volume, the silhouette's dots,
    /// the far dust.
    fn back(&self) -> Drawing {
        let (w, h) = self.size;
        let (g, v, l) = (self.gather.value.clamp(0.0, 1.0), self.view(), self.light());
        let mut d = Drawing::new(w, h);
        dust(&mut d, w, h, self.t, 80);
        if g <= 0.0 { return d; }
        let mut c = self.cache.borrow_mut();
        let want = amenui::budget::grant(VOLUME);
        if c.vkey != (self.form, self.seed, want) { c.vol = volume(self.form, self.seed, want); c.vkey = (self.form, self.seed, want); }
        let rv = v.r * c.radius;
        let ground = v.c.1 + rv * 1.15;
        // A faint spill of light on the haze beneath it.
        for i in 0..60u32 {
            let (a, b) = (hash(i * 3 + 1) * 2.0 - 1.0, hash(i * 3 + 2) * 2.0 - 1.0);
            let r = 1.0 - (a * a + b * b).min(1.0);
            d.dot(v.c.0 + a * rv * 0.95, ground - rv * 0.02 + b * rv * 0.09, 14.0 + 22.0 * hash(i * 3 + 3), Paint { tone: 1.0, accent: 1.0, alpha: 0.012 * g * r, hue: 0 });
        }
        // The volume: only what is seen through the body (the far shell, the inside behind the middle).
        for (i, q) in c.vol.iter().enumerate() {
            let (p, n) = (v.rot(q.p), v.rot(q.n));
            let inside = dot3(q.p, q.p) < 0.7 * c.radius * c.radius;
            if (inside && p[2] > 0.0) || (!inside && n[2] > 0.0) { continue; }
            let ([x, y], k) = v.project(p);
            let paint = Paint { tone: 1.0, accent: 1.0, alpha: 0.20 * g * (0.4 + 0.6 * lit(l, p, n)), hue: 0 };
            d.dot(x, y, q.size * k * 0.9, paint);
            // Its reflection on the ground: squashed, dim, every other one.
            if i % 2 == 0 && y < ground {
                let ry = ground + (ground - y) * 0.22;
                let fade = (1.0 - (ry - ground) / (rv * 0.42)).clamp(0.0, 1.0);
                if fade > 0.0 { d.dot(x + (hash(i as u32) - 0.5) * 4.0, ry, q.size * 0.75, Paint { alpha: (paint.alpha * 0.8 * fade).min(0.3), ..paint }); }
            }
        }
        // Where a cell's glyph has faded out at the rim, the cell is one dot in the colour the light
        // gives it: the lines continue as a dotted contour instead of crushed letters. The far side
        // is the volume's.
        let dot_r = 0.006 * rv;
        for (gi, &k) in c.text.cell.iter().enumerate() {
            let Some(sp) = c.pts.get(k) else { continue };
            if v.rot(sp.n)[2] < -0.25 { continue; }
            let look = cell_look(&v, l, sp, 0.0);
            if look.paint.alpha * FORM_FX.facing_alpha(word_facing(&v, &c, gi)) >= 0.15 { continue; }
            let ([x, y], s) = v.project(v.rot(sp.p));
            d.dot(x, y, dot_r * s, Paint { tone: 1.0, accent: look.paint.accent, alpha: 0.30 * g, hue: 0 });
        }
        d
    }
    /// Over the text: the motes and the readouts' leader lines.
    fn front(&self, labels: &[Readout]) -> Drawing {
        let (w, h) = self.size;
        let mut d = Drawing::new(w, h);
        for m in &self.motes { let f = 1.0 - m.age / MOTE_LIFE; d.dot(m.x, m.y, 0.6 + 0.9 * f, Paint { tone: 1.0, accent: 0.6, alpha: 0.6 * f, hue: 0 }); }
        for l in labels {
            if l.alpha <= 0.0 { continue; }
            let p = Paint::DIM.alpha(l.alpha);
            d.line(l.from, l.elbow, 0.5, p);
            d.line(l.elbow, l.to, 0.5, p);
            d.dot(l.from.0, l.from.1, 1.4, Paint { tone: 1.0, ..p });
        }
        d
    }
    /// The glyph nearest the point the pointer's ray meets on the body: its index in the stream, the
    /// character, and where its cell is on screen.
    pub fn glyph_under_lamp(&self) -> Option<(usize, char, (f32, f32))> {
        self.ptr?;
        let gi = self.under?;
        let v = self.view();
        let c = self.cache.borrow();
        let ch = c.text.listing.chars().filter(|c| !c.is_whitespace()).nth(gi)?;
        let p = v.project(v.rot(c.pts[c.text.cell[gi]].p)).0;
        Some((gi, ch, (p[0], p[1])))
    }
    /// The readouts round the body, in state 04: `GLYPH 0412  t` from the glyph under the lamp (only
    /// while there is a pointer) and `YAW 023°` from the silhouette's lower right. Each leader runs at
    /// 45° then level; the text is measured with the face's real advance and kept inside the stage,
    /// the glyph's on the other side if it does not fit on its own; a readout that fits on neither is not drawn.
    /// None on a narrow window.
    pub fn readouts(&self) -> Vec<Readout> {
        if self.width_class() == Width::Narrow || self.target() < 0.5 { return vec![]; }
        let (v, g) = (self.view(), self.gather.value.clamp(0.0, 1.0));
        let (cx, cy, r) = self.placement();
        let alpha = ((g - 0.85) / 0.15).clamp(0.0, 1.0) * self.dim();
        if alpha <= 0.0 { return vec![]; }
        let st = self.stage();
        let fit = |key: &'static str, text: String, from: (f32, f32), ly: f32, right: bool, flip: bool| -> Option<Readout> {
            let tw = ADVANCE * text.chars().count() as f32;
            let ly = ly.clamp(st[1] + 8.0, self.caption_y() - 8.0);
            let dy = (ly - from.1).abs();
            for right in [right, !right].into_iter().take(1 + flip as usize) {
                let elbow = (if right { from.0 + dy } else { from.0 - dy }, ly);
                let end = if right { elbow.0 + 12.0 } else { elbow.0 - 12.0 };
                let x = if right { end + 6.0 } else { end - 6.0 - tw };
                let xc = x.clamp(st[0], st[2] - tw);
                let end = if right { xc - 6.0 } else { xc + tw + 6.0 };
                if (right && end >= elbow.0) || (!right && end <= elbow.0) {
                    return Some(Readout { key, text: text.clone(), from, elbow, to: (end, ly), at: (xc, ly - LINE * 0.5), alpha });
                }
            }
            None
        };
        let mut out = vec![];
        if let Some((gi, ch, at)) = self.glyph_under_lamp() {
            let right = at.0 >= cx;
            out.extend(fit("glyph", format!("GLYPH {gi:04}  {ch}"), at, cy - r * 1.02, right, true));
        }
        // The silhouette's point farthest towards the lower right (the projected cell farthest that way).
        let fs = {
            let c = self.cache.borrow();
            c.pts.iter().map(|sp| v.project(v.rot(sp.p)).0).max_by(|a, b| (a[0] * 0.85 + a[1] * 0.5).total_cmp(&(b[0] * 0.85 + b[1] * 0.5))).map_or((cx + r * 0.85, cy + r * 0.5), |p| (p[0], p[1]))
        };
        let yaw = (self.yaw.to_degrees().rem_euclid(360.0).round() as u32) % 360;
        out.extend(fit("yaw", format!("YAW {yaw:03}°"), fs, fs.1 + r * 0.15, true, false));
        out
    }
}
/// A readout: its text, the point it names, its leader's bend and end, and where its text is drawn.
pub struct Readout { pub key: &'static str, pub text: String, pub from: (f32, f32), pub elbow: (f32, f32), pub to: (f32, f32), pub at: (f32, f32), pub alpha: f32 }

/// Far dust: slow drift, a few marks from the budget.
fn dust(d: &mut Drawing, w: f32, h: f32, t: f32, n: usize) {
    let n = amenui::budget::grant(n);
    for i in 0..n as u32 {
        let s = i * 7;
        let z = 0.35 * (0.5 + 0.5 * hash(s + 3));
        let x = (hash(s) * w + t * (3.0 + 9.0 * z)).rem_euclid(w);
        let y = (hash(s + 1) * h - t * (1.0 + 2.0 * z) * hash(s + 2)).rem_euclid(h);
        let tw = 0.6 + 0.4 * (t * (0.4 + hash(s + 5)) + hash(s + 6) * 6.3).sin();
        d.dot(x, y, 0.5 + 1.1 * z, Paint { tone: 0.8, accent: 0.4, alpha: (0.06 + 0.22 * z) * tw, hue: 0 });
    }
}

impl Body {
    /// The scene filling a `w × h` window.
    pub fn view_in(&self, key: &str) -> El<Msg> {
        let (w, h) = self.size;
        self.ensure();
        let st = self.stage();
        let s = self.shown();
        let (lx, ly) = (st[0] + GUTTER, st[1]);
        let (g, dim) = (self.gather.value.clamp(0.0, 1.0), self.dim());
        // What is on the stage: the text (the listing or the body) or the specimen; the body's
        // drawings wait for a morph to land.
        let shown = self.shown_state();
        let card = matches!(shown, 2 | 3);
        let drawn = !card && self.t >= self.lands;
        let ink = Paint { tone: 1.0, accent: 0.0, alpha: 1.0, hue: 0 };
        let mono = |t: String, lh: f32, p: Paint, k: &str| text_lh(t, SIZE, lh, p).font(Font::Mono).nowrap().key(format!("{key}/{k}"));
        // The whole text in one element: the listing shows its first lines cut at whole words; the
        // glyphs it leaves out wait unseen where they would have been (the ends of cut lines to the
        // right of the stage, the lines that do not fit under it).
        let mut flat = vec![false; s.map.len() + s.rest_map.len()];
        for &i in &s.map { flat[i] = true; }
        // Its box is the listing's (the cut ends run past it, unseen).
        let cols = self.grid().1;
        let code = mono(self.listing(), LINE, ink, "text").absolute(Some(lx), Some(ly), None, None).width(cols as f32 * ADVANCE)
            .pose(self.poser((lx, ly), Rc::new(flat))).form(FORM_FX).reveal_by(Reveal::Word).hidden_until_morphed(card);
        let numbers: String = (1..=s.rows).map(|i| format!("{i:>3}")).collect::<Vec<_>>().join("\n");
        let numbers = mono(numbers, LINE, Paint::DIM.alpha(if shown == 1 { (1.0 - g).clamp(0.0, 1.0) * dim } else { 0.0 }), "lines").absolute(Some(st[0]), Some(ly), None, None);
        let marks = if drawn { self.readouts() } else { vec![] };
        let empty = || Drawing::new(w, h);
        let mut parts = vec![
            canvas(if drawn { self.back() } else { empty() }).key(format!("{key}/back")).width(w).absolute(Some(0.0), Some(0.0), None, None).respond(amenui::el::Response::Ignore),
            numbers,
            code,
            self.specimen(key).hidden_until_morphed(!card),
            canvas(if drawn { self.front(&marks) } else { empty() }).key(format!("{key}/front")).width(w).absolute(Some(0.0), Some(0.0), None, None).respond(amenui::el::Response::Ignore),
        ];
        for m in &marks {
            parts.push(mono(m.text.clone(), LINE, Paint::DIM.alpha(m.alpha), &format!("readout/{}", m.key)).absolute(Some(m.at.0), Some(m.at.1), None, None));
        }
        // The caption: the state's number in the dim colour, its sentence in the text colour; in the
        // body the sentence dims to the dim colour after 4 s without a change (no further: that is the
        // faintest text that still reads).
        let body = shown == 4 && !self.intro_pending && self.intro_at.is_none();
        let (num, sentence) = match shown {
            _ if body => ("04", "Posed in 3D. Your pointer is the lamp.".to_owned()),
            2 => ("02", "The same ink closes into controls.".to_owned()),
            3 => ("03", "One tree. The material sets the pixels.".to_owned()),
            _ => ("01", format!("Source. {} characters of Rust.", thousands(self.glyphs()))),
        };
        let quiet = body && self.t - self.changed > 4.0;
        let cy = self.caption_y();
        parts.push(mono(num.into(), LINE, Paint::DIM, "caption/n").absolute(Some(st[0]), Some(cy), None, None));
        // A screen reader hears it as it changes.
        parts.push(mono(sentence, LINE, if quiet { Paint::DIM } else { Paint::TEXT }, "caption").absolute(Some(st[0] + GUTTER), Some(cy), None, None).live(amenui::el::Live::Polite));
        let grabbing = self.grab.is_some() && self.dragged;
        let mut stage = layers(parts).key(key.to_owned()).width(w).height(h).clip();
        // The idle turn moves on purpose.
        if self.turning() { stage = stage.ongoing(); }
        stage
            .on_point(|p| Some(Msg::Point(p)))
            .on_key(|k, _| Self::takes(k).then_some(Msg::Key(k)))
            .cursor(if grabbing { amenui::el::Cursor::Grabbing } else if self.over { amenui::el::Cursor::Grab } else { amenui::el::Cursor::Default })
            .role(amenui::el::Role::Img).label(tr("body.label"))
    }
    /// The specimen of states 02 and 03: a card of four working components, centred on the body's
    /// centre, and the line that adds them to a project under it.
    fn specimen(&self, key: &str) -> El<Msg> {
        use amenui::el::{col, row, space};
        let phone = self.width_class() == Width::Narrow;
        let (cx, cy, _) = self.placement();
        let w = if phone { (self.size.0 - 32.0).min(358.0) } else { SPECIMEN.0 };
        let h = if phone { SPECIMEN_PHONE_H } else { SPECIMEN.1 };
        let sp = self.spec;
        let k = |s: &str| format!("{key}/specimen/{s}");
        let label = |t: &str| text_lh(t.to_owned(), SIZE, LINE, Paint::TEXT).font(Font::Mono).width(96.0);
        let line = |name: &str, c: El<Msg>| row().key(k(&format!("row-{name}"))).fill_width().align(taffy::AlignItems::CENTER).child(label(name)).child(c);
        let names: Vec<&str> = SPEC_MATERIALS.iter().map(|m| m.0).collect();
        let mut card = col().key(format!("{key}/specimen")).width(w).height(h).pad(20.0, 18.0).gap(14.0).radius(6.0).border(1.0, Paint::FAINT).fill(Paint::GROUND)
            .material(SPEC_MATERIALS[sp.material].1.id()).loose(if sp.loosen { 1.0 } else { 0.0 })
            .child(row().key(k("head")).fill_width()
                .child(text_lh("SPECIMEN".to_owned(), SIZE, LINE, Paint::DIM).font(Font::Mono)).child(space(0.0, 0.0).grow(1.0))
                .child(text_lh("4 components".to_owned(), SIZE, LINE, Paint::DIM).font(Font::Mono)))
            .child(line("Loosen", row().grow(1.0).child(space(0.0, 0.0).grow(1.0)).child(amenui::widgets::switch(k("loosen"), "", sp.loosen, None, Msg::Spec(SpecMsg::Loosen)).label("Loosen".to_owned()))));
        if !phone {
            let spec = amenui::widgets::Spec { min: 3.0, max: 12.0, step: 1.0, decimals: 0, unit: "px" };
            card = card.child(line("Pitch", row().grow(1.0).gap(12.0).align(taffy::AlignItems::CENTER)
                .child(amenui::widgets::slider(&k("pitch"), sp.pitch, spec, w - 40.0 - 96.0 - 52.0, |v| Msg::Spec(SpecMsg::Pitch(v))))
                .child(text_lh(format!("{} px", sp.pitch as u32), SIZE, LINE, Paint::TEXT).font(Font::Mono))));
        }
        let card = card
            .child(line("Material", amenui::widgets::segmented(&k("material"), &names, sp.material, 28.0, |i| Msg::Spec(SpecMsg::Material(i))).commit()))
            .child(space(0.0, 0.0).grow(1.0))
            .child(row().fill_width().child(space(0.0, 0.0).grow(1.0)).child(amenui::widgets::button_as(k("gather"), "GATHER →", amenui::widgets::Variant::Accent, Msg::Show(4))));
        // The line that adds them; on a phone there is no room for it above the caption.
        let mut b = col().key(format!("{key}/specimen-box")).absolute(Some((cx - w * 0.5).round()), Some((cy - h * 0.5).round()), None, None).gap(10.0).child(card);
        if !phone { b = b.child(text_lh("$ amenui add switch slider choice button".to_owned(), SIZE, LINE, Paint::DIM).font(Font::Mono).key(k("add"))); }
        b
    }
    /// Where the listing's caret is (window px): one cell after the last line shown.
    pub fn caret(&self) -> (f32, f32) {
        let s = self.shown();
        let st = self.stage();
        let last = s.text.rsplit('\n').next().unwrap_or("").chars().count();
        (st[0] + GUTTER + ADVANCE * (last as f32 + 1.5), st[1] + LINE * (s.rows.max(1) as f32 - 0.5))
    }
    /// The front for the host to send now, once (`front`).
    pub fn take_front(&mut self) -> Option<(f32, f32)> { self.front.take() }
    /// A front across the listing is under way (its speed is `FRONT_SPEED`).
    pub fn fronting(&self) -> bool { self.front_at.is_some() || self.front.is_some() || self.fronted.is_some_and(|t| self.t - t < 1.4) }
    /// The body turns by itself (the idle turn), and so asks for every frame.
    pub fn turning(&self) -> bool { self.target() >= 0.5 && !self.reduced && self.omega > 0.0 }
    /// The scene's keys: 1 and 4 the states, Space and Enter between them, F the form, [ and ] the
    /// seed, ← → a turn.
    pub fn takes(k: Key) -> bool { matches!(k, Key::Char(' ' | '1'..='4' | 'f' | 'F' | '[' | ']') | Key::Enter | Key::Left | Key::Right | Key::Escape) }
    /// What the hand does to the surface now (none in the listing).
    pub fn touch(&self) -> Touch {
        let r = self.cache.borrow().radius.max(1e-3);
        Touch { at: self.touch_at, k: self.bulge.value.clamp(0.0, 1.0), rings: self.rings.iter().map(|&(o, t0)| (o, RING_SPEED * r * (self.t - t0))).collect() }
    }
    /// How fast the body turns about the vertical now (rad/s): its free rate and the idle turn's share.
    pub fn turn_rate(&self) -> f32 { let u = self.idle.clamp(0.0, 1.0); self.spin + if self.idle_on() { u * u * (3.0 - 2.0 * u) * self.omega } else { 0.0 } }
    fn idle_on(&self) -> bool { self.target() >= 0.5 && !self.reduced && self.grab.is_none() }
    /// Where the ray through a window point meets the body (view space), with the normal.
    fn hit_at(&self, x: f32, y: f32, v: &View) -> Option<([f32; 3], [f32; 3])> { self.hit((x - v.c.0) / v.r, -(y - v.c.1) / v.r, v) }
    /// The turn that brings `q` (body space) to where the ray through a window point meets the sphere
    /// of `q`'s distance (the arcball): the point under the pointer stays under it. Pitch is clamped.
    fn arcball(&self, q: [f32; 3], rho: f32, x: f32, y: f32, v: &View) -> Option<(f32, f32)> {
        let (px, py) = ((x - v.c.0) / v.r, -(y - v.c.1) / v.r);
        let (o, d) = ([0.0, 0.0, 4.0], norm([px, py, -4.0]));
        let b = dot3(o, d);
        let disc = b * b - (dot3(o, o) - rho * rho);
        if disc < 0.0 { return None; }
        let s = -b - disc.sqrt();
        let t = [o[0] + d[0] * s, o[1] + d[1] * s, o[2] + d[2] * s];
        // About the vertical first: x' = q.x cos Y + q.z sin Y = ρ cos(Y − φ) must be t.x.
        let rxz = q[0].hypot(q[2]);
        if rxz < 1e-4 { return None; }
        let (phi, a) = (q[2].atan2(q[0]), (t[0] / rxz).clamp(-1.0, 1.0).acos());
        let tau = std::f32::consts::TAU;
        let near = |y: f32| y + ((self.yaw - y) / tau).round() * tau;
        let (y1, y2) = (near(phi + a), near(phi - a));
        let yaw = if (y1 - self.yaw).abs() <= (y2 - self.yaw).abs() { y1 } else { y2 };
        // Then about the horizontal: (q.y, z') turned by the pitch is (t.y, t.z).
        let zp = -q[0] * yaw.sin() + q[2] * yaw.cos();
        let mut pitch = t[2].atan2(t[1]) - zp.atan2(q[1]);
        pitch -= (pitch / tau).round() * tau;
        Some((yaw, pitch.clamp(-1.0, 1.0)))
    }
    /// The lamp to the pointer: just in front of the surface where the pointer's ray meets the body,
    /// else on a plane in front of it; back to its rest place when the pointer is gone.
    fn point_lamp(&mut self, at: Option<(f32, f32)>) {
        let v = self.view();
        let body = self.target() >= 0.5;
        let hit = at.and_then(|(x, y)| self.hit_at(x, y, &v)).filter(|_| body);
        self.over = hit.is_some();
        let now = self.t;
        // The bulge goes with the point under the pointer, and dies away where it was when there is none.
        if let Some((h, _)) = hit { self.touch_at = Some(v.unrot(h)); }
        let k = self.bulge.velocity;
        self.bulge.to(if hit.is_some() { 1.0 } else { 0.0 }, k, now);
        let target = match (at, hit) {
            (None, _) => REST_LIGHT,
            (Some(_), Some((h, n))) => [h[0] + 0.55 * n[0], h[1] + 0.55 * n[1], h[2] + 0.55 * n[2]],
            (Some((x, y)), None) => [(x - v.c.0) / v.r, -(y - v.c.1) / v.r, 1.2],
        };
        for (a, t) in self.lamp.iter_mut().zip(target) { let v = a.velocity; a.to(t, v, now); }
    }
    /// A turn by an arrow key: `KEY_TURN` in all, slowing as a flick does (at once with reduced motion).
    fn nudge(&mut self, dir: f32, now: f32) {
        self.last_input = now;
        self.idle = 0.0;
        if self.reduced { self.yaw += dir * KEY_TURN; } else { self.spin += dir * KEY_TURN / FLICK_TAU; }
    }
    /// Where the ray through the screen point (view units) meets the form, with its normal (view space).
    fn hit(&self, px: f32, py: f32, v: &View) -> Option<([f32; 3], [f32; 3])> {
        let o = [0.0, 0.0, 4.0];
        let dir = norm([px, py, -4.0]);
        let mut s = 2.0f32;
        for _ in 0..32 {
            let p = [o[0] + dir[0] * s, o[1] + dir[1] * s, o[2] + dir[2] * s];
            let d = sdf(self.form, self.seed, v.unrot(p));
            if d < 2e-3 { return Some((p, v.rot(gradient(self.form, self.seed, v.unrot(p))))); }
            s += d;
            if s > 6.0 { break; }
        }
        None
    }
}
/// The seed (0..100) whose largest facet faces the viewer most squarely at the resting turn.
pub fn best_seed() -> u32 {
    let v = View { yaw: -0.4, pitch: -0.22, c: (0.0, 0.0), r: 1.0 };
    (0..100u32).max_by(|&a, &b| {
        let score = |s: u32| { let f = facets(s); let (n, _) = f.iter().copied().min_by(|x, y| x.1.total_cmp(&y.1)).unwrap(); v.rot(n)[2] };
        score(a).total_cmp(&score(b))
    }).unwrap_or(DEFAULT_SEED)
}

impl Experiment for Body {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> { self.view_in(key) }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Key(Key::Left) => self.nudge(-1.0, now),
            Msg::Key(Key::Right) => self.nudge(1.0, now),
            Msg::Key(Key::Char(c @ '1'..='4')) => self.update(Msg::Show(c as u8 - b'0'), now),
            // The walk's own press on GATHER → keeps the walk's morph.
            Msg::Show(n) => { let m = if self.walk.is_some() { MORPH_INTRO } else { MORPH_SHORT }; self.go_to(n, now, m); }
            Msg::Key(Key::Escape) | Msg::Skip => self.skip(now),
            Msg::Play => self.play(now),
            Msg::Spec(SpecMsg::Loosen) => self.spec.loosen = !self.spec.loosen,
            Msg::Spec(SpecMsg::Pitch(v)) => self.spec.pitch = v.clamp(3.0, 12.0).round(),
            Msg::Spec(SpecMsg::Material(i)) => {
                // A material other than plain is state 03.
                self.spec.material = i.min(SPEC_MATERIALS.len() - 1);
                let st = if self.spec.material == 0 { 2 } else { 3 };
                if matches!(self.at, 2 | 3) && st != self.at { (self.was, self.at, self.changed) = (self.at, st, now); }
            }
            Msg::Key(Key::Char('f' | 'F')) | Msg::Form => { self.form = 1 - self.form; self.changed = now; }
            Msg::Key(Key::Char('[')) => self.update(Msg::Seed(-1), now),
            Msg::Key(Key::Char(']')) => self.update(Msg::Seed(1), now),
            Msg::Seed(d) => { self.seed = (self.seed as i32 + d).rem_euclid(100) as u32; self.changed = now; }
            Msg::Key(_) | Msg::Gather => { let to = if self.at == 4 { 1 } else { 4 }; self.update(Msg::Show(to), now); }
            Msg::Num(i, Edit::Set(v)) => match i {
                0 => self.go(if v >= 0.5 { 1.0 } else { 0.0 }, now, FADE),
                1 => self.form = (v >= 0.5) as u32,
                2 => self.seed = v.clamp(0.0, 99.0) as u32,
                _ => self.omega = v.clamp(0.0, 1.5),
            },
            Msg::Num(..) => {}
            Msg::Point(p) => {
                self.ptr = p.map(|q| (q.x, q.y));
                self.point_lamp(self.ptr);
                // The glyph drawn under the pointer, as the runtime found it (`Ui::glyph_at`); while
                // dragging none is given, and the last one stays; between glyphs too, while the
                // pointer is on the body.
                match p {
                    Some(q) if q.down => {}
                    Some(q) if q.glyph.is_some() => self.under = q.glyph,
                    Some(_) if self.over => {}
                    _ => self.under = None,
                }
                let body = self.target() >= 0.5;
                let v = self.view();
                match (p, self.grab) {
                    (Some(q), Some(g)) if q.down => {
                        if (q.x - g.x).hypot(q.y - g.y) > 5.0 { self.dragged = true; }
                        if self.dragged && body {
                            // The arcball: the point pressed stays under the pointer. Pressed off the
                            // body, or dragged past its sphere, the turn follows the pointer's travel.
                            let turned = g.on.and_then(|(q0, rho)| self.arcball(q0, rho, q.x, q.y, &v));
                            let (yaw, pitch) = turned.unwrap_or_else(|| {
                                let r = self.placement().2.max(60.0);
                                (g.yaw + (q.x - g.x) / r, (g.pitch + (q.y - g.y) / r).clamp(-1.0, 1.0))
                            });
                            (self.yaw, self.pitch) = (yaw, pitch);
                            self.vel.push(now, yaw, pitch);
                        }
                        self.last_input = now;
                    }
                    (Some(q), None) if q.down => {
                        let on = self.hit_at(q.x, q.y, &v).map(|(h, _)| (v.unrot(h), dot3(h, h).sqrt()));
                        self.grab = Some(Grab { x: q.x, y: q.y, yaw: self.yaw, pitch: self.pitch, on });
                        self.dragged = false;
                        self.vel.clear();
                        self.vel.push(now, self.yaw, self.pitch);
                        // The hand holds it: the turn stops until it lets go.
                        (self.spin, self.tip, self.idle, self.last_input) = (0.0, 0.0, 0.0, now);
                    }
                    (_, Some(g)) => {
                        // Let go: a drag keeps turning at its speed; a click sends a ring from where it
                        // pressed (none with reduced motion) and leaves the body where it is.
                        if self.dragged {
                            let (vy, vp) = self.vel.get();
                            (self.spin, self.tip) = (vy.clamp(-8.0, 8.0), vp.clamp(-4.0, 4.0));
                        } else if body && !self.reduced && let Some((o, _)) = g.on {
                            if self.rings.len() >= 4 { self.rings.remove(0); }
                            self.rings.push((o, now));
                        }
                        self.grab = None;
                        self.dragged = false;
                        self.vel.clear();
                        self.last_input = now;
                    }
                    _ => {}
                }
            }
        }
    }
    fn step(&mut self, now: f32) -> bool {
        let dt = (now - self.t).clamp(0.0, 0.05);
        self.t = now;
        // The short path: the first frame shows the listing; half a second later it gathers (with
        // reduced motion it gives way to the body by an opacity change at once).
        let mut changed = false;
        if self.intro_pending {
            self.intro_pending = false;
            if self.full && !self.reduced { self.walk_from(now, true); } else { self.intro_at = Some(now + if self.reduced { 0.0 } else { INTRO_DELAY }); }
            if !self.reduced { self.front_at = Some(now + FRONT_DELAY); }
            changed = true;
        }
        // The walk's steps, and steps at a time of their own, as their times come.
        if let Some((t0, steps)) = &mut self.walk {
            let t0 = *t0;
            let due: Vec<Step> = { let n = steps.iter().take_while(|s| now >= t0 + s.0).count(); steps.drain(..n).map(|s| s.1).collect() };
            for st in due { self.do_step(st, now); changed = true; }
        }
        if !self.later.is_empty() {
            let (due, rest): (Vec<_>, Vec<_>) = self.later.drain(..).partition(|s| now >= s.0);
            self.later = rest;
            for (_, st) in due { self.do_step(st, now); changed = true; }
        }
        if let Some((t, v)) = self.settle && now >= t { self.gather = Animated::new(v, Spring { ..GATHER }); self.settle = None; changed = true; }
        if let Some(t0) = self.front_at && now >= t0 { self.front_at = None; self.front = Some(self.caret()); self.fronted = Some(now); changed = true; }
        if let Some(t0) = self.intro_at && now >= t0 { self.intro_at = None; self.go(1.0, now, FADE_INTRO); }
        if let Some((t0, to, d)) = self.fade {
            if now - t0 >= d * 0.5 && self.gather.target != to { self.gather = Animated::new(to, Spring { ..GATHER }); }
            if now - t0 >= d { self.fade = None; }
            changed = true;
        }
        changed |= self.gather.step(now);
        for a in &mut self.lamp { changed |= a.step(now); }
        changed |= self.bulge.step(now);
        let body = self.target() >= 0.5;
        if self.grab.is_none() {
            // A flick or a key's turn slows; the idle turn comes back 2 s after the last input, over 1 s.
            let k = (-dt / FLICK_TAU).exp();
            self.spin *= k;
            self.tip *= k;
            if self.spin.abs() < 1e-3 { self.spin = 0.0; }
            if self.tip.abs() < 1e-3 { self.tip = 0.0; }
            if self.idle_on() && now - self.last_input >= IDLE_AFTER { self.idle = (self.idle + dt).min(1.0); }
            let rate = self.turn_rate();
            if rate != 0.0 || self.tip != 0.0 {
                self.yaw += rate * dt;
                self.pitch = (self.pitch + self.tip * dt).clamp(-1.0, 1.0);
                changed = true;
                // What is under a pointer that stays still changes as the body turns under it.
                if self.ptr.is_some() { self.point_lamp(self.ptr); }
            }
        }
        // Rings cross the surface and leave it.
        let r = self.cache.borrow().radius;
        let before = self.rings.len();
        self.rings.retain(|&(_, t0)| RING_SPEED * r * (now - t0) < 2.4 * r + RING_WIDTH);
        changed |= !self.rings.is_empty() || before != self.rings.len();
        for m in &mut self.motes { m.age += dt; m.x += m.vx * dt; m.y += m.vy * dt; m.vy -= 3.0 * dt; }
        let before = self.motes.len();
        self.motes.retain(|m| m.age < MOTE_LIFE);
        changed |= before != self.motes.len() || !self.motes.is_empty();
        // Rim cells shed a few motes while it turns (from the budget; never in the listing).
        if body && self.gather.value > 0.97 && !self.reduced && dt > 0.0 && self.turn_rate().abs() > 0.01 {
            let v = self.view();
            let c = self.cache.borrow();
            let want = amenui::budget::grant(1).min(30usize.saturating_sub(self.motes.len()));
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
        changed || (body && !self.reduced && self.omega > 0.0)
    }
    fn moving(&self) -> bool {
        self.gather.moving() || self.lamp.iter().any(Animated::moving) || self.bulge.moving() || !self.rings.is_empty() || !self.motes.is_empty() || self.grab.is_some()
            || self.spin != 0.0 || self.tip != 0.0 || (self.target() >= 0.5 && !self.reduced && self.omega > 0.0)
            || self.intro_pending || self.intro_at.is_some() || self.front_at.is_some() || self.fade.is_some()
            || self.walk.is_some() || !self.later.is_empty() || self.settle.is_some() || self.t < self.lands
    }
}

/// A runtime of its own (the `render` tool, tests): the scene alone in its look.
#[cfg(not(target_arch = "wasm32"))]
pub fn ui(faces: Vec<amenui::text::Face>, w: u32, h: u32, scale: f32) -> amenui::ui::Ui<Body, Msg> {
    fn view(s: &Body) -> El<Msg> { amenui::el::col().fill_width().fill_height().fonts(amenui::style::LAB.roles()).child(s.view_in("body")) }
    fn update(s: &mut Body, m: Msg) { let now = s.t; s.update(m, now); }
    let mut b = Body::default();
    b.resize(w as f32 / scale, h as f32 / scale);
    amenui::ui::Ui::new(b, view, update, faces, w, h, scale)
        .on_resize(|s, w, h| s.resize(w, h))
        .on_look(|s| look(s.form, s.gather.value))
        .on_tick(|s, t| s.step(t))
        .on_wake(|s, t| s.moving().then_some(t))
        .on_commands(|s| std::mem::take(&mut s.cmds))
        .on_input(|s| s.heard())
}
/// The scene's look for a form, `gathered` of the way from the listing to the body: the site's dark
/// ground, the plain material, a small bloom on the brightest glyphs and a vignette, both only as
/// the body gathers (the listing is plain text on the plain ground, as the page draws it in HTML).
/// Rock: the site's orange and ink, blue for the silhouette's dots; cloud: cyan and pale ink.
pub fn look(form: u32, gathered: f32) -> amenui::style::Look {
    let g = gathered.clamp(0.0, 1.0);
    let lab_t = amenui::style::LAB;
    let theme = if form == 0 {
        amenui::style::Theme { ground: [0.047, 0.051, 0.055], ground2: [0.075, 0.066, 0.064], ink: [0.886, 0.890, 0.878], accent: [1.0, 0.42, 0.17], second: [0.30, 0.62, 1.0], third: [0.55, 0.88, 1.0], ..lab_t }
    } else {
        amenui::style::Theme { ground: [0.040, 0.050, 0.060], ground2: [0.050, 0.066, 0.090], ink: [0.90, 0.97, 1.0], accent: [0.36, 0.78, 1.0], second: [0.30, 0.62, 1.0], third: [1.0, 0.74, 0.42], ..lab_t }
    };
    // The ground's second tone (its gradient) also comes in with the body: the listing's ground is flat.
    let theme = amenui::style::Theme { ground2: std::array::from_fn(|i| theme.ground[i] + (theme.ground2[i] - theme.ground[i]) * g), ..theme };
    let mut l = amenui::style::Look { theme, material: amenui::style::Material::Plain, ..amenui::style::Look::default() };
    l.post.bloom = amenui::post::Bloom { strength: 0.8 * g, threshold: 0.8, radius: 6.0 };
    l.post.vignette = amenui::post::Vignette { strength: 0.35 * g, ..amenui::post::Vignette::default() };
    l
}
