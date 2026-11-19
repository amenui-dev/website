pub const GATHER: f32 = 1.0;      // 0 is this listing, 1 the body
pub const MATERIAL: Material = Material::Plain;
pub const TURN: bool = false;
pub const SEED: u32 = 26;
// Body (the home page, `/`): a stone whose surface is the text of this file, set in lines round it
// like latitude rings, lit by a lamp that follows the pointer. Written only against amenui's public
// API: text elements whose glyphs are posed (`El::pose`), drawings for the dots behind and over it,
// springs for the motion.
//
// - The four lines above are the console: in 02 they close into four controls where they stand.
//   GATHER scrubs this listing onto the stone and back, MATERIAL draws one word in a material (03),
//   TURN lets the stone turn by itself, SEED cuts another stone. The scene starts from their values.
// - The text is the first 2,700 characters of this file, ending at a line end (1,400 on a small body).
// - The listing fills a stage rectangle the host gives (`set_stage`): 14 px lines, as many of the
//   text's lines as fit above the caption, each cut at the last whole word that fits. The web page
//   draws the same listing in HTML while the module loads, at the same pixels.
// - Glyph i of the text under the console sits in one cell of a spiral on the surface; spaces take a
//   cell. The glyph size is the one at which the text fills the surface.
// - The lamp is a point light: opacity keeps the text legible everywhere, the colour carries the light.
// - A drag turns the stone about its upright axis, 1:1 at its middle, and tips it a little; let go,
//   it settles upright. At rest nothing is drawn.
use amenui::draw::Drawing;
use amenui::el::{El, Font, GlyphAt, Key, Paint, Phase, Point, Pose, Reveal, Role, canvas, layers, space, text_lh};
use amenui::experiment::Experiment;
use amenui::field::hash;
use amenui::form::FormFx;
use amenui::gesture::{Animated, Spring};
use amenui::style::Material;
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
/// On a phone the text is cut to what the body can show in glyphs drawn at least `PHONE_GLYPH` px tall
/// where they face the viewer: from the small body's count down, a line at a time.
pub const CHARS_PHONE: usize = CHARS_SMALL;
pub const PHONE_GLYPH: f32 = 11.0;
/// How tall the listing's face draws a glyph for its em (its ink, as the audit measures it).
pub const GLYPH_INK: f32 = 0.65;
/// The listing's line height, the gap between its last line and the caption, and the line numbers'
/// gutter (px). The line height never changes: a shorter stage shows fewer lines.
pub const LINE: f32 = 14.0;
pub const CAPTION_GAP: f32 = 16.0;
pub const GUTTER: f32 = 28.0;
/// The short path: after the first frame the listing waits this long (s), then gathers; the front
/// across the listing starts this long after the first frame (s), from the caret, at this speed.
pub const INTRO_DELAY: f32 = 0.5;
pub const FRONT_DELAY: f32 = 0.15;
pub const FRONT_SPEED: f32 = 1_400.0;
/// The widest the home page lays out (px); a wider window centres it.
pub const FRAME: f32 = 1_600.0;
/// With reduced motion a change of state is an opacity change this long (s): out, then in.
pub const FADE: f32 = 0.15;
pub const REST_LIGHT: [f32; 3] = [-0.9, 0.9, 1.6];
/// The opening view: the turn and the upright pitch the stone settles to.
pub const REST_YAW: f32 = -0.4;
pub const REST_PITCH: f32 = -0.22;
/// The surface round the point under the pointer rises (within this distance, body units); a click's
/// ring crosses the surface at this speed (units/s), this wide.
pub const BULGE: f32 = 0.35;
pub const RING_SPEED: f32 = 1.6;
pub const RING_WIDTH: f32 = 0.12;
/// The hand: a drag starts after this travel (px); a throw is measured over the last 80 ms (none if
/// the pointer rested 50 ms), at most 6 rad/s, and slows with this time constant (s); the tilt gives
/// up to 20° and springs back.
pub const DRAG_START: f32 = 3.0;
pub const THROW_TAU: f32 = 0.45;
pub const THROW_MAX: f32 = 6.0;
pub const TILT: f32 = 0.35;
/// With TURN on: the idle turn (rad/s) comes back this long after the last input (s), easing in over a
/// second, and eases out after this long without input (s); an arrow key turns the stone 15°.
pub const OMEGA: f32 = 0.16;
pub const IDLE_AFTER: f32 = 2.0;
pub const IDLE_QUIET: f32 = 20.0;
pub const KEY_TURN: f32 = 0.261_799_4;
/// The walk's arrival: the stone turns this far (rad) and eases to a stop over this long (s).
pub const ARRIVE: (f32, f32) = (0.5, 1.6);
/// The console: at most this wide (px), rows of two listing lines (three on a phone).
pub const CONSOLE_W: f32 = 644.0;
pub const ROW: f32 = 28.0;
pub const ROW_PHONE: f32 = 44.0;
/// The materials the console offers, the one 03 opens in from the rail, and the walk's.
pub const MATERIALS: [(&str, Material); 4] = [("plain", Material::Plain), ("near", Material::Near), ("dottr", Material::Dottr), ("carvr", Material::Carvr)];
pub const REST_MATERIAL: usize = 2;
/// A material draws what the plain page does not where an element is loose: the word is held this
/// loose, still (it is a picture of its name, not text to read).
pub const WORD_LOOSE: f32 = 0.7;
/// The morphs' cells: at most this many (desktop, phone), each flying once this much of it is ink,
/// departures spread over this share of the flight.
pub const MORPH_CELLS: (f32, f32) = (12_000.0, 6_000.0);
pub const MORPH_THRESHOLD: f32 = 0.02;
pub const MORPH_STAGGER: f32 = 0.15;
/// The morphs (loosen, flight, tighten in ms): the head lines into the console, and the listing into
/// the word or the stone.
pub const MORPH_CONSOLE: (f32, f32, f32) = (120.0, 200.0, 100.0);
pub const MORPH_SHORT: (f32, f32, f32) = (160.0, 560.0, 220.0);
/// The intro's version: the page remembers it once the landing state is reached or the intro skipped.
pub const INTRO_VERSION: u32 = 1;
/// How the text is drawn on the form: nothing turned away, glyphs fading out from facing 0.5 (gone at
/// 0.3), a glyph more than a third covered by nearer ones left out, the far side dimmer.
pub const FORM_FX: FormFx = FormFx { back: 0.0, edge: 0.5, overlap: 0.35, fog: 0.6 };
/// The hairline colour (rules, frames, the console's line numbers).
pub const HAIR: Paint = Paint::FAINT;

/// One cell of the surface: where (unit body space), its outward normal, the line's tangent there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfacePoint { pub p: [f32; 3], pub n: [f32; 3], pub t: [f32; 3], pub k: f32 }
/// A mark of the volume: where (unit space), its normal-ish direction, its size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VolumePoint { pub p: [f32; 3], pub n: [f32; 3], pub size: f32 }

/// What of `SOURCE` the body can be made of: its whole lines to the first line end past `CHARS`
/// characters, copied when the module is compiled, so that the module carries these and not the rest.
const SHOWN: &str = {
    const fn cut(s: &str, most: usize) -> usize {
        let b = s.as_bytes();
        let (mut i, mut n) = (0, 0);
        while i < b.len() {
            if b[i] & 0xc0 != 0x80 { n += 1; }
            if n > most && b[i] == b'\n' { return i; }
            i += 1;
        }
        b.len()
    }
    const N: usize = cut(SOURCE, CHARS);
    const fn head<const N: usize>(s: &str) -> [u8; N] { let mut out = [0u8; N]; let b = s.as_bytes(); let mut i = 0; while i < N { out[i] = b[i]; i += 1; } out }
    const B: [u8; N] = head::<N>(SOURCE);
    match std::str::from_utf8(&B) { Ok(s) => s, Err(_) => panic!("cut inside a character") }
};
/// The text: the listing (the whole lines of `SOURCE` in its first `chars` characters), how many of
/// its first lines the console closes (`head`), and for each glyph of the lines under them, the cell
/// it sits in on the stone. A run of white space takes one cell; `cells` is how many the stone's text
/// takes, `all` how many the whole listing would. `mid` gives each glyph the middle glyph of its word.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stream { pub listing: String, pub lines: usize, pub head: usize, pub cell: Vec<usize>, pub cells: usize, pub all: usize, pub mid: Vec<usize> }
pub fn stream(chars: usize, head: usize) -> Stream {
    let (mut listing, mut cell, mut lines) = (String::new(), vec![], 0usize);
    let (mut at, mut all) = (0usize, 0usize);
    for line in SHOWN.lines() {
        let line = line.trim_end();
        if listing.chars().count() + (lines > 0) as usize + line.chars().count() > chars { break; }
        let mine = lines >= head;
        // Cells of the whole text (for its count) and of the stone's (the lines under the console).
        let (mut k, mut j, mut gap) = (at, all, lines > 0);
        let mut add = vec![];
        for c in line.chars() {
            if c.is_whitespace() { gap = true; continue; }
            if gap && j > 0 { j += 1; }
            if gap && k > 0 && mine { k += 1; }
            gap = false;
            if mine { add.push(k); k += 1; }
            j += 1;
        }
        if lines > 0 { listing.push('\n'); }
        listing.push_str(line);
        cell.extend(add);
        (at, all) = (k, j);
        lines += 1;
    }
    let mut mid = vec![0; cell.len()];
    let mut a = 0;
    for i in 1..=cell.len() {
        if i == cell.len() || cell[i] != cell[i - 1] + 1 { for m in &mut mid[a..i] { *m = (a + i - 1) / 2; } a = i; }
    }
    Stream { listing, lines, head: head.min(lines), cell, cells: at, all, mid }
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

/// The listing as a stage shows it, its first `rows` lines cut to `cols` cells: the console's lines
/// (`head`) and the lines under them (`text`), the stone's glyphs these show (`map`) and those they
/// leave out (`rest_map`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shown { pub head: String, pub text: String, pub rows: usize, pub map: Vec<usize>, pub rest_map: Vec<usize> }
pub fn shown(s: &Stream, rows: usize, cols: usize) -> Shown {
    let mut out = Shown::default();
    let mut g = 0;
    for (l, line) in s.listing.split('\n').enumerate() {
        let keep = if l < rows { cut(line, cols).len() } else { 0 };
        if l < s.head {
            if l > 0 { out.head.push('\n'); }
            out.head.push_str(&line[..keep]);
            continue;
        }
        if l < rows {
            if l > s.head { out.text.push('\n'); }
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
/// What the poses and the drawings share across frames: the cells for (seed, chars, head), the volume
/// for (seed, count), the text.
#[derive(Default)]
pub struct Cache { pub key: (u32, u32, u32), pub pts: Vec<SurfacePoint>, pub radius: f32, pub em: f32, pub vkey: (u32, usize), pub vol: Vec<VolumePoint>, pub text: Stream }

/// A mote shed from the rim while the body turns: where (window px), velocity, age (s).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mote { pub x: f32, pub y: f32, pub vx: f32, pub vy: f32, pub age: f32 }

/// How the window is laid out round the text block: wide (≥ 1080 px), middle, narrow (< 720 px).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width { Wide, Middle, Narrow }

/// How the console's controls are set: two rows (wide), three (a narrow stage), three tall rows (a phone).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit { Wide, Narrow, Phone }
/// The console's box (window px) and how many of the listing's lines it closes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Console { pub x: f32, pub y: f32, pub w: f32, pub row: f32, pub rows: usize, pub lines: usize, pub fit: Fit }
impl Console {
    pub fn h(&self) -> f32 { self.row * self.rows as f32 }
    pub fn rect(&self) -> [f32; 4] { [self.x, self.y, self.x + self.w, self.y + self.h()] }
    fn row_y(&self, i: usize) -> f32 { self.y + self.row * i as f32 }
    /// Row A, GATHER: its box (the whole row takes the pointer) and its track's ends.
    pub fn gather(&self) -> [f32; 4] { [self.x, self.y, self.x + self.w, self.y + self.row] }
    pub fn track(&self) -> (f32, f32) { (self.x + 10.0 * ADVANCE, self.x + self.w - 48.0) }
    /// MATERIAL's segment `i`: eight cells (56 × 20 px), or a quarter of a narrower console.
    pub fn segment(&self, i: usize) -> [f32; 4] {
        let sw = if self.w < 320.0 { (self.w / 4.0).floor() } else { 8.0 * ADVANCE };
        let x0 = if self.fit == Fit::Wide { self.x + 10.0 * ADVANCE } else { self.x };
        let y = self.row_y(1) + ((self.row - 20.0) * 0.5).round();
        let x = x0 + sw * i as f32;
        [x, y, x + sw, y + 20.0]
    }
    fn last_row(&self) -> f32 { if self.fit == Fit::Wide { self.row_y(1) } else { self.row_y(2) } }
    /// SEED's button, right-aligned to the console: `SEED 26 →` with 8 px each side, 22 px high.
    pub fn seed(&self) -> [f32; 4] {
        let w = 9.0 * ADVANCE + 16.0;
        let y = self.last_row() + ((self.row - 22.0) * 0.5).round();
        [self.x + self.w - w, y, self.x + self.w, y + 22.0]
    }
    /// TURN: its label and frame (the switch's box), and the 28 × 14 frame alone.
    pub fn turn(&self) -> [f32; 4] {
        let x = if self.fit == Fit::Wide { self.seed()[0] - 16.0 - 28.0 - 8.0 - 4.0 * ADVANCE } else { self.x };
        let y = self.last_row();
        [x, y, x + 4.0 * ADVANCE + 8.0 + 28.0, y + self.row]
    }
    pub fn turn_frame(&self) -> [f32; 4] { let t = self.turn(); let y = t[1] + ((self.row - 14.0) * 0.5).round(); [t[2] - 28.0, y, t[2], y + 14.0] }
    /// The foot the stone's room starts 16 px under: on a phone in 04 only row A stays.
    pub fn foot(&self) -> f32 { if self.fit == Fit::Phone { self.y + self.row } else { self.y + self.h() } }
}

/// A control of the console, for the hover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ctl { Gather, Material(usize), Turn, Seed }

#[derive(Clone)]
pub struct Body {
    /// The console's values: the seed, whether the stone turns by itself, the material of the word.
    pub seed: u32, pub turn: bool, pub material: usize,
    /// The spring between the listing (0) and the stone (1); the thumb is held (`scrub`).
    pub gather: Animated, pub scrub: Option<f32>,
    /// The turn and the tilt; a throw's rate (rad/s, slowing by `THROW_TAU`); the tilt's spring back
    /// upright; the idle turn's share (0..1), its direction and the last input it waits for.
    pub yaw: f32, pub pitch: f32, pub spin: f32, pub tilt: Animated, pub idle: f32, pub idle_dir: f32, pub last_input: f32,
    /// A turn eased from one view to another: an arrow key, `0`, the walk's arrival.
    pub ease: Option<Ease>,
    /// The tab is hidden, or less than half the stage is in view: nothing turns by itself.
    pub away: bool,
    /// The pointer over the picture (window px) and the lamp following it (view space).
    pub ptr: Option<(f32, f32)>, pub lamp: [Animated; 3],
    /// The glyph of the text drawn under the pointer (its number among the stone's glyphs).
    pub under: Option<usize>,
    /// The pointer is over the body; the point under it (body space) and the bulge's strength; the
    /// rings sent by clicks (body-space origin, start).
    pub over: bool, pub touch_at: Option<[f32; 3]>, pub bulge: Animated, pub rings: Vec<([f32; 3], f32)>,
    grab: Option<Grab>, dragged: bool, clicked: f32, samples: Vec<(f32, f32)>,
    pub motes: Vec<Mote>, made: u32,
    /// The control under the pointer; when TURN last changed and SEED was last pressed (their motion).
    pub hover: Option<Ctl>, pub turned_at: f32, pub flash: f32,
    /// The window (logical px), and the stage (x0, y0, x1, y1) the host's page lays out for the scene.
    pub size: (f32, f32), pub stage: Option<[f32; 4]>,
    /// The optical centre of the page's text block (window px, vertical); none for the site's grid.
    pub text_centre: Option<f32>,
    pub entered: f32,
    pub reduced: bool,
    /// The palette: 0 the site's, 1 and 2 the proposed dark and light (AA.9, AA.10), for review.
    pub palette: u8,
    /// The short path or the walk is to play: waiting for the first frame to be presented, then at a time.
    pub intro_pending: bool, pub drawn: bool, pub intro_at: Option<f32>,
    /// The front across the listing on the short path: when, from where (`take_front`), when it started.
    pub front_at: Option<f32>, pub front: Option<(f32, f32)>, pub fronted: Option<f32>,
    /// The state (1 source, 2 interface, 3 material, 4 body), the one before it, and when the morphs
    /// between them land (the stone's drawings wait for them).
    pub at: u8, pub was: u8, pub lands: f32,
    /// The console opens into its lines once the gather has landed at the listing (04 → 01).
    pub unfold: bool,
    /// The intro (the full walk): when it started and the steps still to come; a person took over.
    pub walk: Option<(f32, Vec<(f32, Step)>)>, pub taken: bool,
    /// The first frame plays the full walk; it has been seen; steps at a time outside the walk; the
    /// gather's value to jump to once a morph lands.
    pub full: bool, pub seen: bool, pub later: Vec<(f32, Step)>, pub settle: Option<(f32, f32)>,
    /// What the scene asks the runtime to do (morphs, presses, focus, fronts, commits).
    pub cmds: Vec<amenui::morph::Cmd>,
    /// With reduced motion, a change of state under way: (start, length).
    pub fade: Option<(f32, f32)>,
    /// When the state last changed (the caption's dimming counts from it).
    pub changed: f32,
    pub t: f32,
    /// How ready the renderer is (`Ui::readiness`); the frames of the first morph in the walk.
    pub ready: f32, pub flight: (u32, f32),
    cache: Rc<RefCell<Cache>>,
}
/// A press on the stone: where, where the drag started (after `DRAG_START`), the view then.
#[derive(Clone, Copy, Debug)]
struct Grab { x: f32, y: f32, t: f32, sx: f32, sy: f32, yaw: f32, pitch: f32, on: Option<[f32; 3]> }
/// A turn eased over time: start, length (s), from and to (yaw, pitch); `out` eases out only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ease { pub t0: f32, pub d: f32, pub from: (f32, f32), pub to: (f32, f32), pub out: bool }
impl Ease {
    fn at(&self, now: f32) -> (f32, f32, bool) {
        let u = ((now - self.t0) / self.d).clamp(0.0, 1.0);
        let e = if self.out { 1.0 - (1.0 - u).powi(3) } else if u < 0.5 { 4.0 * u * u * u } else { 1.0 - (-2.0 * u + 2.0).powi(3) * 0.5 };
        (self.from.0 + (self.to.0 - self.from.0) * e, self.from.1 + (self.to.1 - self.from.1) * e, u >= 1.0)
    }
}
/// The lamp's spring: it trails the hand a little.
const LAMP: Spring = Spring { stiffness: 90.0, damping: 18.0, rest_distance: 0.001, rest_speed: 0.005, mass: 1.0 };
/// The gather spring: about 0.9 s from the listing to the body.
const GATHER_SPRING: Spring = Spring { stiffness: 30.0, damping: 11.0, rest_distance: 0.002, rest_speed: 0.01, mass: 1.0 };
/// The tilt's spring back upright: critically damped, home in about 380 ms.
const TILT_SPRING: Spring = Spring { stiffness: 170.0, damping: 26.0, rest_distance: 0.0005, rest_speed: 0.002, mass: 1.0 };
impl Default for Body {
    fn default() -> Self {
        let lamp = |v: f32| Animated::new(v, LAMP);
        let at = if GATHER >= 0.5 { 4 } else { 1 };
        Self {
            seed: SEED, turn: TURN, material: MATERIALS.iter().position(|m| m.1 == MATERIAL).unwrap_or(0),
            gather: Animated::new(GATHER, GATHER_SPRING), scrub: None,
            yaw: REST_YAW, pitch: REST_PITCH, spin: 0.0, tilt: Animated::new(REST_PITCH, TILT_SPRING), idle: 0.0, idle_dir: 1.0, last_input: f32::NEG_INFINITY,
            ease: None, away: false,
            ptr: None, under: None, over: false, touch_at: None, bulge: Animated::new(0.0, LAMP), rings: vec![], lamp: [lamp(REST_LIGHT[0]), lamp(REST_LIGHT[1]), lamp(REST_LIGHT[2])],
            grab: None, dragged: false, clicked: f32::NEG_INFINITY, samples: vec![], motes: vec![], made: 0,
            hover: None, turned_at: f32::NEG_INFINITY, flash: f32::NEG_INFINITY,
            at, was: at, lands: 0.0, unfold: false, walk: None, taken: false, cmds: vec![], full: false, seen: false, later: vec![], settle: None,
            size: (1440.0, 900.0), stage: None, text_centre: None, entered: 0.0, reduced: false, palette: 0, intro_pending: false, drawn: false, intro_at: None, front_at: None, front: None, fronted: None, fade: None, changed: 0.0,
            t: 0.0, ready: 1.0, flight: (0, 0.0), cache: Rc::default(),
        }
    }
}
/// How much larger than its mean radius the body's silhouette is on screen (perspective, relief).
pub const SILHOUETTE: f32 = 1.12;

/// `Show(n)` state n; `Scrub(phase, v)` the thumb held (0 press, 1 move, 2 release); `GatherTo(v)` a
/// key on the slider; `Material(i)`, `Turn`, `Seed(d)` the console; `Play` the walk; `Skip` to the
/// body; `Away` the stage out of view.
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>), Key(Key), Show(u8), Scrub(u8, f32), GatherTo(f32), Material(usize), Turn, Seed(i32), Play, Skip, Away(bool) }
/// One step of the intro.
#[derive(Clone, Debug, PartialEq)]
pub enum Step { State(u8), Focus(&'static str), Press(&'static str), Via, Lamp, Arrive, End }
impl Step {
    /// A beat that only shows something on the way (a focus, a material's press).
    fn on_the_way(&self) -> bool { matches!(self, Step::Focus(_)) || matches!(self, Step::Press(k) if k.starts_with("material:")) }
}
/// A beat whose frame comes later than this after it (s) is dropped when it is only on the way.
const LATE: f32 = 0.12;
/// How the walk ends (`Body::land`).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Ending { Played, GaveWay, Dropped, Skipped }

/// The key the scene is drawn under (its morphs and the operator's presses name its parts by it).
pub const KEY: &str = "body";
/// The scene's words: English whatever the interface language.
fn tr(key: &str) -> String { crate::install_catalogs(); amenui::i18n::Locale::En.tr(key) }


// ── The stone ────────────────────────────────────────────────────────────────────────────────────

fn noise3(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let (xi, yi, zi) = (x.floor(), y.floor(), z.floor());
    let (fx, fy, fz) = (x - xi, y - yi, z - zi);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let h = |i: f32, j: f32, k: f32| hash(seed ^ (i as i32 as u32).wrapping_mul(73_856_093) ^ (j as i32 as u32).wrapping_mul(19_349_663) ^ (k as i32 as u32).wrapping_mul(83_492_791));
    let l = |a: f32, b: f32, t: f32| a + (b - a) * s(t);
    let c = |dz: f32| l(l(h(xi, yi, zi + dz), h(xi + 1.0, yi, zi + dz), fx), l(h(xi, yi + 1.0, zi + dz), h(xi + 1.0, yi + 1.0, zi + dz), fx), fy);
    l(c(0.0), c(1.0), fz) * 2.0 - 1.0
}
/// The stone's five facets: (normal, offset) of each cutting plane.
pub fn facets(seed: u32) -> [([f32; 3], f32); 5] {
    std::array::from_fn(|f| {
        let f = f as u32;
        let a = |k: u32| hash(seed.wrapping_mul(977) ^ (f * 5 + k)) * 2.0 - 1.0;
        (norm([a(0), a(1) * 0.7, a(2)]), 0.62 + 0.12 * (a(3) * 0.5 + 0.5))
    })
}
/// The signed distance to the stone (unit space, about radius 0.86): low-frequency relief, a middle
/// octave, and five flat facets cut across it.
pub fn sdf(seed: u32, p: [f32; 3]) -> f32 {
    let len = dot3(p, p).sqrt();
    let q = [p[0] * 1.15 + 11.0, p[1] * 1.15 + 3.0, p[2] * 1.15 + 7.0];
    let d = 0.18 * noise3(q[0], q[1], q[2], seed) + 0.10 * noise3(q[0] * 2.4, q[1] * 2.4, q[2] * 2.4, seed ^ 0x55);
    let mut r = len - 0.86 - d;
    for (nrm, off) in facets(seed) { r = r.max(dot3(p, nrm) - off); }
    r
}
fn norm(v: [f32; 3]) -> [f32; 3] { let l = dot3(v, v).sqrt().max(1e-6); [v[0] / l, v[1] / l, v[2] / l] }
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn gradient(seed: u32, p: [f32; 3]) -> [f32; 3] {
    let e = 1e-3;
    let g = |a: usize| { let mut q = p; q[a] += e; let mut w = p; w[a] -= e; sdf(seed, q) - sdf(seed, w) };
    norm([g(0), g(1), g(2)])
}

/// Where the surface is along `dir`, marched in from radius 2, with its normal.
fn on_surface(seed: u32, dir: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let mut r = 2.0f32;
    for _ in 0..56 {
        let d = sdf(seed, [dir[0] * r, dir[1] * r, dir[2] * r]);
        if d.abs() < 1e-3 { break; }
        r = (r - d * 0.85).max(0.05);
    }
    let p = [dir[0] * r, dir[1] * r, dir[2] * r];
    (p, gradient(seed, p))
}
/// The stone's mean radius (unit space), from a few directions.
pub fn mean_radius(seed: u32) -> f32 {
    let n = 48;
    (0..n).map(|i| {
        let z = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
        let a = i as f32 * 2.399_963;
        let s = (1.0 - z * z).sqrt();
        let (p, _) = on_surface(seed, [s * a.cos(), z, s * a.sin()]);
        dot3(p, p).sqrt()
    }).sum::<f32>() / n as f32
}

/// How many cells the spiral has for a step and a pitch (laid on the sphere of directions).
pub fn spiral_count(step: f32, pitch: f32) -> usize {
    let tau = std::f32::consts::TAU;
    let (mut th, mut n) = (pitch * 0.5, 0);
    while th < std::f32::consts::PI - pitch * 0.25 { n += 1; th += step / (1.0 + (tau / pitch * th.sin()).powi(2)).sqrt(); }
    n
}
/// The largest glyph size (units of the mean radius) whose cells (0.62 × 1.45 em) hold `cells` cells.
pub fn em_to_fill(cells: usize) -> f32 {
    let (mut lo, mut hi) = (0.02f32, 0.25f32);
    for _ in 0..24 { let m = (lo + hi) * 0.5; if spiral_count(0.62 * m, 1.45 * m) >= cells { lo = m; } else { hi = m; } }
    lo
}

/// The cells: one spiral from the top to the bottom whose turns are `pitch` apart and whose cells are
/// `step` apart along it, so lines of text run round the stone like latitude rings.
pub fn cells(seed: u32, step: f32, pitch: f32) -> Vec<SurfacePoint> {
    let tau = std::f32::consts::TAU;
    let mut th = pitch * 0.5;
    let mut out = vec![];
    while th < std::f32::consts::PI - pitch * 0.25 {
        let ph = -tau * th / pitch;
        let dir = [th.sin() * ph.cos(), th.cos(), th.sin() * ph.sin()];
        let (p, nrm) = on_surface(seed, dir);
        let t = norm(cross([0.0, 1.0, 0.0], nrm));
        let t = if t[0].is_finite() && (t[0].abs() + t[2].abs()) > 1e-3 { t } else { [1.0, 0.0, 0.0] };
        out.push(SurfacePoint { p, n: nrm, t, k: 1.0 });
        th += step / (1.0 + (tau / pitch * th.sin()).powi(2)).sqrt();
    }
    // The line's real direction and spacing on the stone (a facet bends and draws the cells together).
    let mean = mean_radius(seed);
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
pub fn volume(seed: u32, n: usize) -> Vec<VolumePoint> {
    (0..n as u32).map(|i| {
        let u = hash(i.wrapping_mul(2_654_435_761) ^ seed);
        let v = hash(i.wrapping_mul(40_503) ^ seed ^ 0x51);
        let z = 1.0 - 2.0 * u;
        let a = v * std::f32::consts::TAU;
        let s = (1.0 - z * z).max(0.0).sqrt();
        let (p, nrm) = on_surface(seed, [s * a.cos(), z, s * a.sin()]);
        let inside = hash(i ^ 0x7777) < 0.34;
        let depth = if inside { 0.3 + 0.62 * hash(i ^ 0x1234).powf(0.6) } else { 0.99 };
        VolumePoint { p: [p[0] * depth, p[1] * depth, p[2] * depth], n: nrm, size: 0.55 + 0.8 * hash(i ^ 0x99) }
    }).collect()
}

/// The view of the stone: its turn, where its centre is and its scale (px per unit).
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
/// The paint of a glyph lit `l`. The site's palette: the accent up to 0.5, then towards the ink. The
/// proposed one (`neutral`): from dim to ink by the light, the accent only above 0.8 and at most 35 %
/// of the way to it. The opacity has a high floor and a short range: it keeps the text legible.
pub fn lit_paint(l: f32, neutral: bool) -> Paint {
    if neutral { return Paint { tone: 0.66 + 0.34 * l, accent: 0.35 * ((l - 0.8) / 0.2).clamp(0.0, 1.0), alpha: 0.70 + 0.30 * l, hue: 0 }; }
    let accent = if l <= 0.5 { 1.0 } else { 1.0 - 0.85 * (l - 0.5) / 0.5 };
    Paint { tone: 1.0, accent, alpha: 0.70 + 0.30 * l, hue: 0 }
}

/// How a cell's glyph looks on the stone: its lit paint, how it faces the viewer, its opacity as drawn,
/// and how much the line's tangent (`run`) and the line's height across it (`gap`) are foreshortened.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellLook { pub paint: Paint, pub alpha: f32, pub run: f32, pub gap: f32, pub facing: f32 }
pub fn cell_look(v: &View, light: [f32; 3], sp: &SurfacePoint, boost: f32, neutral: bool) -> CellLook {
    let (p, n, t) = (v.rot(sp.p), v.rot(sp.n), v.rot(sp.t));
    let paint = lit_paint((lit(light, p, n) + boost).min(1.0), neutral);
    let run = (t[0] * t[0] + t[1] * t[1]).sqrt();
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

/// How much of its width glyph `gi`'s cell leaves it on screen (0..1): the line's tangent foreshortened,
/// and never wider than the cell's real spacing to its nearer neighbour.
fn squeeze(v: &View, c: &Cache, gi: usize, sp: &SurfacePoint, em_px: f32) -> f32 {
    let (p, t) = (v.rot(sp.p), v.rot(sp.t));
    let ([sx, sy], k) = v.project(p);
    let run = (t[0] * t[0] + t[1] * t[1]).sqrt();
    let s = (em_px * k / SIZE * sp.k).max(1e-4);
    let cell = c.text.cell[gi];
    let near = |j: usize| c.pts.get(j).map(|q| { let o = v.project(v.rot(q.p)).0; (o[0] - sx).hypot(o[1] - sy) });
    let room = cell.checked_sub(1).and_then(near).unwrap_or(f32::INFINITY).min(near(cell + 1).unwrap_or(f32::INFINITY));
    (s * run).min(room * 0.97 / ADVANCE) / s
}
/// A glyph squeezed under this, or faded by its facing under this opacity, is not drawn; its cell's
/// dot is (the contour is the dots' job).
pub const MIN_SQUEEZE: f32 = 0.35;
pub const MIN_ALPHA: f32 = 0.55;
/// Nor is one foreshortened across its line to under this height on screen (px), nor one under this wide.
pub const MIN_HEIGHT: f32 = 6.0;
pub const MIN_WIDTH: f32 = 4.0;
/// Whether a glyph at scale `s`, squeeze `q`, line gap `gap` and opacity `alpha` is drawn.
pub fn drawn(s: f32, q: f32, gap: f32, alpha: f32) -> bool {
    q >= MIN_SQUEEZE && alpha >= MIN_ALPHA && SIZE * s * gap >= MIN_HEIGHT && ADVANCE * s * q >= MIN_WIDTH
}

/// How far glyph `i` of `n` has gone at gather `g`: reading order leads; exactly 0 at 0 and 1 at 1.
pub fn progress(g: f32, i: usize, n: usize) -> f32 {
    let st = 0.55;
    let u = (g * (1.0 + st) - st * i as f32 / n.max(1) as f32).clamp(0.0, 1.0);
    u * u * u * (u * (u * 6.0 - 15.0) + 10.0)
}

/// The glyph size on the stone in units of the mean radius, and whether it is the small-window case.
pub fn em_for(radius_px: f32) -> (f32, bool) { if em_px(radius_px, 1.0 / 14.0) * 1.33 < SIZE { (1.0 / 10.0, true) } else { (1.0 / 14.0, false) } }
/// A glyph's size on screen at the stone's centre plane (px).
pub fn em_px(radius_px: f32, em: f32) -> f32 { radius_px / SILHOUETTE * em }
/// A phone's body radius as a share of the window's width, and the lines its console closes.
pub const PHONE_R: f32 = 0.385;
pub const PHONE_HEAD: usize = 10;
/// The room a phone's console takes above the stone (row A and the 16 px under it).
pub const PHONE_TOP: f32 = ROW_PHONE + 16.0;
/// A phone's text by the radius it needs: for each cut of this file from `CHARS_PHONE` down (whole
/// lines), its characters, its lines and the least radius (px) at which the stone's glyphs draw
/// `PHONE_GLYPH` px tall or more. The shell reads the same table to write the same listing.
pub fn phone_table() -> &'static [(usize, usize, f32)] {
    static T: std::sync::OnceLock<Vec<(usize, usize, f32)>> = std::sync::OnceLock::new();
    T.get_or_init(|| {
        let (mut out, mut n) = (vec![], CHARS_PHONE);
        while n > 0 {
            let t = stream(n, PHONE_HEAD);
            if t.lines <= PHONE_HEAD { break; }
            out.push((t.listing.chars().count(), t.lines, PHONE_GLYPH / GLYPH_INK * SILHOUETTE / em_to_fill(t.cells)));
            n = t.listing.chars().count().saturating_sub(1);
        }
        out
    })
}
/// The most characters whose glyphs fill a stone of `radius_px` at `PHONE_GLYPH` px or more.
pub fn phone_chars(radius_px: f32) -> usize {
    let t = phone_table();
    t.iter().find(|e| radius_px >= e.2).or(t.last()).map_or(CHARS_PHONE, |e| e.0)
}
/// The word of 03: its size multiple `k` for a listing `width` px wide (the largest whole number with
/// five cells of `7k` in 90 % of it), and the material lattice's pitch for it (a stem is three units).
pub fn word_k(width: f32) -> f32 { (0.9 * width / (5.0 * ADVANCE)).floor().max(1.0) }
pub fn word_pitch(k: f32) -> f32 { (k / 3.0).round().clamp(3.0, 8.0) }

impl Body {
    /// At rest in the listing.
    pub fn flat(&self) -> bool { self.gather.value == 0.0 && !self.gather.moving() && self.fade.is_none() }
    /// The state it is going to: 0 the listing, 1 the stone.
    pub fn target(&self) -> f32 { if self.at == 4 { 1.0 } else { 0.0 } }
    /// The host's window (logical px).
    pub fn resize(&mut self, w: f32, h: f32) { self.size = (w.max(1.0), h.max(1.0)); }
    /// The stage the host laid out (window px), or none for the default one.
    pub fn set_stage(&mut self, r: Option<[f32; 4]>) { self.stage = r.filter(|r| r[2] > r[0] + 1.0 && r[3] > r[1] + 1.0); }
    /// The optical centre of the page's text block (window px, vertical), or none for the site's grid.
    pub fn set_text_centre(&mut self, y: Option<f32>) { self.text_centre = y; }
    /// The pointer's place as a fraction of the window from its centre (-0.5..0.5): the lamp goes there.
    pub fn look_at(&mut self, x: f32, y: f32) { let (w, h) = self.size; self.point_lamp(Some(((x + 0.5) * w, (y + 0.5) * h))); }
    fn set_now(&mut self, st: u8) {
        self.gather = Animated::new(if st == 4 { 1.0 } else { 0.0 }, GATHER_SPRING);
        (self.fade, self.intro_pending, self.intro_at, self.unfold, self.scrub) = (None, false, None, false, None);
        (self.at, self.was, self.lands, self.walk, self.settle) = (st, st, 0.0, None, None);
        if st != 3 { self.material = 0; } else if self.material == 0 { self.material = REST_MATERIAL; }
    }
    /// Open a state from the page's address: `source`, `interface`, `material`, `body`, `body/08`
    /// (and the older `body/cloud/08`, which opens the stone with that seed), at rest; empty is the
    /// stone with the default seed, reached by the short path (`walk` 1) or the full walk (`walk` 2)
    /// from the source, or at rest (0).
    pub fn open(&mut self, state: &str, walk: u8) {
        let mut parts = state.trim_start_matches('#').split('/');
        match parts.next().unwrap_or("") {
            "source" => self.set_now(1),
            "interface" => self.set_now(2),
            "material" => self.set_now(3),
            "body" => {
                if let Some(n) = parts.filter_map(|n| n.parse::<u32>().ok()).next_back() { self.seed = n.min(99); }
                self.set_now(4);
            }
            _ => {
                self.seed = SEED;
                self.set_now(if walk > 0 { 1 } else { 4 });
                self.intro_pending = walk > 0;
                self.full = walk == 2;
            }
        }
        self.changed = self.t;
    }
    /// The state for the page's address (`open` reads it back): `source`, `interface`, `material`,
    /// `body/08`, or empty for the stone with the default seed.
    pub fn state(&self) -> String {
        if self.walk.is_some() || self.intro_pending || self.intro_at.is_some() { return String::new(); }
        match self.at {
            1 => "source".into(),
            2 => "interface".into(),
            3 => "material".into(),
            _ if self.seed == SEED => String::new(),
            _ => format!("body/{:02}", self.seed),
        }
    }
    /// The state on the stage now: during a reduced-motion change, the one before it until half way.
    pub fn shown_state(&self) -> u8 { match self.fade { Some((t0, d)) if self.t - t0 < d * 0.5 => self.was, _ => self.at } }
    /// The walk: 0 none, 1 playing (or about to), 2 stopped by a person (`continue ›`).
    pub fn walking(&self) -> u8 { if self.walk.is_some() || (self.intro_pending && self.full && !self.reduced) { 1 } else if self.taken { 2 } else { 0 } }
    /// A morph between two of the scene's elements; the lattice is the word's pitch.
    fn morph(&mut self, from: &str, to: &str, m: (f32, f32, f32), now: f32) {
        let count_max = if self.width_class() == Width::Narrow { MORPH_CELLS.1 } else { MORPH_CELLS.0 };
        let spec = amenui::morph::MorphSpec { loosen_ms: m.0, duration_ms: m.1, tighten_ms: m.2, count_max, lattice: self.pitch_px(), threshold: MORPH_THRESHOLD, stagger: MORPH_STAGGER, ..Default::default() };
        self.cmds.push(amenui::morph::Cmd::Morph { from: format!("{KEY}/{from}"), to: format!("{KEY}/{to}"), spec });
        self.lands = self.lands.max(now + (m.0 + m.1 + m.2) / 1000.0);
    }
    /// The gather to `to` by its spring (from where it is, at its speed).
    fn spring(&mut self, to: f32, now: f32) { if (self.gather.target - to).abs() > 1e-6 || self.gather.value != to { let v = self.gather.velocity; self.gather.to(to, v, now); } }
    /// To state `to` (1 to 4). The console closes from and opens into the head lines (01 ↔ the rest);
    /// the listing gathers into the word (03) or onto the stone (04); the gather spring joins the
    /// listing and the stone. With reduced motion an opacity change.
    pub fn go_to(&mut self, to: u8, now: f32) {
        let to = to.clamp(1, 4);
        (self.intro_pending, self.intro_at) = (false, None);
        let from = self.at;
        if to == from { if to == 4 { self.spring(1.0, now); } return; }
        (self.was, self.at, self.changed, self.unfold, self.scrub) = (from, to, now, false, None);
        self.settle = None;
        if to == 3 { if self.material == 0 { self.material = REST_MATERIAL; } } else { self.material = 0; }
        if to == 4 && self.walk.is_some() {
            // In the walk the stone is held still while it lands, the lamp behind it, until the walk
            // turns the lamp on.
            self.lamp = [-0.2, 0.3, -2.5].map(|v| Animated::new(v, LAMP));
            (self.spin, self.idle, self.ease) = (0.0, 0.0, None);
        }
        if self.reduced { self.fade = Some((now, FADE)); return; }
        self.lands = now;
        // The console.
        if from == 1 { self.morph("head", "console", MORPH_CONSOLE, now); }
        if to == 1 && from != 4 { self.morph("console", "head", MORPH_CONSOLE, now); self.cmds.push(amenui::morph::Cmd::ClearFocus); }
        // The word.
        if from == 3 {
            self.morph("word", "text", MORPH_SHORT, now);
            if to == 4 { self.gather = Animated::new(1.0, GATHER_SPRING); }
        } else if to == 3 {
            self.morph("text", "word", MORPH_SHORT, now);
            if from == 4 { self.settle = Some((self.lands, 0.0)); }
        }
        // The gather between the listing and the stone.
        match (from, to) {
            (1 | 2, 4) => self.spring(1.0, now),
            (4, 1) => { self.spring(0.0, now); self.unfold = true; }
            (4, 2) => self.spring(0.0, now),
            _ => {}
        }
    }
    /// The full walk from the current state (from the source when elsewhere).
    pub fn play(&mut self, now: f32) { self.walk_from(now, false) }
    fn phone(&self) -> bool { self.width_class() == Width::Narrow }
    /// The walk's beats (s after its start).
    fn beats(&self) -> Vec<(f32, Step)> {
        let mut s = vec![(0.70, Step::State(2)), (1.30, Step::Focus("material")), (1.70, Step::Press("material:2"))];
        if self.phone() {
            s.extend([(2.90, Step::Focus("gather")), (3.30, Step::Via), (5.35, Step::Lamp), (5.35, Step::Arrive), (5.95, Step::End)]);
        } else {
            s.extend([(2.90, Step::Press("material:3")), (3.80, Step::Focus("gather")), (4.20, Step::Via), (6.25, Step::Lamp), (6.25, Step::Arrive), (6.85, Step::End)]);
        }
        s
    }
    fn walk_from(&mut self, now: f32, first: bool) {
        if self.reduced { return self.land(now, Ending::Skipped); }
        self.flight = (0, 0.0);
        let mut steps = self.beats();
        // From where the walk is: the beats before the current state's are done.
        let skip = match self.at { _ if first => 0.0, 2 => 1.30, 3 => 2.90, _ => 0.55 };
        let mut t0 = now - skip;
        if matches!(self.at, 4) || (self.at == 3 && !first && self.phone()) {
            // From the stone (or 03 on a phone): flattened to the source first, then from its start.
            self.go_to(1, now);
            t0 = now + 1.4 - 0.7;
            steps.retain(|s| s.0 >= 0.55);
        } else {
            steps.retain(|s| s.0 >= skip);
        }
        (self.walk, self.taken) = (Some((t0, steps)), false);
    }
    /// A person acted during the walk: it stops where it is (`continue ›` plays on).
    pub fn heard(&mut self) -> bool {
        if self.walk.is_none() { return false; }
        (self.walk, self.taken) = (None, true);
        self.later.retain(|s| matches!(s.1, Step::State(_)));
        true
    }
    /// `skip ›`, Esc or a scroll: straight to the stone, and the walk counts as seen.
    pub fn skip(&mut self, now: f32) { self.land(now, Ending::Skipped) }
    /// The landing state, however the walk ends: the walk is over and seen, the state is the stone,
    /// and the lamp comes on once it lands (at once when skipped).
    fn land(&mut self, now: f32, how: Ending) {
        // The operator's marks leave with the walk.
        if self.walk.is_some() { self.cmds.push(amenui::morph::Cmd::ClearFocus); }
        (self.walk, self.taken, self.seen) = (None, false, true);
        (self.intro_pending, self.intro_at) = (false, None);
        self.later.clear();
        let there = self.at == 4;
        if !there { self.go_to(4, now); }
        match how {
            Ending::Skipped => self.point_lamp(self.ptr),
            Ending::Played if there => {}
            _ => { let at = self.lands.max(now); self.later.push((at, Step::Lamp)); }
        }
    }
    /// The host says how ready it is to draw what the walk shows next (`Ui::readiness`).
    pub fn set_ready(&mut self, r: f32) { self.ready = r.clamp(0.0, 1.0); }
    /// The walk gives way: not ready for 02, it lands at once; a material press whose pipelines are not
    /// ready, or any after a slow first flight (a mean frame over 20 ms), is left out.
    fn gives_way(&mut self, step: &Step, now: f32) -> bool {
        if self.walk.is_none() { return false; }
        let slow = self.flight.0 >= 10 && self.flight.1 / self.flight.0 as f32 > 0.020;
        match step {
            Step::State(2) if self.ready < 1.0 => { self.land(now, Ending::GaveWay); true }
            Step::Focus("material") if slow => true,
            Step::Press(k) if k.starts_with("material:") && (self.ready < 1.0 || slow) => true,
            _ => false,
        }
    }
    fn do_step(&mut self, step: Step, now: f32) {
        if self.gives_way(&step, now) { return; }
        match step {
            Step::State(n) => self.go_to(n, now),
            Step::Focus(k) => self.cmds.push(amenui::morph::Cmd::Focus { key: format!("{KEY}/console/{k}") }),
            Step::Press(k) => self.cmds.push(amenui::morph::Cmd::Press { key: format!("{KEY}/console/{k}") }),
            // GATHER from 03: the word opens into the listing, and when it has landed the stone gathers.
            Step::Via => if self.at == 3 { self.go_to(2, now); self.later.push((self.lands, Step::State(4))); } else { self.go_to(4, now); },
            Step::Lamp => self.point_lamp(self.ptr),
            Step::Arrive => self.ease = Some(Ease { t0: now, d: ARRIVE.1, from: (self.yaw, self.pitch), to: (self.yaw + ARRIVE.0, self.pitch), out: true }),
            Step::End => self.land(now, Ending::Played),
        }
    }
    /// How opaque everything is during a reduced-motion change of state (1 otherwise).
    pub fn dim(&self) -> f32 { self.fade.map_or(1.0, |(t0, d)| (1.0 - 2.0 * ((self.t - t0) / d).clamp(0.0, 1.0)).abs()) }
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
    /// A phone's stage is at least 340 px tall, and 44 % of the window's height when that is more.
    fn narrow_stage_bottom(h: f32) -> f32 { 64.0 + (h * 0.44).max(340.0) }
    /// The stage: the host's, or where the site's page puts it.
    pub fn stage(&self) -> [f32; 4] {
        if let Some(s) = self.stage { return s; }
        let (w, h) = self.size;
        match self.width_class() {
            Width::Wide => { let (fx, fw) = self.frame(); [(self.text_box()[2] + 48.0).round(), 72.0, fx + fw - 32.0, h - 88.0] }
            Width::Middle => [(self.text_box()[2] + 48.0).round(), 72.0, w - 24.0, h - 88.0],
            Width::Narrow => [16.0, 64.0, w - 16.0, Self::narrow_stage_bottom(h)],
        }
    }
    /// The listing's width in whole 7 px cells.
    pub fn cols(&self) -> usize { let s = self.stage(); ((s[2] - s[0] - GUTTER) / ADVANCE).floor().max(1.0) as usize }
    /// The listing's lines and their width in cells: as many of the text's lines (whole, 14 px) as fit
    /// in the stage with the caption 16 px under the last.
    pub fn grid(&self) -> (usize, usize) {
        let s = self.stage();
        self.ensure();
        let lines = self.cache.borrow().text.lines.max(1);
        let rows = (((s[3] - s[1] - CAPTION_GAP - LINE) / LINE).floor().max(1.0) as usize).min(lines);
        (rows, self.cols())
    }
    /// The console: on the listing's text column at the stage's top, `min(listing, 644 px)` wide.
    pub fn console(&self) -> Console {
        let s = self.stage();
        let w = (self.cols() as f32 * ADVANCE).min(CONSOLE_W);
        let (x, y) = (s[0] + GUTTER, s[1]);
        if self.phone() { return Console { x, y, w, row: ROW_PHONE, rows: 3, lines: PHONE_HEAD, fit: Fit::Phone }; }
        if w >= 500.0 { Console { x, y, w, row: ROW, rows: 2, lines: 4, fit: Fit::Wide } } else { Console { x, y, w, row: ROW, rows: 3, lines: 6, fit: Fit::Narrow } }
    }
    /// The caption's top (window px): 16 px under the listing's last line; on a phone at the stage's foot.
    pub fn caption_y(&self) -> f32 {
        let s = self.stage();
        if self.phone() { return s[3] - LINE; }
        s[1] + self.grid().0 as f32 * LINE + CAPTION_GAP
    }
    /// The caption's box (window px), as wide as its longest sentence.
    pub fn caption_box(&self) -> [f32; 4] { let (x, y) = (self.stage()[0], self.caption_y()); [x, y, x + GUTTER + ADVANCE * 44.0, y + LINE] }
    /// The optical centre of the text block, vertical (window px).
    pub fn text_centre(&self) -> f32 { self.text_centre.unwrap_or_else(|| { let b = self.text_box(); (b[1] + b[3]) * 0.5 }) }
    /// The top of the stone's room: 16 px under the console.
    pub fn room_top(&self) -> f32 { self.console().foot() + 16.0 }
    /// Where the stone is in the stage: centre (window px) and visible radius (px). Wide: centred across
    /// the stage, level with the text block's optical centre where the room allows. Middle: the same
    /// height; it may run off the right edge by a third of its radius. Narrow: centred in the room
    /// above the caption. Its circle (1.05 of its radius) stays under the console and off the caption.
    pub fn placement(&self) -> (f32, f32, f32) {
        let (w, h) = self.size;
        let s = self.stage();
        let top = self.room_top();
        let cap = self.caption_box();
        let sw = s[2] - s[0];
        let off_caption = |cx: f32, cy: f32| { let (dx, dy) = ((cap[0] - cx).max(cx - cap[2]).max(0.0), (cap[1] - cy).max(cy - cap[3]).max(0.0)); dx.hypot(dy) / 1.05 };
        let (cx, cy, r) = match self.width_class() {
            Width::Wide => {
                let cy = self.text_centre().max(top + 0.34 * h * 0.6).min(s[3] - 40.0);
                let cx = (s[0] + s[2]) * 0.5;
                let r = (0.34 * h).min(0.23 * self.frame().1).min(sw * 0.5 / 1.25).min((cy - top) / 1.05).min((s[3] - cy) / 1.05).min(off_caption(cx, cy));
                (cx, cy, r)
            }
            Width::Middle => {
                let cy = self.text_centre().max(top + 0.26 * w * 0.6).min(s[3] - 40.0);
                let mut r = (0.26 * w).min((w - s[0]) / (1.25 + 2.0 / 3.0)).min((cy - top) / 1.05).min((s[3] - cy) / 1.05);
                for _ in 0..8 { r = r.min(off_caption(s[0] + 1.25 * r, cy)); }
                (s[0] + 1.25 * r, cy, r)
            }
            Width::Narrow => {
                let half = (cap[1] - 8.0 - top) * 0.5;
                ((s[0] + s[2]) * 0.5, top + half, (PHONE_R * w).min(half / 1.05))
            }
        };
        (cx, cy, r.max(24.0))
    }
    /// The camera now.
    pub fn view(&self) -> View {
        let (cx, cy, r) = self.placement();
        let mean = self.cache.borrow().radius;
        let mean = if mean > 0.0 { mean } else { 0.86 };
        View { yaw: self.yaw, pitch: self.pitch, c: (cx, cy), r: r / (mean * SILHOUETTE) }
    }
    /// The lamp now (view space).
    pub fn light(&self) -> [f32; 3] { [self.lamp[0].value, self.lamp[1].value, self.lamp[2].value] }
    /// The proposed palette's neutral stone.
    fn neutral(&self) -> bool { self.palette > 0 }
    /// Fill the cache for the current seed, text and console.
    fn ensure(&self) {
        let (chars, head) = (self.chars(), self.console().lines);
        let key = (self.seed, chars as u32, head as u32);
        let mut c = self.cache.borrow_mut();
        if c.key == key && !c.pts.is_empty() { return; }
        c.text = stream(chars, head);
        c.em = em_to_fill(c.text.cells);
        c.pts = cells(self.seed, 0.62 * c.em, 1.45 * c.em);
        c.radius = mean_radius(self.seed);
        c.key = key;
    }
    /// The small-window case (fewer, larger glyphs). It depends on the stage, not on the stone's radius.
    pub fn small(&self) -> bool {
        let s = self.stage();
        let r = match self.width_class() { Width::Narrow => (PHONE_R * self.size.0).min((s[3] - s[1]) * 0.5), _ => (0.34 * self.size.1).min(0.23 * self.frame().1) };
        em_for(r).1
    }
    /// How many characters of this file the text is: on a phone as many as keep the facing glyphs at
    /// `PHONE_GLYPH` px on a stone of the phone's radius.
    pub fn chars(&self) -> usize {
        if !self.phone() { return if self.small() { CHARS_SMALL } else { CHARS }; }
        let s = self.stage();
        let r = (PHONE_R * self.size.0).min(((s[3] - s[1]) - LINE - 8.0 - PHONE_TOP) * 0.5 / 1.05);
        phone_chars(r)
    }
    /// How many cells the whole text takes (its glyphs and the spaces between words).
    pub fn glyphs(&self) -> usize { self.ensure(); self.cache.borrow().text.all }
    /// How many glyphs the stone is made of, and the console's lines hold.
    pub fn stone_glyphs(&self) -> usize { self.ensure(); self.cache.borrow().text.cell.len() }
    /// The text (whole lines of this file).
    pub fn listing(&self) -> String { self.ensure(); self.cache.borrow().text.listing.clone() }
    /// The listing as the stage shows it now.
    pub fn shown(&self) -> Shown { self.ensure(); let (rows, cols) = self.grid(); shown(&self.cache.borrow().text, rows, cols) }
    /// The word of 03: its size multiple and its box (window px).
    pub fn word_box(&self) -> (f32, [f32; 4]) {
        let s = self.stage();
        let lw = self.cols() as f32 * ADVANCE;
        let k = word_k(lw);
        let (ww, wh) = (5.0 * ADVANCE * k, SIZE * k);
        let x = (s[0] + GUTTER + (lw - ww) * 0.5).round();
        let lo = self.console().y + self.console().h() + 16.0;
        let hi = self.caption_y() - 16.0 - wh;
        let y = (self.placement().1 - wh * 0.5).min(hi).max(lo).round();
        (k, [x, y, x + ww, y + wh])
    }
    /// The lattice pitch of the materials and the morphs' cells: the word's.
    pub fn pitch_px(&self) -> f32 { word_pitch(self.word_box().0) }

    /// The pose of every glyph of the stone's text: from its place in the listing to its cell on the
    /// stone, with its depth and how it faces the viewer for `El::form`.
    fn poser(&self, origin: (f32, f32), shown: Rc<Vec<bool>>) -> impl Fn(GlyphAt) -> Pose + 'static {
        let (g, v, l, cache, dim, touch, neutral) = (self.gather.value.clamp(0.0, 1.0), self.view(), self.light(), self.cache.clone(), self.dim(), self.touch(), self.neutral());
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
            let (moved, boost) = touch.moved(sp);
            let sp = &moved;
            let (p, t) = (v.rot(sp.p), v.rot(sp.t));
            let ([sx, sy], k) = v.project(p);
            let CellLook { paint, gap, .. } = cell_look(&v, l, sp, boost, neutral);
            let facing = word_facing(&v, &c, gi);
            let ang = (-t[1]).atan2(t[0]);
            let s = em_px * k / SIZE * sp.k;
            let q = squeeze(&v, &c, gi, sp, em_px);
            let shown = if drawn(s, q, gap, paint.alpha * FORM_FX.facing_alpha(facing)) { 1.0 } else { 0.0 };
            Pose {
                dx: (sx - fx) * e, dy: (sy - fy) * e, angle: ang * e,
                sx: 1.0 + (s * q - 1.0) * e, sy: 1.0 + (s * gap - 1.0) * e,
                alpha: flat_alpha + (paint.alpha * dim * shown - flat_alpha) * e,
                accent: paint.accent * e, hue: if e > 0.5 { 0.0 } else { -1.0 },
                z: p[2] * v.r * e, facing: 1.0 + (facing - 1.0) * e,
                ..Pose::default()
            }
        }
    }

    /// Behind the text: the dust, the volume, the silhouette's dots.
    fn back(&self) -> Drawing {
        let (w, h) = self.size;
        let (g, v, l) = (self.gather.value.clamp(0.0, 1.0), self.view(), self.light());
        let mut d = Drawing::new(w, h);
        if g <= 0.0 { return d; }
        let neutral = self.neutral();
        dust(&mut d, &v, self.stage(), g, if neutral { 0.0 } else { 0.4 }, 80);
        let rpx = self.placement().2;
        let mut c = self.cache.borrow_mut();
        let want = amenui::budget::grant(VOLUME);
        if c.vkey != (self.seed, want) { c.vol = volume(self.seed, want); c.vkey = (self.seed, want); }
        let rv = v.r * c.radius;
        for q in c.vol.iter() {
            let (p, n) = (v.rot(q.p), v.rot(q.n));
            let inside = dot3(q.p, q.p) < 0.7 * c.radius * c.radius;
            if (inside && p[2] > 0.0) || (!inside && n[2] > 0.0) { continue; }
            let ([x, y], k) = v.project(p);
            let paint = Paint { tone: 1.0, accent: if neutral { 0.0 } else { 1.0 }, alpha: 0.20 * g * (0.4 + 0.6 * lit(l, p, n)), hue: 0 };
            d.dot(x, y, q.size * k * 0.9, paint);
        }
        // Where a cell's glyph has faded out at the rim, the cell is one dot in the colour the light gives it.
        let dot_r = 0.006 * rv;
        let em = em_px(rpx, c.em);
        for (gi, &k) in c.text.cell.iter().enumerate() {
            let Some(sp) = c.pts.get(k) else { continue };
            if v.rot(sp.n)[2] < -0.25 { continue; }
            let look = cell_look(&v, l, sp, 0.0, neutral);
            let ([x, y], s) = v.project(v.rot(sp.p));
            if drawn(em * s / SIZE * sp.k, squeeze(&v, &c, gi, sp, em), look.gap, look.paint.alpha * FORM_FX.facing_alpha(word_facing(&v, &c, gi))) { continue; }
            d.dot(x, y, dot_r * s, Paint { tone: look.paint.tone, accent: look.paint.accent, alpha: 0.30 * g, hue: 0 });
        }
        d
    }
    /// Over the text: the motes and the readouts' leader lines.
    fn front(&self, labels: &[Readout]) -> Drawing {
        let (w, h) = self.size;
        let mut d = Drawing::new(w, h);
        let accent = if self.neutral() { 0.0 } else { 0.6 };
        for m in &self.motes { let f = 1.0 - m.age / MOTE_LIFE; d.dot(m.x, m.y, 0.6 + 0.9 * f, Paint { tone: 1.0, accent, alpha: 0.6 * f * self.idle, hue: 0 }); }
        for l in labels {
            if l.alpha <= 0.0 { continue; }
            let p = Paint::DIM.alpha(l.alpha);
            d.line(l.from, l.elbow, 0.5, p);
            d.line(l.elbow, l.to, 0.5, p);
            d.dot(l.from.0, l.from.1, 1.4, Paint { tone: 1.0, ..p });
        }
        d
    }
    /// The glyph nearest the point the pointer's ray meets on the stone: its index, the character, and
    /// where its cell is on screen.
    pub fn glyph_under_lamp(&self) -> Option<(usize, char, (f32, f32))> {
        self.ptr?;
        let gi = self.under?;
        let v = self.view();
        let c = self.cache.borrow();
        let head = c.text.head;
        let ch = c.text.listing.split('\n').skip(head).flat_map(|l| l.chars()).filter(|c| !c.is_whitespace()).nth(gi)?;
        let p = v.project(v.rot(c.pts[c.text.cell[gi]].p)).0;
        Some((gi, ch, (p[0], p[1])))
    }
    /// The readouts round the stone, in state 04: `GLYPH 0412  t` from the glyph under the lamp (only
    /// while there is a pointer) and `YAW 023°` from the silhouette's lower right. None on a phone.
    pub fn readouts(&self) -> Vec<Readout> {
        if self.phone() || self.target() < 0.5 { return vec![]; }
        let (v, g) = (self.view(), self.gather.value.clamp(0.0, 1.0));
        let (cx, cy, r) = self.placement();
        let alpha = ((g - 0.85) / 0.15).clamp(0.0, 1.0) * self.dim();
        if alpha <= 0.0 { return vec![]; }
        let st = self.stage();
        let top = self.room_top();
        let fit = |key: &'static str, text: String, from: (f32, f32), ly: f32, right: bool, flip: bool| -> Option<Readout> {
            let tw = ADVANCE * text.chars().count() as f32;
            let ly = ly.clamp(top + 8.0, self.caption_y() - 8.0);
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

/// The dust: `n` marks fixed in the stone's space round it, turning with it and still when it is
/// still; in the stage only, as strong as the gather.
fn dust(d: &mut Drawing, v: &View, st: [f32; 4], g: f32, accent: f32, n: usize) {
    let n = amenui::budget::grant(n);
    for i in 0..n as u32 {
        let s = i * 7;
        let z = 1.0 - 2.0 * hash(s);
        let a = hash(s + 1) * std::f32::consts::TAU;
        let rho = 1.25 + 0.75 * hash(s + 2);
        let q = (1.0 - z * z).max(0.0).sqrt() * rho;
        let p = v.rot([q * a.cos(), z * rho * 0.8, q * a.sin()]);
        let ([x, y], k) = v.project(p);
        let size = (0.5 + 1.1 * hash(s + 3)) * k;
        if x < st[0] + size || x > st[2] - size || y < st[1] + size || y > st[3] - size { continue; }
        let near = (0.5 + 0.5 * p[2] / rho).clamp(0.0, 1.0);
        d.dot(x, y, size, Paint { tone: 0.8, accent, alpha: (0.06 + 0.22 * near) * g, hue: 0 });
    }
}

/// The gather's value, `d.dd`, as wheels of digits (and the point) in a four-glyph window.
fn odometer(g: f32) -> Vec<El<Msg>> {
    let n = (g.clamp(0.0, 1.0) * 100.0).round() as u32;
    let wheel = |x: f32, top: u32, d: u32| {
        let digits = (0..=top).map(|i| i.to_string()).collect::<Vec<_>>().join("\n");
        mono(digits, Paint::INK).absolute(Some(x), Some(0.0), None, None).shift(0.0, -(d as f32) * LINE)
    };
    vec![
        wheel(0.0, 1, n / 100),
        mono(".", Paint::INK).absolute(Some(ADVANCE), Some(0.0), None, None),
        wheel(2.0 * ADVANCE, 9, n / 10 % 10),
        wheel(3.0 * ADVANCE, 9, n % 10),
    ]
}
/// A label or value of the console: Departure Mono at 11 px on a 14 px line.
fn mono(t: impl Into<String>, p: Paint) -> El<Msg> { text_lh(t.into(), SIZE, LINE, p).font(Font::Mono).nowrap() }
/// An element of the console at a box (window px) relative to the console's corner.
fn put(e: El<Msg>, c: &Console, b: [f32; 4]) -> El<Msg> { e.absolute(Some(b[0] - c.x), Some(b[1] - c.y), None, None).width(b[2] - b[0]).height(b[3] - b[1]) }

impl Body {
    /// The console: the four constants closed into the shipped slider, choice, switch and button under
    /// the site's theme (square, hairline frames, no material, no looseness).
    fn console_el(&self, key: &str) -> El<Msg> {
        let c = self.console();
        let k = |s: &str| format!("{key}/console/{s}");
        let g = self.gather.value.clamp(0.0, 1.0);
        let held = self.scrub.is_some();
        let hov = |x: Ctl| self.hover == Some(x);
        let mut parts = vec![];
        // GATHER: the whole row takes the pointer; the thumb is the listing's caret, under the hand.
        let (t0, t1) = c.track();
        let run = t1 - t0;
        let travel = run - 8.0;
        let ink = if held { Paint::ACCENT } else { Paint::INK };
        let big = if hov(Ctl::Gather) || held { 1.0 } else { 0.0 };
        let mid = (c.row * 0.5).floor();
        let spec = amenui::widgets::Spec { min: 0.0, max: 1.0, step: 0.05, decimals: 2, unit: "" };
        let mut s = amenui::widgets::slider(&k("gather"), g, spec, c.w, Msg::GatherTo);
        let (lx, lt) = (t0 - c.x, ((c.row - LINE) * 0.5).round());
        s.children = vec![
            mono("GATHER", Paint::DIM).absolute(Some(0.0), Some(lt), None, None),
            space(run, 1.0).fill(HAIR).absolute(Some(lx), Some(mid), None, None),
            layers(vec![space(run, 2.0).fill(ink).shift((travel * g + 4.0 - run).round(), 0.0)]).width(run).height(2.0).clip().absolute(Some(lx), Some(mid - 1.0), None, None),
            space(8.0, 16.0).fill(ink).absolute(Some(lx), Some(((c.row - 16.0) * 0.5).round()), None, None).shift((travel * g).round(), 0.0),
            // The value as three wheels of digits behind a one-line window: a new value moves them and
            // lays nothing out (a changed string would be measured again).
            layers(odometer(g)).width(4.0 * ADVANCE).height(LINE).clip().absolute(Some(c.w - 4.0 * ADVANCE), Some(lt), None, None),
            // Under the hand the thumb grows to 18 px: a second plate over it, shown by its alpha alone.
            space(8.0, 18.0).fill(ink.alpha(big)).absolute(Some(lx), Some(((c.row - 18.0) * 0.5).round()), None, None).shift((travel * g).round(), 0.0),
        ];
        let s = s.height(c.row).absolute(Some(0.0), Some(0.0), None, None).label("Gather").unframed()
            .on_drag_with(move |d: amenui::el::Drag| {
                let v = ((d.x - lx - 4.0) / travel.max(1.0)).clamp(0.0, 1.0);
                Some(Msg::Scrub(match d.phase { Phase::Start => 0, Phase::Move => 1, Phase::End => 2 }, v))
            });
        parts.push(s);
        // MATERIAL: four segments, the chosen one an ink plate; no frame round the group.
        let phone_rest = if c.fit == Fit::Phone && self.shown_state() == 4 { 0.0 } else { 1.0 };
        if c.fit == Fit::Wide { parts.push(mono("MATERIAL", Paint::DIM).absolute(Some(0.0), Some(c.row + lt), None, None)); }
        let names: Vec<&str> = MATERIALS.iter().map(|m| m.0).collect();
        let mut m = amenui::widgets::segmented(&k("material"), &names, self.material, 20.0, Msg::Material);
        let (s0, s3) = (c.segment(0), c.segment(3));
        (m.border, m.radius) = (None, 0.0);
        m = m.pad(0.0, 0.0).gap(0.0);
        let hover = self.hover;
        m.children = std::mem::take(&mut m.children).into_iter().enumerate().map(|(i, seg)| {
            let on = i == self.material;
            let b = c.segment(i);
            let name = MATERIALS[i].0;
            let over = hover == Some(Ctl::Material(i)) && !on;
            // The underline is always there; hover shows it by its alpha, so the tree keeps its shape.
            let nw = ADVANCE * name.len() as f32;
            let label = vec![
                mono(name, if on { Paint::GROUND } else if over { Paint::INK } else { Paint::TEXT }),
                space(nw, 1.0).fill(Paint::INK.alpha(if over { 1.0 } else { 0.0 })).absolute(Some(((b[2] - b[0] - nw) * 0.5).round()), Some(17.0), None, None),
            ];
            let mut seg = seg.pad(0.0, 0.0).width(b[2] - b[0]).height(20.0).unframed();
            (seg.radius, seg.fill, seg.children) = (0.0, on.then_some(Paint::INK), label);
            seg
        }).collect();
        parts.push(put(m.unframed().label("Material").role(Role::RadioGroup), &c, [s0[0], s0[1], s3[2], s3[3]]).opacity(phone_rest));
        // TURN: a 28 × 14 frame with a block inside, left and dim when off, right and ink when on.
        let on = self.turn && !self.reduced;
        let u = ((self.t - self.turned_at) / 0.14).clamp(0.0, 1.0);
        let e = if u < 0.5 { 4.0 * u * u * u } else { 1.0 - (-2.0 * u + 2.0).powi(3) * 0.5 };
        let slide = if on { e } else { 1.0 - e };
        let (tb, tf) = (c.turn(), c.turn_frame());
        let mut sw = amenui::widgets::switch_sized(k("turn"), "", on, None, 14.0, Msg::Turn);
        sw.children = vec![
            mono("TURN", Paint::DIM).absolute(Some(0.0), Some(lt), None, None),
            layers(vec![space(12.0, 10.0).fill(if on { Paint::INK } else { Paint::DIM }).absolute(Some(1.0), Some(1.0), None, None).shift((12.0 * slide).round(), 0.0)])
                .width(28.0).height(14.0).border(1.0, if hov(Ctl::Turn) { Paint::INK } else { HAIR }).absolute(Some(tf[0] - tb[0]), Some(tf[1] - tb[1]), None, None),
        ];
        sw = sw.pad(0.0, 0.0).gap(0.0).label("Turn by itself").unframed().disabled(self.reduced);
        parts.push(put(sw, &c, tb).opacity(phone_rest));
        // SEED: a hairline frame; pressed, an ink plate for 80 ms, and its two digits recompose.
        let flash = self.t - self.flash < 0.08;
        let sb = c.seed();
        let mut b = amenui::widgets::button(k("seed"), "", Msg::Seed(1));
        b.children = vec![mono(format!("SEED {:02} →", self.seed), if flash { Paint::GROUND } else { Paint::TEXT }).recompose(true).key(k("seed/n"))];
        (b.fill, b.radius) = (flash.then_some(Paint::INK), 0.0);
        b = b.border(1.0, if hov(Ctl::Seed) { Paint::INK } else { HAIR }).pad(8.0, 0.0).label(format!("Next seed, {:02}", self.seed)).unframed();
        parts.push(put(b, &c, sb).opacity(phone_rest));
        layers(parts).key(format!("{key}/console")).absolute(Some(c.x), Some(c.y), None, None).width(c.w).height(c.h()).calm()
            .role(Role::Group).label("Parameters of this page's source")
    }
    /// The control of the console at a point (window px).
    fn ctl_at(&self, x: f32, y: f32) -> Option<Ctl> {
        if self.shown_state() == 1 { return None; }
        let c = self.console();
        let inside = |b: [f32; 4]| x >= b[0] && x < b[2] && y >= b[1] && y < b[3];
        if inside(c.gather()) { return Some(Ctl::Gather); }
        if let Some(i) = (0..MATERIALS.len()).find(|&i| inside(c.segment(i))) { return Some(Ctl::Material(i)); }
        if inside(c.turn()) { return Some(Ctl::Turn); }
        if inside(c.seed()) { return Some(Ctl::Seed); }
        None
    }

    /// The scene filling a `w × h` window.
    pub fn view_in(&self, key: &str) -> El<Msg> {
        let (w, h) = self.size;
        self.ensure();
        let st = self.stage();
        let s = self.shown();
        let head = self.cache.borrow().text.head;
        let (lx, ly) = (st[0] + GUTTER, st[1]);
        let ty = ly + head as f32 * LINE;
        let (g, dim) = (self.gather.value.clamp(0.0, 1.0), self.dim());
        let shown = self.shown_state();
        // 04 → 01 keeps the console until the glyphs have landed, then it opens into the head lines.
        let head_on = shown == 1 && !self.unfold;
        let stone = self.t >= self.lands && shown != 3;
        let ink = Paint::INK;
        let txt = |t: String, p: Paint, k: &str| text_lh(t, SIZE, LINE, p).font(Font::Mono).nowrap().key(format!("{key}/{k}"));
        let cols = self.cols();
        // The head lines: the listing's first lines in 01, the console everywhere else.
        let head_el = txt(s.head.clone(), ink, "head").absolute(Some(lx), Some(ly), None, None).width(cols as f32 * ADVANCE).height(head as f32 * LINE)
            .opacity(dim).hidden_until_morphed(!head_on);
        // The text under them in one element: the listing's lines cut at whole words; the glyphs it
        // leaves out wait unseen where they would have been.
        let mut flat = vec![false; s.map.len() + s.rest_map.len()];
        for &i in &s.map { flat[i] = true; }
        let rest = self.cache.borrow().text.listing.split('\n').skip(head).collect::<Vec<_>>().join("\n");
        let code = txt(rest, ink, "text")
            .absolute(Some(lx), Some(ty), None, None).width(cols as f32 * ADVANCE).height(s.rows as f32 * LINE)
            .pose(self.poser((lx, ty), Rc::new(flat))).form(FORM_FX).reveal_by(Reveal::Word).hidden_until_morphed(shown == 3);
        let nums = |a: usize, b: usize| (a..=b).map(|i| format!("{i:>3}")).collect::<Vec<_>>().join("\n");
        // The line numbers: dim in the listing; the console's lines keep theirs in the hairline colour.
        let head_nums = txt(nums(1, head), if head_on { Paint::DIM } else { HAIR }, "lines-head").absolute(Some(st[0]), Some(ly), None, None).opacity(dim);
        let rest_alpha = if shown == 3 { 0.0 } else { (1.0 - g).clamp(0.0, 1.0) * dim };
        let numbers = txt(nums(head + 1, head + s.rows), Paint::DIM.alpha(rest_alpha), "lines").absolute(Some(st[0]), Some(ty), None, None);
        let marks = if stone { self.readouts() } else { vec![] };
        let empty = || Drawing::new(w, h);
        // The piece: the drawings and the stone's text, a stage the governor may draw at a lower scale
        // while it moves. The console, the word, the caption and the readouts are outside it.
        let piece = layers(vec![
            canvas(if stone { self.back() } else { empty() }).key(format!("{key}/back")).width(w).absolute(Some(0.0), Some(0.0), None, None).respond(amenui::el::Response::Ignore).pass(true),
            code,
            canvas(if stone { self.front(&marks) } else { empty() }).key(format!("{key}/front")).width(w).absolute(Some(0.0), Some(0.0), None, None).respond(amenui::el::Response::Ignore).pass(true),
        ]).key(format!("{key}/piece")).width(w).height(h).absolute(Some(0.0), Some(0.0), None, None)
            .role(Role::Img).label(tr("body.label"));
        // Not a stage while the text is a morph's end (to and from the word): a morph's tiles are taken
        // from the page's own pass, which a stage's content is not drawn in.
        let worded = shown == 3 || (self.was == 3 && self.t < self.lands);
        let piece = if worded { piece } else { piece.stage() };
        let piece = if self.turning() { piece.ongoing() } else { piece };
        // The word of 03: the material's name, as large as the listing allows, drawn in that material;
        // a change of material is a commit inside the stage only.
        let (k, wb) = self.word_box();
        let word = text_lh(MATERIALS[self.material].0.to_owned(), SIZE * k, SIZE * k, ink).font(Font::Mono).nowrap().key(format!("{key}/word"))
            .absolute(Some(wb[0] - st[0]), Some(wb[1] - st[1]), None, None).material(MATERIALS[self.material].1.id())
            .hidden_until_morphed(shown != 3).opacity(dim).legible(false).loose(WORD_LOOSE).role(Role::Img).label(MATERIALS[self.material].0.to_owned());
        let word_box = layers(vec![word]).key(format!("{key}/word-box")).absolute(Some(st[0]), Some(st[1]), None, None).width(st[2] - st[0]).height(st[3] - st[1]).commit_scope().pass(true);
        let mut parts = vec![head_nums, numbers, head_el, piece, word_box, self.console_el(key).hidden_until_morphed(head_on).opacity(dim)];
        for m in &marks {
            parts.push(txt(m.text.clone(), Paint::DIM.alpha(m.alpha), &format!("readout/{}", m.key)).absolute(Some(m.at.0), Some(m.at.1), None, None));
        }
        // The caption: the state's number in the dim colour, its sentence in the text colour; in 04 the
        // sentence dims after 4 s without a change.
        let body = shown == 4 && !self.intro_pending && self.intro_at.is_none();
        let (num, sentence) = match shown {
            _ if body => ("04", "Posed in 3D. Your pointer is the lamp.".to_owned()),
            2 => ("02", "The same ink closes into controls.".to_owned()),
            3 => ("03", "One tree. The material sets the pixels.".to_owned()),
            _ => ("01", format!("Source. {} characters of Rust.", thousands(self.glyphs()))),
        };
        let quiet = body && self.t - self.changed > 4.0;
        let cy = self.caption_y();
        parts.push(txt(num.into(), Paint::DIM, "caption/n").absolute(Some(st[0]), Some(cy), None, None).opacity(dim).recompose(!self.reduced));
        let gutter = if w < 360.0 { GUTTER - ADVANCE } else { GUTTER };
        parts.push(txt(sentence, if quiet { Paint::DIM } else { Paint::TEXT }, "caption").absolute(Some(st[0] + gutter), Some(cy), None, None).live(amenui::el::Live::Polite).opacity(dim).recompose(!self.reduced));
        // The line that adds the console's components, in 02, where it clears the caption by 48 px.
        let add = "$ amenui add switch slider choice button";
        let ax = st[2] - ADVANCE * add.chars().count() as f32;
        if shown == 2 && !self.phone() && ax >= self.caption_box()[2] + 48.0 {
            parts.push(txt(add.into(), Paint::DIM, "add").absolute(Some(ax.floor()), Some(cy), None, None).opacity(dim));
        }
        let grabbing = self.grab.is_some() && self.dragged;
        layers(parts).key(key.to_owned()).width(w).height(h).clip()
            .on_point(|p| Some(Msg::Point(p)))
            .on_key(|k, _| Self::takes(k).then_some(Msg::Key(k)))
            .cursor(if grabbing { amenui::el::Cursor::Grabbing } else if self.over { amenui::el::Cursor::Grab } else { amenui::el::Cursor::Default })
    }
    /// Where the listing's caret is (window px): one cell after the last line shown.
    pub fn caret(&self) -> (f32, f32) {
        let s = self.shown();
        let st = self.stage();
        let head = self.cache.borrow().text.head;
        let last = s.text.rsplit('\n').next().unwrap_or("").chars().count();
        (st[0] + GUTTER + ADVANCE * (last as f32 + 1.5), st[1] + LINE * ((head + s.rows).max(1) as f32 - 0.5))
    }
    /// The front for the host to send now, once (`front`).
    pub fn take_front(&mut self) -> Option<(f32, f32)> { self.front.take() }
    /// A front across the listing is under way (its speed is `FRONT_SPEED`).
    pub fn fronting(&self) -> bool { self.front_at.is_some() || self.front.is_some() || self.fronted.is_some_and(|t| self.t - t < 1.4) }
    /// The stone turns by itself now (the idle turn), and so asks for every frame.
    pub fn turning(&self) -> bool { self.idle > 0.0 && !self.reduced }
    /// Whether the idle turn is let run: TURN on, the stone there, nothing holding it, the page in view.
    fn idle_on(&self) -> bool { self.turn && self.at == 4 && !self.reduced && self.grab.is_none() && !self.away && self.gather.value > 0.97 }
    /// The scene's keys: 1 to 4 the states, Space and Enter between 02 and 04, [ and ] the seed, ← → a
    /// turn, 0 the opening view, Esc the walk's skip.
    pub fn takes(k: Key) -> bool { matches!(k, Key::Char(' ' | '0'..='4' | '[' | ']') | Key::Enter | Key::Left | Key::Right | Key::Escape) }
    /// What the hand does to the surface now (none in the listing).
    pub fn touch(&self) -> Touch {
        let r = self.cache.borrow().radius.max(1e-3);
        Touch { at: self.touch_at, k: self.bulge.value.clamp(0.0, 1.0), rings: self.rings.iter().map(|&(o, t0)| (o, RING_SPEED * r * (self.t - t0))).collect() }
    }
    /// How fast the stone turns about the vertical now (rad/s): a throw's rate and the idle turn's share.
    pub fn turn_rate(&self) -> f32 { let u = self.idle.clamp(0.0, 1.0); self.spin + u * u * (3.0 - 2.0 * u) * OMEGA * self.idle_dir }
    /// Where the ray through a window point meets the stone (view space), with the normal.
    fn hit_at(&self, x: f32, y: f32, v: &View) -> Option<([f32; 3], [f32; 3])> { self.hit((x - v.c.0) / v.r, -(y - v.c.1) / v.r, v) }
    /// The lamp to the pointer: just in front of the surface where the pointer's ray meets the stone,
    /// else on a plane in front of it; back to its rest place when the pointer is gone.
    fn point_lamp(&mut self, at: Option<(f32, f32)>) {
        let v = self.view();
        let body = self.target() >= 0.5;
        let hit = at.and_then(|(x, y)| self.hit_at(x, y, &v)).filter(|_| body);
        self.over = hit.is_some();
        let now = self.t;
        if let Some((h, _)) = hit { self.touch_at = Some(v.unrot(h)); }
        let k = self.bulge.velocity;
        self.bulge.to(if hit.is_some() { 1.0 } else { 0.0 }, k, now);
        let target = match (at, hit) {
            (None, _) => REST_LIGHT,
            (Some(_), Some((h, n))) => [h[0] + 0.55 * n[0], h[1] + 0.55 * n[1], h[2] + 0.55 * n[2]],
            (Some((x, y)), None) => [(x - v.c.0) / v.r, -(y - v.c.1) / v.r, 1.2],
        };
        for (a, t) in self.lamp.iter_mut().zip(target) { if (a.target - t).abs() > 1e-6 { let v = a.velocity; a.to(t, v, now); } }
    }
    /// An eased turn to (yaw, pitch) over `d` s (at once with reduced motion).
    fn ease_to(&mut self, yaw: f32, pitch: f32, d: f32, now: f32) {
        (self.spin, self.idle, self.last_input) = (0.0, 0.0, now);
        if self.reduced { (self.yaw, self.pitch, self.ease) = (yaw, pitch, None); self.tilt.hold(pitch); return; }
        self.tilt.hold(pitch);
        self.ease = Some(Ease { t0: now, d, from: (self.yaw, self.pitch), to: (yaw, pitch), out: false });
    }
    /// Back to the opening view, the nearest turn of it.
    fn home(&mut self, now: f32) {
        let tau = std::f32::consts::TAU;
        let yaw = REST_YAW + ((self.yaw - REST_YAW) / tau).round() * tau;
        self.ease_to(yaw, REST_PITCH, 0.4, now);
    }
    /// Where the ray through the screen point (view units) meets the stone, with its normal (view space).
    fn hit(&self, px: f32, py: f32, v: &View) -> Option<([f32; 3], [f32; 3])> {
        let o = [0.0, 0.0, 4.0];
        let dir = norm([px, py, -4.0]);
        let mut s = 2.0f32;
        for _ in 0..32 {
            let p = [o[0] + dir[0] * s, o[1] + dir[1] * s, o[2] + dir[2] * s];
            let d = sdf(self.seed, v.unrot(p));
            if d < 2e-3 { return Some((p, v.rot(gradient(self.seed, v.unrot(p))))); }
            s += d;
            if s > 6.0 { break; }
        }
        None
    }
    /// The tilt for a vertical drag of `dy` px from a tilt of `p0`: it gives under the hand, stiffens
    /// smoothly, and never passes ±20° from upright.
    pub fn tilt_for(&self, p0: f32, dy: f32) -> f32 {
        let r = self.placement().2.max(24.0);
        let u0 = ((p0 - REST_PITCH) / TILT).clamp(-0.999, 0.999).atanh();
        REST_PITCH + TILT * (u0 + 0.6 * dy / (r * TILT)).tanh()
    }
    /// The time of the next frame the scene needs (now while it moves; the idle turn's return when it
    /// waits for it), or none.
    pub fn wake(&self, now: f32) -> Option<f32> {
        if self.moving() { return Some(now); }
        if self.idle_on() && self.idle < 1.0 && now - self.last_input < IDLE_AFTER { return Some(self.last_input + IDLE_AFTER); }
        None
    }
    fn input(&mut self, now: f32) { self.last_input = now; }
}
/// The seed whose largest facet faces the viewer most squarely at the opening view.
pub fn best_seed() -> u32 {
    let v = View { yaw: REST_YAW, pitch: REST_PITCH, c: (0.0, 0.0), r: 1.0 };
    (0..100u32).max_by(|&a, &b| {
        let score = |s: u32| { let f = facets(s); let (n, _) = f.iter().copied().min_by(|x, y| x.1.total_cmp(&y.1)).unwrap(); v.rot(n)[2] };
        score(a).total_cmp(&score(b))
    }).unwrap_or(SEED)
}

impl Experiment for Body {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> { self.view_in(key) }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Key(Key::Left) => { let y = self.yaw - KEY_TURN; let p = self.pitch; self.ease_to(y, p, 0.2, now); }
            Msg::Key(Key::Right) => { let y = self.yaw + KEY_TURN; let p = self.pitch; self.ease_to(y, p, 0.2, now); }
            Msg::Key(Key::Char('0')) => self.home(now),
            Msg::Key(Key::Char(c @ '1'..='4')) => self.update(Msg::Show(c as u8 - b'0'), now),
            Msg::Key(Key::Char(' ') | Key::Enter) => { let to = if self.at == 4 { 2 } else { 4 }; self.update(Msg::Show(to), now); }
            Msg::Key(Key::Char('[')) => self.update(Msg::Seed(-1), now),
            Msg::Key(Key::Char(']')) => self.update(Msg::Seed(1), now),
            Msg::Key(Key::Escape) | Msg::Skip => self.skip(now),
            Msg::Key(_) => {}
            Msg::Show(n) => self.go_to(n, now),
            Msg::Play => self.play(now),
            Msg::Away(a) => { self.away = a; if a { self.idle = 0.0; } else { self.input(now); } }
            Msg::Material(i) => {
                let i = i.min(MATERIALS.len() - 1);
                match self.at {
                    3 if i == 0 => self.go_to(2, now),
                    3 if i != self.material => {
                        // A commit from the pressed segment, inside the stage: the old word ahead of the
                        // front, the new behind.
                        let b = self.console().segment(i);
                        self.cmds.push(amenui::morph::Cmd::Commit { x: (b[0] + b[2]) * 0.5, y: (b[1] + b[3]) * 0.5, scope: Some(format!("{KEY}/word-box")) });
                        self.material = i;
                    }
                    2 | 4 if i != 0 => { self.go_to(3, now); self.material = i; }
                    _ => {}
                }
            }
            Msg::Turn => if !self.reduced {
                self.turn = !self.turn;
                self.turned_at = now;
                if self.turn { self.last_input = now - IDLE_AFTER; }
            },
            Msg::Seed(d) => { self.seed = (self.seed as i32 + d).rem_euclid(100) as u32; self.changed = now; self.flash = now; }
            Msg::GatherTo(v) => {
                if self.at == 3 { return self.go_to(2, now); }
                let v = v.clamp(0.0, 1.0);
                if v >= 1.0 { return self.go_to(4, now); }
                if v <= 0.0 { if self.at == 4 { self.go_to(2, now); } self.gather.hold(0.0); return; }
                self.gather.hold(v);
                self.at = if v >= 0.5 { 4 } else { 2 };
            }
            Msg::Scrub(phase, v) => {
                // The thumb is held: the gather is its value on the same frame, no spring between.
                if self.at == 3 { if phase == 0 { self.go_to(2, now); } return; }
                if now < self.lands || self.at == 1 { return; }
                match phase {
                    0 | 1 => {
                        self.scrub = Some(v);
                        self.gather.hold(v);
                        if self.at == 4 && v < 1.0 { (self.ease, self.spin) = (None, 0.0); }
                    }
                    _ => {
                        self.scrub = None;
                        let to = if v < 0.5 { 2 } else { 4 };
                        self.gather.hold(v);
                        if to != self.at { (self.was, self.at, self.changed) = (self.at, to, now); }
                        if to == 4 && self.walk.is_none() { self.point_lamp(self.ptr); }
                        if self.reduced { self.fade = Some((now, FADE)); } else { self.spring(if to == 4 { 1.0 } else { 0.0 }, now); }
                    }
                }
            }
            Msg::Point(p) => {
                self.ptr = p.map(|q| (q.x, q.y));
                self.hover = p.filter(|q| !q.down).and_then(|q| self.ctl_at(q.x, q.y));
                self.input(now);
                let body = self.target() >= 0.5 && self.gather.value > 0.97;
                // Over the console the stone's marks keep what they show: the glyphs there are the
                // console's, and a hover changes only the control under it.
                let on_console = self.shown_state() != 1 && self.ptr.is_some_and(|(x, y)| { let b = self.console().rect(); x >= b[0] && x < b[2] && y >= b[1] && y < b[3] });
                match p {
                    Some(q) if q.down => {}
                    _ if on_console => {}
                    Some(q) if q.glyph.is_some() => self.under = q.glyph,
                    Some(_) if self.over => {}
                    _ => self.under = None,
                }
                // While the hand turns the stone the lamp stays where it was at the press, and over the
                // console.
                if self.grab.is_none() && !on_console { self.point_lamp(self.ptr); }
                let v = self.view();
                match (p, self.grab) {
                    (Some(q), Some(mut g)) if q.down => {
                        if !self.dragged && (q.x - g.x).hypot(q.y - g.y) > DRAG_START {
                            // Measured from where it began, so nothing jumps.
                            self.dragged = true;
                            (g.sx, g.sy) = (q.x, q.y);
                            self.grab = Some(g);
                        }
                        if self.dragged && body {
                            let r = self.placement().2.max(24.0);
                            self.yaw = g.yaw + (q.x - g.sx) / r;
                            self.pitch = self.tilt_for(g.pitch, q.y - g.sy);
                            self.tilt.hold(self.pitch);
                            self.samples.push((q.t.max(now), self.yaw));
                            let cut = q.t.max(now) - 0.2;
                            self.samples.retain(|s| s.0 >= cut);
                        }
                    }
                    (Some(q), None) if q.down => {
                        // A press on the console is the console's; on the stone it holds it.
                        if self.ctl_at(q.x, q.y).is_some() { return; }
                        let on = self.hit_at(q.x, q.y, &v).map(|(h, _)| v.unrot(h));
                        self.grab = Some(Grab { x: q.x, y: q.y, t: now, sx: q.x, sy: q.y, yaw: self.yaw, pitch: self.pitch, on });
                        self.dragged = false;
                        self.samples.clear();
                        (self.spin, self.idle, self.ease) = (0.0, 0.0, None);
                    }
                    (_, Some(g)) => {
                        // Let go: a throw keeps turning at its speed and slows; the tilt springs upright.
                        if self.dragged {
                            let end = p.map_or(now, |q| q.t.max(now));
                            let recent: Vec<(f32, f32)> = self.samples.iter().copied().filter(|s| s.0 >= end - 0.08).collect();
                            let still = self.samples.last().is_none_or(|s| end - s.0 > 0.05);
                            let speed = match (recent.first(), recent.last()) { (Some(a), Some(b)) if b.0 - a.0 > 1e-3 && !still => (b.1 - a.1) / (b.0 - a.0), _ => 0.0 };
                            self.spin = if self.reduced { 0.0 } else { speed.clamp(-THROW_MAX, THROW_MAX) };
                            if self.spin != 0.0 { self.idle_dir = self.spin.signum(); }
                            if self.reduced { self.pitch = REST_PITCH; self.tilt.hold(REST_PITCH); } else { self.tilt.hold(self.pitch); self.tilt.to(REST_PITCH, 0.0, now); }
                        } else if body && now - g.t < 0.3 {
                            if now - self.clicked < 0.35 { self.home(now); self.clicked = f32::NEG_INFINITY; }
                            else {
                                self.clicked = now;
                                if !self.reduced && let Some(o) = g.on {
                                    if self.rings.len() >= 4 { self.rings.remove(0); }
                                    self.rings.push((o, now));
                                }
                            }
                        }
                        self.grab = None;
                        self.dragged = false;
                        self.samples.clear();
                        if p.is_some() { self.point_lamp(self.ptr); }
                    }
                    _ => {}
                }
            }
        }
    }
    fn step(&mut self, now: f32) -> bool {
        let dt = (now - self.t).clamp(0.0, 1.0);
        if self.walk.is_some() && matches!(self.at, 2 | 3) && now < self.lands && self.was == 1 { self.flight = (self.flight.0 + 1, self.flight.1 + (now - self.t).max(0.0)); }
        self.t = now;
        let mut changed = false;
        // The clock of the short path and of the walk starts at the first frame after one has been presented.
        if self.intro_pending && !self.drawn { self.drawn = true; changed = true; }
        else if self.intro_pending {
            self.intro_pending = false;
            if self.full && !self.reduced { self.walk_from(now, true); } else { self.intro_at = Some(now + if self.reduced { 0.0 } else { INTRO_DELAY }); }
            if !self.reduced { self.front_at = Some(now + FRONT_DELAY); }
            changed = true;
        }
        // The walk's beats keep the clock's time: a late frame takes every beat that is due, and a beat
        // that only shows something on the way is dropped when its frame comes 120 ms after it.
        if let Some((t0, steps)) = &self.walk && steps.len() > 1 && steps.last().is_some_and(|s| now - (t0 + s.0) > LATE) {
            self.land(now, Ending::Dropped);
            changed = true;
        }
        if let Some((t0, steps)) = &mut self.walk {
            let t0 = *t0;
            let due: Vec<(f32, Step)> = { let n = steps.iter().take_while(|s| now >= t0 + s.0).count(); steps.drain(..n).collect() };
            for (at, st) in due {
                if now - (t0 + at) > LATE && st.on_the_way() { continue; }
                self.do_step(st, now);
                changed = true;
            }
        }
        if !self.later.is_empty() {
            let (due, rest): (Vec<_>, Vec<_>) = self.later.drain(..).partition(|s| now >= s.0);
            self.later = rest;
            for (_, st) in due { self.do_step(st, now); changed = true; }
        }
        if let Some((t, v)) = self.settle && now >= t { self.gather = Animated::new(v, GATHER_SPRING); self.settle = None; changed = true; }
        if let Some(t0) = self.front_at && now >= t0 { self.front_at = None; self.front = Some(self.caret()); self.fronted = Some(now); changed = true; }
        if let Some(t0) = self.intro_at && now >= t0 { self.intro_at = None; self.go_to(4, now); changed = true; }
        if let Some((t0, d)) = self.fade {
            let to = self.target();
            if now - t0 >= d * 0.5 && self.gather.value != to { self.gather = Animated::new(to, GATHER_SPRING); }
            if now - t0 >= d { self.fade = None; }
            changed = true;
        }
        changed |= self.gather.step(now);
        // 04 → 01: once the glyphs have landed in the listing, the console opens into its lines.
        if self.unfold && self.gather.value <= 0.02 {
            self.unfold = false;
            self.gather = Animated::new(0.0, GATHER_SPRING);
            self.morph("console", "head", MORPH_CONSOLE, now);
            // The console's controls go with it, and so does a focus on one of them.
            self.cmds.push(amenui::morph::Cmd::ClearFocus);
            changed = true;
        }
        for a in &mut self.lamp { changed |= a.step(now); }
        changed |= self.bulge.step(now);
        if self.grab.is_none() {
            if let Some(e) = self.ease {
                let (y, p, done) = e.at(now);
                (self.yaw, self.pitch) = (y, p);
                self.tilt.hold(p);
                if done { self.ease = None; }
                changed = true;
            } else {
                // A throw slows, and ends under 0.02 rad/s; the tilt springs upright.
                self.spin *= (-dt / THROW_TAU).exp();
                if self.spin.abs() < 0.02 { self.spin = 0.0; }
                if self.tilt.step(now) { self.pitch = self.tilt.value; changed = true; }
            }
            // The idle turn: with TURN on it comes back 2 s after the last input over 1 s, and eases
            // out over 600 ms after 20 s without input, when TURN goes off or the page is out of view.
            let quiet = now - self.last_input;
            let want = self.idle_on() && (IDLE_AFTER..IDLE_AFTER + IDLE_QUIET).contains(&quiet);
            let before = self.idle;
            self.idle = if want { (self.idle + dt).min(1.0) } else if self.away || self.reduced { 0.0 } else { (self.idle - dt / 0.6).max(0.0) };
            changed |= self.idle != before;
            // The turn's motes go with it, so nothing is drawn once it has eased out.
            if before > 0.0 && self.idle == 0.0 { self.motes.clear(); changed = true; }
            let rate = self.turn_rate();
            if rate != 0.0 {
                self.yaw += rate * dt;
                changed = true;
                if self.ptr.is_some() { self.point_lamp(self.ptr); }
            }
        }
        if now - self.turned_at < 0.2 || now - self.flash < 0.12 { changed = true; }
        // Rings cross the surface and leave it.
        let r = self.cache.borrow().radius;
        let before = self.rings.len();
        self.rings.retain(|&(_, t0)| RING_SPEED * r * (now - t0) < 2.4 * r + RING_WIDTH);
        changed |= !self.rings.is_empty() || before != self.rings.len();
        for m in &mut self.motes { m.age += dt; m.x += m.vx * dt; m.y += m.vy * dt; m.vy -= 3.0 * dt; }
        let before = self.motes.len();
        self.motes.retain(|m| m.age < MOTE_LIFE);
        changed |= before != self.motes.len() || !self.motes.is_empty();
        // Rim cells shed a few motes while it turns by itself (from the budget; never in the listing).
        if self.at == 4 && self.gather.value > 0.97 && !self.reduced && dt > 0.0 && self.idle > 0.0 {
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
        changed
    }
    fn moving(&self) -> bool {
        self.gather.moving() || self.lamp.iter().any(Animated::moving) || self.bulge.moving() || !self.rings.is_empty() || !self.motes.is_empty() || (self.grab.is_some() && self.dragged)
            || self.spin != 0.0 || self.tilt.moving() || self.ease.is_some() || self.idle > 0.0 || self.unfold
            || self.intro_pending || self.intro_at.is_some() || self.front_at.is_some() || self.fade.is_some()
            || self.walk.is_some() || !self.later.is_empty() || self.settle.is_some() || self.t < self.lands
            || self.t - self.turned_at < 0.2 || self.t - self.flash < 0.12
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
        .on_look(|s| { let mut l = look(s.palette, s.gather.value); l.fx.pitch = s.pitch_px(); l })
        .on_tick(|s, t| s.step(t))
        .on_wake(|s, t| s.wake(t))
        .on_commands(|s| std::mem::take(&mut s.cmds))
        .on_input(|s| s.heard())
}
/// The palettes (AA.9, AA.10): the site's dark, and the proposed dark and light, as (ground, ink,
/// accent, second).
pub fn palette(p: u8) -> ([f32; 3], [f32; 3], [f32; 3], [f32; 3]) {
    let c = |v: u32| [((v >> 16) & 255) as f32 / 255.0, ((v >> 8) & 255) as f32 / 255.0, (v & 255) as f32 / 255.0];
    match p {
        1 => (c(0x1a1b1d), c(0xf2f2ee), c(0xf47a3c), c(0x7db0f7)),
        2 => (c(0xf1f0eb), c(0x141517), c(0xb23f0d), c(0x1a5cc7)),
        _ => ([0.047, 0.051, 0.055], [0.886, 0.890, 0.878], [1.0, 0.42, 0.17], [0.30, 0.62, 1.0]),
    }
}
/// The scene's look, `gathered` of the way from the listing to the stone: the palette's flat ground,
/// the plain material, and a small bloom on the brightest glyphs as the stone gathers (none on paper).
pub fn look(palette_id: u8, gathered: f32) -> amenui::style::Look {
    let g = gathered.clamp(0.0, 1.0);
    let (ground, ink, accent, second) = palette(palette_id);
    let theme = amenui::style::Theme { ground, ground2: ground, ink, accent, second, third: [0.55, 0.88, 1.0], ..amenui::style::LAB };
    let mut l = amenui::style::Look { theme, material: Material::Plain, ..amenui::style::Look::default() };
    l.post.bloom = amenui::post::Bloom { strength: if palette_id == 2 { 0.0 } else { 0.8 * g }, threshold: 0.8, radius: 6.0 };
    l
}
