//! The film (DESIGN.md §6): one continuous piece in fourteen chapters. A prologue shows one small
//! interface as it is and, past a divider, in another material; then thirteen chapters from the most
//! ordinary surface to the most material one, everything together, and the one place that is really
//! three-dimensional. A 2.39:1 stage is centred in the window and letterboxed by the page ground; the
//! bars around it carry a slate (title, chapter, preset), a hairline timeline with a tick per chapter
//! that fills toward the next one while autoplay runs, the chapter's caption, the way back and on, and
//! a hint that stays only until the first input.
//!
//! The stage is display, not copy: text appears only where a chapter's own little interface needs it,
//! and differs in every chapter. Three actors are shared. The chapter's short title, set small as a
//! label at the same place in every chapter, is one element across the whole film, so a cut recomposes
//! it glyph by glyph from the old word to the new one. The number and the sign (a disc with a square
//! set on its lower right quarter, drawn as shapes with true outlines) carry flow ids, so every cut is
//! a match cut: they travel to their new places while everything else changes by the target preset's
//! transition. Chapter 13 is the mechanism (`mechanism.rs`): gears, a cam and a follower, in
//! perspective. Placement is shared: one 12-column grid with fixed margins in a 1600-unit-wide stage,
//! scaled to the window; widgets scale with the stage too.
//!
//! Autoplay moves on every `dwell_ms` and plays a scripted "ghost" interaction in each chapter, so the
//! film plays untouched; any input pauses it in the same frame and it resumes after `idle_ms` (unless
//! the viewer paused it with Space).
use super::mechanism::{self, MMsg, Mech};
use super::{Gallery, Msg, Page, STYLE_SLOT};
use crate::demo::{DUSK, SKY, TERRAIN};
use crate::el::{Drag, El, Font, Key, Paint, Phase, Response, Scope, col, picture, row, space, stack, text, text_lh};
use crate::morph::{Cmd, Ease, FlowSpec, MorphSpec};
use crate::style::{Gaze, GazeMode, Look, Material, gaze};
use crate::widgets::{Edit, Spec, TextEdit, Variant, button_as, dialog, input, number, segmented, switch};
use crate::wrap::{Around, Silhouette};
use taffy::AlignItems;

pub const CHAPTERS: usize = 14;

/// One chapter: the preset it is in (index into `style::styles()`) and its name. Its short title,
/// caption (what it shows of the framework, in plain words) and hint (what you can do) are messages
/// `film.cNN.word`, `film.cNN.caption` and `film.cNN.hint` in the catalogs, NN being its index.
pub struct Chapter { pub style: usize, pub name: &'static str }
impl Chapter {
    pub fn word(i: usize) -> String { l(&format!("film.c{i:02}.word")) }
    pub fn caption(&self, i: usize) -> String { l(&format!("film.c{i:02}.caption")) }
    pub fn hint(&self, i: usize) -> String { l(&format!("film.c{i:02}.hint")) }
}
pub const CH: [Chapter; CHAPTERS] = [
    Chapter { style: 7, name: "Prologue" },
    Chapter { style: 7, name: "Normcore" }, Chapter { style: 8, name: "Cyberminimalism" }, Chapter { style: 6, name: "Tech-Manual" },
    Chapter { style: 0, name: "Algorithmic Gaze" }, Chapter { style: 1, name: "Neo-Dotmatrix" }, Chapter { style: 2, name: "Lo-Fi Print" },
    Chapter { style: 4, name: "Metamorphosis Linocut" }, Chapter { style: 3, name: "Glitch" }, Chapter { style: 5, name: "Transparent Electronics" },
    Chapter { style: 9, name: "Bauhaus" }, Chapter { style: 10, name: "Neo-Y2K" }, Chapter { style: 11, name: "Mixed media" },
    Chapter { style: 6, name: "Mechanism" },
];
/// A message in the current locale (the runtime sets it from the state while the view is built).
fn l(key: &str) -> String { crate::i18n::tr(key) }

/// The film's numbers (named, with defaults, each a number control in its strip of the gallery).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilmSpec {
    /// How long autoplay stays in a chapter, and how long after the last input it waits to resume.
    pub dwell_ms: f32,
    pub idle_ms: f32,
    /// The shared parts' flight between chapters.
    pub flow_ms: f32,
    /// Particles of the finale (up to 1,000,000; the default stays smooth on the WebGL2 tier).
    pub particles: f32,
    /// How loose the prologue's right side is held, so its material shows (0–1).
    pub split_loose: f32,
    /// How long before a cut the way on pulses, ms.
    pub cue_ms: f32,
}
impl Default for FilmSpec {
    fn default() -> Self { Self { dwell_ms: 9000.0, idle_ms: 12000.0, flow_ms: 640.0, particles: 60000.0, split_loose: 0.5, cue_ms: 900.0 } }
}
pub const COUNT_SPEC: Spec = Spec { min: 1000.0, max: 1_000_000.0, step: 1000.0, decimals: 0, unit: "" };
pub const N_SPEC: Spec = Spec { min: 0.0, max: 99.0, step: 1.0, decimals: 0, unit: "" };
pub const GAP_SPEC: Spec = Spec { min: 0.0, max: 60.0, step: 1.0, decimals: 0, unit: "px" };
pub const EXPLODE_SPEC: Spec = Spec { min: 0.0, max: 1.0, step: 0.05, decimals: 2, unit: "" };

/// What the film does.
#[derive(Clone, Debug, PartialEq)]
pub enum FMsg {
    Go(usize),
    Next,
    Prev,
    Play,
    Split(Drag),
    SplitTo(f32),
    Mat(usize),
    Remind,
    Note(TextEdit),
    Invert,
    Num(Edit),
    Query(TextEdit),
    Run(usize),
    Size(Drag),
    Identify(String),
    Lit,
    Obj(f32, f32),
    Gap(Edit),
    Form,
    Scrub(Drag),
    Tear(Drag),
    Error(bool),
    Retry,
    Explode,
    Knob(f32),
    Squares,
    Plane(usize),
    MenuSel(usize),
    Finale,
    Count(Edit),
    Mech(MMsg),
    MechExplode(Edit),
}

/// What the ghost does at a moment of a chapter (seconds after it was entered).
#[derive(Clone, Debug)]
enum Ghost { Msg(FMsg), Press(&'static str), Focus(&'static str), Type(&'static str), Hover(Option<&'static str>), Ptr(f32, f32) }

pub struct Film {
    pub ch: usize,
    pub spec: FilmSpec,
    /// Autoplay on; paused by the viewer with Space (it does not resume by itself then).
    pub playing: bool,
    pub held: bool,
    /// When the chapter was entered, the last input, the last tick, and the next ghost step.
    pub entered: f32,
    pub last_input: f32,
    pub last_tick: f32,
    pub ghost: usize,
    /// When input last paused autoplay (so the Space that did it is read as "pause").
    pub yielded: Option<f32>,
    /// The note edited in chapter 1 and pinned up in the collage; the number; the sign's colours swapped.
    pub note: TextEdit,
    pub n: f32,
    pub inverted: bool,
    /// What the pointer is over, and where it is in the stage (0..1 each way).
    pub hover: Option<String>,
    pub ptr: (f32, f32),
    pub drafts: Option<(u8, String)>,
    // 0: the divider (0..1 of the frame), at the drag's start; the material on its right; the switch
    pub split: f32,
    pub split0: Option<f32>,
    pub mat: usize,
    pub remind: bool,
    // 2
    pub query: TextEdit,
    pub sel: usize,
    // 3: the sign's ring diameter on the drawing (units), and at the drag's start
    pub dia: f32,
    pub dia0: Option<f32>,
    // 4: what is being identified, and since when
    pub ident: Option<(String, f32)>,
    pub known: Vec<String>,
    // 5: the rows' numbers as shown (they refresh one after another); the sign lit up
    pub rows: [f32; 4],
    pub board_at: f32,
    pub lit: bool,
    // 6
    pub obj: (f32, f32),
    pub gap: f32,
    // 7
    pub form: usize,
    pub scrub_from: Option<usize>,
    // 8
    pub tear: Option<f32>,
    pub locks: Vec<(f32, f32)>,
    pub error: bool,
    // 9
    pub exploded: bool,
    // 10
    pub size: f32,
    pub squares: bool,
    pub plane: usize,
    // 11
    pub menu_sel: usize,
    // 12
    pub finale: bool,
    // 13
    pub mech: Mech,
}
impl Default for Film {
    fn default() -> Self {
        Self {
            ch: 0, spec: FilmSpec::default(), playing: true, held: false, entered: 0.0, last_input: -1e9, last_tick: 0.0, ghost: 0, yielded: None,
            note: TextEdit::new(&l("film.c01.note-default")), n: 12.0, inverted: false, hover: None, ptr: (0.5, 0.5), drafts: None,
            split: 0.32, split0: None, mat: 0, remind: true,
            query: TextEdit::default(), sel: 0, dia: 300.0, dia0: None, ident: None, known: vec![],
            rows: [12.0; 4], board_at: 0.0, lit: false, obj: (330.0, 40.0), gap: 8.0, form: 0, scrub_from: None, tear: None, locks: vec![], error: false,
            exploded: false, size: 0.55, squares: false, plane: 0, menu_sel: 1, finale: false, mech: Mech::default(),
        }
    }
}
impl Film {
    /// No input since this chapter was entered: its hint and its cue still show.
    pub fn untouched(&self) -> bool { self.last_input < self.entered }
    /// No input since the film opened: the key legend still shows.
    pub fn fresh(&self) -> bool { self.last_input < -1e8 }
    /// How far autoplay is toward the next cut (0..1), in `PROGRESS_STEPS` steps.
    pub fn progress(&self, now: f32) -> f32 {
        if !self.playing { return 0.0; }
        let p = ((now - self.entered) / (self.spec.dwell_ms / 1000.0).max(0.1)).clamp(0.0, 1.0);
        (p * PROGRESS_STEPS).floor() / PROGRESS_STEPS
    }
    /// The way on pulses for `cue_ms` before autoplay cuts.
    pub fn cueing(&self, now: f32) -> bool { self.playing && now - self.entered >= (self.spec.dwell_ms - self.spec.cue_ms) / 1000.0 }
}
/// The timeline's fill moves in this many steps per chapter (frames only when a step is taken).
pub const PROGRESS_STEPS: f32 = 48.0;

// ── Geometry: the stage and its grid ─────────────────────────────────────────────────────────────

/// The stage in the window (logical px) and `k`, logical px per stage unit (the stage is 1600 units wide).
#[derive(Clone, Copy, Debug)]
pub struct Geo { pub sx: f32, pub sy: f32, pub sw: f32, pub sh: f32, pub k: f32, pub bar: f32 }
pub const ASPECT: f32 = 2.39;
pub const UNITS: f32 = 1600.0;
/// The bars' least heights: the slate on top, the timeline, caption and hint below.
const TOP: f32 = 46.0;
const BOTTOM: f32 = 112.0;
pub fn geo(w: f32, h: f32) -> Geo {
    let side = (w * 0.03).clamp(16.0, 48.0);
    let sw = (w - side * 2.0).min((h - TOP - BOTTOM) * ASPECT).max(160.0);
    let sh = sw / ASPECT;
    let sy = ((h - sh) * 0.5).clamp(TOP, (h - BOTTOM - sh).max(TOP));
    // The slate's type: small, a little larger on large windows, never below 10 px.
    let bar = (sw / 1600.0 * 12.0).clamp(10.0, 15.0);
    Geo { sx: (w - sw) * 0.5, sy, sw, sh, k: sw / UNITS, bar }
}
/// Grid: 72-unit margins, 12 columns, 24-unit gutters.
const MARGIN: f32 = 72.0;
const GUTTER: f32 = 24.0;
const CW: f32 = (UNITS - 2.0 * MARGIN - 11.0 * GUTTER) / 12.0;
pub fn colx(c: f32) -> f32 { MARGIN + c * (CW + GUTTER) }
pub fn span(n: f32) -> f32 { n * CW + (n - 1.0).max(0.0) * GUTTER }
/// The stage's height in units.
pub const STAGE_H: f32 = UNITS / ASPECT;
/// Where the chapter's title sits in every chapter (units), and its size: a label, not a headline.
pub const WORD_AT: (f32, f32) = (MARGIN, 36.0);
pub const WORD_SIZE: f32 = 24.0;
/// What a chapter's content keeps clear of: the title's line.
pub const TOP_Y: f32 = 96.0;
/// The invisible edge zones that go back and on (units wide).
pub const EDGE_W: f32 = 52.0;

/// The type sizes of a chapter (units): display (the number), body, caption.
#[derive(Clone, Copy)]
struct Type { d: f32, b: f32, c: f32 }

/// The sign: a disc `d` across and a square `SIGN_SQ × d` whose corner sits on the disc's centre, over
/// its lower right quarter and out past it. Its bounding box is `SIGN_W × d` each way.
pub const SIGN_SQ: f32 = 0.58;
pub const SIGN_W: f32 = 0.5 + SIGN_SQ;
/// A ring's band, as a fraction of its diameter (the Bauhaus variant).
pub const SIGN_BAND: f32 = 0.2;

/// Building one chapter: the state, the geometry, the chapter, its type.
struct Cx<'a> { g: &'a Gallery, f: &'a Film, geo: Geo, c: usize, t: Type }
impl Cx<'_> {
    fn u(&self, v: f32) -> f32 { v * self.geo.k }
    /// How much widgets (made for a 1280 px window) are scaled on this stage.
    fn z(&self) -> f32 { (self.geo.k / 0.75).clamp(0.55, 1.6) }
    fn key(&self, name: &str) -> String { format!("film/c{:02}/{name}", self.c) }
    /// Place an element at (x, y) units in the stage.
    fn at(&self, el: El<Msg>, x: f32, y: f32) -> El<Msg> { el.absolute(Some(self.u(x)), Some(self.u(y)), None, None) }
    fn display(&self) -> f32 { self.u(self.t.d) }
    fn body(&self) -> f32 { self.u(self.t.b).max(10.0) }
    fn caption(&self) -> f32 { self.u(self.t.c).max(9.0) }
    fn note(&self) -> String { let s = self.f.note.text.trim(); if s.is_empty() { l("film.c01.note-default") } else { s.to_owned() } }
    /// The number actor (it recomposes when it changes).
    fn number(&self, size: f32, paint: Paint) -> El<Msg> { self.number_keyed(&self.key("number"), size, paint).flow("film-number") }
    fn number_keyed(&self, k: &str, size: f32, paint: Paint) -> El<Msg> {
        text(format!("{:02}", self.f.n.round() as i64), size, paint).font(Font::Display).key(k.to_owned())
    }
    /// The sign actor, `d` units across its disc, in the chapter's two colours (swapped when inverted).
    fn sign(&self, d: f32, disc: Paint, square: Paint) -> El<Msg> { self.sign_keyed(&self.key("sign"), d, disc, square).flow("film-sign") }
    fn sign_keyed(&self, k: &str, d: f32, disc: Paint, square: Paint) -> El<Msg> {
        let (disc, square) = if self.f.inverted { (square, disc) } else { (disc, square) };
        sign_el(k, self.u(d), disc, square)
    }
    fn label(&self, s: impl Into<String>, size: f32, paint: Paint) -> El<Msg> { text(s, size, paint) }
    fn mono(&self, s: impl Into<String>, size: f32, paint: Paint) -> El<Msg> { text(s, size, paint).font(Font::Mono) }
}

/// The sign as elements: a disc `d` px across and a square over its lower right quarter, in a box
/// `SIGN_W × d` each way. Its parts are shapes, so every material and the wrap-around text see its true
/// outline.
pub fn sign_el<M>(k: &str, d: f32, disc: Paint, square: Paint) -> El<M> {
    let (w, q) = (d * SIGN_W, d * SIGN_SQ);
    col().key(k.to_owned()).width(w).height(w)
        .child(space(d, d).radius(d * 0.5).fill(disc).key(format!("{k}/disc")).absolute(Some(0.0), Some(0.0), None, None))
        .child(space(q, q).fill(square).key(format!("{k}/square")).absolute(Some(d * 0.5), Some(d * 0.5), None, None))
}
/// The sign's outline, rows of a text box's coordinates (logical px): the sign at (x, y), `d` px across.
pub fn sign_outline(x: f32, y: f32, d: f32) -> Silhouette {
    let (r, w) = (d * 0.5, d * SIGN_W);
    let top = y.floor();
    let rows: Vec<(f32, f32)> = (0..(w.ceil() as usize + 1)).map(|i| {
        let yy = top + i as f32 + 0.5 - y;
        let dy = yy - r;
        let mut span = if dy.abs() <= r { let h = (r * r - dy * dy).max(0.0).sqrt(); (x + r - h, x + r + h) } else { (f32::INFINITY, f32::NEG_INFINITY) };
        if yy >= r && yy <= w { span = (span.0.min(x + r), span.1.max(x + w)); }
        span
    }).collect();
    Silhouette { top, rows }
}

// ── The page ─────────────────────────────────────────────────────────────────────────────────────

/// A chapter's parts and its cue: the box (units) of the one thing to touch first.
type Parts = (Vec<El<Msg>>, Option<[f32; 4]>);

pub fn view(s: &Gallery) -> El<Msg> {
    let f = &s.film;
    let geo = geo(s.width, s.height);
    let th = s.theme();
    let c = f.ch.min(CHAPTERS - 1);
    let t = TYPES[c];
    let cx = Cx { g: s, f, geo, c, t };
    let (parts, cue) = match c {
        0 => prologue(&cx),
        1 => normcore(&cx),
        2 => cyber(&cx),
        3 => manual(&cx),
        4 => detector(&cx),
        5 => board(&cx),
        6 => zine(&cx),
        7 => poster(&cx),
        8 => monitor(&cx),
        9 => device(&cx),
        10 => bauhaus(&cx),
        11 => chrome(&cx),
        12 => collage(&cx),
        _ => machine(&cx),
    };
    // Everything of a chapter lives inside the stage; its own presses reach the stage (DESIGN.md §1.15).
    let mut stage = col().key(format!("film/stage/{c:02}")).width(geo.sw).height(geo.sh).clip().ripple_scope().gaze_scope()
        .absolute(Some(geo.sx), Some(geo.sy), None, None);
    for p in parts { stage = stage.child(p); }
    if let Some(b) = cue.filter(|_| f.untouched()) { stage = stage.child(cue_el(&cx, b)); }
    stage = stage.child(edge(&cx, false)).child(edge(&cx, true));
    // The chapter's title: one element for the whole film, so a cut recomposes it glyph by glyph.
    let word = text(Chapter::word(c), cx.u(WORD_SIZE).max(11.0), Paint::INK).font(Font::Display).key("film/word").recompose(true).nowrap()
        .absolute(Some(geo.sx + cx.u(WORD_AT.0)), Some(geo.sy + cx.u(WORD_AT.1)), None, None);
    col().key("film").fonts(th.roles()).fill_width().fill_height()
        .child(corners(&geo))
        .child(stage)
        .child(word)
        .child(slate(s, &geo))
        .child(bottom(s, &geo))
}

/// The cue: four corner brackets drawn once around the thing to touch, gone after the first input.
fn cue_el(x: &Cx, b: [f32; 4]) -> El<Msg> {
    let (m, len) = (10.0, 16.0);
    let (x0, y0, x1, y1) = (b[0] - m, b[1] - m, b[0] + b[2] + m, b[1] + b[3] + m);
    let w = 1.5;
    let mut c = col().key("film/cue").absolute(Some(0.0), Some(0.0), None, None).width(0.0).height(0.0).respond(Response::Ignore).material(Material::Plain.id()).calm();
    for (i, (cx, cy, sx, sy)) in [(x0, y0, 1.0, 1.0), (x1, y0, -1.0, 1.0), (x0, y1, 1.0, -1.0), (x1, y1, -1.0, -1.0)].into_iter().enumerate() {
        let hx = if sx > 0.0 { cx } else { cx - len };
        let vy = if sy > 0.0 { cy } else { cy - len };
        c = c.child(x.at(space(x.u(len), w).fill(Paint::ACCENT).key(format!("film/cue/h{i}")), hx, cy - if sy > 0.0 { 0.0 } else { w / x.geo.k }))
             .child(x.at(space(w, x.u(len)).fill(Paint::ACCENT).key(format!("film/cue/v{i}")), cx - if sx > 0.0 { 0.0 } else { w / x.geo.k }, vy));
    }
    c
}

/// An edge of the stage that goes back (left) or on (right): invisible until the pointer is over it,
/// then a faint arrow; a press or a sideways swipe from it moves.
fn edge(x: &Cx, on: bool) -> El<Msg> {
    let k = if on { "film/edge/next" } else { "film/edge/prev" };
    let hot = x.f.hover.as_deref() == Some(k);
    let msg = if on { FMsg::Next } else { FMsg::Prev };
    let w = x.u(EDGE_W);
    let mut z = col().key(k).width(w).height(x.geo.sh).align(AlignItems::CENTER).justify(taffy::JustifyContent::CENTER)
        .absolute(if on { None } else { Some(0.0) }, Some(0.0), if on { Some(0.0) } else { None }, None)
        .on_press(Msg::Film(msg)).ripple(Scope::Off).focusable(false).calm().material(Material::Plain.id())
        .label({ let i = if on { (x.c + 1) % CHAPTERS } else { (x.c + CHAPTERS - 1) % CHAPTERS }; crate::t!(crate::i18n::current(), if on { "film.next" } else { "film.prev" }, name = Chapter::word(i)) })
        .on_drag_with(move |d: Drag| (d.phase == Phase::End && d.dx.abs() > 40.0).then_some(Msg::Film(if d.dx < 0.0 { FMsg::Next } else { FMsg::Prev })));
    if hot { z = z.child(text(if on { "›" } else { "‹" }, x.u(40.0).max(18.0), Paint::DIM).key(format!("{k}/arrow"))); }
    z
}

/// Four crop marks just outside the stage's corners: the frame, without a box.
fn corners(g: &Geo) -> El<Msg> {
    let (l, w) = (10.0, 1.0);
    let mut c = col().key("film/frame").absolute(Some(0.0), Some(0.0), None, None).width(0.0).height(0.0).respond(Response::Ignore).material(Material::Plain.id());
    for (i, (x, y, sx, sy)) in [(g.sx, g.sy, -1.0, -1.0), (g.sx + g.sw, g.sy, 1.0, -1.0), (g.sx, g.sy + g.sh, -1.0, 1.0), (g.sx + g.sw, g.sy + g.sh, 1.0, 1.0)].into_iter().enumerate() {
        let hx = if sx < 0.0 { x - 5.0 - l } else { x + 5.0 };
        let vy = if sy < 0.0 { y - 5.0 - l } else { y + 5.0 };
        c = c.child(space(l, w).fill(Paint::DIM).key(format!("film/frame/h{i}")).absolute(Some(hx), Some(y - w * 0.5), None, None))
             .child(space(w, l).fill(Paint::DIM).key(format!("film/frame/v{i}")).absolute(Some(x - w * 0.5), Some(vy), None, None));
    }
    c
}

/// The slate above the stage: the work's title on the left; the chapter and its preset on the right.
fn slate(s: &Gallery, g: &Geo) -> El<Msg> {
    let f = &s.film;
    let st = if f.playing { "▶" } else { "||" };
    row().key("film/slate").material(Material::Plain.id()).width(g.sw).absolute(Some(g.sx), Some((g.sy - g.bar * 1.4 - 22.0).max(4.0)), None, None).align(AlignItems::CENTER).calm()
        .child(text(l("film.title"), g.bar, Paint::TEXT).key("film/title"))
        .child(space(0.0, 0.0).grow(1.0))
        .child(text(format!("{st}  {:02} / {:02}   {}", f.ch, CHAPTERS - 1, CH[f.ch].name), g.bar, Paint::TEXT).font(Font::Mono).key("film/slate/at").recompose(false))
        // The language, next to where the chapter is: the other one's name switches to it.
        .child(space(18.0, 0.0))
        .child({ let other = crate::i18n::Locale::ALL.into_iter().find(|l| *l != s.locale).unwrap_or_default(); button_as("film/lang", other.name(), Variant::Quiet, Msg::Locale(other)).height(g.bar * 1.9).label(l("film.language")) })
}

/// Below the stage: the timeline (a hairline with a tick per chapter; hover a tick for its name, press
/// it to go there; while autoplay runs, a fill grows from this chapter's tick toward the next), then
/// the way back, the caption and the way on, then the hint, the key legend and the way to the gallery.
fn bottom(s: &Gallery, g: &Geo) -> El<Msg> {
    let f = &s.film;
    let y = g.sy + g.sh + 16.0;
    let tick_x = |i: usize| i as f32 / (CHAPTERS - 1) as f32 * (g.sw - 1.0);
    let mut line = col().key("film/timeline").width(g.sw).height(26.0);
    line = line.child(space(g.sw, 1.0).fill(Paint::FAINT).key("film/timeline/rule").absolute(Some(0.0), Some(12.0), None, None));
    let p = f.progress(s.clock);
    if p > 0.0 && f.ch + 1 < CHAPTERS {
        let (a, b) = (tick_x(f.ch), tick_x(f.ch + 1));
        line = line.child(space(((b - a) * p).max(1.0), 1.0).fill(Paint::ACCENT).key("film/timeline/fill").absolute(Some(a), Some(12.0), None, None).recompose(false));
    }
    for (i, chapter) in CH.iter().enumerate() {
        let x = tick_x(i);
        let on = i == f.ch;
        let k = format!("film/tick/{i:02}");
        let hot = f.hover.as_deref() == Some(k.as_str());
        let h = if on { 13.0 } else if hot { 10.0 } else { 7.0 };
        let mark = space(if on { 2.0 } else { 1.0 }, h).fill(if on { Paint::ACCENT } else if hot { Paint::INK } else { Paint::DIM });
        let tick = col().key(k.clone()).width(18.0).height(26.0).align(AlignItems::CENTER).justify(taffy::JustifyContent::CENTER)
            .on_press(Msg::Film(FMsg::Go(i))).ripple(Scope::Off).label(format!("{i:02} {}", Chapter::word(i))).focusable(false)
            .absolute(Some((x - 9.0).clamp(-8.0, g.sw - 10.0)), Some(0.0), None, None)
            .child(mark.key(format!("{k}/mark")));
        line = line.child(tick);
        if hot {
            let name = format!("{i:02} {} · {}", Chapter::word(i), chapter.name);
            let w = name.chars().count() as f32 * g.bar * 0.62 + 8.0;
            line = line.child(text(name, g.bar, Paint::INK).font(Font::Mono).key("film/tick/name").absolute(Some((x - w * 0.5).clamp(0.0, g.sw - w)), Some(-g.bar * 1.5), None, None));
        }
    }
    let ch = &CH[f.ch];
    let prev = (f.ch + CHAPTERS - 1) % CHAPTERS;
    let next = (f.ch + 1) % CHAPTERS;
    let cue = f.cueing(s.clock);
    let way = |k: &str, label: String, m: FMsg, hot: bool| text(label, g.bar, if hot { Paint::ACCENT } else { Paint::DIM }).font(Font::Mono).key(k.to_owned())
        .on_press(Msg::Film(m)).focusable(false).role(crate::el::Role::Link).nowrap();
    let mut notes = row().key("film/notes").width(g.sw).gap(18.0).align(AlignItems::CENTER);
    if f.untouched() { notes = notes.child(text(crate::t!(crate::i18n::current(), "film.try", hint = ch.hint(f.ch)), g.bar, Paint::DIM).key("film/hint").shrink()); }
    notes = notes.child(space(0.0, 0.0).grow(1.0));
    if f.fresh() { notes = notes.child(text(l("film.keys"), g.bar, Paint::FAINT).font(Font::Mono).key("film/keys").nowrap()); }
    // The performance readout, as a word: off, one line, every number (F2 does the same).
    notes = notes.child(text(if s.overlay > 0 { "fps ●" } else { "fps" }, g.bar, if s.overlay > 0 { Paint::ACCENT } else { Paint::DIM }).font(Font::Mono).key("film/fps")
        .on_press(Msg::Perf(0)).focusable(false).role(crate::el::Role::Button).label(l("film.fps")));
    notes = notes.child(button_as("film/gallery", &l("film.gallery"), Variant::Quiet, Msg::Go(Page::Comp(0))));
    col().key("film/bottom").width(g.sw).gap(4.0).absolute(Some(g.sx), Some(y), None, None).calm().material(Material::Plain.id())
        .child(line)
        .child(row().key("film/ways").width(g.sw).gap(18.0).align(AlignItems::CENTER)
            .child(way("film/prev", crate::t!(crate::i18n::current(), "film.prev", name = Chapter::word(prev)), FMsg::Prev, false))
            .child(text(ch.caption(f.ch), g.bar + 1.0, Paint::TEXT).key("film/caption").grow(1.0).shrink().min_width(0.0))
            .child(way("film/next", crate::t!(crate::i18n::current(), "film.next", name = Chapter::word(next)), FMsg::Next, cue)))
        .child(notes)
}

/// Each chapter's three sizes (units): display (the number), body, caption.
const TYPES: [Type; CHAPTERS] = [
    Type { d: 44.0, b: 18.0, c: 12.0 },
    Type { d: 44.0, b: 19.0, c: 13.0 }, Type { d: 30.0, b: 18.0, c: 13.0 }, Type { d: 30.0, b: 16.0, c: 12.0 }, Type { d: 44.0, b: 16.0, c: 12.0 },
    Type { d: 56.0, b: 30.0, c: 14.0 }, Type { d: 64.0, b: 18.0, c: 12.0 }, Type { d: 48.0, b: 120.0, c: 13.0 }, Type { d: 36.0, b: 16.0, c: 12.0 },
    Type { d: 30.0, b: 15.0, c: 12.0 }, Type { d: 64.0, b: 18.0, c: 13.0 }, Type { d: 40.0, b: 20.0, c: 13.0 }, Type { d: 40.0, b: 16.0, c: 12.0 },
    Type { d: 40.0, b: 16.0, c: 12.0 },
];

// ── 0 Prologue: one interface, as it is and in a material ───────────────────────────────────────
/// The materials the prologue's chips choose for the right side, with their names' messages.
pub const SPLIT_MATS: [Material; 9] = [Material::Halftone, Material::Dot, Material::Ascii, Material::Print, Material::Lino, Material::Glitch, Material::Xray, Material::Chrome, Material::Liquid];
/// The frame the interface sits in (units): left, top, right, bottom.
pub const FRAME: [f32; 4] = [MARGIN, TOP_Y, UNITS - MARGIN, 604.0];
/// The divider's place in stage units.
pub fn split_x(f: &Film) -> f32 { FRAME[0] + f.split.clamp(0.0, 1.0) * (FRAME[2] - FRAME[0]) }
/// The small interface, built once per side: `side` prefixes its keys, `actors` gives the number its
/// flow id (only the left copy carries it, so the cut flies from one place).
fn split_panel(x: &Cx, side: &str, actors: bool) -> El<Msg> {
    let f = x.f;
    let k = |n: &str| format!("film/c00/{side}/{n}");
    let z = x.z();
    let (x0, y0) = (FRAME[0] + 40.0, FRAME[1] + 40.0);
    let pw = span(6.0) - 40.0;
    let pic = col().key(k("pic")).width(x.u(pw)).child(picture(DUSK, super::pages::ratio(DUSK)).key(k("picture")).pic_fx(STYLE_SLOT));
    let tx = colx(7.0);
    let num = { let n = x.number_keyed(&k("number"), x.display() * 1.6, Paint::INK); if actors { n.flow("film-number") } else { n } };
    let draft = f.drafts.as_ref().filter(|d| d.0 == 0).map(|d| d.1.as_str());
    let text_col = col().key(k("text")).width(x.u(span(4.0))).gap(x.u(14.0))
        .child(x.mono(l("film.c00.eyebrow"), x.caption(), Paint::DIM).key(k("eyebrow")))
        .child(text(l("film.c00.head"), x.u(34.0), Paint::INK).font(Font::Display).key(k("head")))
        .child(text_lh(l("film.c00.body"), x.body(), x.body() * 1.6, Paint::TEXT).width(x.u(span(4.0))).key(k("body")))
        .child(row().key(k("reading")).gap(x.u(10.0)).align(AlignItems::BASELINE).child(num).child(x.mono(l("film.c00.unit"), x.caption(), Paint::DIM).key(k("unit"))))
        .child(row().key(k("controls")).gap(x.u(22.0)).align(AlignItems::CENTER)
            .child(switch(k("remind"), &l("film.c00.remind"), f.remind, None, Msg::Film(FMsg::Remind)).zoom(z))
            .child(number(&k("count"), &l("film.c00.minutes"), f.n, N_SPEC, draft, |e| Msg::Film(FMsg::Num(e))).zoom(z)))
        .child(row().key(k("act")).child(button_as(k("go"), &l("film.c00.go"), Variant::Accent, Msg::Film(FMsg::Next)).zoom(z)));
    col().key(k("panel")).width(x.u(UNITS)).height(x.u(STAGE_H))
        .child(x.at(pic, x0, y0))
        .child(x.at(text_col, tx, y0))
}
fn prologue(x: &Cx) -> Parts {
    let f = x.f;
    let sx = split_x(f);
    let mat = SPLIT_MATS[f.mat.min(SPLIT_MATS.len() - 1)];
    let (fx0, fy0, fx1, fy1) = (FRAME[0], FRAME[1], FRAME[2], FRAME[3]);
    let mut out = vec![];
    // Hairlines along the frame's edges, running out to the stage's edges: the page's own grid.
    let faint = Paint::FAINT;
    for (i, yy) in [fy0, fy1].iter().enumerate() { out.push(x.at(space(x.u(UNITS), 1.0).fill(faint).key(x.key(&format!("hl{i}"))).respond(Response::Ignore), 0.0, *yy)); }
    for (i, xx) in [fx0, fx1].iter().enumerate() { out.push(x.at(space(1.0, x.u(STAGE_H)).fill(faint).key(x.key(&format!("vl{i}"))).respond(Response::Ignore), *xx, 0.0)); }
    // The two sides: the same interface twice, each clipped to its side of the divider.
    let (lw, rw) = (x.u(sx), x.u(UNITS - sx));
    let left = col().key(x.key("left")).width(lw).height(x.u(STAGE_H)).clip().material(Material::Plain.id())
        .child(split_panel(x, "a", true).absolute(Some(0.0), Some(0.0), None, None));
    let right = col().key(x.key("right")).width(rw).height(x.u(STAGE_H)).clip().material(mat.id()).loosen(f.spec.split_loose)
        .child(split_panel(x, "b", false).absolute(Some(-lw), Some(0.0), None, None));
    out.push(x.at(left, 0.0, 0.0));
    out.push(x.at(right, sx, 0.0));
    // The corner tags, small caps.
    let tag = |s: String, k: &str| x.mono(s.to_uppercase(), x.caption(), Paint::DIM).key(x.key(k)).respond(Response::Ignore);
    out.push(x.at(tag(l("film.c00.before"), "before"), fx0 + 14.0, fy0 + 12.0));
    let after = crate::t!(crate::i18n::current(), "film.c00.after", name = l(&format!("film.c00.mat{}", f.mat)));
    out.push(tag(after, "after").absolute(None, Some(x.u(fy0 + 12.0)), Some(x.u(UNITS - fx1 + 14.0)), None));
    // The divider: a hairline with a round handle; drag it, or Tab to it and use ← →.
    let hd = 30.0;
    out.push(x.at(space(1.5, x.u(fy1 - fy0)).fill(Paint::INK).key(x.key("divider")).respond(Response::Ignore).material(Material::Plain.id()), sx, fy0));
    let handle = col().key(x.key("handle")).width(x.u(hd)).height(x.u(hd)).radius(x.u(hd * 0.5)).fill(Paint::GROUND).border(1.5, Paint::INK)
        .align(AlignItems::CENTER).justify(taffy::JustifyContent::CENTER).material(Material::Plain.id())
        .cursor(crate::el::Cursor::ColResize).focusable(true).label(l("film.c00.divider")).value_range(f.split, 0.0, 1.0)
        .on_drag_with(|d: Drag| Some(Msg::Film(FMsg::Split(d))))
        .on_key({ let v = f.split; move |k, sh| { let st = if sh { 0.2 } else { 0.05 }; match k { Key::Left => Some(Msg::Film(FMsg::SplitTo(v - st))), Key::Right => Some(Msg::Film(FMsg::SplitTo(v + st))), _ => None } } })
        .child(x.mono("‹›", x.caption(), Paint::INK).key(x.key("handle/glyph")));
    out.push(x.at(handle, sx - hd * 0.5, (fy0 + fy1) * 0.5 - hd * 0.5));
    // The chips: plain rectangles of text along the frame's bottom edge; the chosen one filled.
    let mut chips = row().key(x.key("chips")).gap(x.u(6.0)).material(Material::Plain.id());
    for i in 0..SPLIT_MATS.len() {
        let on = i == f.mat;
        let k = x.key(&format!("chip{i}"));
        chips = chips.child(col().key(k.clone()).pad(x.u(10.0), x.u(5.0)).fill(if on { Paint::INK } else { Paint::GROUND })
            .on_press(Msg::Film(FMsg::Mat(i))).focusable(true).role(crate::el::Role::Radio).label(l(&format!("film.c00.mat{i}")))
            .child(x.mono(l(&format!("film.c00.mat{i}")).to_uppercase(), x.caption(), if on { Paint::GROUND } else { Paint::TEXT }).key(format!("{k}/t"))));
    }
    out.push(x.at(chips, fx0 + 14.0, fy1 - 42.0));
    // The sign: the work's mark, outside the frame, opposite the title.
    out.push(x.at(x.sign(42.0, Paint::INK, Paint::ACCENT), fx1 - 42.0 * SIGN_W, 34.0));
    (out, Some([sx - hd * 0.5, (fy0 + fy1) * 0.5 - hd * 0.5, hd, hd]))
}

// ── 1 Normcore: a plain settings form ────────────────────────────────────────────────────────────
fn normcore(x: &Cx) -> Parts {
    let f = x.f;
    let z = x.z();
    let lab = |s: &str| x.label(s, x.body(), Paint::DIM).width(x.u(150.0));
    let draft = x.g.film.drafts.as_ref().filter(|d| d.0 == 1).map(|d| d.1.as_str());
    let form = col().key(x.key("form")).gap(x.u(26.0)).width(x.u(span(6.0)))
        .child(row().key(x.key("row-note")).align(AlignItems::CENTER).child(lab(&l("film.c01.note")))
            .child(input(&x.key("note"), &f.note, &l("film.c01.note"), (x.u(span(6.0) - 150.0) / z).max(120.0), 24, |e| Msg::Film(FMsg::Note(e))).zoom(z)))
        .child(space(x.u(span(6.0)), 1.0).fill(Paint::FAINT).key(x.key("rule")))
        .child(row().key(x.key("row-invert")).align(AlignItems::CENTER).child(lab(&l("film.invert")))
            .child(switch(x.key("invert"), "", f.inverted, None, Msg::Film(FMsg::Invert)).zoom(z)))
        .child(row().key(x.key("row-count")).align(AlignItems::CENTER).child(lab(&l("film.count")))
            .child(number(&x.key("count"), &l("film.count"), f.n, N_SPEC, draft, |e| Msg::Film(FMsg::Num(e))).zoom(z)));
    // The preview: the sign, the note under it and the count, on a quiet card, as a settings page shows
    // what it changes.
    let (cw, ch) = (span(5.0), 440.0);
    let card = col().key(x.key("preview")).width(x.u(cw)).height(x.u(ch)).border(1.0, Paint::FAINT).radius(x.u(10.0))
        .align(AlignItems::CENTER).justify(taffy::JustifyContent::CENTER).gap(x.u(22.0))
        .child(x.sign(200.0, Paint::INK, Paint::ACCENT))
        .child(x.label(x.note(), x.body(), Paint::INK).key(x.key("preview-note")))
        .child(row().key(x.key("preview-count")).gap(x.u(8.0)).align(AlignItems::BASELINE).child(x.number(x.display(), Paint::INK)).child(x.label(l("film.count"), x.caption(), Paint::DIM)));
    let (fx, fy) = (colx(0.0), TOP_Y + 40.0);
    (vec![
        x.at(x.label(l("film.c01.preview"), x.caption(), Paint::DIM).key(x.key("preview-label")), colx(7.0), TOP_Y - 22.0),
        x.at(form, fx, fy),
        x.at(card, colx(7.0), TOP_Y),
    ], Some([fx + 150.0, fy - 4.0, span(6.0) - 150.0, 40.0]))
}

// ── 2 Cyberminimalism: a command palette ─────────────────────────────────────────────────────────
/// The palette's commands: the chapters, then two that act here.
pub fn commands() -> Vec<String> {
    let mut v: Vec<String> = CH.iter().enumerate().map(|(i, c)| format!("go {i:02} {}", c.name.to_lowercase())).collect();
    v.push("invert sign".into());
    v.push("count +1".into());
    v
}
/// The commands the query matches (every character of it, in order), as indices.
pub fn matches(q: &str) -> Vec<usize> {
    let q: Vec<char> = q.to_lowercase().chars().filter(|c| !c.is_whitespace()).collect();
    commands().iter().enumerate().filter(|(_, c)| {
        let mut it = c.chars();
        q.iter().all(|ch| it.any(|x| x == *ch))
    }).map(|(i, _)| i).collect()
}
fn cyber(x: &Cx) -> Parts {
    let f = x.f;
    let found = matches(&f.query.text);
    let cmds = commands();
    let sel = f.sel.min(found.len().saturating_sub(1));
    let w = span(6.0);
    let mut list = col().key(x.key("results")).gap(x.u(4.0)).width(x.u(w));
    // Rows are keyed by place, so a new filter recomposes the characters that changed in each row.
    for r in 0..6 {
        let (s, on) = match found.get(r) { Some(&i) => (cmds[i].clone(), r == sel), None => (String::new(), false) };
        let row_el = row().key(x.key(&format!("row{r}"))).height(x.u(40.0)).gap(x.u(14.0)).align(AlignItems::CENTER).pad(x.u(12.0), 0.0)
            .child(space(2.0, x.u(22.0)).fill(if on { Paint::ACCENT } else { Paint::GROUND }).key(x.key(&format!("row{r}/bar"))))
            .child(x.mono(s, x.body(), if on { Paint::ACCENT } else { Paint::TEXT }).key(x.key(&format!("row{r}/text"))));
        let row_el = match found.get(r) { Some(&i) => row_el.on_press(Msg::Film(FMsg::Run(i))).focusable(false), None => row_el };
        list = list.child(row_el);
    }
    let z = x.z();
    let field = input(&x.key("query"), &f.query, &l("film.c02.placeholder"), (x.u(w - 40.0) / z).max(120.0), 32, |e| Msg::Film(FMsg::Query(e))).zoom(z);
    (vec![
        x.at(x.mono("results", x.caption(), Paint::DIM).key(x.key("n-label")), colx(10.0), TOP_Y + 40.0),
        x.at(x.number(x.display(), Paint::TEXT).font(Font::Mono), colx(10.0), TOP_Y + 60.0),
        x.at(row().key(x.key("prompt")).gap(x.u(12.0)).align(AlignItems::CENTER).child(x.mono("›", x.body() * 1.3, Paint::ACCENT)).child(field), colx(3.0), 168.0),
        x.at(space(x.u(w), 1.0).fill(Paint::FAINT).key(x.key("rule")), colx(3.0), 226.0),
        x.at(list, colx(3.0), 240.0),
        x.at(x.mono(crate::t!(crate::i18n::current(), "film.c02.status", found = found.len(), all = cmds.len()), x.caption(), Paint::DIM).key(x.key("count")).recompose(false), colx(3.0), 520.0),
        x.at(x.sign(80.0, Paint::DIM, Paint::ACCENT), colx(11.0) - 22.0, 500.0),
    ], None)
}

// ── 3 Tech-Manual: the sign on a drawing sheet, dimensioned ──────────────────────────────────────
/// The sign's parts on the drawing.
pub const PARTS: [&str; 4] = ["film.c03.part0", "film.c03.part1", "film.c03.part2", "film.c03.part3"];
fn manual(x: &Cx) -> Parts {
    let f = x.f;
    let d = f.dia.clamp(160.0, 320.0);
    let (w, q) = (d * SIGN_W, d * SIGN_SQ);
    let (vw, vh) = (span(7.0), 500.0);
    let (ox, oy) = (colx(0.0), TOP_Y);
    let (sx, sy) = (ox + (vw - w) * 0.5, oy + (vh - w) * 0.5 - 10.0);
    let mut out = vec![];
    // The sheet: a frame, the sign at its middle (drag across it to resize it).
    out.push(x.at(space(x.u(vw), x.u(vh)).border(1.0, Paint::INK).key(x.key("frame")).respond(Response::Ignore), ox, oy));
    // Centre lines through the disc, as a drawing has them.
    out.push(x.at(space(x.u(d + 40.0), 1.0).fill(Paint::DIM).key(x.key("axis-h")).respond(Response::Ignore), sx - 20.0, sy + d * 0.5));
    out.push(x.at(space(1.0, x.u(d + 40.0)).fill(Paint::DIM).key(x.key("axis-v")).respond(Response::Ignore), sx + d * 0.5, sy - 20.0));
    let sign = x.sign(d, Paint::INK, Paint::hue(0)).cursor(crate::el::Cursor::ColResize).focusable(true).label(l("film.c03.size"))
        .on_drag_with(|dr: Drag| Some(Msg::Film(FMsg::Size(dr))))
        .on_key(move |k, sh| { let s = if sh { 40.0 } else { 10.0 }; match k { Key::Right | Key::Up => Some(Msg::Film(FMsg::Size(Drag { phase: Phase::End, x: 0.0, y: 0.0, w: 1.0, h: 1.0, dx: s, dy: 0.0 }))), Key::Left | Key::Down => Some(Msg::Film(FMsg::Size(Drag { phase: Phase::End, x: 0.0, y: 0.0, w: 1.0, h: 1.0, dx: -s, dy: 0.0 }))), _ => None } });
    out.push(x.at(sign, sx, sy));
    // Callouts: a small mark on each part, numbered; the row under the pointer marks its callout.
    let marks = [(sx + d * 0.22, sy + d * 0.3), (sx + d * 0.5 + q * 0.62, sy + d * 0.5 + q * 0.62), (sx + w, sy + w * 0.5), (sx + d * 0.5, sy + d * 0.5)];
    for (i, (px, py)) in marks.iter().enumerate() {
        let on = f.hover.as_deref().is_some_and(|h| h.ends_with(&format!("part{i}")));
        let s = if on { 14.0 } else { 9.0 };
        let mk = space(x.u(s).max(5.0), x.u(s).max(5.0)).radius(x.u(s)).fill(if on { Paint::ACCENT } else { Paint::INK }).border(1.5, Paint::GROUND).key(x.key(&format!("callout{i}")))
            .annotate(l(PARTS[i])).respond(Response::Ignore);
        out.push(x.at(mk, px - s * 0.5, py - s * 0.5));
    }
    // Dimensions from the sign's own geometry: the overall width under it, the disc's diameter at its
    // left, the square's side at its right.
    let dim = |a: (f32, f32), b: (f32, f32), name: &str, value: f32, out: &mut Vec<El<Msg>>| {
        let horizontal = (a.1 - b.1).abs() < 0.5;
        if horizontal {
            out.push(x.at(space(x.u(b.0 - a.0), 1.0).fill(Paint::INK).key(x.key(&format!("dim-{name}"))), a.0, a.1));
            for (j, px) in [a.0, b.0].iter().enumerate() { out.push(x.at(space(1.0, x.u(16.0)).fill(Paint::INK).key(x.key(&format!("dim-{name}/{j}"))), *px, a.1 - 8.0)); }
            out.push(x.at(x.mono(format!("{value:.1}"), x.caption(), Paint::INK).key(x.key(&format!("dim-{name}/v"))).recompose(false), (a.0 + b.0) * 0.5 - 24.0, a.1 + 6.0));
        } else {
            out.push(x.at(space(1.0, x.u(b.1 - a.1)).fill(Paint::INK).key(x.key(&format!("dim-{name}"))), a.0, a.1));
            for (j, py) in [a.1, b.1].iter().enumerate() { out.push(x.at(space(x.u(16.0), 1.0).fill(Paint::INK).key(x.key(&format!("dim-{name}/{j}"))), a.0 - 8.0, *py)); }
            // A dimension on the left reads from the left of its line.
            let lx = if name == "d" { a.0 - 76.0 } else { a.0 + 10.0 };
            out.push(x.at(x.mono(format!("{value:.1}"), x.caption(), Paint::INK).key(x.key(&format!("dim-{name}/v"))).recompose(false), lx, (a.1 + b.1) * 0.5 - 8.0));
        }
    };
    dim((sx, sy + w + 26.0), (sx + w, sy + w + 26.0), "w", w, &mut out);
    dim((sx - 40.0, sy), (sx - 40.0, sy + d), "d", d, &mut out);
    dim((sx + w + 30.0, sy + d * 0.5), (sx + w + 30.0, sy + w), "q", q, &mut out);
    // The parts list.
    let values = [format!("Ø {d:.1}"), format!("{q:.1} × {q:.1}"), format!("{w:.1}"), format!("{:.1}, {:.1}", d * 0.5, d * 0.5)];
    let mut list = col().key(x.key("parts")).width(x.u(span(4.0))).gap(0.0)
        .child(row().key(x.key("parts/head")).height(x.u(34.0)).align(AlignItems::CENTER).gap(x.u(16.0))
            .child(x.mono("№", x.caption(), Paint::DIM).width(x.u(40.0))).child(x.mono(l("film.c03.part"), x.caption(), Paint::DIM).width(x.u(120.0))).child(x.mono(l("film.c03.dimension"), x.caption(), Paint::DIM)));
    for (i, name) in PARTS.iter().enumerate() {
        let k = x.key(&format!("part{i}"));
        let on = f.hover.as_deref() == Some(k.as_str());
        list = list.child(space(x.u(span(4.0)), 1.0).fill(Paint::FAINT).key(x.key(&format!("part{i}/rule"))))
            .child(row().key(k).height(x.u(40.0)).align(AlignItems::CENTER).gap(x.u(16.0)).on_press(Msg::Film(FMsg::Num(Edit::Set(f.n)))).focusable(false)
                .child(x.mono(format!("{:02}", i + 1), x.body(), if on { Paint::ACCENT } else { Paint::INK }).width(x.u(40.0)))
                .child(x.label(l(name), x.body(), if on { Paint::ACCENT } else { Paint::INK }).width(x.u(120.0)))
                .child(x.mono(values[i].clone(), x.caption(), Paint::DIM).key(x.key(&format!("part{i}/v"))).recompose(false)));
    }
    out.push(x.at(list, colx(8.0), TOP_Y));
    // The title block: what the drawing is, its name, the scale with the number.
    let block = col().key(x.key("titleblock")).width(x.u(span(4.0))).pad(x.u(16.0), x.u(12.0)).gap(x.u(6.0)).border(1.0, Paint::INK)
        .child(x.mono(l("film.c03.figure"), x.caption(), Paint::DIM).key(x.key("figure")))
        .child(x.label(l("film.c03.drawing"), x.body() * 1.5, Paint::INK).key(x.key("drawing")))
        .child(row().gap(x.u(10.0)).align(AlignItems::BASELINE).key(x.key("scale")).child(x.mono(l("film.c03.scale"), x.caption(), Paint::DIM)).child(x.number(x.body() * 1.4, Paint::INK).font(Font::Mono)));
    out.push(x.at(block, colx(8.0), TOP_Y + 360.0));
    (out, Some([sx, sy, w, w]))
}

// ── 4 Algorithmic Gaze: the stage as a detector sees it ──────────────────────────────────────────
/// How long each step of identifying takes (s): loosen, scan, then label and form again.
const ID_LOOSEN: f32 = 0.35;
const ID_SCAN: f32 = 0.8;
const ID_FORM: f32 = 0.35;
/// What identifying `k` is doing at `now`: its looseness, the scan line's place (0..1 of its height, or
/// none), whether its facts are written.
fn ident_at(f: &Film, k: &str, now: f32) -> (f32, Option<f32>, bool) {
    let known = f.known.iter().any(|x| x == k);
    let Some((ik, t0)) = &f.ident else { return (0.0, None, known) };
    if ik != k { return (0.0, None, known); }
    let e = now - t0;
    if e < ID_LOOSEN { return ((e / ID_LOOSEN).min(1.0), None, false); }
    if e < ID_LOOSEN + ID_SCAN { return (1.0, Some((e - ID_LOOSEN) / ID_SCAN), false); }
    let r = ((e - ID_LOOSEN - ID_SCAN) / ID_FORM).clamp(0.0, 1.0);
    (1.0 - r, None, true)
}
/// A thing the detector can identify: its element, key, kind, the facts it writes once known, and its
/// box in stage units (for the scan).
type Thing = (El<Msg>, String, String, String, [f32; 4]);
fn detector(x: &Cx) -> Parts {
    let f = x.f;
    let now = x.g.clock;
    let mut out = vec![];
    let label = l("film.c04.label");
    let ls = x.t.d * 0.72;
    let lw = label.chars().count() as f32 * ls * 0.9;
    let sd = 250.0;
    let things: Vec<Thing> = vec![
        (x.label(label.clone(), x.u(ls), Paint::INK).font(Font::Display).key(x.key("label")).nowrap(), x.key("label"), l("film.c04.text"), crate::t!(crate::i18n::current(), "film.c04.chars", n = label.chars().count()), [colx(0.0), TOP_Y + 40.0, lw, ls * 1.3]),
        (x.number(x.display(), Paint::ACCENT), x.key("number"), l("film.c04.number"), crate::t!(crate::i18n::current(), "film.c04.value", v = format!("{:02}", f.n.round() as i64)), [colx(11.0), TOP_Y + 40.0, x.t.d * 1.2, x.t.d * 1.3]),
        (x.sign(sd, Paint::INK, Paint::ACCENT), x.key("sign"), l("film.sign"), l("film.c04.sign-facts"), [colx(4.5), 220.0, sd * SIGN_W, sd * SIGN_W]),
        (col().key(x.key("picture")).width(x.u(span(3.0))).child(picture(TERRAIN, super::pages::ratio(TERRAIN)).key(x.key("terrain")).pic_fx(STYLE_SLOT)), x.key("picture"), l("film.c04.picture"),
            format!("{:.2} : 1", 1.0 / super::pages::ratio(TERRAIN)), [colx(9.0), 300.0, span(3.0), span(3.0) * super::pages::ratio(TERRAIN)]),
        (x.mono(format!("frame {:04}", (f.n * 37.0) as i64 % 10000), x.body(), Paint::DIM).key(x.key("caption")), x.key("caption"), l("film.c04.text"), l("film.c04.caption-facts"), [colx(0.0), 560.0, 10.0 * x.t.b * 0.6, x.t.b * 1.3]),
    ];
    for (el, k, kind, facts, b) in things {
        let (loose, scan, known) = ident_at(f, &k, now);
        let note = if known { crate::t!(crate::i18n::current(), "film.c04.known", kind = kind, facts = facts.as_str()) } else { crate::t!(crate::i18n::current(), "film.c04.unknown", kind = kind) };
        let el = el.annotate(note).legible(false).respond(Response::Loosen).loose(loose).on_press(Msg::Film(FMsg::Identify(k.clone()))).focusable(true);
        out.push(x.at(el, b[0], b[1]));
        // The scan: a bright line sweeping down with two fainter ones after it, inside the thing's box.
        if let Some(sp) = scan {
            for (j, (lag, a)) in [(0.0, 1.0), (0.05, 0.45), (0.11, 0.2)].iter().enumerate() {
                let yy = b[1] + b[3] * (sp - lag).clamp(0.0, 1.0);
                out.push(x.at(space(x.u(b[2] + 16.0), 1.5).fill(Paint::ACCENT.alpha(*a)).key(format!("{k}/scan{j}")).respond(Response::Ignore).material(Material::Plain.id()), b[0] - 8.0, yy));
            }
        }
    }
    (out, Some([colx(4.5), 220.0, sd * SIGN_W, sd * SIGN_W]))
}

// ── 5 Neo-Dotmatrix: a departure board ───────────────────────────────────────────────────────────
pub const DEST: [&str; 4] = ["film.c05.dest0", "film.c05.dest1", "film.c05.dest2", "film.c05.dest3"];
fn board(x: &Cx) -> Parts {
    let f = x.f;
    let mut b = col().key(x.key("board")).width(x.u(span(6.0))).gap(x.u(14.0))
        .child(row().key(x.key("board/head")).gap(x.u(24.0))
            .child(x.label(l("film.c05.time"), x.caption(), Paint::DIM).width(x.u(130.0))).child(x.label(l("film.c05.to"), x.caption(), Paint::DIM).width(x.u(330.0))).child(x.label(l("film.c05.gate"), x.caption(), Paint::DIM)));
    for (i, d) in DEST.iter().enumerate() {
        let v = f.rows[i].round() as i64 + i as i64 * 7;
        b = b.child(row().key(x.key(&format!("row{i}"))).gap(x.u(24.0)).align(AlignItems::CENTER)
            .child(x.label(format!("{:02}:{:02}", (9 + v / 60) % 24, v % 60), x.body(), Paint::ACCENT).key(x.key(&format!("row{i}/time"))).width(x.u(130.0)))
            .child(x.label(l(d), x.body(), Paint::INK).key(x.key(&format!("row{i}/dest"))).width(x.u(330.0)).nowrap())
            .child(x.label(format!("{}", (v % 9) + 1), x.body(), Paint::INK).key(x.key(&format!("row{i}/gate")))));
    }
    // The sign is dots until the pointer is on it (or it was pressed): then it condenses.
    let on = f.lit || f.hover.as_deref() == Some(x.key("sign").as_str());
    let sd = 330.0;
    let sign = x.sign(sd, Paint::INK, Paint::ACCENT).loose(if on { 0.0 } else { 0.8 }).respond(Response::Loosen).legible(false)
        .on_press(Msg::Film(FMsg::Lit)).label(l("film.sign"));
    let (sx, sy) = (colx(7.5), 170.0);
    (vec![
        x.at(b, colx(0.0), TOP_Y + 110.0),
        x.at(x.label(l("film.c05.minute"), x.caption(), Paint::DIM).key(x.key("n-label")), colx(0.0), TOP_Y + 4.0),
        x.at(x.number(x.display(), Paint::ACCENT), colx(0.0), TOP_Y + 24.0),
        x.at(sign, sx, sy),
    ], Some([sx, sy, sd * SIGN_W, sd * SIGN_W]))
}

// ── 6 Lo-Fi Print: a zine spread ─────────────────────────────────────────────────────────────────
const ZINE_D: f32 = 210.0;
fn zine(x: &Cx) -> Parts {
    let f = x.f;
    let (tx, ty, tw, th) = (colx(0.0), 196.0, span(6.0) - 30.0, 420.0);
    let (ow, oh) = (ZINE_D * SIGN_W, ZINE_D * SIGN_W);
    let (bx, by) = (f.obj.0.clamp(-60.0, tw - ow + 60.0), f.obj.1.clamp(-40.0, th - oh + 40.0));
    // The obstacle in the text's own box, logical px: the sign's true outline.
    // On the wrap's grid, so dragging by less than a step never moves a line break.
    let q = |v: f32| (v / crate::wrap::STEP).round() * crate::wrap::STEP;
    let sil = sign_outline(q(x.u(bx)), q(x.u(by)), x.u(ZINE_D));
    let body = text_lh(l("film.c06.zine"), x.body(), x.body() * 1.7, Paint::INK).key(x.key("body")).width(x.u(tw)).around(Around::new(vec![sil], x.u(f.gap)));
    let draft = x.g.film.drafts.as_ref().filter(|d| d.0 == 6).map(|d| d.1.as_str());
    let sign = x.sign(ZINE_D, Paint::INK, Paint::ACCENT).cursor(crate::el::Cursor::Grab).focusable(true).label(l("film.sign")).unframed()
        .on_drag_with({ let (bx, by) = (bx, by); let k = x.geo.k; move |d: Drag| Some(Msg::Film(FMsg::Obj(bx + d.dx / k, by + d.dy / k))) })
        .on_key({ let (bx, by) = (bx, by); move |key, sh| { let s = if sh { 40.0 } else { 10.0 }; let (dx, dy) = match key { Key::Left => (-s, 0.0), Key::Right => (s, 0.0), Key::Up => (0.0, -s), Key::Down => (0.0, s), _ => return None }; Some(Msg::Film(FMsg::Obj(bx + dx, by + dy))) } });
    let z = x.z();
    (vec![
        x.at(x.label(l("film.c06.head"), x.u(x.t.d), Paint::INK).font(Font::Display).weight(1.5).key(x.key("head")).nowrap(), colx(0.0), TOP_Y),
        x.at(body, tx, ty),
        x.at(sign, tx + bx, ty + by),
        x.at(space(1.0, x.u(500.0)).fill(Paint::FAINT).key(x.key("gutter")), 800.0, TOP_Y),
        x.at(text_lh(l("film.c06.zine2"), x.body(), x.body() * 1.7, Paint::INK).key(x.key("body2")).width(x.u(span(5.0))), colx(6.5), ty),
        x.at(number(&x.key("gap"), &l("film.c06.gap"), f.gap, GAP_SPEC, draft, |e| Msg::Film(FMsg::Gap(e))).width((x.u(300.0) / z).max(220.0)).zoom(z), colx(6.5), 470.0),
        x.at(row().key(x.key("folio")).gap(x.u(10.0)).align(AlignItems::BASELINE).child(x.label("—", x.body(), Paint::DIM)).child(x.number(x.body() * 1.2, Paint::INK)).child(x.label("—", x.body(), Paint::DIM)), 1170.0, 570.0),
    ], None)
}

// ── 7 Metamorphosis Linocut: a poster that becomes itself ────────────────────────────────────────
pub const FORMS: usize = 3;
pub fn form_key(i: usize) -> String { format!("film/c07/form{i}") }
fn poster(x: &Cx) -> Parts {
    let f = x.f;
    let (w, h) = (span(7.0), 470.0);
    let forms = [
        col().key(form_key(0)).width(x.u(w)).height(x.u(h)).align(AlignItems::CENTER).justify(taffy::JustifyContent::CENTER)
            .child(x.sign_keyed("film/c07/sign-form", 360.0, Paint::INK, Paint::hue(0))),
        col().key(form_key(1)).width(x.u(w)).height(x.u(h)).align(AlignItems::CENTER).justify(taffy::JustifyContent::CENTER)
            .child(text(l("film.c07.carve"), x.u(380.0), Paint::INK).font(Font::Display).weight(x.u(3.0)).key("film/c07/letter").nowrap()),
        col().key(form_key(2)).width(x.u(w * 0.9)).child(picture(SKY, super::pages::ratio(SKY)).key("film/c07/print").pic_fx(STYLE_SLOT).loose(1.0)),
    ];
    let current = f.form % FORMS;
    let st = stack(forms.into_iter().enumerate().map(|(i, el)| el.hidden_until_morphed(i != current).respond(Response::Loosen).loose(0.55)))
        .key(x.key("forms")).width(x.u(w)).height(x.u(h)).focusable(true).label(l("film.c07.morph")).cursor(crate::el::Cursor::Grab)
        .on_drag_with(|d: Drag| Some(Msg::Film(FMsg::Scrub(d))))
        .on_key(|k, _| matches!(k, Key::Enter | Key::Char(' ')).then_some(Msg::Film(FMsg::Form)));
    let (px, py) = (colx(2.5), 140.0);
    (vec![
        x.at(x.label("No.", x.caption(), Paint::DIM).key(x.key("n-label")), colx(10.0) + 30.0, TOP_Y - 20.0),
        x.at(x.number(x.display(), Paint::hue(0)).weight(2.0), colx(10.0) + 30.0, TOP_Y),
        x.at(st, px, py),
        // The sign actor stands small in the corner, as the poster's mark.
        x.at(x.sign(70.0, Paint::INK, Paint::hue(0)), colx(11.0) - 6.0, 560.0),
        x.at(x.label(l("film.c07.note"), x.caption(), Paint::DIM).key(x.key("note")), colx(0.0), 610.0),
    ], Some([px + w * 0.5 - 200.0, py + h * 0.5 - 200.0, 400.0, 400.0]))
}

// ── 8 Glitch: a signal monitor ───────────────────────────────────────────────────────────────────
pub const BARS: usize = 96;
fn monitor(x: &Cx) -> Parts {
    let f = x.f;
    let (wx, wy, ww, wh) = (colx(0.0), 190.0, span(12.0), 300.0);
    let bw = ww / BARS as f32;
    let mut wave = col().key(x.key("wave")).width(x.u(ww)).height(x.u(wh)).cursor(crate::el::Cursor::Grab)
        .on_drag_with(|d: Drag| Some(Msg::Film(FMsg::Tear(d)))).focusable(false).respond(Response::Loosen);
    let band = 46.0;
    for i in 0..BARS {
        let ph = i as f32 / BARS as f32 * std::f32::consts::TAU;
        let a = (0.45 * (ph * 3.0 + f.n * 0.3).sin() + 0.3 * (ph * 7.0).sin() + 0.15 * (ph * 17.0 + 1.0).sin()).abs();
        let bh = (a * wh).clamp(6.0, wh);
        let (y0, y1) = (wh * 0.5 - bh * 0.5, wh * 0.5 + bh * 0.5);
        // Cut the bar where a band is torn (loose while held) or locked (moved sideways, at rest).
        let mut cuts: Vec<(f32, f32, f32, f32)> = vec![(y0, y1, 0.0, 0.0)];
        let mut bands: Vec<(f32, f32, f32, f32)> = f.locks.iter().map(|&(c, dx)| (c * wh - band * 0.5, c * wh + band * 0.5, dx, 0.0)).collect();
        if let Some(c) = f.tear { bands.push((c * wh - band * 0.5, c * wh + band * 0.5, 0.0, 1.0)); }
        for (b0, b1, dx, loose) in bands {
            let mut next = vec![];
            for (a0, a1, d0, l0) in cuts {
                if a1 <= b0 || a0 >= b1 { next.push((a0, a1, d0, l0)); continue; }
                if a0 < b0 { next.push((a0, b0, d0, l0)); }
                next.push((a0.max(b0), a1.min(b1), d0 + dx, l0.max(loose)));
                if a1 > b1 { next.push((b1, a1, d0, l0)); }
            }
            cuts = next;
        }
        for (j, (a0, a1, dx, loose)) in cuts.into_iter().enumerate() {
            let hue = if (i * 5) % 11 == 0 { Paint::hue(1) } else { Paint::INK };
            wave = wave.child(space(x.u(bw * 0.55).max(1.0), x.u(a1 - a0).max(1.0)).fill(hue).key(x.key(&format!("bar{i}/{j}"))).loose(loose)
                .absolute(Some(x.u(i as f32 * bw + dx)), Some(x.u(a0)), None, None));
        }
    }
    let close = Msg::Film(FMsg::Error(false));
    let z = x.z();
    let dlg = dialog(&x.key("error"), f.error, &l("film.c08.lost"), 380.0, 0.35, close.clone(),
        col().gap(14.0).child(x.label(l("film.c08.body"), 14.0, Paint::TEXT).width(330.0))
            .child(row().gap(10.0).child(button_as(x.key("retry"), &l("film.c08.retry"), Variant::Accent, Msg::Film(FMsg::Retry))).child(button_as(x.key("ignore"), &l("film.c08.ignore"), Variant::Default, close))));
    (vec![
        x.at(x.mono("SNR dB", x.caption(), Paint::DIM).key(x.key("n-label")), colx(10.0), TOP_Y - 20.0),
        x.at(x.number(x.display(), Paint::hue(1)).font(Font::Mono), colx(10.0), TOP_Y),
        x.at(x.mono("CH 08", x.caption(), Paint::DIM).key(x.key("channel")), colx(0.0), wy - 30.0),
        x.at(space(x.u(ww), 1.0).fill(Paint::FAINT).key(x.key("axis")), wx, wy + wh * 0.5),
        x.at(wave, wx, wy),
        x.at(x.mono(crate::t!(crate::i18n::current(), "film.c08.status", n = f.locks.len()), x.caption(), Paint::DIM).key(x.key("status")).recompose(false), colx(0.0), 540.0),
        x.at(button_as(x.key("raise"), &l("film.c08.raise"), Variant::Default, Msg::Film(FMsg::Error(true))).zoom(z), colx(6.0), 532.0),
        x.at(x.sign(96.0, Paint::INK, Paint::hue(1)), colx(11.0) - 40.0, 520.0),
        dlg,
    ], Some([wx + ww * 0.3, wy + wh * 0.3, ww * 0.4, wh * 0.4]))
}

// ── 9 Transparent Electronics: a clear handheld, its layers ──────────────────────────────────────
pub const LAYERS: [&str; 4] = ["film.c09.layer0", "film.c09.layer1", "film.c09.layer2", "film.c09.layer3"];
/// The device's size (units) and where it stands when assembled.
const DEV: (f32, f32) = (340.0, 500.0);
fn device(x: &Cx) -> Parts {
    let f = x.f;
    let (dw, dh) = DEV;
    let cx0 = (UNITS - dw) * 0.5;
    let cy0 = TOP_Y;
    let (px, py) = (f.ptr.0 - 0.5, f.ptr.1 - 0.5);
    let u = |v: f32| x.u(v);
    let at = |el: El<Msg>, ex: f32, ey: f32| el.absolute(Some(u(ex)), Some(u(ey)), None, None);
    // The wheel: a ring low on the face with the sign as its button.
    let (wc, wd) = ((dw * 0.5, 370.0), 180.0);
    let mut out = vec![];
    for (i, name) in LAYERS.iter().enumerate() {
        let depth = i as f32 - 1.5;
        // Apart: the layers stand side by side in equal steps on the stage's centre line, each named;
        // together they are one body. The pointer moves the nearer ones a little more.
        let step = if f.exploded { dw + 40.0 } else { 0.0 };
        let (lx, ly) = (cx0 + depth * step + px * (i as f32) * 8.0, cy0 + py * (i as f32) * 6.0);
        let k = x.key(&format!("layer{i}"));
        let mut layer = col().key(k.clone()).width(u(dw)).height(u(dh)).radius(u(36.0)).flow(format!("film-layer{i}"));
        if f.exploded { layer = layer.child(at(x.mono(format!("{:02} {}", i + 1, l(name)), x.caption(), Paint::DIM).key(format!("{k}/name")), 0.0, -26.0)); }
        layer = match i {
            // The board: one bus of five traces across, three down to the wheel, a ring around it.
            0 => {
                let mut lay = layer.border(1.0, Paint::DIM);
                for t in 0..5 {
                    let y = 240.0 + t as f32 * 14.0;
                    lay = lay.child(at(space(u(dw - 60.0), 2.0).fill(Paint::hue(0)).key(format!("{k}/bus{t}")), 30.0, y));
                    for (v, vx) in [24.0, dw - 36.0].iter().enumerate() { lay = lay.child(at(space(u(10.0), u(10.0)).radius(u(5.0)).border(1.5, Paint::hue(0)).key(format!("{k}/via{t}{v}")), *vx, y - 4.0)); }
                }
                for t in 0..3 { lay = lay.child(at(space(2.0, u(40.0)).fill(Paint::hue(0)).key(format!("{k}/down{t}")), dw * 0.5 - 20.0 + t as f32 * 20.0, 300.0)); }
                lay.child(at(space(u(wd + 24.0), u(wd + 24.0)).radius(u((wd + 24.0) * 0.5)).border(1.5, Paint::hue(0)).key(format!("{k}/ring")), wc.0 - (wd + 24.0) * 0.5, wc.1 - (wd + 24.0) * 0.5))
            }
            // The components: a chip on the bus, two capacitors, a crystal.
            1 => {
                let (cw, chh) = (110.0, 70.0);
                let (cxp, cyp) = (dw * 0.5 - cw * 0.5, 236.0);
                let mut lay = layer.child(at(col().key(format!("{k}/chip")).width(u(cw)).height(u(chh)).fill(Paint::tone(0.75)).radius(u(4.0)).align(AlignItems::CENTER).justify(taffy::JustifyContent::CENTER)
                    .child(x.mono("U1", x.caption(), Paint::GROUND).key(format!("{k}/chip/label"))), cxp, cyp));
                for p in 0..6 {
                    lay = lay.child(at(space(u(5.0), u(9.0)).fill(Paint::INK).key(format!("{k}/pin{p}a")), cxp + 10.0 + p as f32 * 17.0, cyp - 9.0))
                         .child(at(space(u(5.0), u(9.0)).fill(Paint::INK).key(format!("{k}/pin{p}b")), cxp + 10.0 + p as f32 * 17.0, cyp + chh));
                }
                for (c, ccx) in [40.0, dw - 66.0].iter().enumerate() { lay = lay.child(at(space(u(26.0), u(26.0)).radius(u(13.0)).fill(Paint::hue(1)).key(format!("{k}/cap{c}")), *ccx, 250.0)); }
                lay.child(at(space(u(64.0), u(20.0)).radius(u(10.0)).border(1.5, Paint::INK).key(format!("{k}/xtal")), dw * 0.5 - 32.0, 200.0))
                    .child(at(space(u(wd), u(wd)).radius(u(wd * 0.5)).border(2.0, Paint::INK).key(format!("{k}/wheel")), wc.0 - wd * 0.5, wc.1 - wd * 0.5))
                    .child(at(x.sign(64.0, Paint::INK, Paint::hue(0)), wc.0 - 32.0, wc.1 - 32.0))
            }
            // The screen: a window with the track and its number.
            2 => {
                let (sw, sh) = (dw - 64.0, 150.0);
                layer.child(at(col().key(format!("{k}/window")).width(u(sw)).height(u(sh)).border(1.5, Paint::INK).radius(u(10.0)).pad(u(18.0), u(16.0)).gap(u(8.0))
                    .child(x.label(l("film.c09.track"), x.body(), Paint::INK).key(format!("{k}/track")))
                    .child(row().key(format!("{k}/count")).gap(u(8.0)).align(AlignItems::BASELINE).child(x.number(x.display(), Paint::hue(0)).font(Font::Mono)).child(x.mono("/ 24", x.caption(), Paint::DIM)))
                    .child(space(u(sw - 36.0), 2.0).fill(Paint::DIM).key(format!("{k}/bar"))), 32.0, 36.0))
            }
            // The glass: its edge, and two highlights along the top left.
            _ => layer.border(2.0, Paint::INK).respond(Response::Ignore)
                .child(at(space(u(120.0), u(3.0)).fill(Paint::INK.alpha(0.5)).key(format!("{k}/glint0")), 36.0, 18.0))
                .child(at(space(u(60.0), u(3.0)).fill(Paint::INK.alpha(0.35)).key(format!("{k}/glint1")), 36.0, 26.0)),
        };
        out.push(x.at(layer, lx, ly));
    }
    let z = x.z();
    let sw = row().key(x.key("explode-row")).child(switch(x.key("explode"), &l("film.apart"), f.exploded, None, Msg::Film(FMsg::Explode)).zoom(z));
    out.push(x.at(sw, colx(0.0), 560.0));
    (out, Some([colx(0.0) - 6.0, 552.0, 150.0, 44.0]))
}

// ── 10 Bauhaus: controls as geometry ─────────────────────────────────────────────────────────────
/// The composition's ground line (units): everything stands on it.
const BAR_Y: f32 = 500.0;
fn bauhaus(x: &Cx) -> Parts {
    let f = x.f;
    let lw = span(3.0);
    let r = 15.0;
    // The slider: a line and a circle on it.
    let slider = col().key(x.key("slider")).width(x.u(lw)).height(x.u(40.0)).focusable(true).label(l("film.c10.size")).cursor(crate::el::Cursor::Grab)
        .value_range(f.size, 0.0, 1.0)
        .on_drag_with(|d: Drag| Some(Msg::Film(FMsg::Knob(d.fx()))))
        .on_key({ let v = f.size; move |k, sh| { let st = if sh { 0.2 } else { 0.05 }; match k { Key::Left => Some(Msg::Film(FMsg::Knob(v - st))), Key::Right => Some(Msg::Film(FMsg::Knob(v + st))), _ => None } } })
        .child(space(x.u(lw), 2.0).fill(Paint::INK).key(x.key("slider/line")).absolute(Some(0.0), Some(x.u(20.0) - 1.0), None, None))
        .child(space(x.u(r * 2.0), x.u(r * 2.0)).radius(x.u(r)).fill(Paint::hue(0)).key(x.key("slider/knob")).absolute(Some(x.u((lw - r * 2.0) * f.size)), Some(x.u(20.0 - r)), None, None));
    // The switch: two squares, the one that is on filled.
    let sq = |on: bool, k: &str| space(x.u(34.0), x.u(34.0)).key(x.key(k)).fill(if on { Paint::INK } else { Paint::GROUND }).border(2.0, Paint::INK);
    let toggle = row().key(x.key("toggle")).gap(x.u(6.0)).on_press(Msg::Film(FMsg::Squares)).label(l("film.invert")).child(sq(!f.squares, "toggle/a")).child(sq(f.squares, "toggle/b"));
    // The tabs: three planes of colour; the chosen one is taller, all on one baseline.
    let mut tabs = row().key(x.key("planes")).gap(x.u(8.0)).align(AlignItems::END).height(x.u(56.0));
    for i in 0..3usize {
        tabs = tabs.child(space(x.u(70.0), x.u(if i == f.plane { 56.0 } else { 34.0 })).fill(Paint::hue(i as u8)).key(x.key(&format!("plane{i}")))
            .on_press(Msg::Film(FMsg::Plane(i))).label(l(&format!("film.c10.plane{i}"))));
    }
    // The composition, standing on the bar: a plate with the number in cols 4–6, the sign recomposed
    // from the primitives the planes choose in cols 8–11, sized by the slider and never past its columns.
    let big = 200.0 + 190.0 * f.size;
    let (a, b) = if f.squares { (Paint::hue(1), Paint::hue(0)) } else { (Paint::hue(0), Paint::hue(1)) };
    let k = x.key("sign");
    let (d, w, q) = (x.u(big), x.u(big * SIGN_W), x.u(big * SIGN_SQ));
    let (first, second) = match f.plane {
        0 => (space(d, d).radius(d * 0.5).fill(a), space(q, q).fill(b)),
        1 => (space(d, d).fill(a), space(q, q).radius(q * 0.5).fill(b)),
        _ => (space(d, d).radius(d * 0.5).border(d * SIGN_BAND, a), space(q, q).fill(Paint::INK)),
    };
    let sign = col().key(k.clone()).flow("film-sign").width(w).height(w)
        .child(first.key(format!("{k}/disc")).absolute(Some(0.0), Some(0.0), None, None))
        .child(second.key(format!("{k}/square")).absolute(Some(d * 0.5), Some(d * 0.5), None, None));
    let ph = 130.0;
    let plate = col().key(x.key("plate")).width(x.u(span(3.0))).height(x.u(ph)).fill(b).pad(x.u(18.0), x.u(14.0)).justify(taffy::JustifyContent::END)
        .child(x.number(x.display(), Paint::GROUND));
    let (c0, ly) = (colx(0.0), TOP_Y + 40.0);
    (vec![
        x.at(x.label(l("film.c10.size"), x.caption(), Paint::DIM).key(x.key("l-size")), c0, ly),
        x.at(slider, c0, ly + 24.0),
        x.at(x.label(l("film.invert"), x.caption(), Paint::DIM).key(x.key("l-toggle")), c0, ly + 110.0),
        x.at(toggle, c0, ly + 134.0),
        x.at(x.label(l("film.c10.form"), x.caption(), Paint::DIM).key(x.key("l-planes")), c0, ly + 220.0),
        x.at(tabs, c0, ly + 244.0),
        x.at(plate, colx(4.0), BAR_Y - ph),
        x.at(space(x.u(span(12.0)), x.u(14.0)).fill(Paint::INK).key(x.key("bar")), colx(0.0), BAR_Y),
        x.at(sign, colx(8.0) + (span(4.0) - big * SIGN_W) * 0.5, BAR_Y - big * SIGN_W),
    ], Some([c0 + (lw - r * 2.0) * f.size - 4.0, ly + 24.0 + 1.0, 38.0, 38.0]))
}

// ── 11 Neo-Y2K: a chrome sign over a liquid dock ─────────────────────────────────────────────────
pub const MENU: [&str; 5] = ["film.c11.item0", "film.c11.item1", "film.c11.item2", "film.c11.item3", "film.c11.item4"];
fn chrome(x: &Cx) -> Parts {
    let f = x.f;
    let liquid = Material::Liquid.id();
    // The dock: blobs in a row whose fields touch; the hovered one swells into its neighbours, the
    // chosen one rises out of the row. Labels sit on one line under them.
    let pitch = 110.0;
    let n = MENU.len() as f32;
    let dx0 = (UNITS - pitch * n) * 0.5;
    let base = 520.0;
    let mut out = vec![];
    for (i, name) in MENU.iter().enumerate() {
        let k = x.key(&format!("item{i}"));
        let hot = f.hover.as_deref() == Some(k.as_str());
        let sel = i == f.menu_sel;
        let r = if hot { 50.0 } else { 40.0 };
        let lift = if sel { 34.0 } else { 0.0 };
        let cx = dx0 + pitch * (i as f32 + 0.5);
        let blob = space(x.u(r * 2.0), x.u(r * 2.0)).radius(x.u(r)).fill(if sel { Paint::hue(1) } else { Paint::hue(0) }).material(liquid).key(format!("{k}/blob")).respond(Response::Loosen);
        let item = col().key(k).width(x.u(pitch)).height(x.u(150.0)).on_press(Msg::Film(FMsg::MenuSel(i))).label(l(name)).focusable(true)
            .child(blob.absolute(Some(x.u(pitch * 0.5 - r)), Some(x.u(60.0 - r - lift)), None, None))
            .child(col().key(format!("film/c11/item{i}/under")).width(x.u(pitch)).align(AlignItems::CENTER).absolute(Some(0.0), Some(x.u(118.0)), None, None)
                .child(x.label(l(name), x.caption(), if sel { Paint::INK } else { Paint::TEXT }).key(format!("film/c11/item{i}/label"))));
        out.push(x.at(item, cx - pitch * 0.5, base - 60.0));
    }
    let sd = 300.0;
    out.push(x.at(x.sign(sd, Paint::INK, Paint::INK), (UNITS - sd * SIGN_W) * 0.5, TOP_Y - 10.0));
    out.push(x.at(x.number(x.display(), Paint::INK).font(Font::Mono), colx(11.0), TOP_Y));
    (out, Some([dx0 + pitch * 2.0 + 15.0, base - 40.0, pitch - 30.0, 80.0]))
}

// ── 12 Mixed media: the collage, and the finale ──────────────────────────────────────────────────
fn collage(x: &Cx) -> Parts {
    let f = x.f;
    let m = |mat: Material| mat.id();
    let paper = |el: El<Msg>, k: &str, mat: Material| col().key(x.key(k)).pad(x.u(14.0), x.u(12.0)).fill(Paint::tone(0.08)).material(m(mat)).child(el);
    let frags: Vec<(El<Msg>, f32, f32)> = vec![
        (paper(row().key(x.key("f1/r")).gap(10.0).align(AlignItems::CENTER).child(x.label(l("film.invert"), x.body(), Paint::TEXT)).child(switch(x.key("f1/switch"), "", f.inverted, None, Msg::Film(FMsg::Invert)).zoom(x.z())), "f1", Material::Plain), colx(0.0), TOP_Y),
        (paper(x.mono("› go 05 neo-dotmatrix", x.body(), Paint::ACCENT).key(x.key("f2/t")), "f2", Material::Plain), colx(3.0), TOP_Y),
        (paper(row().gap(6.0).key(x.key("f3/r")).child(space(x.u(90.0), 1.0).fill(Paint::INK)).child(x.mono("Ø 300.0", x.caption(), Paint::INK)), "f3", Material::Plain), colx(6.0), TOP_Y),
        (paper(x.number(x.u(54.0), Paint::ACCENT).loose(0.0), "f5", Material::Dot), colx(0.0), 200.0),
        (paper(text_lh(l("film.c12.ink"), x.body(), x.body() * 1.5, Paint::INK).width(x.u(220.0)).key(x.key("f6/t")), "f6", Material::Print), colx(2.0), 200.0),
        (paper(text(l("film.c12.carve"), x.u(90.0), Paint::INK).font(Font::Display).weight(3.0).key(x.key("f7/t")).loose(0.6), "f7", Material::Lino), colx(5.0), 190.0),
        (paper(row().gap(3.0).key(x.key("f8/w")).children((0..18).map(|i| space(3.0, x.u(12.0 + ((i * 7) % 11) as f32 * 4.0)).fill(Paint::INK).key(x.key(&format!("f8/b{i}"))))), "f8", Material::Glitch), colx(6.5), 200.0),
        (paper(col().key(x.key("f9/p")).width(x.u(150.0)).height(x.u(100.0)).radius(x.u(14.0)).fill(Paint::INK), "f9", Material::Xray), colx(0.0), 360.0),
        (paper(row().gap(x.u(8.0)).key(x.key("f10/r")).child(space(x.u(70.0), x.u(70.0)).radius(x.u(70.0)).fill(Paint::hue(0)).loose(0.8)).child(space(x.u(60.0), x.u(60.0)).fill(Paint::hue(1)).loose(0.8)), "f10", Material::Halftone), colx(2.0), 380.0),
        (paper(row().gap(0.0).key(x.key("f11/r")).child(space(x.u(56.0), x.u(56.0)).radius(x.u(56.0)).fill(Paint::hue(0))).child(space(x.u(46.0), x.u(46.0)).radius(x.u(46.0)).fill(Paint::hue(0))), "f11", Material::Liquid), colx(4.0), 380.0),
        // The note from the first chapter's form, pinned up in print.
        (paper(text(x.note(), x.body() * 1.3, Paint::INK).key(x.key("note/t")).nowrap(), "note", Material::Print), colx(6.0), 400.0),
    ];
    let mut board = col().key("film/c12/board").width(x.u(UNITS)).height(x.u(STAGE_H)).respond(Response::Loosen);
    for (el, fx, fy) in frags { board = board.child(x.at(el, fx, fy)); }
    board = board.child(x.at(col().key(x.key("f12")).pad(x.u(18.0), x.u(18.0)).fill(Paint::tone(0.08)).material(m(Material::Lino)).child(x.sign(230.0, Paint::INK, Paint::hue(0)).loose(0.35)), colx(8.5), TOP_Y + 40.0));
    // The finale: everything loosens into particles that make the sign, alone and large.
    let finale = col().key("film/c12/finale").width(x.u(UNITS)).height(x.u(STAGE_H)).align(AlignItems::CENTER).justify(taffy::JustifyContent::CENTER)
        .child(x.sign_keyed("film/c12/finale/sign", 380.0, Paint::INK, Paint::hue(0)));
    let draft = x.g.film.drafts.as_ref().filter(|d| d.0 == 12).map(|d| d.1.as_str());
    let mut out = vec![];
    if f.finale {
        out.push(x.at(finale, 0.0, 0.0));
        out.push(x.at(board.hidden_until_morphed(true), 0.0, 0.0));
    } else {
        out.push(x.at(board, 0.0, 0.0));
        out.push(x.at(finale.hidden_until_morphed(true), 0.0, 0.0));
    }
    let z = x.z();
    out.push(x.at(row().key(x.key("controls")).gap(x.u(18.0)).align(AlignItems::CENTER)
        .child(button_as(x.key("close"), &if f.finale { l("film.c12.open") } else { l("film.c12.close") }, Variant::Accent, Msg::Film(FMsg::Finale)).zoom(z))
        .child(number(&x.key("count"), &l("film.c12.particles"), f.spec.particles, COUNT_SPEC, draft, |e| Msg::Film(FMsg::Count(e))).width(360.0).zoom(z)), colx(0.0), 560.0));
    (out, Some([colx(0.0) - 4.0, 556.0, 110.0, 44.0]))
}

// ── 13 Mechanism: the one place that is three-dimensional ────────────────────────────────────────
fn machine(x: &Cx) -> Parts {
    let f = x.f;
    let m = &f.mech;
    let z = x.z();
    let (vw, vh) = (span(8.0), 540.0);
    let view = mechanism::view(m, &x.key("machine"), x.u(vw), x.u(vh), x.caption(), |mm| Msg::Film(FMsg::Mech(mm)));
    let draft = x.g.film.drafts.as_ref().filter(|d| d.0 == 13).map(|d| d.1.as_str());
    let mut parts = col().key(x.key("parts")).width(x.u(span(3.5))).gap(0.0);
    for (i, p) in mechanism::PARTS.iter().enumerate() {
        let on = m.hover.or(m.chosen) == Some(i);
        let teeth = if p.teeth > 0 { format!("z {}", p.teeth) } else { String::new() };
        parts = parts.child(row().key(x.key(&format!("part{i}"))).height(x.u(30.0)).gap(x.u(12.0)).align(AlignItems::CENTER)
            .child(x.mono(format!("{:02}", i + 1), x.caption(), if on { Paint::ACCENT } else { Paint::DIM }).width(x.u(30.0)))
            .child(x.label(l(p.name), x.body(), if on { Paint::ACCENT } else { Paint::INK }).width(x.u(140.0)))
            .child(x.mono(teeth, x.caption(), Paint::DIM)));
    }
    let names: Vec<String> = mechanism::MATS.iter().map(|m| l(m.0)).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let mats = segmented(&x.key("mat"), &names, m.mat, 28.0, |i| Msg::Film(FMsg::Mech(MMsg::Material(i)))).zoom(z);
    let explode = number(&x.key("explode"), &l("film.apart"), m.explode_to, EXPLODE_SPEC, draft, |e| Msg::Film(FMsg::MechExplode(e))).width(300.0).zoom(z);
    (vec![
        x.at(view, colx(0.0), TOP_Y - 20.0),
        x.at(row().key(x.key("ratio")).gap(x.u(10.0)).align(AlignItems::BASELINE).child(x.mono("i =", x.caption(), Paint::DIM)).child(x.number(x.body() * 1.6, Paint::ACCENT).font(Font::Mono)).child(x.mono("24 : 12 : 18", x.caption(), Paint::DIM)), colx(8.5), TOP_Y),
        x.at(parts, colx(8.5), TOP_Y + 56.0),
        x.at(explode, colx(8.5), 460.0),
        x.at(mats, colx(8.5), 520.0),
    ], Some([colx(0.0) + span(8.0) * 0.25, TOP_Y + 120.0, span(8.0) * 0.4, 300.0]))
}

// ── Update, the clock, keys ──────────────────────────────────────────────────────────────────────

/// Go to chapter `i` (wrapping): the target preset's transition from `at` (logical px; the stage's
/// centre when `None`), and the actors flow.
pub fn go(g: &mut Gallery, i: usize, at: Option<(f32, f32)>) {
    let i = i % CHAPTERS;
    let geo = geo(g.width, g.height);
    let (x, y) = at.unwrap_or((geo.sx + geo.sw * 0.5, geo.sy + geo.sh * 0.5));
    g.cmds.push(Cmd::Commit { x, y });
    g.cmds.push(Cmd::Flow(FlowSpec { flow_ms: g.film.spec.flow_ms, flow_loose: 0.2, out_ms: 160.0 }));
    let f = &mut g.film;
    f.ch = i;
    f.entered = g.clock;
    f.ghost = 0;
    f.hover = None;
    f.tear = None;
    f.error = false;
    f.finale = false;
    f.scrub_from = None;
    f.ident = None;
    f.known.clear();
    f.drafts = None;
    f.mech.hover = None;
    f.mech.chosen = None;
    f.mech.loose = None;
    f.mech.vturn = 0.0;
    f.mech.orbit.vyaw = 0.0;
    f.mech.orbit.vpitch = 0.0;
    f.mech.explode = f.mech.explode_to;
    g.apply_style(CH[i].style);
    // Keyboard first: the palette has the focus when it arrives.
    if i == 2 { g.cmds.push(Cmd::Focus { key: "film/c02/query".into() }); }
}

pub fn update(g: &mut Gallery, m: FMsg) {
    let now = g.clock;
    match m {
        FMsg::Go(i) => { let at = tick_at(g, i); go(g, i, at); }
        FMsg::Next => { let n = g.film.ch + 1; go(g, n, None); }
        FMsg::Prev => { let n = g.film.ch + CHAPTERS - 1; go(g, n, None); }
        FMsg::Play => {
            // Space pauses what was playing (even when this key's own input has just paused it) and plays
            // what was paused.
            let f = &mut g.film;
            let was = f.playing || f.yielded == Some(now);
            f.playing = !was;
            f.held = was;
            f.yielded = None;
            if f.playing { f.entered = now; f.ghost = 0; }
        }
        FMsg::Note(t) => g.film.note = t,
        FMsg::Split(d) => {
            let f = &mut g.film;
            let k = geo(g.width, g.height).k.max(1e-3) * (FRAME[2] - FRAME[0]);
            match d.phase {
                Phase::Start => f.split0 = Some(f.split),
                Phase::Move | Phase::End => { f.split = (f.split0.unwrap_or(f.split) + d.dx / k).clamp(0.0, 1.0); if d.phase == Phase::End { f.split0 = None; } }
            }
        }
        FMsg::SplitTo(v) => g.film.split = v.clamp(0.0, 1.0),
        FMsg::Mat(i) => { g.film.mat = i.min(SPLIT_MATS.len() - 1); }
        FMsg::Remind => g.film.remind = !g.film.remind,
        FMsg::Invert => g.film.inverted = !g.film.inverted,
        FMsg::Num(Edit::Draft(d)) => g.film.drafts = Some((if g.film.ch == 0 { 0 } else { 1 }, d)),
        FMsg::Num(Edit::Set(v)) => { g.film.n = N_SPEC.clamp(v); g.film.drafts = None; }
        FMsg::Num(Edit::Cancel) | FMsg::Gap(Edit::Cancel) | FMsg::Count(Edit::Cancel) | FMsg::MechExplode(Edit::Cancel) => g.film.drafts = None,
        FMsg::Query(t) => { g.film.query = t; g.film.sel = 0; }
        FMsg::Run(i) => run_command(g, i),
        FMsg::Size(d) => {
            let f = &mut g.film;
            let k = geo(g.width, g.height).k.max(1e-3);
            match d.phase {
                Phase::Start => f.dia0 = Some(f.dia),
                Phase::Move => f.dia = (f.dia0.unwrap_or(f.dia) + d.dx / k / SIGN_W).clamp(160.0, 340.0),
                Phase::End => { f.dia = (f.dia0.unwrap_or(f.dia) + d.dx / if f.dia0.is_some() { k * SIGN_W } else { 1.0 }).clamp(160.0, 340.0); f.dia0 = None; }
            }
        }
        FMsg::Identify(k) => {
            let f = &mut g.film;
            if f.ident.as_ref().is_some_and(|(ik, t)| *ik == k && now - t < ID_LOOSEN + ID_SCAN + ID_FORM) { return; }
            f.ident = Some((k.clone(), now));
            if !f.known.contains(&k) { f.known.push(k); }
        }
        FMsg::Lit => g.film.lit = !g.film.lit,
        FMsg::Obj(x, y) => g.film.obj = (x, y),
        FMsg::Gap(Edit::Draft(d)) => g.film.drafts = Some((6, d)),
        FMsg::Gap(Edit::Set(v)) => { g.film.gap = GAP_SPEC.clamp(v); g.film.drafts = None; }
        FMsg::Form => next_form(g, false),
        FMsg::Scrub(d) => scrub(g, d),
        FMsg::Tear(d) => {
            let y = d.fy();
            match d.phase {
                Phase::Start | Phase::Move => g.film.tear = Some(y),
                Phase::End => {
                    g.film.tear = None;
                    let dx = ((y * 997.0).sin() * 0.5 + 0.5) * 60.0 - 30.0;
                    g.film.locks.push((y, if dx.abs() < 12.0 { dx.signum() * 18.0 } else { dx }));
                    if g.film.locks.len() > 3 { g.film.locks.remove(0); }
                    if g.film.locks.len() == 3 { g.film.error = true; }
                }
            }
        }
        FMsg::Error(on) => g.film.error = on,
        FMsg::Retry => { g.film.locks.clear(); g.film.error = false; }
        FMsg::Explode => { g.cmds.push(Cmd::Flow(FlowSpec { flow_ms: 520.0, flow_loose: 0.15, out_ms: 0.0 })); g.film.exploded = !g.film.exploded; g.cmds.push(Cmd::FrontAt { key: "film/c09/layer1".into(), strength: 0.8 }); }
        FMsg::Knob(v) => g.film.size = v.clamp(0.0, 1.0),
        FMsg::Squares => g.film.squares = !g.film.squares,
        FMsg::Plane(i) => { g.cmds.push(Cmd::Flow(FlowSpec { flow_ms: 420.0, flow_loose: 0.3, out_ms: 120.0 })); g.film.plane = i.min(2); }
        FMsg::MenuSel(i) => { g.cmds.push(Cmd::Flow(FlowSpec { flow_ms: 380.0, flow_loose: 0.3, out_ms: 0.0 })); g.film.menu_sel = i.min(MENU.len() - 1); }
        FMsg::Finale => {
            let f = &g.film;
            let spec = MorphSpec { duration_ms: 1800.0, loosen_ms: 420.0, tighten_ms: 520.0, count_max: f.spec.particles, curl: 90.0, stagger: 0.55, ease: Ease::InOutCubic, lattice: 2.0, ..MorphSpec::default() };
            let (from, to) = if f.finale { ("film/c12/finale", "film/c12/board") } else { ("film/c12/board", "film/c12/finale") };
            g.cmds.push(Cmd::Morph { from: from.into(), to: to.into(), spec });
            g.film.finale = !g.film.finale;
        }
        FMsg::Count(Edit::Draft(d)) => g.film.drafts = Some((12, d)),
        FMsg::Count(Edit::Set(v)) => { g.film.spec.particles = COUNT_SPEC.clamp(v); g.film.drafts = None; }
        FMsg::Mech(mm) => {
            let key = "film/c13/machine";
            let mut cmds = vec![];
            g.film.mech.update(mm, now, &mut cmds, key);
            g.cmds.extend(cmds);
            // Turning the train counts the big gear's turns.
            g.film.n = ((g.film.mech.turn / 360.0).floor().rem_euclid(100.0)).max(0.0);
        }
        FMsg::MechExplode(Edit::Draft(d)) => g.film.drafts = Some((13, d)),
        FMsg::MechExplode(Edit::Set(v)) => { g.film.mech.explode_to = EXPLODE_SPEC.clamp(v); g.film.drafts = None; }
    }
}

/// Where a tick of the timeline is (its centre), for the transition to start there.
fn tick_at(g: &Gallery, i: usize) -> Option<(f32, f32)> {
    let geo = geo(g.width, g.height);
    Some((geo.sx + i as f32 / (CHAPTERS - 1) as f32 * geo.sw, geo.sy + geo.sh + 28.0))
}

fn run_command(g: &mut Gallery, i: usize) {
    if i < CHAPTERS { go(g, i, None); return; }
    match i - CHAPTERS { 0 => g.film.inverted = !g.film.inverted, _ => g.film.n = N_SPEC.clamp(g.film.n + 1.0) }
}

/// The poster's next form: a morph from what it shows to the next one.
fn next_form(g: &mut Gallery, scrubbed: bool) {
    let f = &mut g.film;
    let (a, b) = (f.form % FORMS, (f.form + 1) % FORMS);
    let spec = MorphSpec { duration_ms: 1300.0, loosen_ms: 260.0, tighten_ms: 420.0, curl: 70.0, stagger: 0.5, lattice: 4.0, count_max: 16000.0, ..MorphSpec::default() };
    g.cmds.push(Cmd::Morph { from: form_key(a), to: form_key(b), spec });
    if scrubbed { g.cmds.push(Cmd::Scrub(Some(0.0))); }
    f.scrub_from = Some(a);
    f.form = b;
}

/// Dragging across the poster: the drag's start starts the morph held at 0, moving holds it where the
/// drag is, letting go runs it on (past half way) or back.
fn scrub(g: &mut Gallery, d: Drag) {
    let u = (d.dx / d.w.max(1.0) * 1.6).clamp(0.0, 1.0);
    match d.phase {
        Phase::Start => next_form(g, true),
        Phase::Move => g.cmds.push(Cmd::Scrub(Some(u))),
        Phase::End => {
            g.cmds.push(Cmd::Scrub(None));
            // A press without a drag is a press: the morph runs on. A short drag goes back.
            if d.dx.abs() > 6.0 && u < 0.5 {
                let f = &mut g.film;
                let (a, b) = (f.form, f.scrub_from.unwrap_or((f.form + FORMS - 1) % FORMS));
                g.cmds.push(Cmd::Morph { from: form_key(a), to: form_key(b), spec: MorphSpec { duration_ms: 700.0, ..MorphSpec::default() } });
                f.form = b;
                f.scrub_from = Some(a);
            }
        }
    }
}

/// The ghost's script for chapter `c`: (seconds after entering, what it does).
fn script(c: usize) -> Vec<(f32, Ghost)> {
    use Ghost::*;
    let drag = |phase: Phase, dx: f32| Drag { phase, x: 100.0, y: 50.0, w: 400.0, h: 300.0, dx, dy: 0.0 };
    match c {
        0 => {
            // The divider sweeps left, right and back; the chips step through four materials.
            let mut v = vec![];
            let path = [(0.8, 0.32), (2.0, 0.12), (3.6, 0.62), (5.0, 0.32)];
            for w in path.windows(2) { for i in 1..=12 { let t = i as f32 / 12.0; let e = t * t * (3.0 - 2.0 * t); v.push((w[0].0 + (w[1].0 - w[0].0) * t, Msg(FMsg::SplitTo(w[0].1 + (w[1].1 - w[0].1) * e)))); } }
            for (t, m) in [(2.6, 1), (4.2, 3), (5.8, 4), (7.2, 0)] { v.push((t, Msg(FMsg::Mat(m)))); }
            v.sort_by(|a, b| a.0.total_cmp(&b.0));
            v
        }
        1 => vec![(1.4, Focus("film/c01/title")), (2.6, Press("film/c01/invert")), (4.2, Press("film/c01/count/more")), (4.8, Press("film/c01/count/more")), (6.8, Press("film/c01/invert"))],
        2 => vec![(1.0, Type("g")), (1.4, Type("o")), (1.8, Type("0")), (2.4, Type("5")), (3.4, Msg(FMsg::Query(TextEdit::new("go")))), (4.2, Type(" n")), (5.0, Msg(FMsg::Query(TextEdit::default())))],
        3 => vec![(1.0, Hover(Some("film/c03/part0"))), (2.0, Msg(FMsg::Size(drag(Phase::Start, 0.0)))), (2.4, Msg(FMsg::Size(drag(Phase::Move, 60.0)))), (2.8, Msg(FMsg::Size(drag(Phase::End, 80.0)))),
                 (3.6, Hover(Some("film/c03/part2"))), (4.6, Hover(Some("film/c03/part3"))), (5.6, Msg(FMsg::Size(drag(Phase::Start, 0.0)))), (6.0, Msg(FMsg::Size(drag(Phase::End, -80.0)))), (6.6, Hover(None))],
        4 => vec![(0.8, Focus("film/c04/sign")), (1.0, Press("film/c04/sign")), (3.0, Focus("film/c04/label")), (3.2, Press("film/c04/label")), (5.2, Focus("film/c04/picture")), (5.4, Press("film/c04/picture"))],
        5 => vec![(2.0, Hover(Some("film/c05/sign"))), (4.6, Hover(None))],
        6 => vec![(1.2, Msg(FMsg::Obj(80.0, 140.0))), (3.0, Msg(FMsg::Obj(360.0, 50.0))), (5.0, Msg(FMsg::Gap(Edit::Set(24.0)))), (6.6, Msg(FMsg::Gap(Edit::Set(8.0)))), (7.2, Msg(FMsg::Obj(330.0, 40.0)))],
        7 => vec![(1.0, Msg(FMsg::Form)), (3.6, Msg(FMsg::Form)), (6.2, Msg(FMsg::Form))],
        8 => vec![(1.2, Msg(FMsg::Tear(tear(Phase::Start, 0.3)))), (1.8, Msg(FMsg::Tear(tear(Phase::End, 0.3)))), (2.8, Msg(FMsg::Tear(tear(Phase::Start, 0.62)))), (3.4, Msg(FMsg::Tear(tear(Phase::End, 0.62)))),
                 (4.4, Msg(FMsg::Tear(tear(Phase::Start, 0.45)))), (5.0, Msg(FMsg::Tear(tear(Phase::End, 0.45)))), (6.8, Press("film/c08/retry"))],
        9 => vec![(0.5, Ptr(0.3, 0.4)), (1.5, Ptr(0.7, 0.6)), (2.4, Msg(FMsg::Explode)), (4.0, Ptr(0.35, 0.3)), (5.2, Ptr(0.65, 0.7)), (6.4, Msg(FMsg::Explode)), (7.4, Ptr(0.5, 0.5))],
        10 => vec![(1.0, Msg(FMsg::Knob(0.9))), (2.4, Press("film/c10/toggle")), (3.6, Press("film/c10/plane1")), (5.0, Press("film/c10/plane2")), (6.4, Msg(FMsg::Knob(0.4))), (7.4, Press("film/c10/plane0"))],
        11 => vec![(0.8, Hover(Some("film/c11/item2"))), (1.6, Ptr(0.2, 0.8)), (2.4, Hover(Some("film/c11/item3"))), (3.2, Press("film/c11/item3")), (4.4, Ptr(0.8, 0.8)), (5.0, Hover(Some("film/c11/item0"))), (6.0, Press("film/c11/item1")), (6.6, Hover(None))],
        12 => vec![(2.2, Msg(FMsg::Finale))],
        13 => vec![(0.6, Msg(FMsg::Mech(MMsg::Swing(40.0)))), (1.8, Msg(FMsg::Mech(MMsg::Spin(80.0)))), (3.2, Msg(FMsg::Mech(MMsg::Explode(1.0)))), (4.4, Msg(FMsg::Mech(MMsg::Loosen(6)))),
                   (6.2, Msg(FMsg::Mech(MMsg::Explode(0.0)))), (7.4, Msg(FMsg::Mech(MMsg::Spin(0.0)))), (7.6, Msg(FMsg::Mech(MMsg::Swing(-30.0))))],
        _ => vec![],
    }
}
fn tear(phase: Phase, fy: f32) -> Drag { Drag { phase, x: 100.0, y: fy * 300.0, w: 1000.0, h: 300.0, dx: 0.0, dy: 0.0 } }

/// What the ghost does now.
fn act(g: &mut Gallery, a: Ghost) {
    match a {
        Ghost::Msg(m) => update(g, m),
        Ghost::Press(k) => g.cmds.push(Cmd::Press { key: k.into() }),
        Ghost::Focus(k) => g.cmds.push(Cmd::Focus { key: k.into() }),
        Ghost::Type(s) => { let mut t = g.film.query.clone(); t.text.push_str(s); t.caret = t.text.chars().count(); t.anchor = t.caret; g.film.query = t; g.film.sel = 0; }
        Ghost::Hover(k) => g.film.hover = k.map(str::to_owned),
        Ghost::Ptr(x, y) => {
            g.film.ptr = (x, y);
            // The chrome's reflection follows the pointer: the ghost moves it too.
            if g.film.ch == 11 { let geo = geo(g.width, g.height); g.cmds.push(Cmd::Pointer { x: geo.sx + x * geo.sw, y: geo.sy + y * geo.sh }); }
        }
    }
}

/// The film's clock (`Ui::on_tick`): autoplay, the ghost, the board's rows, identifying, the machine.
pub fn tick(g: &mut Gallery, t: f32) -> bool {
    if g.page != Page::Film { return false; }
    let before = g.film.last_tick;
    g.film.last_tick = t;
    let mut changed = false;
    let f = &mut g.film;
    if !f.playing && !f.held && t - f.last_input >= f.spec.idle_ms / 1000.0 { f.playing = true; f.entered = t; f.ghost = 0; changed = true; }
    if g.film.playing {
        let el = t - g.film.entered;
        if g.film.progress(t) != g.film.progress(before) || g.film.cueing(t) != g.film.cueing(before) { changed = true; }
        let steps = script(g.film.ch);
        while let Some((at, a)) = steps.get(g.film.ghost).cloned() && el >= at {
            g.film.ghost += 1;
            act(g, a);
            changed = true;
        }
        if el >= g.film.spec.dwell_ms / 1000.0 { let n = g.film.ch + 1; go(g, n, None); return true; }
    }
    let f = &mut g.film;
    // Identifying runs on the clock while it lasts.
    if f.ch == 4 && f.ident.as_ref().is_some_and(|(_, t0)| t - t0 < ID_LOOSEN + ID_SCAN + ID_FORM + 0.05) { changed = true; }
    if f.ch == 13 && f.mech.step(t) {
        changed = true;
        f.n = (f.mech.turn / 360.0).floor().rem_euclid(100.0);
    }
    // The board: once a second the number moves on, and the rows follow one after another.
    if f.ch == 5 && f.playing {
        if t - f.board_at >= 1.0 { f.board_at = t; f.n = (f.n + 1.0) % 100.0; changed = true; }
        for i in 0..4 { if f.rows[i] != f.n && t - f.board_at >= 0.12 * (i + 1) as f32 { f.rows[i] = f.n; changed = true; } }
    }
    changed
}

/// When the film wants its next tick (`Ui::on_wake`).
pub fn wake(g: &Gallery, t: f32) -> Option<f32> {
    if g.page != Page::Film { return None; }
    let f = &g.film;
    if f.ch == 4 && f.ident.as_ref().is_some_and(|(_, t0)| t - t0 < ID_LOOSEN + ID_SCAN + ID_FORM + 0.05) { return Some(t); }
    if f.ch == 13 && f.mech.moving(t) { return Some(t); }
    if f.playing {
        let mut w = f.entered + f.spec.dwell_ms / 1000.0;
        if let Some((at, _)) = script(f.ch).get(f.ghost) { w = w.min(f.entered + at); }
        if f.ch == 5 { w = w.min(f.board_at + 1.0).min(f.board_at + 0.5); }
        // The timeline's fill steps, and the way on's pulse.
        let dwell = f.spec.dwell_ms / 1000.0;
        let next_step = f.entered + ((f.progress(t) * PROGRESS_STEPS).floor() + 1.0) / PROGRESS_STEPS * dwell;
        w = w.min(next_step).min(if f.cueing(t) { f32::INFINITY } else { f.entered + dwell - f.spec.cue_ms / 1000.0 });
        return Some(w.max(t));
    }
    if !f.held { return Some(f.last_input + f.spec.idle_ms / 1000.0); }
    None
}

/// A key nothing on the page took: chapters, play, the gallery; the palette's selection.
pub fn key(g: &mut Gallery, k: Key, _shift: bool) -> bool {
    if g.page != Page::Film { return false; }
    let ch = g.film.ch;
    if ch == 2 {
        let n = matches(&g.film.query.text).len();
        match k {
            Key::Up => { g.film.sel = g.film.sel.saturating_sub(1); return true; }
            Key::Down => { g.film.sel = (g.film.sel + 1).min(n.saturating_sub(1)); return true; }
            Key::Enter => { if let Some(&i) = matches(&g.film.query.text).get(g.film.sel) { run_command(g, i); } return true; }
            _ => {}
        }
    }
    match k {
        Key::Right => update(g, FMsg::Next),
        Key::Left => update(g, FMsg::Prev),
        Key::Char(' ') => update(g, FMsg::Play),
        Key::Char(c @ '1'..='9') => go(g, c as usize - '0' as usize, None),
        Key::Char('0') => go(g, 10, None),
        Key::Char('-') => go(g, 11, None),
        Key::Char('=') => go(g, 12, None),
        Key::Home => go(g, 0, None),
        Key::End => go(g, CHAPTERS - 1, None),
        Key::Escape => { super::update(g, Msg::Go(Page::Comp(0))); }
        _ => return false,
    }
    true
}

/// A person's input (a press, the wheel, a key): autoplay yields at once.
pub fn heard(g: &mut Gallery) -> bool {
    if g.page != Page::Film { return false; }
    let f = &mut g.film;
    f.last_input = g.clock;
    let was = f.playing;
    if was { f.yielded = Some(g.clock); }
    f.playing = false;
    was
}

/// The pointer over the window: where it is in the stage, and it counts as input.
pub fn pointer(g: &mut Gallery, x: f32, y: f32) -> bool {
    if g.page != Page::Film { return false; }
    let geo = geo(g.width, g.height);
    let p = (((x - geo.sx) / geo.sw).clamp(0.0, 1.0), ((y - geo.sy) / geo.sh).clamp(0.0, 1.0));
    let was = heard(g);
    let follows = matches!(g.film.ch, 9);
    let moved = follows && p != g.film.ptr;
    g.film.ptr = p;
    was || moved
}

/// What is under the pointer (ticks, parts, menu items, the sign).
pub fn hovered(g: &mut Gallery, k: Option<&str>) -> bool {
    if g.page != Page::Film { return false; }
    let k = k.map(str::to_owned);
    if k == g.film.hover { return false; }
    g.film.hover = k;
    true
}

/// The film's own changes to the look: the annotation each chapter wants.
pub fn look(g: &Gallery, l: &mut Look) {
    if g.page != Page::Film { return; }
    let f = &g.film;
    if f.ch == 4 {
        // The detector boxes what is focused while the ghost plays, what the pointer is near otherwise.
        l.gaze = Gaze { mode: if f.playing { GazeMode::Focus } else if f.hover.is_some() { GazeMode::Hover } else { GazeMode::All }, kinds: gaze::FRAME | gaze::KIND | gaze::SIZE | gaze::CROSS, density: 6.0, ..l.gaze };
    }
}
