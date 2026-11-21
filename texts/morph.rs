//! 变换 (DESIGN.md §1.10): a morph samples what an element draws into lattice tiles that fly to and
//! become another element; a flow moves the parts two pages share to their new place. Everything here
//! is plain data and pure functions; the runtime (`ui.rs`) runs them and the renderer draws the tiles.
//!
//! A tile is one cell of the material's lattice (or an integer multiple of it, when there would be more
//! than `count_max`). It carries no colour of its own: it shows what the source drew in its cell (read
//! from a picture of the source's stage-1 output) and, as it lands, what the target draws in its cell.
//! So at the hand-over a tile is exactly the piece of the element it replaces, whatever the material.
use crate::scene::{GLYPH, Instance, MESH, PICTURE};
use crate::text::{ATLAS, SPREAD};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ease { Linear, #[default] InOutCubic, OutExpo }
impl Ease {
    pub const ALL: [Ease; 3] = [Ease::Linear, Ease::InOutCubic, Ease::OutExpo];
    pub const NAMES: [&'static str; 3] = ["线性", "缓入缓出", "指数减速"];
    pub fn index(self) -> usize { self as usize }
    pub fn from_index(i: usize) -> Self { Self::ALL[i.min(2)] }
    /// 0..1 → 0..1, exactly 0 and 1 at the ends. The shader has the same three curves.
    pub fn at(self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        match self {
            Ease::Linear => x,
            Ease::InOutCubic => if x < 0.5 { 4.0 * x * x * x } else { 1.0 - (-2.0 * x + 2.0).powi(3) * 0.5 },
            Ease::OutExpo => (1.0 - (-10.0 * x).exp2()) / (1.0 - (-10.0f32).exp2()),
        }
    }
}

/// How a morph goes. Times in milliseconds, lengths in logical px.
#[derive(Clone, Debug, PartialEq)]
pub struct MorphSpec {
    /// The source loosens 0 → 1 over this, then hands over to the tiles.
    pub loosen_ms: f32,
    /// The flight, from the first tile leaving to the last one landing.
    pub duration_ms: f32,
    /// The target, shown at looseness 1 when the tiles land, tightens 1 → 0 over this.
    pub tighten_ms: f32,
    /// At most this many tiles; past it the tile becomes 2, 3 … lattice cells wide.
    pub count_max: f32,
    /// A cell flies when this much of it is covered (the material's four-point average); thinner cells
    /// stay where they are and fade.
    pub threshold: f32,
    /// How far the swarm swings off the straight line, mid-flight.
    pub curl: f32,
    /// Fraction of the flight over which departures are spread.
    pub stagger: f32,
    pub ease: Ease,
    /// The material's lattice pitch (`Fx::pitch`), so the tiles are its cells.
    pub lattice: f32,
    /// Take the source from the frame drawn last, not from the tree (`Ui::morph_from_last`): the
    /// glyphs that frame drew for this key (for `to`'s key when `None`), which fly, each whole, to the
    /// new element's over `duration_ms` (the other fields are not used then).
    pub from_last: Option<String>,
}
impl Default for MorphSpec {
    fn default() -> Self {
        Self { loosen_ms: 220.0, duration_ms: 900.0, tighten_ms: 260.0, count_max: 40000.0, threshold: 0.5, curl: 48.0, stagger: 0.35, ease: Ease::InOutCubic, lattice: 5.0, from_last: None }
    }
}

/// How shared parts move when the page changes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowSpec {
    /// Flight of a shared part from its old rect to its new one (in-out cubic).
    pub flow_ms: f32,
    /// Looseness at the middle of that flight.
    pub flow_loose: f32,
    /// Parts of the old page that are not shared loosen and fade over this.
    pub out_ms: f32,
    /// With less motion (`Ui::set_reduced_motion`) nothing flies: the old page fades out and the new one
    /// in over this (default 150). It stays the fade for what comes in and goes until the next flow.
    pub fade_ms: f32,
}
impl Default for FlowSpec {
    fn default() -> Self { Self { flow_ms: 420.0, flow_loose: 0.25, out_ms: 180.0, fade_ms: 150.0 } }
}

/// How the changed glyphs of a text whose string changed recompose (DESIGN.md §1.16): the old glyphs'
/// tiles travel to the new glyphs' shape. Times in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Recompose {
    /// From the first tile leaving to the last one landing.
    pub duration_ms: f32,
    /// Fraction of the flight over which departures are spread.
    pub stagger: f32,
    /// At most this many flying tiles per changed glyph; past it the tiles grow by whole device px.
    pub count_max: f32,
    /// Unchanged glyphs that moved (the text got longer or shorter) slide to their new place over this.
    pub slide_ms: f32,
    /// A text that changes again sooner than this after its last change is replaced at once, without
    /// tiles (a readout that changes every frame never recomposes).
    #[serde(default = "default_min_interval")]
    pub min_interval_ms: f32,
    /// At most this many recomposition tiles are alive at once over the whole window; a change that
    /// would need more is replaced at once.
    #[serde(default = "default_max_tiles")]
    pub max_tiles: f32,
    /// The tiles' size to start from (logical px, whole device px): 1 px tiles in flight read as noise,
    /// 2 px as the glyph's own pixels changing places.
    #[serde(default = "default_tile")]
    pub tile: f32,
}
fn default_tile() -> f32 { 2.0 }
fn default_min_interval() -> f32 { 80.0 }
fn default_max_tiles() -> f32 { 24000.0 }
impl Default for Recompose {
    fn default() -> Self { Self { duration_ms: 160.0, stagger: 0.2, count_max: 600.0, slide_ms: 120.0, min_interval_ms: default_min_interval(), max_tiles: default_max_tiles(), tile: default_tile() } }
}

/// The GPU particle path (DESIGN.md §1.10): when a morph would need more tiles than `gpu_above`, the
/// tiles are a grid the GPU simulates (state in textures), not records the CPU builds. Forces act on
/// each tile's offset from its eased path, so the ends of the flight stay exact. Lengths are logical px,
/// accelerations logical px/s².
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particles {
    /// Above this many tiles a morph runs on the GPU (where the GPU can; else on the CPU, capped).
    pub gpu_above: f32,
    /// Curl noise: strength and scale.
    pub curl: f32,
    pub curl_scale: f32,
    /// How fast an offset's speed dies (1/s), and how hard it is pulled back to the path (1/s²).
    pub damping: f32,
    pub spring: f32,
    /// The pointer: strength (+ pulls, − pushes) and reach.
    pub pointer: f32,
    pub pointer_radius: f32,
    /// How hard a passing front pushes.
    pub front: f32,
}
impl Default for Particles {
    fn default() -> Self { Self { gpu_above: 20000.0, curl: 1600.0, curl_scale: 110.0, damping: 2.5, spring: 6.0, pointer: -9000.0, pointer_radius: 140.0, front: 30000.0 } }
}

/// A morph on the GPU path: the same-sized grid over each element (device px: x0, y0, tile w, tile h),
/// its columns and rows, and, once turned around, how (`Turn`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid { pub a: [f32; 4], pub b: [f32; 4], pub cols: u32, pub rows: u32, pub turned: Option<Turn> }

/// A GPU morph turned around any number of times. Every turn after the first sends all tiles off at
/// once over the same time, so each tile's progress at the start of the current leg is an affine
/// function of where it was at the first turn: `alpha × u(first) + beta`, and the leg goes from there to
/// `to` (0 or 1) over `dur` seconds. `first` is the forward flight's clock at the first turn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Turn { pub first: f32, pub alpha: f32, pub beta: f32, pub to: f32, pub dur: f32 }
impl Turn {
    /// The turn after this one, `clock` seconds into this leg (on `ease`), going back over `dur`.
    pub fn again(self, clock: f32, ease: Ease, dur: f32) -> Turn {
        let e = ease.at(clock / self.dur.max(1e-4));
        Turn { first: self.first, alpha: self.alpha * (1.0 - e), beta: self.beta * (1.0 - e) + self.to * e, to: 1.0 - self.to, dur }
    }
    /// A tile's progress `clock` seconds into the current leg, given its forward delay and duration (the
    /// shader's `grid_u` does the same).
    pub fn progress(&self, delay: f32, fwd: f32, clock: f32, ease: Ease) -> f32 {
        let at = ease.at((self.first - delay) / fwd.max(1e-4));
        let from = self.alpha * at + self.beta;
        from + (self.to - from) * ease.at(clock / self.dur.max(1e-4))
    }
}
impl Grid {
    pub fn count(&self) -> u32 { self.cols * self.rows }
    /// The grid for two elements' ink bounds at lattice `pitch` (device px): tiles of a whole number of
    /// px, at least the pitch, big enough that there are at most `count_max`.
    pub fn new(a: Bounds, b: Bounds, pitch: f32, count_max: f32) -> Self {
        let s = SPREAD as f32;
        let wide = |b: Bounds| [(b[0] - s).floor(), (b[1] - s).floor(), (b[2] + s).ceil(), (b[3] + s).ceil()];
        let (a, b) = (wide(a), wide(b));
        let (w, h) = ((a[2] - a[0]).max(b[2] - b[0]).max(1.0), (a[3] - a[1]).max(b[3] - b[1]).max(1.0));
        let mut tile = pitch.round().max(1.0);
        while (w / tile).ceil() * (h / tile).ceil() > count_max.max(1.0) { tile += 1.0; }
        let (cols, rows) = ((w / tile).ceil() as u32, (h / tile).ceil() as u32);
        Self { a: [a[0], a[1], tile, tile], b: [b[0], b[1], tile, tile], cols, rows, turned: None }
    }
    /// How many tiles the CPU would cut for these bounds at `pitch` (to choose the path).
    pub fn estimate(a: Bounds, b: Bounds, pitch: f32) -> f32 {
        let s = SPREAD as f32 * 2.0;
        let area = |b: Bounds| ((b[2] - b[0] + s) * (b[3] - b[1] + s)).max(0.0);
        area(a).max(area(b)) / (pitch * pitch).max(1.0)
    }
}

/// What `update` asks the runtime to do; the application keeps a list in its state and the runtime
/// takes it after every update (`Ui::on_commands`).
#[derive(Clone, Debug, PartialEq)]
pub enum Cmd {
    /// Put the field's pointer at (x, y), logical px, without a lens (what follows the pointer — the
    /// chrome's reflection — follows it): a ghost's pointer in a scripted interaction.
    Pointer { x: f32, y: f32 },
    /// Morph the element `from` into the element `to` (keys). Asking for the reverse of the morph in
    /// progress turns it around from where its tiles are.
    Morph { from: String, to: String, spec: MorphSpec },
    /// The page is about to change: parts with the same flow id move instead of being replaced.
    Flow(FlowSpec),
    /// A ripplr (a front) from (x, y), logical px, of this strength (0..1): for things that disturb the
    /// page by themselves, such as a beat.
    Ripplr { x: f32, y: f32, strength: f32 },
    /// A ripplr from the centre of the element `key` as laid out by this update.
    RipplrAt { key: String, strength: f32 },
    /// Stir the field map (DESIGN.md §1.29) at a point, at an element's centre, or along a path: `energy`
    /// (0..1) and flow `v` (logical px are not involved: a direction whose length is how much; along a
    /// path `v` is [along the path, across it]) over `radius` logical px, on what `scope` reaches.
    Stir { at: StirAt, v: [f32; 2], energy: f32, radius: f32, scope: crate::el::Scope },
    /// A gust over the field map (DESIGN.md §1.29): a band that crosses `scope`, worked out from the time
    /// at every node and laid over the map each step, with a wake behind it, gone without a trace once it
    /// has passed. One command; the application does nothing while it crosses.
    Gust { gust: crate::fieldmap::Gust, scope: crate::el::Scope },
    /// Press the element `key` as a person would (its message, its front, its condensing): for scripted
    /// demonstrations and tests. It does not count as a person's input (`Ui::on_input` is not told).
    Press { key: String },
    /// Keep the frame as it is now and play the new look's transition from (x, y), logical px: a commit
    /// that no press made (a key, a timer). Taken before the layout, like a flow. With `scope` (the key
    /// of an element marked `El::commit_scope`), it plays only within that element's box.
    Commit { x: f32, y: f32, scope: Option<String> },
    /// Give the element `key` the focus (as the keyboard would), after this layout.
    Focus { key: String },
    /// Take the focus away: nothing has it, its corners leave as they do when it moves on, and the next
    /// Tab goes on from the element that had it (`Ui::clear_focus`).
    ClearFocus,
    /// Empty the plate `key` (`El::plate`) in this frame, at once: a cut for a beat. For a soft end
    /// use `Plate::fade_s`.
    ClearPlate { key: String },
    /// Scroll so the element `key` is at the top of its scroller (an outline's link), after this layout.
    ScrollTo { key: String },
    /// Hold the morph in progress at this point of its flight (0 = the source's tiles, 1 = the target's),
    /// or let it run on from where it is held (`None`).
    Scrub(Option<f32>),
    /// Stop the beat clock (DESIGN.md §1.32) for this many steps: what falls on the beat stands still,
    /// the frame byte for byte. Nothing when the look has no beat.
    Hold(u8),
    /// Play the beat clock's current step again from its start.
    Retrigger,
    /// Take the beat clock back this many beats (never before its start): what the beat carries (the
    /// fronts and a transition's steps) goes back; the materials' own clocks (the ground's drift,
    /// scribblr's pose) and the application's state do not.
    Rewind(f32),
    /// Set one number of the look by its name (`Look::set`) from now on, over what the application's
    /// look says; a switch (`post.invert`) given 1 turns over, given 0 goes back to the look's own.
    Set { name: String, value: f32 },
}

/// Where a `Cmd::Stir` stirs: a point (logical px), the centre of an element, or a polyline (logical
/// px) from `from` to `to` of its arc length (0..1), a stir every half cell, so a path drawn by progress
/// stirs the part drawn since the last command.
#[derive(Clone, Debug, PartialEq)]
pub enum StirAt { Point([f32; 2]), Key(String), Path { points: std::rc::Rc<Vec<[f32; 2]>>, from: f32, to: f32 } }

/// What a tile is in the pairing.
pub mod kind {
    /// One source cell to one target cell.
    pub const PRIMARY: f32 = 0.0;
    /// An extra target served by an already used source: leaves with it, fades in over the first 20%.
    pub const SPLIT: f32 = 1.0;
    /// An extra source onto an already used target: fades out over the last 20%.
    pub const MERGE: f32 = 2.0;
    /// A source cell too thin to fly: stays and fades out over the first 20%.
    pub const SRC_DUST: f32 = 3.0;
    /// A target cell too thin to fly: fades in where it is over the last 20%.
    pub const DST_DUST: f32 = 4.0;
    /// Added to the kind: the tile starts on a mesh surface point that is behind the mesh now (hidden
    /// until it has lifted off), and lands on one that is (it fades before it lands).
    pub const SRC_HIDDEN: f32 = 8.0;
    pub const DST_HIDDEN: f32 = 16.0;
    /// The kind without the two flags.
    pub fn base(k: f32) -> f32 { ((k as u32) & 7) as f32 }
}

fn smooth(a: f32, b: f32, x: f32) -> f32 { let t = ((x - a) / (b - a)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) }
/// A tile's opacity at progress `u` (the shader has the same).
pub fn alpha(k: f32, u: f32) -> f32 {
    let code = k as u32;
    let a = match code & 7 {
        1 => smooth(0.0, 0.2, u),
        2 => 1.0 - smooth(0.8, 1.0, u),
        3 => 1.0 - smooth(0.0, 0.2, u),
        4 => smooth(0.8, 1.0, u),
        _ => 1.0,
    };
    let a = if code & 8 != 0 { a * smooth(0.06, 0.3, u) } else { a };
    if code & 16 != 0 { a * (1.0 - smooth(0.7, 0.94, u)) } else { a }
}

/// One tile, as the GPU reads it (instance rate).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    /// Top-left of its source cell and of its target cell, device px.
    pub ab: [f32; 4],
    /// Progress at the start and the end of this run, delay and duration (seconds) on the run's clock.
    /// Forward: 0 → 1. A reversal rewrites these from where the tile is.
    pub run: [f32; 4],
    /// Kind (`kind`), size (device px), seed in 0..1, and where (in progress) a tile whose source and
    /// target are drawn differently (a picture and text) changes from one to the other.
    pub misc: [f32; 4],
}

/// Progress of a tile `clock` seconds into its run.
pub fn progress(p: &Particle, clock: f32, ease: Ease) -> f32 {
    let l = ((clock - p.run[2]) / p.run[3].max(1e-4)).clamp(0.0, 1.0);
    let e = ease.at(l);
    p.run[0] + (p.run[1] - p.run[0]) * e
}

/// The integer hash the shader uses too (lowbias32).
pub fn hash(mut x: u32) -> u32 {
    x ^= x >> 16; x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15; x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}
fn unit(h: u32) -> f32 { (h >> 8) as f32 / 16_777_216.0 }
/// Smooth value noise in -1..1 on both axes (the shader has the same).
pub fn noise2(x: f32, y: f32) -> [f32; 2] {
    let (ix, iy) = (x.floor(), y.floor());
    let (fx, fy) = (x - ix, y - iy);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let corner = |dx: i32, dy: i32, salt: u32| {
        let h = hash((ix as i32 + dx) as u32 ^ hash(((iy as i32 + dy) as u32) ^ salt));
        unit(h) * 2.0 - 1.0
    };
    let v = |salt: u32| {
        let a = corner(0, 0, salt) + (corner(1, 0, salt) - corner(0, 0, salt)) * sx;
        let b = corner(0, 1, salt) + (corner(1, 1, salt) - corner(0, 1, salt)) * sx;
        a + (b - a) * sy
    };
    [v(0x68e3_1da4), v(0xb529_7a4d)]
}

/// Where a tile's top-left is at progress `u` (device px): along the line, plus a swing to one side
/// that is largest mid-flight. Cells that stay (dust) do not swing. `curl` is device px.
pub fn position(p: &Particle, u: f32, curl: f32) -> [f32; 2] {
    let (a, b) = ([p.ab[0], p.ab[1]], [p.ab[2], p.ab[3]]);
    let mut q = [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u];
    if kind::base(p.misc[0]) < kind::SRC_DUST && u > 0.0 && u < 1.0 {
        let d = [b[0] - a[0], b[1] - a[1]];
        let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let dir = if len > 0.5 { [d[0] / len, d[1] / len] } else { [1.0, 0.0] };
        let n = noise2(a[0] / 120.0, a[1] / 120.0);
        let s = [n[0] + (p.misc[2] - 0.5) * 0.6, n[1] + (unit(hash((p.misc[2] * 16_777_216.0) as u32)) - 0.5) * 0.6];
        let k = curl * (std::f32::consts::PI * u).sin();
        let (side, along) = (0.8 + 0.5 * s[0], 0.35 * s[1]);
        q[0] += k * (-dir[1] * side + dir[0] * along);
        q[1] += k * (dir[0] * side + dir[1] * along);
    }
    q
}

/// Pair `ns` sources with `nt` targets by rank (both sorted by the same order). Every source and every
/// target is used at least once and there are `max(ns, nt)` pairs; when the counts differ the extra
/// ones are `SPLIT` (a target sharing the source of the rank nearest it) or `MERGE` (a source sharing
/// a target). Deterministic. Empty when either side is.
pub fn pair(ns: usize, nt: usize) -> Vec<(usize, usize, f32)> {
    if ns == 0 || nt == 0 { return vec![]; }
    let mut out = Vec::with_capacity(ns.max(nt));
    if ns >= nt {
        let mut last = usize::MAX;
        for i in 0..ns {
            let j = i * nt / ns;
            out.push((i, j, if j == last { kind::MERGE } else { kind::PRIMARY }));
            last = j;
        }
    } else {
        let mut last = usize::MAX;
        for j in 0..nt {
            let i = j * ns / nt;
            out.push((i, j, if i == last { kind::SPLIT } else { kind::PRIMARY }));
            last = i;
        }
    }
    out
}

/// Morton (Z-order) code of a point given as fractions 0..1 of its element's bounds.
pub fn morton(fx: f32, fy: f32) -> u32 {
    let spread = |v: f32| {
        let mut x = (v.clamp(0.0, 1.0) * 65535.0) as u32;
        x = (x | (x << 8)) & 0x00FF_00FF;
        x = (x | (x << 4)) & 0x0F0F_0F0F;
        x = (x | (x << 2)) & 0x3333_3333;
        (x | (x << 1)) & 0x5555_5555
    };
    spread(fx) | (spread(fy) << 1)
}

/// One lattice cell of an element: its top-left (device px), how much of it is covered, and a tag the
/// caller can follow it by (a mesh surface point; `NO_TAG` for cells of the screen).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell { pub x: f32, pub y: f32, pub cover: f32, pub tag: u32 }
pub const NO_TAG: u32 = u32::MAX;

/// Device px rect x0, y0, x1, y1.
pub type Bounds = [f32; 4];

/// Where an element's instances put ink, device px. Glyph rects carry the near spread; it is taken off.
pub fn ink_bounds(insts: &[Instance]) -> Option<Bounds> {
    let mut b: Option<Bounds> = None;
    for i in insts {
        // Where it really is: a turned, scaled or posed instance covers its transformed box.
        let pad = if i.paint[3] == GLYPH { SPREAD as f32 } else { 0.0 };
        let mut g = *i;
        g.rect = [i.rect[0] + pad, i.rect[1] + pad, (i.rect[2] - 2.0 * pad).max(0.0), (i.rect[3] - 2.0 * pad).max(0.0)];
        let r = g.bounds(0.0);
        b = Some(match b { None => r, Some(o) => [o[0].min(r[0]), o[1].min(r[1]), o[2].max(r[2]), o[3].max(r[3])] });
    }
    b
}

/// Signed distance to a rounded box (as the shader's `box_d`).
fn box_d(lx: f32, ly: f32, w: f32, h: f32, r: f32) -> f32 {
    let rr = r.min(w.min(h) * 0.5);
    let qx = (lx - w * 0.5).abs() - (w * 0.5 - rr);
    let qy = (ly - h * 0.5).abs() - (h * 0.5 - rr);
    (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - rr
}

/// Device point (x, y) taken into instance `i`'s own box: the inverse of its transform about the box's
/// centre (`Instance::xf`), as the shader's `exact_i` does, so a turned or posed instance is sampled
/// where it is drawn.
pub fn to_box(i: &Instance, x: f32, y: f32) -> (f32, f32) {
    let r = i.rect;
    let x4 = i.xf;
    if x4 == [0.0; 4] { return (x, y); }
    let (cx, cy) = (r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
    let (a, b, c, d) = (1.0 + x4[0], x4[1], x4[2], 1.0 + x4[3]);
    let det = a * d - b * c;
    if det.abs() < 1e-6 { return (f32::NAN, f32::NAN); }
    let (qx, qy) = (x - cx, y - cy);
    (cx + (d * qx - b * qy) / det, cy + (a * qy - c * qx) / det)
}

/// How much one instance's true outline covers device point (x, y), whatever its colour: glyphs from
/// the atlas, shapes by their distance, pictures fully; turned and posed instances where they are drawn.
/// A fill in the ground's own tone is still an outline (a dialog's panel): it covers.
pub fn cover_at(i: &Instance, x: f32, y: f32, atlas: &[u8]) -> f32 { cover(i, x, y, atlas, false) }
/// `cover_at`, but only ink that shows against the ground: tone or accent at least 0.1 (what a morph
/// flies, since a tile in the ground's colour would be invisible).
pub fn ink_at(i: &Instance, x: f32, y: f32, atlas: &[u8]) -> f32 { cover(i, x, y, atlas, true) }
fn cover(i: &Instance, x: f32, y: f32, atlas: &[u8], visible: bool) -> f32 {
    if x < i.clip[0] || y < i.clip[1] || x >= i.clip[2] || y >= i.clip[3] { return 0.0; }
    let (x, y) = to_box(i, x, y);
    let r = i.rect;
    if !(x >= r[0] && y >= r[1] && x < r[0] + r[2] && y < r[1] + r[3]) { return 0.0; }
    let a = i.paint[2];
    if visible && i.paint[3] != PICTURE && i.paint[3] != MESH && i.paint[0].max(i.paint[1]) < 0.1 { return 0.0; }
    match i.paint[3] {
        k if k == GLYPH => {
            let (tx, ty) = ((i.uv[0] * ATLAS as f32).round() as i64 + (x - r[0]).floor() as i64, (i.uv[1] * ATLAS as f32).round() as i64 + (y - r[1]).floor() as i64);
            if tx < 0 || ty < 0 || tx >= ATLAS as i64 || ty >= ATLAS as i64 { return 0.0; }
            let o = ((ty * ATLAS as i64 + tx) * 2) as usize;
            // A weighted glyph moves its outline by `extra[0]` device px, as stage 1 draws it.
            let c = if i.extra[0] != 0.0 {
                atlas.get(o + 1).map_or(0.0, |&n| (0.5 + (n as f32 / 255.0 - 0.5) * 2.0 * SPREAD as f32 + i.extra[0]).clamp(0.0, 1.0))
            } else { atlas.get(o).map_or(0.0, |&c| c as f32 / 255.0) };
            c * a
        }
        // A 3D element counts as covering its box; its tiles show what it drew there.
        k if k == PICTURE || k == MESH => a,
        _ => {
            let (lx, ly) = (x - r[0], y - r[1]);
            let mut d = box_d(lx, ly, r[2], r[3], i.uv[0]);
            if i.extra[0] > 0.0 { d = (d + i.extra[0] * 0.5).abs() - i.extra[0] * 0.5; }
            (0.5 - d).clamp(0.0, 1.0) * a
        }
    }
}

/// Every tile of an element at tile size `tile` (device px) on the global lattice, over its ink bounds
/// widened by the near spread (so the tiles also carry the nearness around the ink). The cover of a
/// cell is the average of four points at ± a quarter tile from its centre, as dottr reads it.
/// Only ink that shows counts (`ink_at`): these are the tiles a morph flies.
pub fn cells(insts: &[Instance], tile: f32, atlas: &[u8]) -> Vec<Cell> { cells_by(insts, tile, atlas, true) }
/// The cells of an element's true outline, whatever its colours (`cover_at`): what a decode covers.
pub fn outline_cells(insts: &[Instance], tile: f32, atlas: &[u8]) -> Vec<Cell> { cells_by(insts, tile, atlas, false) }
fn cells_by(insts: &[Instance], tile: f32, atlas: &[u8], visible: bool) -> Vec<Cell> {
    let Some(b) = ink_bounds(insts) else { return vec![] };
    let s = SPREAD as f32;
    let (c0, r0) = (((b[0] - s) / tile).floor() as i64, ((b[1] - s) / tile).floor() as i64);
    let (c1, r1) = (((b[2] + s) / tile).ceil() as i64, ((b[3] + s) / tile).ceil() as i64);
    let (nc, nr) = ((c1 - c0).max(0) as usize, (r1 - r0).max(0) as usize);
    // Per sample point (four per cell), what is left uncovered: 1 − cover, multiplied over instances.
    let mut open = vec![1.0f32; nc * nr * 4];
    let q = [(-0.25, -0.25), (0.25, -0.25), (-0.25, 0.25), (0.25, 0.25)];
    for i in insts {
        let b = i.bounds(0.0);
        let r = [b[0], b[1], b[2] - b[0], b[3] - b[1]];
        let (ic0, ir0) = (((r[0] / tile).floor() as i64 - c0).max(0) as usize, ((r[1] / tile).floor() as i64 - r0).max(0) as usize);
        let (ic1, ir1) = ((((r[0] + r[2]) / tile).ceil() as i64 - c0).clamp(0, nc as i64) as usize, (((r[1] + r[3]) / tile).ceil() as i64 - r0).clamp(0, nr as i64) as usize);
        for cy in ir0..ir1 { for cx in ic0..ic1 {
            let (ox, oy) = ((cx as i64 + c0) as f32 * tile, (cy as i64 + r0) as f32 * tile);
            for (k, (dx, dy)) in q.iter().enumerate() {
                let c = cover(i, ox + tile * (0.5 + dx), oy + tile * (0.5 + dy), atlas, visible);
                if c > 0.0 { open[(cy * nc + cx) * 4 + k] *= 1.0 - c.min(1.0); }
            }
        } }
    }
    let mut out = Vec::with_capacity(nc * nr);
    for cy in 0..nr { for cx in 0..nc {
        let o = &open[(cy * nc + cx) * 4..(cy * nc + cx) * 4 + 4];
        let cover = o.iter().map(|v| 1.0 - v).sum::<f32>() * 0.25;
        out.push(Cell { x: (cx as i64 + c0) as f32 * tile, y: (cy as i64 + r0) as f32 * tile, cover, tag: NO_TAG });
    } }
    out
}

/// The tiles of one morph, and their size; per tile, the tags of its source and target cells.
pub struct Tiles { pub particles: Vec<Particle>, pub tile: f32, pub tags: Vec<[u32; 2]> }

/// Sample both elements on the lattice (`pitch` device px), pair the cells that fly, and give every
/// tile its run. The tile grows to a whole multiple of the lattice until there are at most `count_max`.
/// Times are seconds on the flight's clock.
pub fn tiles(src: &[Instance], dst: &[Instance], pitch: f32, spec: &MorphSpec, atlas: &[u8]) -> Tiles {
    tiles_with(|t| cells(src, t, atlas), |t| cells(dst, t, atlas), ink_bounds(src), ink_bounds(dst), pitch, spec)
}

/// `tiles` with each side's cells from a function of the tile size (a mesh side samples its surface).
pub fn tiles_with(mut sa: impl FnMut(f32) -> Vec<Cell>, mut sb: impl FnMut(f32) -> Vec<Cell>, ba: Option<Bounds>, bb: Option<Bounds>, pitch: f32, spec: &MorphSpec) -> Tiles {
    let mut m = 1.0f32;
    loop {
        let tile = pitch * m;
        let (a, b) = (sa(tile), sb(tile));
        let count = |cs: &[Cell]| cs.iter().filter(|c| c.cover >= spec.threshold).count();
        let (fa, fb) = (count(&a), count(&b));
        let total = fa.max(fb) + (a.len() - fa) + (b.len() - fb);
        if (total as f32) <= spec.count_max.max(1.0) || m > 64.0 {
            let (particles, tags) = build(&a, &b, ba, bb, tile, spec);
            return Tiles { particles, tile, tags };
        }
        // Jump close to the right multiple, then go up one at a time.
        m = (m * (total as f32 / spec.count_max.max(1.0)).sqrt()).ceil().max(m + 1.0);
    }
}

fn build(a: &[Cell], b: &[Cell], ba: Option<Bounds>, bb: Option<Bounds>, tile: f32, spec: &MorphSpec) -> (Vec<Particle>, Vec<[u32; 2]>) {
    let order = |cs: &[Cell], bd: Option<Bounds>| {
        let bd = bd.unwrap_or([0.0, 0.0, 1.0, 1.0]);
        let (w, h) = ((bd[2] - bd[0]).max(1.0), (bd[3] - bd[1]).max(1.0));
        let mut fly: Vec<(u32, Cell)> = cs.iter().filter(|c| c.cover >= spec.threshold)
            .map(|c| (morton((c.x + tile * 0.5 - bd[0]) / w, (c.y + tile * 0.5 - bd[1]) / h), *c)).collect();
        fly.sort_by_key(|(k, c)| (*k, c.y as i64, c.x as i64));
        fly.into_iter().map(|(_, c)| c).collect::<Vec<_>>()
    };
    let (fa, fb) = (order(a, ba), order(b, bb));
    let flight = spec.duration_ms / 1000.0;
    let dur = flight * (1.0 - spec.stagger.clamp(0.0, 0.9));
    // Departures: the side of the source facing the target leaves first, with some noise, so the swarm
    // peels off rather than moving as a block. Delays stay just short of the stagger so every tile has
    // landed when the flight is over.
    let centre = |bd: Option<Bounds>| bd.map_or([0.0, 0.0], |b| [(b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5]);
    let (ca, cb) = (centre(ba), centre(bb));
    let d = [cb[0] - ca[0], cb[1] - ca[1]];
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let reach = ba.map_or(1.0, |b| ((b[2] - b[0]).abs() + (b[3] - b[1]).abs()).max(1.0) * 0.5);
    let delay = |x: f32, y: f32, seed: f32| {
        let lead = if len > 20.0 { (0.5 - ((x - ca[0]) * d[0] + (y - ca[1]) * d[1]) / len / reach).clamp(0.0, 1.0) }
                   else { noise2(x / 90.0, y / 90.0)[0] * 0.5 + 0.5 };
        spec.stagger.clamp(0.0, 0.9) * flight * 0.95 * (0.6 * lead + 0.4 * seed).clamp(0.0, 1.0)
    };
    let seed = |n: usize| unit(hash(n as u32 ^ 0x9e37_79b9));
    let mut out = Vec::with_capacity(fa.len().max(fb.len()) + a.len() + b.len());
    let mut tags = Vec::with_capacity(out.capacity());
    for (n, (i, j, k)) in pair(fa.len(), fb.len()).into_iter().enumerate() {
        let (s, t) = (fa[i], fb[j]);
        let sd = seed(n);
        out.push(Particle { ab: [s.x, s.y, t.x, t.y], run: [0.0, 1.0, delay(s.x, s.y, sd), dur], misc: [k, tile, sd, 0.3 + 0.4 * seed(n + 7_777_777)] });
        tags.push([s.tag, t.tag]);
    }
    // Cells that do not fly, and every cell when one side has nothing that flies.
    let none = fa.is_empty() || fb.is_empty();
    let n0 = out.len();
    for (n, c) in a.iter().filter(|c| none || c.cover < spec.threshold).enumerate() {
        let sd = seed(n0 + n);
        out.push(Particle { ab: [c.x, c.y, c.x, c.y], run: [0.0, 1.0, delay(c.x, c.y, sd), dur], misc: [kind::SRC_DUST, tile, sd, 0.5] });
        tags.push([c.tag, NO_TAG]);
    }
    let n0 = out.len();
    for (n, c) in b.iter().filter(|c| none || c.cover < spec.threshold).enumerate() {
        let sd = seed(n0 + n);
        out.push(Particle { ab: [c.x, c.y, c.x, c.y], run: [0.0, 1.0, delay(c.x, c.y, sd), dur], misc: [kind::DST_DUST, tile, sd, 0.5] });
        tags.push([NO_TAG, c.tag]);
    }
    (out, tags)
}

/// Turn every tile around at `clock` (seconds into the current run): from where it is now back to the
/// other end, over `dur` seconds, all leaving at once.
pub fn reverse(ps: &mut [Particle], clock: f32, ease: Ease, dur: f32) {
    for p in ps {
        let u = progress(p, clock, ease);
        let to = if p.run[1] >= p.run[0] { 0.0 } else { 1.0 };
        p.run = [u, to, 0.0, dur];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_use_everyone() {
        for (ns, nt) in [(1, 1), (5, 5), (7, 3), (3, 7), (1, 9), (9, 1), (100, 37), (37, 100), (40000, 12345)] {
            let p = pair(ns, nt);
            assert_eq!(p.len(), ns.max(nt), "{ns} {nt}");
            let mut si = vec![0; ns]; let mut ti = vec![0; nt];
            for &(i, j, _) in &p { si[i] += 1; ti[j] += 1; }
            assert!(si.iter().all(|&c| c > 0) && ti.iter().all(|&c| c > 0), "{ns} {nt}: someone unused");
            // One primary per target (more sources) or per source (more targets); the rest are extras.
            let prim = p.iter().filter(|x| x.2 == kind::PRIMARY).count();
            assert_eq!(prim, ns.min(nt), "{ns} {nt}");
            assert_eq!(p, pair(ns, nt), "deterministic");
            // Ranks stay in order: the pairing does not cross itself.
            assert!(p.windows(2).all(|w| w[0].0 <= w[1].0 && w[0].1 <= w[1].1));
        }
        assert!(pair(0, 4).is_empty() && pair(4, 0).is_empty());
    }

    #[test]
    fn ease_ends() {
        for e in Ease::ALL {
            assert_eq!(e.at(0.0), 0.0);
            assert!((e.at(1.0) - 1.0).abs() < 1e-6, "{e:?}");
            assert!((0..=10).map(|i| e.at(i as f32 / 10.0)).collect::<Vec<_>>().windows(2).all(|w| w[1] >= w[0]));
        }
        assert!((Ease::InOutCubic.at(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn morton_orders_quadrants() {
        assert!(morton(0.1, 0.1) < morton(0.9, 0.1));
        assert!(morton(0.9, 0.1) < morton(0.1, 0.9));
        assert!(morton(0.1, 0.9) < morton(0.9, 0.9));
    }

    #[test]
    fn positions_are_exact_at_the_ends() {
        let p = Particle { ab: [10.0, 20.0, 300.0, 40.0], run: [0.0, 1.0, 0.1, 0.5], misc: [0.0, 5.0, 0.3, 0.5] };
        assert_eq!(position(&p, 0.0, 48.0), [10.0, 20.0]);
        assert_eq!(position(&p, 1.0, 48.0), [300.0, 40.0]);
        let mid = position(&p, 0.5, 48.0);
        // It swings off the straight line.
        assert!((mid[1] - 30.0).abs() > 10.0, "{mid:?}");
        assert_eq!(progress(&p, 0.05, Ease::Linear), 0.0);
        assert_eq!(progress(&p, 0.6, Ease::Linear), 1.0);
    }

    #[test]
    fn reversal_starts_where_the_tiles_are() {
        let mut ps = vec![
            Particle { ab: [0.0, 0.0, 200.0, 0.0], run: [0.0, 1.0, 0.0, 0.6], misc: [0.0, 5.0, 0.1, 0.5] },
            Particle { ab: [5.0, 0.0, 205.0, 50.0], run: [0.0, 1.0, 0.2, 0.6], misc: [1.0, 5.0, 0.7, 0.5] },
        ];
        let clock = 0.35;
        let before: Vec<[f32; 2]> = ps.iter().map(|p| position(p, progress(p, clock, Ease::OutExpo), 48.0)).collect();
        reverse(&mut ps, clock, Ease::OutExpo, 0.5);
        let after: Vec<[f32; 2]> = ps.iter().map(|p| position(p, progress(p, 0.0, Ease::OutExpo), 48.0)).collect();
        assert_eq!(before, after);
        // And they end back at their sources.
        for p in &ps { assert_eq!(position(p, progress(p, 0.5, Ease::OutExpo), 48.0), [p.ab[0], p.ab[1]]); }
        // A second reversal goes forward again.
        reverse(&mut ps, 0.2, Ease::OutExpo, 0.5);
        for p in &ps { assert_eq!(p.run[1], 1.0); }
    }

    #[test]
    fn a_gpu_turn_composes_any_number_of_times() {
        // One tile's progress followed leg by leg must match the closed form at every turn.
        let (delay, fwd, ease) = (0.1, 0.6, Ease::InOutCubic);
        let mut t = Turn { first: 0.35, alpha: 1.0, beta: 0.0, to: 0.0, dur: 0.4 };
        let mut u = ease.at((0.35 - delay) / fwd);
        assert!((t.progress(delay, fwd, 0.0, ease) - u).abs() < 1e-6);
        for leg in 0..12 {
            let c = 0.05 + 0.03 * leg as f32;
            let at = u + (t.to - u) * ease.at(c / t.dur);
            assert!((t.progress(delay, fwd, c, ease) - at).abs() < 1e-5, "leg {leg}");
            let next = t.again(c, ease, 0.4);
            // The next leg starts where this one was, and heads the other way.
            assert!((next.progress(delay, fwd, 0.0, ease) - at).abs() < 1e-5, "leg {leg}");
            assert_eq!(next.to, 1.0 - t.to);
            (t, u) = (next, at);
        }
        // Run to the end of a leg: exactly the end.
        assert!((t.progress(delay, fwd, t.dur, ease) - t.to).abs() < 1e-6);
    }

    #[test]
    fn cap_raises_the_tile_to_a_multiple() {
        // A 400 × 400 opaque picture at pitch 2: 40 000 cells and more, so the tile doubles or more.
        let pic = Instance { rect: [0.0, 0.0, 400.0, 400.0], uv: [0.0, 0.0, 1.0, 1.0], paint: [1.0, 0.0, 1.0, PICTURE], clip: [-1e5, -1e5, 1e5, 1e5], extra: [0.0; 4], look: [0.0; 4], resp: [0.0; 4], xf: [0.0; 4] };
        let spec = MorphSpec { count_max: 10000.0, ..MorphSpec::default() };
        let t = tiles(&[pic], &[pic], 2.0, &spec, &[]);
        assert!(t.particles.len() <= 10000, "{}", t.particles.len());
        assert_eq!(t.tile % 2.0, 0.0);
        assert!(t.tile >= 4.0);
        // Every picture cell flies; only the spread band around it is dust.
        assert!(t.particles.iter().filter(|p| p.misc[0] == kind::PRIMARY).count() >= (400.0 / t.tile).powi(2) as usize);
    }

    #[test]
    fn the_gpu_grid_covers_both_elements_in_whole_pixels() {
        let (a, b): (Bounds, Bounds) = ([10.3, 20.0, 410.0, 220.7], [600.0, 40.0, 700.0, 500.0]);
        for cap in [1000.0, 50_000.0, 1e6] {
            let g = Grid::new(a, b, 1.0, cap);
            assert!(g.count() as f32 <= cap, "{} > {cap}", g.count());
            assert_eq!(g.a[2].fract(), 0.0);
            assert_eq!(g.a[2], g.b[2]);
            for (o, bd) in [(g.a, a), (g.b, b)] {
                // The grid starts on a whole pixel and reaches past the ink and its nearness.
                assert_eq!(o[0].fract(), 0.0);
                assert!(o[0] <= bd[0] - SPREAD as f32 && o[0] + g.cols as f32 * o[2] >= bd[2] + SPREAD as f32);
                assert!(o[1] <= bd[1] - SPREAD as f32 && o[1] + g.rows as f32 * o[3] >= bd[3] + SPREAD as f32);
            }
        }
        // Never finer than the lattice.
        assert!(Grid::new(a, b, 5.0, 1e9).a[2] >= 5.0);
        assert!(Grid::estimate(a, b, 1.0) > 90_000.0 && Grid::estimate(a, b, 2.0) < 30_000.0);
    }
}
