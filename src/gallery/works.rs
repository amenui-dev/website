//! Works (site-design M, N): `/works/`, four windows onto one day outside, and each built piece at
//! `/works/<slug>/`: the picture full bleed under the bar, the title block at the bottom left and the
//! piece strip at the bottom right (M.2). `ground truth` (W-03) is built; the other three are a window
//! of their colours on the index until they are (N.2 item 1): they take no focus, do not widen, have no
//! page of their own (their addresses go on to the index) and are not links from a piece's strip.
//!
//! Calm when idle (M.2): one piece runs at a time (the piece on its page, or the live window of the
//! index); between events a piece wants no frame; what moves by itself is paced at 30 frames a
//! second, what the hand moves is not; 60 s without input the piece rests after its current event.
//!
//! What is drawn over a picture is drawn in its own colour on the pixel grid, its edges blended with the
//! picture by coverage (N.4 (1)). A faint shade is the survey ink at the shade's alpha over the picture
//! (N.2 item 5). A colour of its own (a scan character's) is a mix of the page's ground, its ink and one
//! of its three hues, found as the nearest such mix (`InkSpace::paint`); on a piece's page the ground,
//! which the picture covers, is set to a ramp that follows the sky, so the mixes over the sky are the
//! sky's.
use super::{BAR, Gallery, Msg, NARROW, Page, Section};
pub use super::touch::TOUCH_GRASS;
use super::touch;
use amenui::draw::Drawing;
use amenui::el::{El, Font, Paint, Point, Pose, Response, Role, canvas, col, picture, row, space, text};
use amenui::morph::FlowSpec;
use amenui::style::{Look, Markr, Material, Theme};
use std::sync::OnceLock;
use taffy::AlignItems;

pub const COUNT: usize = 4;

/// A piece: its address, its title (English in both languages), its number and hour, its tones (N.4
/// (3): a tone names the ground it sits on, `light` a pale ground and ink type), where its centre of
/// interest is (a fraction of its width), the phrase its live window shows (N.4 (8)), whether it is
/// built, and its recipe in the docs.
pub struct Piece {
    pub slug: &'static str,
    pub title: &'static str,
    pub n: &'static str,
    pub hour: &'static str,
    /// The hour as a number, for the day's ruler.
    pub at: f32,
    /// The status line's tone: whether the ground at the foot of the picture is light.
    pub light: bool,
    /// The bar's tone: whether the sky under the bar is light.
    pub bar_light: bool,
    pub focus: f32,
    pub live: &'static str,
    /// Whether it has its page (built, or made and waiting for its review: then its window on the
    /// index and its name in a strip stay as an unbuilt piece's).
    pub page: bool,
    pub built: bool,
    /// The docs page `how this is made ↗` opens, under `docs/`.
    pub recipe: &'static str,
}
pub const PIECES: [Piece; COUNT] = [
    Piece { slug: "touch-grass", title: "touch grass", n: "W-01", hour: "12:00", at: 12.0, light: true, bar_light: true, focus: 380.0 / 1440.0, live: "WIND OVER A FIELD OF 1,570 STALKS", page: true, built: true, recipe: "recipes/touch-grass/" },
    Piece { slug: "line-too-long", title: "line too long", n: "W-02", hour: "15:20", at: 15.0 + 20.0 / 60.0, light: false, bar_light: false, focus: 790.0 / 1440.0, live: "ONE STRING, 1,204 PX, ONE KNOT", page: false, built: false, recipe: "concepts/annotation/" },
    Piece { slug: "ground-truth", title: "ground truth", n: "W-03", hour: "18:40", at: 18.0 + 40.0 / 60.0, light: false, bar_light: true, focus: 870.0 / 1440.0, live: "TRACKING 7 BIRDS AND ONE POINTER", page: true, built: true, recipe: "recipes/ground-truth/" },
    Piece { slug: "nightly", title: "nightly", n: "W-04", hour: "23:10", at: 23.0 + 10.0 / 60.0, light: false, bar_light: false, focus: 650.0 / 1440.0, live: "540 RAYS ON ONE LINE OF TYPE", page: false, built: false, recipe: "concepts/annotation/" },
];
pub const GROUND_TRUTH: usize = 2;
/// The first built piece in the day's order: the wide window with no pointer (N.2 item 1).
pub fn first_built() -> usize { PIECES.iter().position(|p| p.built).unwrap_or(0) }
/// The nearest built piece from `i` in direction `dir` (±1), round the day; none when `i` is the only one.
pub fn step(i: usize, dir: isize) -> Option<usize> {
    (1..COUNT as isize).map(|k| (i as isize + dir * k).rem_euclid(COUNT as isize) as usize).find(|&j| PIECES[j].built)
}
/// The links a piece's page prints (its recipe), under the site's base: the build checks each resolves.
pub fn links() -> Vec<String> { PIECES.iter().filter(|p| p.page).map(|p| format!("docs/{}", p.recipe)).collect() }

/// The pieces' pictures: `PIC + i` (`picture`), each 960 × 600, shown scaled. W-03's ridges; W-01's sky
/// and ground, which its page draws the field over; an unbuilt piece's colours alone, with the piece's
/// grain.
pub const PIC: u32 = 100;
/// W-01's picture on a phone's page, its horizon at 62 %.
pub const PIC_PHONE: u32 = PIC + COUNT as u32;
/// W-01's still in the index's windows: the field at rest (`touch::still_pixels`).
pub const PIC_STILL: u32 = PIC + COUNT as u32 + 1;
/// The `Markr` slot a piece's picture is drawn through: the picture as it is.
pub const PIC_SLOT: u8 = 5;

pub(crate) const fn rgb(h: u32) -> [f32; 3] { [((h >> 16) & 255) as f32 / 255.0, ((h >> 8) & 255) as f32 / 255.0, (h & 255) as f32 / 255.0] }
/// W-03's survey ink (`Paint::hue(2)`).
const SURVEY: [f32; 3] = rgb(0x231f33);
/// The accent on a dark ground and on a light one (M.2, N.2 item 6).
const ACCENT_DARK: [f32; 3] = rgb(0xff6b2b);
const ACCENT_LIGHT: [f32; 3] = rgb(0xdb4d14);
/// Survey ink at full strength.
const INK_FULL: Paint = Paint { tone: 1.0, accent: 1.0, alpha: 1.0, hue: 2 };
/// The horizon's dashes, as built (N.2 item 5: it stays): survey ink mixed toward the pale ink.
const HORIZON_INK: Paint = Paint { tone: 1.0, accent: 90.0 / 255.0, alpha: 1.0, hue: 2 };
/// A coordinate on the pixel grid for a 1 px line (at any whole scale), in a subtree drawn at `z` times
/// its size (the index's windows); the subtree's origin is on the grid.
fn px(v: f32, z: f32) -> f32 { ((v * z).floor() + 0.5) / z }
/// The chrome over a piece: ink, and the small type (the small line and the strip) at 150 / 255 of it;
/// on W-01 at 230 / 255 (S.2 item 1). The status line's dim words are the site's.
fn dim(i: usize) -> Paint { Paint::INK.alpha(if i == TOUCH_GRASS { 230.0 } else { 150.0 } / 255.0) }
/// An unbuilt piece's name, in the index and in a strip: 40 % of the ink (N.2 item 1).
fn unbuilt() -> Paint { Paint::INK.alpha(0.4) }

/// A piece's colours: the chrome's ink by its foot's tone, the accent for that ground (the title
/// block's number, the pointer's box over a dark part of the picture), the accent for a light ground as
/// the second hue (the pointer's box over a light part), and the survey ink as the third. The title and
/// the number are at full strength in every state (Q.1 (4), (5)): only the small line, the strip and the
/// status line fade. Check (P.2 item 4): the median of the title's ink is within 3 of the declared colour
/// in every channel, at 0.4 s and after the chrome has faded, at both sizes.
pub fn theme(i: usize) -> Theme {
    let p = &PIECES[i];
    let (ink, accent) = if p.light { (rgb(0x17181a), ACCENT_LIGHT) } else { (rgb(0xe2e3e0), ACCENT_DARK) };
    // W-01: the mast's and the notes' ink, the heads as its second hue, the stalk upright as its third
    // and leaning as its ground (which the picture covers): a blade's two paints differ only in tone and
    // accent, so the one it leans to is the ground with no hue (`touch::field`).
    if i == TOUCH_GRASS { return Theme { ink: touch::INK, accent, second: touch::HEAD, third: touch::STALK, ground: touch::STALK_LEAN, ground2: touch::STALK_LEAN, ..amenui::style::DARK }; }
    Theme { ink, accent, second: ACCENT_LIGHT, third: SURVEY, ..amenui::style::DARK }
}
/// The bar's tone on a page: 0 none (the site's), 1 light, 2 dark.
pub fn tone(p: Page) -> u8 { match p { Page::Work(i) => if PIECES[i as usize].bar_light { 1 } else { 2 }, _ => 0 } }
/// Frames a second for what moves by itself on a page (`Ui::ambient_fps`): 30 on a page of Works.
pub fn ambient_fps(p: Page) -> f32 { if p.section() == Section::Works { 30.0 } else { 60.0 } }
/// The status line's tone on a page (the ground at the foot of the picture), as `tone`.
pub fn foot_tone(p: Page) -> u8 { match p { Page::Work(i) => if PIECES[i as usize].light { 1 } else { 2 }, _ => 0 } }

// ── State ──────────────────────────────────────────────────────────────────────────────────────────

/// The section's state, built on entering it and dropped on leaving.
#[derive(Clone, Debug)]
pub struct Works {
    /// The index: the wide window, the widths it grows from and since when, the live one, the band a
    /// first tap widened on a phone.
    pub wide: usize,
    pub from: [f32; COUNT],
    pub wide_at: f32,
    pub live: Option<usize>,
    pub tapped: Option<usize>,
    /// The last input (state clock), the chrome's fade (from, to, since), whether the piece rests.
    pub last_input: f32,
    pub fade: (f32, f32, f32),
    pub resting: bool,
    /// `Space`: the picture held.
    pub held: bool,
    /// The last tick.
    pub t: f32,
    pub ground: Ground,
    pub grass: touch::Grass,
}
impl Default for Works {
    fn default() -> Self {
        Self { wide: first_built(), from: [-1.0; COUNT], wide_at: -1.0, live: None, tapped: None, last_input: 0.0, fade: (1.0, 1.0, -1.0), resting: false, held: false, t: 0.0, ground: Ground::default(), grass: touch::Grass::default() }
    }
}

/// W-03: the flight of birds (seconds into it; its first frame is the rest pose, halfway), when the
/// next one starts, the scan window (its centre, the pointer, pinned), how human the pointer looks,
/// and the pointer box's accent (1 the dark ground's, 0 the light one's: from, to, since).
#[derive(Clone, Debug)]
pub struct Ground {
    pub flight: Option<f32>,
    /// When the flight last stepped (state clock; below 0 before the first step).
    pub stepped: f32,
    pub next: f32,
    pub scan: Option<(f32, f32)>,
    pub pointer: Option<(f32, f32)>,
    pub down: bool,
    pub pinned: bool,
    pub human: f32,
    /// The pointer's last three places (x, y, clock), and when it last moved.
    pub trail: Vec<(f32, f32, f32)>,
    pub moved_at: f32,
    pub dark: (f32, f32, f32),
    /// The `HUMAN?` label's ink (1 the foot's ink, 0 the survey ink: from, to, since).
    pub label: (f32, f32, f32),
}
impl Default for Ground {
    fn default() -> Self { Self { flight: Some(FLIGHT * 0.5), stepped: -1.0, next: f32::INFINITY, scan: None, pointer: None, down: false, pinned: false, human: 0.62, trail: vec![], moved_at: -10.0, dark: UNSET, label: UNSET } }
}
/// A flight takes 16 s; one starts every 40 s.
const FLIGHT: f32 = 16.0;
const EVERY: f32 = 40.0;
const BIRDS: usize = 7;
/// The flight: each bird's place from the leader (px), a loose line. No two boxes are nearer than
/// 12 px (N.2 item 9), no echo touches another bird's box, and the three labels under their boxes
/// touch no box, echo or other label, at full spacing and at a phone's (`tests/works.rs`).
const FLOCK: [(f32, f32); BIRDS] = [(0.0, 0.0), (55.0, -18.0), (93.0, 21.0), (136.0, 41.0), (185.0, 3.0), (235.0, 23.0), (278.0, 44.0)];
/// A bird's box, about its centre (22 × 16).
const BOX: (f32, f32, f32, f32) = (-11.0, -10.0, 11.0, 6.0);
/// The echoes: how far behind the bird and at what share of the ink (N.2 item 5).
const ECHOES: [(f32, f32); 3] = [(30.0, 0.45), (60.0, 0.25), (90.0, 0.12)];
/// The scan window's cells (N.2 item 4: 14 px square until the framework has a 7 × 14 pitch), and
/// how many: 21 × 14 on a desktop, 15 × 10 on a phone.
const CELL: f32 = 14.0;
fn scan_cells(w: f32) -> (usize, usize) { if w < NARROW { (15, 10) } else { (21, 14) } }
fn scan_size(w: f32) -> (f32, f32) { let (c, r) = scan_cells(w); (c as f32 * CELL, r as f32 * CELL) }
/// The characters of the scan window, lightest first: seven levels and no blank, so the window is a
/// full grid wherever it is (O.2 item 2). A cell's level is by eighths of its luminance, as built, the
/// two lightest eighths both `.`.
const RAMP: [char; 7] = ['.', ':', '-', '=', '+', '*', '#'];
fn level(l: f32) -> usize { (((1.0 - l) * 8.0).floor() as usize).clamp(1, 7) - 1 }
/// Below this luminance under the pointer its box takes the dark ground's accent (N.2 item 6).
const DARK_BELOW: f32 = 0.25;
/// Below this mean luminance under `HUMAN?` the label is in the foot's ink, at or above it in the survey
/// ink (O.2 item 3).
const LABEL_BELOW: f32 = 0.18;
/// Each side of those two thresholds holds until the luminance has crossed it by this much, so neither
/// flickers along a ridge.
const HOLD: f32 = 0.03;
/// The chrome fades after this long without input; the piece rests after this.
const IDLE: f32 = 3.0;
const REST: f32 = 60.0;
const FADE: f32 = 0.15;
/// The pointer box's change of accent, by opacity.
const RETONE: f32 = 0.12;
/// The index's windows widen over this, cubic in and out.
const WIDEN: f32 = 0.32;
const WIDE: f32 = 560.0;
/// Frames for what moves by itself, a second.
const PACE: f32 = 30.0;
/// Departure Mono at 11 px with 0.06 em tracking: the advance of a character, and where a line's ink
/// lies in its box (from its left, from its top, its height), measured on renders.
pub(crate) const ADV: f32 = 69.0 / 9.0;
const INK_X: f32 = 1.0;
const INK_TOP: f32 = 5.0;
const INK_H: f32 = 8.0;
/// Departure Mono's advance at 11 px, without tracking.
const MONO_ADV: f32 = 7.0;
/// A note is drawn only when its ink box is this far inside the window, on screen (N.2 item 2).
const INSIDE: f32 = 8.0;
/// And never this near the title block (M.2, N.2 item 7).
const CLEAR: f32 = 24.0;
/// How far outside the scan window a line it crosses is cut (O.2 item 6).
const CUT: f32 = 8.0;
/// How far the scan window's corner marks stand outside it.
const BRACKET: f32 = 6.0;

#[derive(Clone, Debug, PartialEq)]
pub enum WMsg {
    /// The pointer over window `i` of the index (in the piece's own space), or gone from it.
    Hover(usize, Option<(f32, f32)>),
    /// A press on window `i` (a phone's first tap widens it), on a piece's address.
    Tap(usize),
    Open(usize),
    Index,
    /// The pointer over the piece on its page.
    Point(Option<Point>),
    /// The piece's recipe in the docs.
    Recipe(usize),
    /// A drag over W-01's field (its own space).
    Walk(f32, f32),
}

/// Entering the index or a piece: from outside the section, everything is built new. An unbuilt
/// piece has no page: its address is the index's.
pub fn enter(s: &mut Gallery, p: Page) {
    let p = match p { Page::Work(i) if !PIECES[i as usize].page => { s.page = Page::Works; Page::Works } _ => p };
    if s.style.is_some() { s.site_look(); }
    let inside = s.from.section() == Section::Works && s.from != p;
    if !inside {
        s.works = Works { last_input: s.clock, t: s.clock, ..Works::default() };
        if let Page::Work(i) = p { s.works.wide = i as usize; }
    }
    s.works.last_input = s.clock;
    s.works.fade = (1.0, 1.0, -1.0);
    s.works.resting = false;
    s.works.held = false;
    if let Page::Work(i) = p { s.works.live = None; s.works.wide = i as usize; }
    if touch::on(p) { touch::enter(s); }
    let g = &mut s.works.ground;
    if g.next.is_infinite() { g.next = s.clock + (FLIGHT * 0.5) + (EVERY - FLIGHT); }
}

fn running(s: &Gallery, i: usize) -> bool {
    match s.page { Page::Work(j) => j as usize == i, Page::Works => s.works.live == Some(i), _ => false }
}
fn reduced(s: &Gallery) -> bool { s.body.reduced }

/// The size a piece is laid out at: the window on its page; on the index the window too (the canvas
/// lies under the bar and above the status line there); a phone's bands show the 16 : 10 piece.
fn piece_size(s: &Gallery) -> (f32, f32) {
    match s.page {
        Page::Work(_) => (s.width, s.height),
        _ if s.width < NARROW => (1440.0, 900.0),
        _ => (s.width, s.height + BAR + 28.0),
    }
}

pub fn update(s: &mut Gallery, m: WMsg) {
    let now = s.clock;
    s.works.last_input = now;
    match m {
        WMsg::Hover(i, _) if !PIECES[i].built => {}
        WMsg::Hover(i, p) => {
            if p.is_some() { widen(s, i); s.works.live = Some(i); } else if s.works.live == Some(i) { s.works.live = None; }
            if i == GROUND_TRUTH { point(s, p); }
        }
        WMsg::Tap(i) | WMsg::Open(i) if !PIECES[i].built => {}
        WMsg::Tap(i) => {
            if s.width < NARROW && s.works.tapped != Some(i) { s.works.tapped = Some(i); widen(s, i); s.works.live = Some(i); } else { go(s, Page::Work(i as u8)); }
        }
        WMsg::Open(i) => go(s, Page::Work(i as u8)),
        WMsg::Index => go(s, Page::Works),
        WMsg::Point(p) => {
            let down = p.is_some_and(|p| p.down);
            if down && !s.works.ground.down && running(s, GROUND_TRUTH) {
                // A press pins the scan window where it is; a second frees it.
                let g = &mut s.works.ground;
                g.pinned = !g.pinned;
            }
            s.works.ground.down = down;
            if running(s, GROUND_TRUTH) { point(s, p.map(|p| (p.x, p.y))); }
        }
        WMsg::Recipe(i) => s.navigate = Some(format!("docs/{}", PIECES[i].recipe)),
        WMsg::Walk(x, y) => if touch::on(s.page) { walk(s, x, y); },
    }
}
/// Go to a page of the section as a navigation does (its transition, the address).
fn go(s: &mut Gallery, p: Page) { let path = p.path(); s.route(&path); }

/// The index's wide window becomes `i`: the widths grow from where they are now.
fn widen(s: &mut Gallery, i: usize) {
    if s.works.wide == i { return; }
    let now = s.clock;
    s.works.from = widths(s, now);
    s.works.wide = i;
    s.works.wide_at = now;
}
/// The index's four widths at `t` (desktop).
fn widths(s: &Gallery, t: f32) -> [f32; COUNT] {
    let total = s.width - 64.0 - 16.0 * (COUNT as f32 - 1.0);
    let narrow = ((total - WIDE) / (COUNT as f32 - 1.0)).max(0.0);
    let to: [f32; COUNT] = std::array::from_fn(|j| if j == s.works.wide { WIDE.min(total) } else { narrow });
    let w = &s.works;
    if w.from[0] < 0.0 || reduced(s) { return to; }
    let k = ((t - w.wide_at) / WIDEN).clamp(0.0, 1.0);
    let k = if k < 0.5 { 4.0 * k * k * k } else { 1.0 - (-2.0 * k + 2.0).powi(3) * 0.5 };
    std::array::from_fn(|j| w.from[j] + (to[j] - w.from[j]) * k)
}

/// The pointer over W-03 (its own space): the scan window follows it, and it is judged.
fn point(s: &mut Gallery, p: Option<(f32, f32)>) {
    let now = s.clock;
    let z = zoom_of(s);
    let (w, h) = piece_size(s);
    let fit = fit_for(s, w, h);
    let g = &mut s.works.ground;
    g.pointer = p;
    let Some((x, y)) = p else { g.trail.clear(); return };
    // The box's accent and the label's ink follow the picture under each, changing by opacity (N.2
    // item 6, O.2 item 3).
    let (px0, py0) = fit.to_picture(x, y);
    let l = mean_lum(px0 - 16.0 / fit.s, py0 - 16.0 / fit.s, px0 + 16.0 / fit.s, py0 + 16.0 / fit.s);
    g.dark = retone(g.dark, l, DARK_BELOW, now);
    let li = human_ink(x, y, z);
    let (a, b) = (fit.to_picture(li.0, li.1), fit.to_picture(li.2, li.3));
    g.label = retone(g.label, mean_lum(a.0, a.1, b.0, b.1), LABEL_BELOW, now);
    g.trail.push((x, y, now));
    if g.trail.len() > 3 { g.trail.remove(0); }
    if g.trail.len() == 3 {
        let (a, b, c) = (g.trail[0], g.trail[1], g.trail[2]);
        let (v1, v2) = ((b.0 - a.0, b.1 - a.1), (c.0 - b.0, c.1 - b.1));
        let (l1, l2) = (v1.0.hypot(v1.1), v2.0.hypot(v2.1));
        if l1 > 0.5 && l2 > 0.5 {
            // A hand turns and changes speed; a still or dead-straight pointer does neither.
            let turn = ((v1.0 * v2.1 - v1.1 * v2.0) / (l1 * l2)).abs();
            let (s1, s2) = (l1 / (b.2 - a.2).max(1e-3), l2 / (c.2 - b.2).max(1e-3));
            let change = (s2 - s1).abs() / s1.max(s2).max(1e-3);
            let hand = turn > 0.035 || change > 0.15;
            let to = if hand { 0.99 } else { 0.50 };
            g.human += (to - g.human) * 0.06;
            g.moved_at = now;
        }
    }
}
/// How far a change of tone `(from, to, since)` is at `t` (toward 1, the dark ground's side).
fn tone_at((a, b, at): (f32, f32, f32), t: f32) -> f32 { a + (b - a) * ((t - at) / RETONE).clamp(0.0, 1.0) }
/// How far the pointer box is toward the dark ground's accent at `t`.
fn dark_at(g: &Ground, t: f32) -> f32 { tone_at(g.dark, t) }
/// A tone not set yet (no pointer so far).
const UNSET: (f32, f32, f32) = (0.0, 0.0, f32::NEG_INFINITY);
/// A tone that follows luminance `l` across `below`: the dark side (1) under it, the light side (0) at
/// or over it, each side held until `l` has crossed by `HOLD`; the first is set at once.
fn retone(c: (f32, f32, f32), l: f32, below: f32, now: f32) -> (f32, f32, f32) {
    if c.2 == f32::NEG_INFINITY { let to = (l < below) as u8 as f32; return (to, to, f32::MIN); }
    let to = if c.1 >= 0.5 { (l < below + HOLD) as u8 as f32 } else { (l < below - HOLD) as u8 as f32 };
    if to == c.1 { return c; }
    (tone_at(c, now), to, now)
}
/// The ink box of `HUMAN?` beside a pointer at (x, y), in a piece drawn at `z`.
fn human_ink(x: f32, y: f32, z: f32) -> R { label_ink("HUMAN? 0.00", x.round() + 22.0 / z, y.round() - 16.0 / z, z) }
/// The scale a piece is drawn at in the view on screen: 1 on its page, its window's on the index.
fn zoom_of(s: &Gallery) -> f32 {
    let (pw, ph) = piece_size(s);
    match s.page {
        Page::Work(_) => 1.0,
        _ if s.width < NARROW => (s.width - 48.0) / pw,
        _ => (pw * (ph - 400.0).max(320.0) / ph).round() / pw,
    }
}

/// The input that moves the pointer anywhere: the chrome comes back, a resting piece wakes. Over W-01
/// the pointer walks its path, which the next frame takes without a new view of its own.
pub fn pointer(s: &mut Gallery, x: f32, y: f32) -> bool {
    if touch::on(s.page) { walk(s, x, y); }
    heard(s)
}
/// The pointer at (x, y) on W-01's page.
fn walk(s: &mut Gallery, x: f32, y: f32) {
    let (w, h) = (s.width, s.height);
    let fit = Fit::new(w, h, PIECES[TOUCH_GRASS].focus);
    touch::point(s, Some((x, y)), w, &fit);
}
/// A press, a wheel turn or a key: the same.
pub fn heard(s: &mut Gallery) -> bool {
    if s.page.section() != Section::Works { return false; }
    let was = s.works.resting || s.works.fade.1 < 1.0;
    s.works.last_input = s.clock;
    was
}

/// Keys no control used. The index: ← → move the wide window to the next built piece, Enter opens it.
/// A piece: ← → the nearest built piece that way (or nothing), Esc the index, Space holds and
/// releases the picture.
pub fn key(s: &mut Gallery, k: amenui::el::Key) -> bool {
    use amenui::el::Key;
    s.works.last_input = s.clock;
    let dir = match k { Key::Left => -1, Key::Right => 1, _ => 0 };
    match (s.page, k) {
        (Page::Works, Key::Left | Key::Right) => { if let Some(i) = step(s.works.wide, dir) { widen(s, i); s.works.live = Some(i); } true }
        (Page::Works, Key::Enter) => { if PIECES[s.works.wide].built { go(s, Page::Work(s.works.wide as u8)); } true }
        (Page::Work(i), Key::Left | Key::Right) => { if let Some(j) = step(i as usize, dir) { go(s, Page::Work(j as u8)); } true }
        (Page::Work(_), Key::Escape) => { go(s, Page::Works); true }
        (Page::Work(_), Key::Char(' ')) => { s.works.held = !s.works.held; true }
        _ => false,
    }
}

/// The chrome's opacity at `t`.
fn fade_at(w: &Works, t: f32) -> f32 {
    let (a, b, at) = w.fade;
    a + (b - a) * ((t - at) / FADE).clamp(0.0, 1.0)
}

pub fn tick(s: &mut Gallery, t: f32) -> bool {
    if s.page.section() != Section::Works { return false; }
    let dt = (t - s.works.t).clamp(0.0, 0.1);
    s.works.t = t;
    let mut changed = false;
    if matches!(s.page, Page::Work(_)) {
        let w = &mut s.works;
        let to = if t - w.last_input < IDLE { 1.0 } else { 0.0 };
        if to != w.fade.1 { w.fade = (fade_at(w, t), to, t); changed = true; }
        if t - w.fade.2 <= FADE + 0.02 { changed = true; }
        let resting = t - w.last_input >= REST;
        if resting != w.resting { w.resting = resting; changed = true; }
    }
    if s.page == Page::Works && t - s.works.wide_at <= WIDEN + 0.02 { changed = true; }
    if running(s, GROUND_TRUTH) { changed |= ground_tick(s, t, dt); }
    if touch::on(s.page) { let (w, h) = (s.width, s.height); let fit = Fit::new(w, h, PIECES[TOUCH_GRASS].focus); changed |= touch::tick(s, t, w, &fit); }
    changed
}
fn ground_tick(s: &mut Gallery, t: f32, dt: f32) -> bool {
    let (w, h) = piece_size(s);
    let rest = rest_place(s, w, h);
    let (held, resting, still) = (s.works.held, s.works.resting, reduced(s));
    let g = &mut s.works.ground;
    let mut changed = false;
    if still {
        g.flight = Some(FLIGHT * 0.5);
    } else if let Some(f) = g.flight {
        // What moves by itself moves at 30 fps (M.7): a tick between two steps changes nothing.
        let since = if g.stepped < 0.0 { 1.0 / PACE } else { t - g.stepped };
        if !held && since >= 1.0 / PACE - 1e-3 {
            let f = f + since.min(0.1);
            g.stepped = t;
            if f >= FLIGHT { g.flight = None; g.next = t + EVERY - FLIGHT; } else { g.flight = Some(f); }
            changed = true;
        }
    } else if !held && !resting && t >= g.next {
        g.flight = Some(0.0);
        g.stepped = t;
        changed = true;
    }
    // The scan window: on a spring 120 ms behind the pointer, pinned where a press left it, at its
    // rest place with no pointer.
    let target = if g.pinned { g.scan.unwrap_or(rest) } else { g.pointer.unwrap_or(rest) };
    let at = g.scan.unwrap_or(rest);
    let next = if still { target } else {
        let k = 1.0 - (-dt / 0.12).exp();
        let n = (at.0 + (target.0 - at.0) * k, at.1 + (target.1 - at.1) * k);
        if (n.0 - target.0).hypot(n.1 - target.1) < 0.3 { target } else { n }
    };
    if g.scan != Some(next) { g.scan = Some(next); changed = true; }
    if t - g.dark.2.max(g.label.2) <= RETONE + 0.02 { changed = true; }
    // Held still, the pointer looks less human.
    if g.pointer.is_some() && t - g.moved_at > 0.25 && (g.human - 0.5).abs() > 0.004 {
        let before = (g.human * 100.0).round();
        g.human += (0.5 - g.human) * (1.0 - (-dt / 1.5).exp());
        if (g.human * 100.0).round() != before { changed = true; }
    }
    changed
}

/// When the section next wants a frame: `Some(now)` for what the hand moves, the next 30th of a second
/// for what moves by itself, the next event otherwise.
pub fn wake(s: &Gallery, t: f32) -> Option<f32> {
    if s.page.section() != Section::Works { return None; }
    let w = &s.works;
    let mut at: Option<f32> = None;
    let mut want = |v: f32| at = Some(at.map_or(v, |a: f32| a.min(v)));
    let paced = w.ground.stepped + 1.0 / PACE;
    if matches!(s.page, Page::Work(_)) {
        if t - w.fade.2 < FADE { want(t); } else if w.fade.1 > 0.5 { want(w.last_input + IDLE); }
        if !w.resting { want(w.last_input + REST); }
    }
    if s.page == Page::Works && w.wide_at >= 0.0 && t - w.wide_at < WIDEN { want(t); }
    if touch::on(s.page) && let Some(v) = touch::wake(s, t) { want(v); }
    if running(s, GROUND_TRUTH) && !reduced(s) {
        let g = &w.ground;
        if g.flight.is_some() { if !w.held { want(paced); } } else if !w.held && !w.resting { want(g.next); }
        let (pw, ph) = piece_size(s);
        let target = if g.pinned { g.scan } else { Some(g.pointer.unwrap_or(rest_place(s, pw, ph))) };
        if g.scan.is_none() || target.is_some_and(|p| Some(p) != g.scan) { want(t); }
        if t - g.dark.2.max(g.label.2) < RETONE { want(t); }
        if g.pointer.is_some() && (g.human - 0.5).abs() > 0.004 { want(paced.max(g.moved_at + 0.25)); }
    }
    at
}

// ── Look ───────────────────────────────────────────────────────────────────────────────────────────

pub fn look(s: &Gallery, l: &mut Look) {
    if s.page.section() != Section::Works { return; }
    l.pics[PIC_SLOT as usize] = Markr { pitch: 1.0, ..Markr::default() };
    l.material = Material::Plain;
    // A piece's first frame is its rest pose, whole: nothing condenses.
    l.fx.enter = 0.0;
    let ink = ink_space(s);
    match s.page {
        Page::Work(i) => {
            l.theme = theme(i as usize);
            (l.theme.ground, l.theme.ground2) = (ink.top, ink.bottom);
            // The finishing pass's fixed grain, and nothing else (M.2).
            l.post = amenui::post::PostSpec { grain: amenui::post::Grain { amount: 0.04, size: 1.0 }, ..amenui::post::PostSpec::default() }.look();
        }
        _ => { l.theme.second = ACCENT_LIGHT; l.theme.third = SURVEY; }
    }
}

/// The index → piece change and back: until the framework's Flow grows the window (N.3 item 1), a
/// 120 ms opacity change, not a cut.
pub fn flow(_s: &Gallery, from: Page, to: Page) -> Option<FlowSpec> {
    match (from, to) {
        (Page::Works, Page::Work(_)) | (Page::Work(_), Page::Works) => Some(FlowSpec { flow_ms: 0.0, flow_loose: 0.0, out_ms: 120.0, fade_ms: 120.0 }),
        _ => None,
    }
}

/// The colours a page can draw over a picture: its ground (a ramp from `top` at y = 0 to `bottom` at
/// y = `h`), its ink, and its three hues.
#[derive(Clone, Copy, Debug)]
pub struct InkSpace { pub top: [f32; 3], pub bottom: [f32; 3], pub h: f32, pub ink: [f32; 3], pub hues: [[f32; 3]; 3] }
fn ink_space(s: &Gallery) -> InkSpace {
    match s.page {
        Page::Work(i) => {
            let th = theme(i as usize);
            let (top, bottom) = if i as usize == GROUND_TRUTH { sky_ramp(&fit_for(s, s.width, s.height), s.height) } else { (th.ground, th.ground2) };
            InkSpace { top, bottom, h: s.height, ink: th.ink, hues: [th.accent, th.second, th.third] }
        }
        _ => {
            let th = s.theme();
            InkSpace { top: th.ground, bottom: th.ground2, h: s.height, ink: th.ink, hues: [th.accent, ACCENT_LIGHT, SURVEY] }
        }
    }
}
/// The straight ramp nearest W-03's sky down the window over the band the faint shades are drawn in
/// (the flight and the cloud: 190 to 390 px of the picture; least squares, its two ends kept
/// to colours): the ground a page's mixes
/// over the sky start from.
fn sky_ramp(fit: &Fit, h: f32) -> ([f32; 3], [f32; 3]) {
    let (a, b) = (fit.at(0.0, 190.0).1, fit.at(0.0, 390.0).1);
    let n = 48;
    let (mut sy, mut syy) = (0.0f32, 0.0f32);
    let (mut sc, mut syc) = ([0.0f32; 3], [0.0f32; 3]);
    for k in 0..n {
        let y = a + (b - a) * (k as f32 + 0.5) / n as f32;
        let v = y / h;
        let c = sky_at(fit.to_picture(0.0, y).1);
        sy += v; syy += v * v;
        for j in 0..3 { sc[j] += c[j]; syc[j] += v * c[j]; }
    }
    let nf = n as f32;
    let d = nf * syy - sy * sy;
    let (mut top, mut bottom) = ([0.0; 3], [0.0; 3]);
    for j in 0..3 {
        let b = (nf * syc[j] - sy * sc[j]) / d;
        let a = (sc[j] - b * sy) / nf;
        // An end past a colour is held at it and the other end fitted again.
        let (mut t, mut e) = (a, a + b);
        if !(0.0..=1.0).contains(&e) {
            e = e.clamp(0.0, 1.0);
            // c ≈ t (1 − v) + e v: least squares for t alone.
            let (num, den) = (sc[j] - e * sy - (syc[j] - e * syy), nf - 2.0 * sy + syy);
            t = num / den;
        }
        if !(0.0..=1.0).contains(&t) {
            t = t.clamp(0.0, 1.0);
            e = ((syc[j] - t * (sy - syy)) / syy).clamp(0.0, 1.0);
        }
        top[j] = t.clamp(0.0, 1.0);
        bottom[j] = e;
    }
    (top, bottom)
}
impl InkSpace {
    /// The colour paint `p` is at screen height `y`.
    fn colour(&self, p: Paint, y: f32) -> [f32; 3] { mix3(mix3(self.ground(y), self.ink, p.tone), self.hues[p.hue.min(2) as usize], p.accent) }
    fn ground(&self, y: f32) -> [f32; 3] { let k = (y / self.h.max(1.0)).clamp(0.0, 1.0); std::array::from_fn(|j| self.top[j] + (self.bottom[j] - self.top[j]) * k) }
    /// The paint nearest colour `c` at screen height `y`: of the three mixes ground → ink → hue, the
    /// one whose triangle comes closest.
    pub fn paint(&self, c: [f32; 3], y: f32) -> Paint {
        let g = self.ground(y);
        let mut best = (f32::INFINITY, Paint::INK);
        for (hue, h) in self.hues.iter().enumerate() {
            let (u, v, e) = nearest_in_triangle(g, self.ink, *h, c);
            if e < best.0 {
                let tone = if v < 1.0 { (u / (1.0 - v)).clamp(0.0, 1.0) } else { 0.0 };
                best = (e, Paint { tone, accent: v.clamp(0.0, 1.0), alpha: 1.0, hue: hue as u8 });
            }
        }
        best.1
    }
}
/// The point of triangle g, i, h nearest `c`, as (share of i, share of h, squared distance).
fn nearest_in_triangle(g: [f32; 3], i: [f32; 3], h: [f32; 3], c: [f32; 3]) -> (f32, f32, f32) {
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let (e1, e2, d) = (sub(i, g), sub(h, g), sub(c, g));
    let at = |u: f32, v: f32| { let p = [g[0] + u * e1[0] + v * e2[0], g[1] + u * e1[1] + v * e2[1], g[2] + u * e1[2] + v * e2[2]]; let q = sub(p, c); dot(q, q) };
    let (a, b, cc) = (dot(e1, e1), dot(e1, e2), dot(e2, e2));
    let (p, q) = (dot(e1, d), dot(e2, d));
    let det = a * cc - b * b;
    if det.abs() > 1e-9 {
        let u = (cc * p - b * q) / det;
        let v = (a * q - b * p) / det;
        if u >= 0.0 && v >= 0.0 && u + v <= 1.0 { return (u, v, at(u, v)); }
    }
    // On an edge: the best of the three.
    let seg = |p0: (f32, f32), p1: (f32, f32)| {
        let mut best = (p0.0, p0.1, at(p0.0, p0.1));
        for k in 1..=32 { let t = k as f32 / 32.0; let (u, v) = (p0.0 + (p1.0 - p0.0) * t, p0.1 + (p1.1 - p0.1) * t); let e = at(u, v); if e < best.2 { best = (u, v, e); } }
        best
    };
    [seg((0.0, 0.0), (1.0, 0.0)), seg((0.0, 0.0), (0.0, 1.0)), seg((1.0, 0.0), (0.0, 1.0))].into_iter().min_by(|x, y| x.2.total_cmp(&y.2)).unwrap_or((0.0, 0.0, 0.0))
}
fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] { std::array::from_fn(|k| a[k] + (b[k] - a[k]) * t) }
/// The WCAG contrast of two colours.
fn contrast(a: [f32; 3], b: [f32; 3]) -> f32 { let (a, b) = (lum(a), lum(b)); (a.max(b) + 0.05) / (a.min(b) + 0.05) }
/// WCAG relative luminance of an sRGB colour (0–1).
fn lum(c: [f32; 3]) -> f32 {
    let l = |v: f32| if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
    0.2126 * l(c[0]) + 0.7152 * l(c[1]) + 0.0722 * l(c[2])
}

// ── Pictures ───────────────────────────────────────────────────────────────────────────────────────

/// Give a renderer every piece's picture (native hosts and offscreen renders; the web makes each when
/// a page first draws it).
pub fn set_pictures(r: &mut amenui::render::Renderer) {
    for i in 0..=COUNT as u32 + 1 { if let Some((w, h, px)) = picture_pixels(PIC + i) { r.set_picture(PIC + i, w, h, &px); } }
}
/// The size of piece picture `id` (`PIC + i`).
pub fn picture_size(id: u32) -> Option<(u32, u32)> {
    match id.checked_sub(PIC)? as usize { i if i <= COUNT + 1 => Some((960, 600)), _ => None }
}
/// W-03's picture, made once (the scan window and the faint shades read its colours).
fn ground_pixels() -> &'static [u8] {
    static PX: OnceLock<Vec<u8>> = OnceLock::new();
    PX.get_or_init(|| {
        let (w, h) = (960u32, 600u32);
        let mut px = Vec::with_capacity((w * h * 4) as usize);
        // The ridges' profiles, once per column of each.
        let cols: Vec<[f64; 5]> = (0..w).map(|x| { let dx = (x as f64 + 0.5) * 1.5; std::array::from_fn(|r| ridge_y(dx, r)) }).collect();
        for y in 0..h {
            for x in 0..w { put(&mut px, ground_at((x as f64 + 0.5) * 1.5, (y as f64 + 0.5) * 1.5, &cols[x as usize])); }
        }
        px
    })
}
fn put(px: &mut Vec<u8>, c: [f64; 3]) { for v in c { px.push((v.clamp(0.0, 255.0)).round() as u8); } px.push(255); }
/// W-03's colour at (x, y) in px of its 1440 × 900 picture (0–1).
fn picture_at(x: f32, y: f32) -> [f32; 3] {
    let px = ground_pixels();
    let (i, j) = (((x / 1.5) as i32).clamp(0, 959) as usize, ((y / 1.5) as i32).clamp(0, 599) as usize);
    let o = (j * 960 + i) * 4;
    [px[o] as f32 / 255.0, px[o + 1] as f32 / 255.0, px[o + 2] as f32 / 255.0]
}
/// The mean colour of a rectangle of W-03's picture (px of 1440 × 900), from a 4 × 4 grid of samples.
fn mean_at(x0: f32, y0: f32, x1: f32, y1: f32) -> [f32; 3] {
    let mut c = [0.0; 3];
    for j in 0..4 { for i in 0..4 {
        let p = picture_at(x0 + (x1 - x0) * (i as f32 + 0.5) / 4.0, y0 + (y1 - y0) * (j as f32 + 0.5) / 4.0);
        for k in 0..3 { c[k] += p[k] / 16.0; }
    } }
    c
}
fn mean_lum(x0: f32, y0: f32, x1: f32, y1: f32) -> f32 { lum(mean_at(x0, y0, x1, y1)) }
/// A cell of W-03's picture: its mean colour, from a 4 × 4 grid of samples, and its median luminance,
/// from an 8 × 8 grid (about a sample a pixel of the picture: a cell across a ridge's edge has its
/// median where its pixels have it).
fn cell_at(x0: f32, y0: f32, x1: f32, y1: f32) -> ([f32; 3], f32) {
    // Each channel's share of the luminance by its byte, worked out once.
    static LIN: OnceLock<[[f32; 256]; 3]> = OnceLock::new();
    let lin = LIN.get_or_init(|| std::array::from_fn(|k| std::array::from_fn(|v| { let mut c = [0.0; 3]; c[k] = v as f32 / 255.0; lum(c) })));
    let px = ground_pixels();
    let mut ls = [0.0f32; 64];
    for (n, l) in ls.iter_mut().enumerate() {
        let (x, y) = (x0 + (x1 - x0) * ((n % 8) as f32 + 0.5) / 8.0, y0 + (y1 - y0) * ((n / 8) as f32 + 0.5) / 8.0);
        let o = (((y / 1.5) as i32).clamp(0, 599) as usize * 960 + ((x / 1.5) as i32).clamp(0, 959) as usize) * 4;
        *l = lin[0][px[o] as usize] + lin[1][px[o + 1] as usize] + lin[2][px[o + 2] as usize];
    }
    ls.sort_by(f32::total_cmp);
    (mean_at(x0, y0, x1, y1), (ls[31] + ls[32]) * 0.5)
}
/// Piece picture `id`, RGBA8.
pub fn picture_pixels(id: u32) -> Option<(u32, u32, Vec<u8>)> {
    let (w, h) = picture_size(id)?;
    let i = (id - PIC) as usize;
    if i == GROUND_TRUTH { return Some((w, h, ground_pixels().to_vec())); }
    if id == PIC_STILL { return Some((w, h, touch::still_pixels())); }
    if i == TOUCH_GRASS || id == PIC_PHONE { return Some((w, h, touch::pixels(id == PIC_PHONE))); }
    let stops = stops(i);
    // An unbuilt piece: its colours and the piece's grain (0.04, as the finishing pass's), nothing else.
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let c = ramp(stops, (y as f64 + 0.5) / h as f64);
        for x in 0..w {
            let n = (h01(((y as i64) << 16) ^ x as i64 ^ (i as i64) << 40) - 0.5) * 0.04 * 255.0;
            put(&mut px, [c[0] + n, c[1] + n, c[2] + n]);
        }
    }
    Some((w, h, px))
}
/// An unbuilt piece's colours, top to bottom.
fn stops(i: usize) -> &'static [(f64, u32)] {
    match i {
        0 => &[(0.0, 0xbcd3e4), (0.64, 0xeeeee2), (0.64, 0xd9cc92), (1.0, 0xc2a95a)],
        1 => &[(0.0, 0x2b66c4), (0.55, 0x5f9ae0), (1.0, 0xa9cdee)],
        _ => &[(0.0, 0x070b16), (0.60, 0x0c1626), (0.80, 0x12233a), (0.80, 0x0a1320), (1.0, 0x070b14)],
    }
}
fn hex(c: u32) -> [f64; 3] { [((c >> 16) & 255) as f64, ((c >> 8) & 255) as f64, (c & 255) as f64] }
fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] { std::array::from_fn(|k| a[k] + (b[k] - a[k]) * t) }
/// A vertical ramp through `stops` (position, colour) at `v`; two stops at one place are a hard edge.
fn ramp(stops: &[(f64, u32)], v: f64) -> [f64; 3] {
    for p in stops.windows(2) {
        let ((a, ca), (b, cb)) = (p[0], p[1]);
        if v >= a && v <= b && b > a { return mix(hex(ca), hex(cb), (v - a) / (b - a)); }
    }
    hex(stops.last().map_or(0, |s| s.1))
}
/// W-03's sky alone at height `y` (px of 1440 × 900), 0–1.
const SKY: [(f64, u32); 5] = [(0.0, 0x7d8fb4), (0.30, 0xc9b3b4), (0.52, 0xf2c99b), (0.66, 0xf6dcae), (1.0, 0xf6dcae)];
fn sky_at(y: f32) -> [f32; 3] { let c = ramp(&SKY, (y / 900.0) as f64); [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0] }

// The value noise of the mock-ups (`works/src/lib.py`), so the profiles are the drawn ones.
fn h01(n: i64) -> f64 {
    let n = n.wrapping_mul(374_761_393).wrapping_add(668_265_263) & 0xffff_ffff;
    let n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177) & 0xffff_ffff;
    ((n ^ (n >> 16)) & 0xffff) as f64 / 65535.0
}
fn vnoise(x: f64, y: f64, seed: i64) -> f64 {
    let (xi, yi) = (x.floor() as i64, y.floor() as i64);
    let (fx, fy) = (x - xi as f64, y - yi as f64);
    let (fx, fy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let g = |i: i64, j: i64| h01(i.wrapping_mul(73_856_093) ^ j.wrapping_mul(19_349_663) ^ seed.wrapping_mul(83_492_791));
    (g(xi, yi) * (1.0 - fx) + g(xi + 1, yi) * fx) * (1.0 - fy) + (g(xi, yi + 1) * (1.0 - fx) + g(xi + 1, yi + 1) * fx) * fy
}
fn fbm(x: f64, y: f64, octaves: u32, seed: i64) -> f64 {
    let (mut v, mut a, mut t) = (0.0, 0.5, 0.0);
    for k in 0..octaves { let s = (1u32 << k) as f64; v += a * vnoise(x * s, y * s, seed + k as i64); t += a; a *= 0.5; }
    v / t
}
/// Ridge `i`'s height at `x`, both in px of the 1440 × 900 picture.
pub fn ridge_y(x: f64, i: usize) -> f64 {
    let base = [0.60, 0.655, 0.71, 0.775, 0.86][i];
    let amp = [0.050, 0.055, 0.06, 0.07, 0.06][i];
    let sc = [520.0, 430.0, 360.0, 300.0, 420.0][i];
    (base - amp * (fbm(x / sc + i as f64 * 13.7, i as f64 * 3.1, 5, 21 + i as i64) - 0.5) * 2.0) * 900.0
}
/// W-03's sky, sun and cloud and the five ridges at (x, y) in px of the 1440 × 900 picture.
const SUN: (f64, f64, f64) = (980.0, 0.575 * 900.0, 17.0);
pub const CLOUD: (f64, f64) = (1058.0, 356.0);
fn ground_at(x: f64, y: f64, ridges: &[f64; 5]) -> [f64; 3] {
    let v = y / 900.0;
    let mut c = ramp(&SKY, v);
    let d = (x - SUN.0).hypot(y - SUN.1);
    let glow = (-d / 230.0).exp();
    c = [c[0] + glow * 70.0, c[1] + glow * 52.0, c[2] + glow * 22.0];
    c = mix(c, hex(0xfff4d6), ((SUN.2 - d) / 1.5 + 0.5).clamp(0.0, 1.0));
    // The one small pale cloud (two soft ellipses).
    for (cx, cy, rx, ry, a) in [(CLOUD.0 + 2.0, CLOUD.1, 42.0, 5.0, 150.0 / 255.0), (CLOUD.0 + 3.0, CLOUD.1 - 3.0, 23.0, 6.0, 160.0 / 255.0)] {
        let e = (((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2)).sqrt();
        let cov = ((1.0 - e) * ry / 1.5 + 0.5).clamp(0.0, 1.0);
        c = mix(c, hex(0xfbe9cf), cov * a);
    }
    let cols = [0xc7a59a, 0xa3879a, 0x7a6a8c, 0x51496f, 0x2c2a47];
    for (i, &ry) in ridges.iter().enumerate() {
        let m = ((y - ry) / 1.5 + 0.5).clamp(0.0, 1.0);
        if m <= 0.0 { continue; }
        let haze = ((y - ry) / 160.0).clamp(0.0, 1.0) * (0.30 - 0.05 * i as f64);
        let r = mix(hex(cols[i]), hex(0xf2c99b), haze);
        c = mix(c, r, m);
    }
    c
}

// ── Views ──────────────────────────────────────────────────────────────────────────────────────────

pub(crate) fn mono<M>(s: impl Into<String>, paint: Paint) -> El<M> { text(s, 11.0, paint).font(Font::Face(amenui::text::MONO)).tracking(0.06) }
fn serif<M>(s: impl Into<String>, size: f32, track: f32, paint: Paint) -> El<M> { text(s, size, paint).font(Font::Whisper).tracking(track) }
pub(crate) fn at<M>(e: El<M>, x: f32, y: f32) -> El<M> { e.absolute(Some(x), Some(y), None, None) }
fn at_right<M>(e: El<M>, right: f32, y: f32) -> El<M> { e.absolute(None, Some(y), Some(right), None) }
fn k(slug: &str, part: &str) -> String { format!("work/{slug}/{part}") }

/// What of the section is English in every language (O.4 (10)): what is inside a picture, that is the
/// picture's layer, the titles under the windows and in the title block, the heading `works` and the
/// index's live line. The bar, the lead, the keys line, `how this is made ↗`, the status line and the
/// console follow the locale.
pub fn english(k: &str) -> bool {
    let k = k.strip_prefix("works/window/").unwrap_or(k);
    k == "works/title"
        || k.starts_with("work/") && (k.ends_with("/layer") || k.ends_with("/title"))
        || k.starts_with("works/w") && (k.ends_with("/title") || k.ends_with("/live"))
}

pub fn page(s: &Gallery) -> El<Msg> {
    match s.page { Page::Work(i) => piece_page(s, i as usize), _ if s.width < NARROW => index_phone(s), _ => index(s) }
}

/// A piece on its page: the picture, and the frame over it.
fn piece_page(s: &Gallery, i: usize) -> El<Msg> {
    let (w, h) = (s.width, s.height);
    let p = &PIECES[i];
    let pic = picture_layer(s, i, w, h, 1.0, Mode::Page, (0.0, 0.0, w, h)).flow(format!("work/{}", p.slug));
    // W-01 hears a mouse that moves without making a new view (`pointer`), and a drag (a finger) as
    // the path too.
    let pic = if i == TOUCH_GRASS { pic.on_drag_with(|d| Some(Msg::Works(WMsg::Walk(d.x, d.y)))).unframed() } else { pic.on_point(|p| Some(Msg::Works(WMsg::Point(p)))) };
    // Fronts do not move a piece, except W-01's field, which the map bends (`touch`).
    let root = col().key("work/page").width(w).height(h).clip().child(pic);
    let root = if i == TOUCH_GRASS { { let fit = Fit::new(w, h, p.focus); root.child(touch::area(touch::field(s, w, h, &fit), w, h, &fit)) } } else { root.calm() };
    let page = frame(s, i, w, h, root);
    if i == GROUND_TRUTH { return page.child(scan_and_pointer(s, w, h, &fit_for(s, w, h), 1.0, Mode::Page)); }
    if i == TOUCH_GRASS {
        let fit = Fit::new(w, h, p.focus);
        return page.child(touch::callout(s, w, &fit));
    }
    page
}

/// Where the title block's parts are on a piece's page: the number's, the title's and the small line's
/// tops, and the small line's wrapping width on a phone.
struct Block { n: f32, title: f32, small: f32, lines: f32 }
fn block(s: &Gallery, i: usize, w: f32, h: f32) -> Block { block_with(i, w, h, &micro(s, i)) }
fn block_with(_i: usize, w: f32, h: f32, small: &str) -> Block {
    if w < NARROW {
        let lines = (small.chars().count() as f32 * ADV / (w - 64.0)).ceil().max(1.0);
        let y_m = h - 120.0 - 14.0 * lines;
        Block { n: y_m - 76.0, title: y_m - 48.0, small: y_m, lines }
    } else {
        Block { n: h - 132.0, title: h - 104.0, small: h - 56.0, lines: 1.0 }
    }
}
/// Where the title's ink starts below its box's top (the serif at 30 px, measured on renders).
const TITLE_INK_TOP: f32 = 12.0;
/// The title block's box on a piece's page (x0, y0, x1, y1): from the number's ink to the small line's;
/// from the title's ink where the number is not drawn (P.2 item 1).
fn title_box(s: &Gallery, i: usize, w: f32, h: f32) -> (f32, f32, f32, f32) {
    let b = block(s, i, w, h);
    let small_w = if w < NARROW { w - 64.0 } else { micro(s, i).chars().count() as f32 * ADV };
    let title_w = PIECES[i].title.chars().count() as f32 * 30.0 * 0.8;
    let top = if number_paint(s, i, w, h).is_some() { b.n + INK_TOP } else { b.title + TITLE_INK_TOP };
    (32.0, top, 32.0 + small_w.max(title_w), b.small + 14.0 * b.lines)
}

/// The title block and the piece strip (M.2): the strip and the small line fade after 3 s without
/// input and return on the next. The title and the number stay as they are, in one ink at full strength
/// whatever else shows (P.2 item 4, Q.1 (4)). A neighbour that is not built is named at 40 % and is not a link (N.2 item 1).
fn frame(s: &Gallery, i: usize, w: f32, h: f32, root: El<Msg>) -> El<Msg> {
    let p = &PIECES[i];
    let op = fade_at(&s.works, s.clock);
    let narrow = w < NARROW;
    let (prev, next) = ((i + COUNT - 1) % COUNT, (i + 1) % COUNT);
    let name = |j: usize| format!("{:02} {}", j + 1, PIECES[j].title).to_uppercase();
    // A link: what it reads, and what it is called (the full name, where a phone shows the number only).
    let link = |key: String, shown: String, label: String, m: WMsg| row().key(key).height(28.0).align(AlignItems::CENTER).on_press(Msg::Works(m)).role(Role::Link).label(label).child(mono(shown, dim(i)));
    let step_link = |key: String, shown: String, label: String, j: usize| if PIECES[j].built { link(key, shown, label, WMsg::Open(j)) } else { row().key(key).height(28.0).align(AlignItems::CENTER).child(mono(shown, unbuilt())) };
    let made = format!("{} ↗", amenui::i18n::tr("works.made")).to_uppercase();
    let (prev_l, next_l) = (format!("← {}", name(prev)), format!("{} →", name(next)));
    let (prev_s, next_s) = if narrow { (format!("← {:02}", prev + 1), format!("{:02} →", next + 1)) } else { (prev_l.clone(), next_l.clone()) };
    let steps = row().key(k(p.slug, "steps")).gap(21.0).align(AlignItems::CENTER)
        .child(step_link(k(p.slug, "prev"), prev_s, prev_l, prev))
        .child(mono("·", dim(i)))
        .child(step_link(k(p.slug, "next"), next_s, next_l, next));
    let strip = if narrow { col().key(k(p.slug, "strip")) } else { row().key(k(p.slug, "strip")).gap(56.0).align(AlignItems::CENTER) }
        .opacity(op).child(link(k(p.slug, "made"), made.clone(), made, WMsg::Recipe(i))).child(steps);
    let title = serif(p.title, 30.0, 0.3, Paint::INK).key(k(p.slug, "title")).flow(format!("work/{}/title", p.slug)).heading(1);
    let small = mono(micro(s, i), dim(i)).key(k(p.slug, "micro")).flow(format!("work/{}/meta", p.slug)).opacity(op);
    let b = block(s, i, w, h);
    // On W-01 the type refuses the field map and the gusts' fronts (S.2 item 7): the gust bends the
    // stalks behind it and never moves a line of type. (The callout refuses them as a whole, `touch`.)
    let (strip, title, small) = if i == TOUCH_GRASS { (strip.respond(Response::Ignore), title.respond(Response::Ignore), small.respond(Response::Ignore)) } else { (strip, title, small) };
    let number = |paint: Paint| { let n = mono(p.n, paint).key(k(p.slug, "n")); if i == TOUCH_GRASS { n.respond(Response::Ignore) } else { n } };
    // The number where it reads; where it does not, nothing in its place and nothing else moves.
    let root = match number_paint(s, i, w, h) { Some(paint) => root.child(at(number(paint), 32.0, b.n)), None => root };
    if narrow {
        // On a phone, from the foot up: the strip's two lines (the recipe, then the steps by number),
        // the small line wrapped to the width, the title, the number. Every place is a whole pixel: type
        // over a picture at a fraction of one would be read as the picture's colours at its edges.
        return root.child(at(title, 32.0, b.title)).child(at(small.width(w - 64.0), 32.0, b.small))
            .child(at(strip, 32.0, h - 100.0));
    }
    root.child(at(title, 32.0, b.title))
        .child(at(small, 32.0, b.small))
        .child(at_right(strip, 32.0, h - 64.0))
}
/// W-01's text boxes as laid out at this size (R.2 item 4), with `small` as the small line: the number
/// (where it is drawn), the title and the small line (on a phone each of its lines, among the second
/// set, S.2 item 2); then the strip's links and the status line. Mono at 11 px advances `ADV` a character (a CJK one 11.66) in a box 18 px high; the
/// title's box is 272 × 48 (the serif at 30 px, measured on renders); the status line is the shell's,
/// right-aligned in the bottom 28 px with the margin, taken at the width it has on renders (240 px).
pub(crate) fn touch_text_boxes(locale: amenui::i18n::Locale, w: f32, h: f32, small: &str) -> (Vec<[f32; 4]>, Vec<[f32; 4]>) {
    let i = TOUCH_GRASS;
    let narrow = w < NARROW;
    let adv = |t: &str| t.chars().map(|c| if (c as u32) >= 0x2e80 { 11.66 } else { ADV }).sum::<f32>();
    let line = |t: &str, x: f32, y: f32| [x, y, x + adv(t), y + 18.0];
    let b = block_with(i, w, h, small);
    let (mut types, mut heads) = (vec![], vec![]);
    if number_paint_in(i, &Fit::new(w, h, PIECES[i].focus), &b).is_some() { types.push(line(PIECES[i].n, 32.0, b.n)); }
    types.push([32.0, b.title, 32.0 + 272.0, b.title + 48.0]);
    if narrow {
        // Wrapped to the width: each line's box (the last one as long as what is left).
        let per = ((w - 64.0) / ADV).floor().max(1.0) as usize;
        let chars: Vec<char> = small.chars().collect();
        let mut at = 0;
        let mut y = b.small;
        while at < chars.len() {
            let mut end = (at + per).min(chars.len());
            if end < chars.len() && let Some(sp) = chars[at..end].iter().rposition(|c| *c == ' ') { end = at + sp; }
            let t: String = chars[at..end].iter().collect();
            heads.push(line(t.trim_end(), 32.0, y));
            at = end; while at < chars.len() && chars[at] == ' ' { at += 1; }
            y += 18.0;
        }
    } else {
        types.push(line(small, 32.0, b.small));
    }
    let made = format!("{} ↗", locale.tr("works.made")).to_uppercase();
    let (prev, next) = ((i + COUNT - 1) % COUNT, (i + 1) % COUNT);
    let name = |j: usize| format!("{:02} {}", j + 1, PIECES[j].title).to_uppercase();
    if narrow {
        // The strip's column: the recipe's line over the steps by number, 28 px each.
        let steps = adv(&format!("← {:02}", prev + 1)) + 21.0 + 8.0 + 21.0 + adv(&format!("{:02} →", next + 1));
        heads.push([32.0, h - 100.0, 32.0 + steps.max(adv(&made)), h - 44.0]);
    } else {
        let steps = adv(&format!("← {}", name(prev))) + 21.0 + 8.0 + 21.0 + adv(&format!("{} →", name(next)));
        let x1 = w - 32.0;
        let x0 = x1 - steps - 56.0 - adv(&made);
        heads.push([x0, h - 64.0, x1, h - 36.0]);
    }
    let m = if w >= 1080.0 { 32.0 } else if w >= 720.0 { 24.0 } else { 16.0 };
    heads.push([w - m - 240.0, h - 28.0, w - m, h]);
    (types, heads)
}

/// W-01's stalks as planted on its page: each one's upright extent with its head, and its head's box.
pub fn touch_extents(s: &Gallery) -> Vec<([f32; 4], Option<[f32; 4]>)> {
    let (w, h) = (s.width, s.height);
    touch::planted(s.locale, w, h, &Fit::new(w, h, PIECES[TOUCH_GRASS].focus)).iter().map(|st| st.extent()).collect()
}

/// The title block's number (P.2 item 1): in the accent where the accent measures at least 4.5 : 1
/// against the median luminance of the picture under the number at this size, otherwise in the foot's
/// ink where that does, otherwise not drawn.
fn number_paint(s: &Gallery, i: usize, w: f32, h: f32) -> Option<Paint> { number_paint_at(s, i, w, h, &block(s, i, w, h)) }
fn number_paint_at(s: &Gallery, i: usize, w: f32, h: f32, b: &Block) -> Option<Paint> { number_paint_in(i, &piece_fit(s, i, w, h), b) }
fn number_paint_in(i: usize, fit: &Fit, b: &Block) -> Option<Paint> {
    let (a, ink) = number_contrast_in(i, fit, b);
    if a >= 4.5 { Some(Paint::ACCENT) } else if ink >= 4.5 { Some(Paint::INK) } else { None }
}
/// How a piece's picture lies in a `w × h` piece (W-03's own fit; the others about their centre).
fn piece_fit(s: &Gallery, i: usize, w: f32, h: f32) -> Fit { if i == GROUND_TRUTH { fit_for(s, w, h) } else { Fit::new(w, h, PIECES[i].focus) } }
/// The accent's and the foot ink's contrast against the median luminance under the title block's number.
pub fn number_contrast(s: &Gallery, i: usize, w: f32, h: f32) -> (f32, f32) { number_contrast_at(s, i, w, h, &block(s, i, w, h)) }
fn number_contrast_at(s: &Gallery, i: usize, w: f32, h: f32, b: &Block) -> (f32, f32) { number_contrast_in(i, &piece_fit(s, i, w, h), b) }
fn number_contrast_in(i: usize, fit: &Fit, b: &Block) -> (f32, f32) {
    let r = label_ink(PIECES[i].n, 32.0, b.n, 1.0);
    let mut ls: Vec<f32> = vec![];
    let (mut y, mut x);
    y = r.1 + 0.5;
    while y < r.3 {
        x = r.0 + 0.5;
        while x < r.2 { let (qx, qy) = fit.to_picture(x, y); ls.push(lum(picture_colour(i, qx, qy))); x += 1.0; }
        y += 1.0;
    }
    ls.sort_by(f32::total_cmp);
    let m = ls.get(ls.len() / 2).copied().unwrap_or(0.0);
    let th = theme(i);
    let c = |a: f32| (a.max(m) + 0.05) / (a.min(m) + 0.05);
    (c(lum(th.accent)), c(lum(th.ink)))
}
/// A piece's picture's colour at (x, y) in px of 1440 × 900, without its grain.
fn picture_colour(i: usize, x: f32, y: f32) -> [f32; 3] {
    if i == GROUND_TRUTH { return picture_at(x, y); }
    let c = ramp(stops(i), (y / 900.0) as f64);
    [c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0]
}

/// The small line: the piece's real state. A piece not drawn yet has only its number and hour.
fn micro(s: &Gallery, i: usize) -> String {
    let p = &PIECES[i];
    match i {
        GROUND_TRUTH => format!("{} · {} · 5 RIDGES · {BIRDS} TRACKED · THE BOX FOLLOWS WHAT MOVES", p.n, p.hour),
        TOUCH_GRASS => { let (w, h) = (s.width, s.height); touch::micro(s, w, h, &Fit::new(w, h, p.focus)) }
        _ => format!("{} · {}", p.n, p.hour),
    }
}

/// How much of W-03 a view draws (N.2 item 2): its page; the index's live window (no scan label); a
/// window that is not live (no scan window, no pointer); a phone's band (bird boxes, the horizon
/// line and one spot height).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode { Page, Live, Still, Band }

/// A piece's picture with what is drawn over it, `w × h`, its own space from its top left; `view` is
/// the part of it the window shows (in that space).
fn picture_layer(s: &Gallery, i: usize, w: f32, h: f32, z: f32, mode: Mode, view: (f32, f32, f32, f32)) -> El<Msg> {
    let p = &PIECES[i];
    let fit = fit_for(s, w, h);
    // W-01 on a phone's page has its horizon lower: a picture of its own; in the index's windows its
    // still, the field at rest.
    let id = match (i, mode) {
        (TOUCH_GRASS, Mode::Page) if w < NARROW => PIC_PHONE,
        (TOUCH_GRASS, Mode::Live | Mode::Still | Mode::Band) => PIC_STILL,
        _ => PIC + i as u32,
    };
    let pic = picture(id, 600.0 / 960.0).key(k(p.slug, "picture")).markr(PIC_SLOT).respond(Response::Ignore).width(fit.pw);
    let layer = col().key(k(p.slug, "layer")).width(w).height(h).child(at(pic, fit.ox, fit.oy));
    if i == GROUND_TRUTH { ground_truth(s, layer, w, h, &fit, z, mode, view).0 } else { layer }
}

/// How the 1440 × 900 picture lies in a `w × h` piece: cover, about its centre of interest.
pub struct Fit { s: f32, pw: f32, ox: f32, oy: f32 }
impl Fit {
    pub(crate) fn new(w: f32, h: f32, focus: f32) -> Self {
        let s = (w / 1440.0).max(h / 900.0);
        let pw = 1440.0 * s;
        let ox = (w * 0.5 - focus * pw).clamp(w - pw, 0.0);
        Self { s, pw, ox, oy: (h - 900.0 * s) * 0.5 }
    }
    /// A point of the picture (px of 1440 × 900) in the piece.
    pub(crate) fn at(&self, x: f32, y: f32) -> (f32, f32) { (self.ox + x * self.s, self.oy + y * self.s) }
    /// A point of the piece in the picture.
    fn to_picture(&self, x: f32, y: f32) -> (f32, f32) { ((x - self.ox) / self.s, (y - self.oy) / self.s) }
}
/// W-03's fit: about its centre of interest; on a phone's page about its cloud, 56 % across, so the
/// cloud's box is whole (N.2 item 7).
fn fit_for(s: &Gallery, w: f32, h: f32) -> Fit {
    let mut f = Fit::new(w, h, PIECES[GROUND_TRUTH].focus);
    if matches!(s.page, Page::Work(_)) && w < NARROW { f.ox = (0.56 * w - CLOUD.0 as f32 * f.s).clamp(w - f.pw, 0.0); }
    f
}
/// Where the scan window rests with no pointer: over the second ridge, 30 % across; on a phone's
/// page at the left margin, 24 px above the title block (N.2 item 7), as it is with its number: where
/// the number is not drawn nothing else moves (P.2 item 1).
fn rest_place(s: &Gallery, w: f32, h: f32) -> (f32, f32) {
    if matches!(s.page, Page::Work(_)) && w < NARROW {
        let (sw, sh) = scan_size(w);
        let top = block(s, GROUND_TRUTH, w, h).n + INK_TOP;
        return (32.0 + sw * 0.5, top - CLEAR - BRACKET - sh * 0.5);
    }
    fit_for(s, w, h).at(450.0, 565.0)
}

/// A rectangle of four strokes.
fn rect(d: &mut Drawing, (x0, y0, x1, y1): (f32, f32, f32, f32), z: f32, paint: Paint) {
    let (x0, y0, x1, y1, w) = (px(x0, z), px(y0, z), px(x1, z), px(y1, z), 1.0 / z);
    d.line((x0, y0), (x1, y0), w, paint); d.line((x1, y0), (x1, y1), w, paint);
    d.line((x1, y1), (x0, y1), w, paint); d.line((x0, y1), (x0, y0), w, paint);
}
type R = (f32, f32, f32, f32);
fn meets(a: R, b: R) -> bool { a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3 }
fn grow(r: R, d: f32) -> R { (r.0 - d, r.1 - d, r.2 + d, r.3 + d) }
fn union(a: R, b: R) -> R { (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)) }

/// A label's ink box when its box's top left is at (x, y), in a subtree drawn at `z` (type keeps its
/// size there, so it is wider in the subtree's own units).
fn label_ink(t: &str, x: f32, y: f32, z: f32) -> R {
    let n = t.chars().count() as f32;
    let top = if z < 1.0 { 1.0 } else { INK_TOP };
    (x + INK_X / z, y + top / z, x + (n * ADV - 2.0) / z, y + (top + INK_H) / z)
}
/// Where a label goes so its ink starts at the left edge of a box whose bottom line is at `y1`, 4 px
/// below it (N.4 (5)).
fn under(x0: f32, y1: f32, z: f32) -> (f32, f32) {
    let top = if z < 1.0 { 1.0 } else { INK_TOP };
    (px(x0, z) - 0.5 / z - INK_X / z, px(y1, z) + 0.5 / z + 4.0 / z - top / z)
}

/// The notes a view keeps: each is drawn whole or not at all (N.2 item 2): its ink box at least 8 px
/// inside the window, clear of what hides notes (the scan window, its label's plate, the title block
/// and 24 px round it), and its label clear of every label already kept.
struct Notes { inside: R, hidden: Vec<R>, labels: Vec<R>, inks: Vec<R> }
impl Notes {
    fn keep(&mut self, ink: R, label: Option<R>) -> bool {
        let all = label.map_or(ink, |l| union(ink, l));
        let (a, b) = (self.inside, all);
        if b.0 < a.0 || b.1 < a.1 || b.2 > a.2 || b.3 > a.3 { return false; }
        if self.hidden.iter().any(|&r| meets(r, all)) { return false; }
        if self.labels.iter().any(|&r| meets(r, ink)) { return false; }
        if self.inks.iter().any(|&r| meets(r, ink)) { return false; }
        if let Some(l) = label { if self.labels.iter().chain(&self.inks).any(|&r| meets(r, l)) { return false; } self.labels.push(l); }
        self.inks.push(ink);
        true
    }
    /// A bird's box: kept when it is inside, whatever it meets (O.2 item 1).
    fn subject(&mut self, ink: R) -> bool {
        let a = self.inside;
        if ink.0 < a.0 || ink.1 < a.1 || ink.2 > a.2 || ink.3 > a.3 { return false; }
        self.inks.push(ink);
        true
    }
    /// A label on its own (a bird's): inside, clear of what hides notes and of every label and box kept.
    fn keep_label(&mut self, l: R) -> bool {
        let a = self.inside;
        if l.0 < a.0 || l.1 < a.1 || l.2 > a.2 || l.3 > a.3 { return false; }
        if self.hidden.iter().chain(&self.labels).chain(&self.inks).any(|&r| meets(r, l)) { return false; }
        self.labels.push(l);
        true
    }
}

/// What W-03's survey drew in a view (for the checks): the birds' boxes, the echoes', the cloud's box
/// and label, the birds' labels.
#[derive(Clone, Debug, Default)]
pub struct Drawn {
    /// Every bird's box, drawn or not, and each label's place under its bird, drawn or not.
    pub flock: Vec<R>,
    pub tags: Vec<R>,
    /// What was drawn.
    pub birds: Vec<R>,
    pub echoes: Vec<R>,
    pub cloud: Option<(R, R)>,
    pub labels: Vec<R>,
    /// The cloud label's contrast against the mean of the picture under it.
    pub cloud_contrast: f32,
    /// The scan window's rect, where the view has one.
    pub scan: Option<R>,
}
/// What W-03's survey draws on its page as the state is now.
pub fn drawn(s: &Gallery) -> Drawn {
    let (w, h) = (s.width, s.height);
    ground_truth(s, col(), w, h, &fit_for(s, w, h), 1.0, Mode::Page, (0.0, 0.0, w, h)).1
}
/// The flight's row (the leader's centre) on screen: as built, raised 40 px (60 on a phone's page), and
/// higher still where that is needed for the bottom of its lowest label to be 24 px above the top of
/// the cloud's box (O.2 item 1). A phone's band shows no cloud and keeps the row as built.
fn flight_row(fit: &Fit, w: f32, z: f32, mode: Mode, cloud_top: Option<f32>) -> f32 {
    let built = fit.at(0.0, 290.0).1;
    let Some(top) = cloud_top else { return built };
    let lift = if mode == Mode::Page && w < NARROW { 60.0 } else { 40.0 };
    // The lowest label's bottom below the row: its bird's offset, its box, the gap and the type
    // (one pixel more for the rounding of the bird's place).
    let lowest = (0..BIRDS).filter(|&b| BIRD_LABELS[b].is_some()).map(|b| {
        let (lx, ly) = under(0.0, FLOCK[b].1 + BOX.3, z);
        label_ink("BIRD", lx, ly, z).3
    }).fold(f32::MIN, f32::max) + 1.0 / z;
    (built - lift).min(top - CLEAR / z - lowest).round()
}
/// The flight in a view as it is now: how far into it, the speed (px a second) and each bird's centre.
/// Under 720 px the spacing is scaled so the seven span at most 70 % of the width (N.2 item 7).
type Flight = (f32, f32, Vec<(f32, f32)>);
fn flock_at(s: &Gallery, w: f32, fit: &Fit, z: f32, mode: Mode) -> Option<Flight> {
    let f = s.works.ground.flight?;
    let raw = FLOCK[BIRDS - 1].0 + (BOX.2 - BOX.0);
    let sx = if mode == Mode::Page && w < NARROW { (0.7 * w / raw).min(1.0) } else { 1.0 };
    // The flock's centre crosses at an even speed and is at the window's centre halfway (the rest
    // pose: all seven on screen at any width), from 40 px past the right edge to where its last
    // echo is 40 px past the left.
    let fw = FLOCK[BIRDS - 1].0 * sx;
    let reach = w * 0.5 + fw * 0.5 + BOX.2 + ECHOES[ECHOES.len() - 1].0 + 40.0;
    let speed = 2.0 * reach / FLIGHT;
    let lead = w * 0.5 + reach - f * speed - fw * 0.5;
    // The cloud's box's top, which the flight keeps above (a phone's band shows no cloud).
    let cloud_top = (mode != Mode::Band).then(|| (fit.at(CLOUD.0 as f32, CLOUD.1 as f32).1 - 15.0).round());
    let y0 = flight_row(fit, w, z, mode, cloud_top);
    Some((f, speed, FLOCK.iter().map(|&(dx, dy)| ((lead + dx * sx).round(), (y0 + dy).round())).collect()))
}
/// The three birds that carry a label.
const BIRD_LABELS: [Option<&str>; BIRDS] = [Some("BIRD 0.97"), None, None, Some("BIRD 0.94"), None, None, Some("BIRD 0.91")];

/// W-03's scan window in a view: its rect (x0, y0, x1, y1), its cells' size and how many.
struct Scan { r: R, cell: f32, n: (usize, usize) }
fn scan_rect(s: &Gallery, w: f32, h: f32, z: f32, mode: Mode, view: R) -> Scan {
    let (cw, ch) = scan_size(if mode == Mode::Page { w } else { 1440.0 });
    // Inside a window drawn smaller, the cells keep their size on screen (as the type does), and the
    // window keeps its size in the piece: fewer cells.
    let cell = CELL / z;
    let n = (((cw / cell).floor() as usize).max(1), ((ch / cell).floor() as usize).max(1));
    let (sw, sh) = (n.0 as f32 * cell, n.1 as f32 * cell);
    let (cx, cy) = s.works.ground.scan.unwrap_or_else(|| rest_place(s, w, h));
    // Whole inside the window, and on a page clear of the title block, with its corner marks.
    let m = (INSIDE + BRACKET) / z;
    let mut x0 = (cx - sw * 0.5).clamp(view.0 + m, (view.2 - m - sw).max(view.0 + m));
    let mut y0 = (cy - sh * 0.5).clamp(view.1 + m, (view.3 - m - sh).max(view.1 + m));
    if mode == Mode::Page {
        let t = grow(title_box(s, GROUND_TRUTH, w, h), CLEAR + BRACKET);
        if meets(t, (x0, y0, x0 + sw, y0 + sh)) { y0 = (t.1 - sh).max(view.1 + m); }
    }
    (x0, y0) = (x0.round(), y0.round());
    Scan { r: (x0, y0, x0 + sw, y0 + sh), cell, n }
}
/// One character of the scan window: its cell, its level of the ramp, its paint, and whether it is on
/// the pale side (mixed toward the foot's ink).
#[derive(Clone, Copy)]
struct Glyph { i: usize, j: usize, n: usize, paint: Paint, pale: bool }
/// A character reads at 3 : 1 against its cell (O.4 (1)); worked out at 3.15 against the cell's median
/// as sampled, which a render's pixels and grain move by up to about 0.05. Where even the survey ink
/// at full strength does not read, a cell's character goes to the pale side; it comes back once the
/// luminance is `HOLD` above that (P.2 item 2).
/// The target sits between two limits (Q.1 (1), (2)): the dark survey ink `#231f33` reaches it from a cell
/// luminance of 0.157 up, and the foot's ink (0.763) holds it up to 0.208. That band of 0.051 has to
/// hold `HOLD`'s 0.03; it is 0.029 at 3.30, so do not raise this past 3.28.
const READS: f32 = 3.15;
fn reads(a: f32, b: f32) -> bool { (a.max(b) + 0.05) / (a.min(b) + 0.05) >= READS }
/// The scan window's characters (O.2 item 4): worked out again only when what they depend on changes
/// (the window's place and cells, the hole, how the picture lies, the ground the paints mix from), so a
/// window at rest costs one comparison a frame. In each cell one character of the ramp by the cell's
/// luminance, in the cell's colour mixed toward the survey ink by 0.62, or by the least more that reads
/// at 3 : 1 against the median of the cell; where the survey ink cannot, toward the foot's ink by the
/// least that does (P.2 item 2). The hole's cells are left out.
fn scan_glyphs(sc: &Scan, hole: Option<(R, R)>, fit: &Fit, ink: &InkSpace, z: f32) -> std::rc::Rc<Vec<Glyph>> {
    type Key = Vec<u32>;
    thread_local! { static LAST: std::cell::RefCell<Option<(Key, std::rc::Rc<Vec<Glyph>>)>> = const { std::cell::RefCell::new(None) }; }
    let (b, p) = hole.unwrap_or_default();
    let key: Key = [sc.r.0, sc.r.1, sc.r.2, sc.r.3, sc.cell, sc.n.0 as f32, sc.n.1 as f32, hole.is_some() as u8 as f32, b.0, b.1, b.2, b.3, p.0, p.1, p.2, p.3, fit.s, fit.ox, fit.oy, z, ink.h]
        .into_iter().chain(ink.top).chain(ink.bottom).chain(ink.ink).chain(ink.hues.into_iter().flatten()).map(f32::to_bits).collect();
    // Each cell's side, kept from the last time so it holds until the luminance has crossed by `HOLD`.
    thread_local! { static SIDES: std::cell::RefCell<Vec<bool>> = const { std::cell::RefCell::new(vec![]) }; }
    if let Some(v) = LAST.with_borrow(|l| l.as_ref().filter(|l| l.0 == key).map(|l| l.1.clone())) { return v; }
    let (x0, y0) = (sc.r.0, sc.r.1);
    let mut v = Vec::with_capacity(sc.n.0 * sc.n.1);
    let full = lum(SURVEY);
    SIDES.with_borrow_mut(|sides| {
        if sides.len() != sc.n.0 * sc.n.1 { *sides = vec![false; sc.n.0 * sc.n.1]; }
        for j in 0..sc.n.1 {
            for i in 0..sc.n.0 {
                let (cx, cy) = (x0 + i as f32 * sc.cell, y0 + j as f32 * sc.cell);
                let cr = (cx, cy, cx + sc.cell, cy + sc.cell);
                if hole.is_some_and(|(b, p)| meets(b, cr) || meets(p, cr)) { continue; }
                let (qa, qb) = fit.to_picture(cx, cy);
                let (qc, qd) = fit.to_picture(cx + sc.cell, cy + sc.cell);
                let (c, median) = cell_at(qa, qb, qc, qd);
                let y = (cy + sc.cell * 0.5) * z;
                let side = &mut sides[j * sc.n.0 + i];
                *side = !reads(full, median) || *side && !reads(full, median - HOLD);
                let (to, from) = if *side { (ink.ink, 0.0) } else { (SURVEY, 0.62) };
                // The least mix that reads: the fixed one where it does, else found by halving toward
                // the full ink on the colours themselves.
                let ok = |t: f32| reads(lum(mix3(c, to, t)), median);
                let mut t = from;
                if !ok(t) {
                    let (mut lo, mut hi) = (from, 1.0);
                    for _ in 0..8 { let m = (lo + hi) * 0.5; if ok(m) { hi = m } else { lo = m } }
                    t = hi;
                }
                // Made a paint, which lies near that colour: a little more where the paint does not read.
                let paint = loop {
                    let p = if t < 1.0 { ink.paint(mix3(c, to, t), y) } else if *side { Paint::INK } else { INK_FULL };
                    if t >= 1.0 || reads(lum(ink.colour(p, y)), median) { break p; }
                    t = (t + 0.04).min(1.0);
                };
                v.push(Glyph { i, j, n: level(lum(c)), paint, pale: *side });
            }
        }
    });
    let v = std::rc::Rc::new(v);
    LAST.with_borrow_mut(|l| *l = Some((key, v.clone())));
    v
}
/// The pointer's box and its label's plate, the two parts of the scan window's hole (N.4 (2)).
fn pointer_hole(s: &Gallery, z: f32) -> Option<(R, R, String)> {
    let g = &s.works.ground;
    let (qx, qy) = g.pointer?;
    let human = format!("HUMAN? {:.2}", g.human);
    let (lx, ly) = (qx.round() + 22.0 / z, qy.round() - 16.0 / z);
    let ink = label_ink(&human, lx, ly, z);
    let plate = (ink.0 - 6.0 / z, ink.1 - 3.0 / z, ink.2 + 6.0 / z, ink.3 + 3.0 / z);
    Some(((qx - 16.0 / z, qy - 16.0 / z, qx + 16.0 / z, qy + 16.0 / z), plate, human))
}

/// W-03 `ground truth`: the survey over the ridges, the flight of birds and what tracks them. The scan
/// window and the pointer come after (`scan_and_pointer`), on a page over the frame too.
#[allow(clippy::too_many_arguments)]
fn ground_truth(s: &Gallery, layer: El<Msg>, w: f32, h: f32, fit: &Fit, z: f32, mode: Mode, view: R) -> (El<Msg>, Drawn) {
    let lw = 1.0 / z;
    let slug = PIECES[GROUND_TRUTH].slug;
    // A faint shade: the survey ink mixed toward the picture under it (N.2 item 5), by its alpha.
    let faint = |a: f32| -> Paint { if a >= 1.0 { INK_FULL } else { INK_FULL.alpha(a) } };
    let m = INSIDE / z;
    let mut hidden = vec![];
    if mode == Mode::Page { hidden.push(grow(title_box(s, GROUND_TRUTH, w, h), CLEAR)); }
    // The scan window hides the notes it meets with its corner marks, and cuts a line 8 px outside its
    // rect (O.2 item 6).
    let mut drawn = Drawn::default();
    let (mut cut, mut line_hidden, mut scan) = (None, hidden.clone(), None);
    if matches!(mode, Mode::Page | Mode::Live) {
        let r = scan_rect(s, w, h, z, mode, view).r;
        scan = Some(r);
        drawn.scan = scan;
        cut = Some(grow(r, CUT / z));
        hidden.push(grow(r, BRACKET / z));
        if let Some((_, plate, _)) = pointer_hole(s, z) { hidden.push(plate); line_hidden.push(plate); }
    }
    let mut notes = Notes { inside: (view.0 + m, view.1 + m, view.2 - m, view.3 - m), hidden, labels: vec![], inks: vec![] };
    let mut d = Drawing::new(w, h);
    let mut labels: Vec<El<Msg>> = vec![];
    let label = |labels: &mut Vec<El<Msg>>, key: &str, t: &str, paint: Paint, (x, y): (f32, f32)| labels.push(at(mono(t, paint).key(k(slug, key)), x, y));
    let band = mode == Mode::Band;
    // The one wrong call: the small pale cloud, its box at 45 % and its label at 70 % (O.2 item 3), the
    // label under the box (N.4 (5)). It is placed first: the flight keeps clear of it, not it of the
    // flight (O.2 item 1).
    let cloud = (!band).then(|| {
        let (cx, cy) = fit.at(CLOUD.0 as f32, CLOUD.1 as f32);
        let r = ((cx - 48.0).round(), (cy - 15.0).round(), (cx + 52.0).round(), (cy + 11.0).round());
        let (lx, ly) = under(r.0, r.3, z);
        (r, (lx, ly), label_ink("BIRD 0.03", lx, ly, z))
    });
    if let Some((r, (lx, ly), li)) = cloud && notes.keep(r, Some(li)) {
        rect(&mut d, r, z, faint(0.45));
        let paint = faint(0.70);
        label(&mut labels, "cloud", "BIRD 0.03", paint, (lx, ly));
        let (a, b) = (fit.to_picture(li.0, li.1), fit.to_picture(li.2, li.3));
        let under = mean_at(a.0, a.1, b.0, b.1);
        drawn.cloud_contrast = contrast(mix3(under, SURVEY, paint.alpha), under);
        drawn.cloud = Some((r, li));
    }
    // The flight: seven birds in a loose line, right to left, each in its box with three echoes.
    if let Some((f, speed, birds)) = flock_at(s, w, fit, z, mode) {
        let echoes = !reduced(s);
        // The boxes first, then the labels (clear of every box), then the echoes (clear of both).
        let mut kept = vec![];
        for (b, &(bx, by)) in birds.iter().enumerate() {
            let r = (bx + BOX.0, by + BOX.1, bx + BOX.2, by + BOX.3);
            drawn.flock.push(r);
            if BIRD_LABELS[b].is_some() { let (lx, ly) = under(r.0, r.3, z); drawn.tags.push(label_ink("BIRD 0.00", lx, ly, z)); }
            let i = notes.inside;
            if r.0 < i.0 || r.1 < i.1 || r.2 > i.2 || r.3 > i.3 { continue; }
            // A bird the scan window reaches is read as characters (P.2 item 5): its box, label and
            // echoes are notes, hidden while its box meets the window grown by 8 px; its mark stays
            // while its centre is outside the window, and inside it the bird is a cell's `#`.
            let near = scan.map(|sr| (meets(grow(sr, CUT / z), r), bx >= sr.0 && bx < sr.2 && by >= sr.1 && by < sr.3));
            if near.is_none_or(|n| !n.0) {
                // A bird and its box are the subject: absent only when the box is not 8 px inside the
                // window, never for meeting a note (O.2 item 1).
                notes.subject(r);
                drawn.birds.push(r);
                kept.push((b, bx, by, r));
                rect(&mut d, r, z, INK_FULL);
            }
            if near.is_some_and(|n| n.1) { continue; }
            // Two strokes on the pixel grid, one pixel a column (a stroke antialiased over the picture
            // would mix its colour code with the sky's).
            let span = 5.0 + (b % 3) as f32;
            let n = (span * z).round().max(1.0) as i32;
            for c in -n..=n {
                let rise = if c < 0 { 3.0 } else { 3.5 } * (c as f32 / n as f32).abs();
                let (x, y) = (px(bx + c as f32 / z, z), px(by - rise, z));
                d.line((x, y), (x, y), lw, INK_FULL);
            }
        }
        for &(b, _, _, r) in &kept {
            let Some(t) = BIRD_LABELS[b].filter(|_| !band) else { continue };
            let (lx, ly) = under(r.0, r.3, z);
            let li = label_ink(t, lx, ly, z);
            if notes.keep_label(li) { label(&mut labels, &format!("bird{b}"), t, INK_FULL, (lx, ly)); drawn.labels.push(li); }
        }
        if echoes && !band {
            for &(_, _, _, r) in &kept {
                for &(back, a) in &ECHOES {
                    if f * speed < back { break; }
                    let er = (r.0 + back, r.1, r.2 + back, r.3);
                    if notes.keep(er, None) { rect(&mut d, er, z, faint(a)); drawn.echoes.push(er); }
                }
            }
        }
    }
    // The level line at the sun's height, as built, across the window inside its margins, and its label
    // over its right end.
    let (_, yl) = fit.at(0.0, SUN.1 as f32);
    let yl = px(yl, z);
    let (xa, xb) = ((32.0f32).max(view.0 + m), (w - 32.0).min(view.2 - m));
    if !band {
        let t = "HORIZON 0.0°";
        let lx = (w - 32.0).min(view.2 - m) - (t.chars().count() as f32 * ADV) / z;
        let ly = yl - 16.0 / z;
        let li = label_ink(t, lx, ly, z);
        if notes.keep(li, None) { notes.labels.push(li); label(&mut labels, "horizon", t, faint(170.0 / 255.0), (lx, ly)); }
    }
    // The dashes: none by the title block or under the pointer's plate, and the scan window cuts the line
    // 8 px outside its rect.
    let cut = cut.filter(|c| c.1 < yl + 0.5 && yl - 0.5 < c.3);
    let mut dashes: Vec<(f32, f32)> = vec![];
    let mut x = 32.0;
    while x < w - 32.0 {
        let (a, b) = (x.max(xa), (x + 6.0).min(xb));
        x += 14.0;
        if b <= a || line_hidden.iter().any(|&r| meets(r, (a, yl - 0.5, b, yl + 0.5))) { continue; }
        match cut {
            Some(c) if a < c.2 && c.0 < b => {
                if a < c.0 { dashes.push((a, c.0)); }
                if c.2 < b { dashes.push((c.2, b)); }
            }
            _ => dashes.push((a, b)),
        }
    }
    // After the cut, a run of fewer than three dashes is not drawn, whatever ends it (P.2 item 3): a run
    // is dashes no further apart than the line's own gap.
    let mut run: Vec<(f32, f32)> = vec![];
    for dash in dashes.into_iter().map(Some).chain([None]) {
        if let (Some(&(_, end)), Some(next)) = (run.last(), dash) && next.0 - end <= 8.5 { run.push(next); continue; }
        if run.len() >= 3 { for &(a, b) in &run { d.line((a, yl), (b, yl), lw, HORIZON_INK); } }
        run.clear();
        run.extend(dash);
    }
    // Spot heights on true summits of the drawn profiles (a phone's band: the first that fits).
    let mut heights = 0;
    for (n, (x0, r, t, up)) in [(700.0, 1, "▲ 1,204", 52.0), (206.0, 0, "▲ 1,731", 40.0), (1236.0, 2, "▲ 986", 64.0)].into_iter().enumerate() {
        if band && heights > 0 { break; }
        let best = (-60..60).map(|o| x0 + o as f64).min_by(|a, b| ridge_y(*a, r).total_cmp(&ridge_y(*b, r))).unwrap_or(x0);
        let (bx, by) = fit.at(best as f32, ridge_y(best, r) as f32);
        let (lx, ly) = (bx + 6.0, by - up - 4.0);
        let lead = (px(bx, z) - 0.5, by - up, px(bx, z) + 0.5, by - 3.0);
        let li = label_ink(t, lx, ly, z);
        if !notes.keep(union(lead, li), Some(li)) { continue; }
        heights += 1;
        d.line((px(bx, z), by - 3.0), (px(bx, z), by - up), lw, faint(170.0 / 255.0));
        label(&mut labels, &format!("height{n}"), t, faint(235.0 / 255.0), (lx, ly));
    }
    let mut layer = layer.child(at(canvas(d).key(k(slug, "survey")).width(w).pass(true), 0.0, 0.0));
    for l in labels { layer = layer.child(l.pass(true)); }
    // On the index the scan window goes over the survey here; on the piece's page, over the frame too.
    (if mode == Mode::Live { layer.child(scan_and_pointer_in(s, w, h, fit, z, mode, view)) } else { layer }, drawn)
}

/// W-03's scan window over what is drawn before it, then the pointer, surveyed too (on its page).
fn scan_and_pointer(s: &Gallery, w: f32, h: f32, fit: &Fit, z: f32, mode: Mode) -> El<Msg> { scan_and_pointer_in(s, w, h, fit, z, mode, (0.0, 0.0, w, h)) }
/// The scan window (N.2 item 4): corner marks, and in each cell one character from the ramp by the
/// cell's luminance, in the cell's colour mixed 0.62 toward the survey ink, over the picture as it
/// is; its label says what is drawn. The pointer's box and its label's plate are left clear. Then the
/// pointer's box and `HUMAN?` in the accent for the ground under it.
#[allow(clippy::too_many_arguments)]
fn scan_and_pointer_in(s: &Gallery, w: f32, h: f32, fit: &Fit, z: f32, mode: Mode, view: R) -> El<Msg> {
    let g = &s.works.ground;
    let slug = PIECES[GROUND_TRUTH].slug;
    let lw = 1.0 / z;
    let ink = ink_space(s);
    let sc = scan_rect(s, w, h, z, mode, view);
    let (x0, y0, x1, y1) = sc.r;
    let hole = pointer_hole(s, z);
    let mut c = col().key(k(slug, "scan-over")).width(w).height(h).pass(true);
    let mut d = Drawing::new(w, h);
    for (cx, cy, ax, ay) in [(x0, y0, 1.0, 1.0), (x1, y0, -1.0, 1.0), (x0, y1, 1.0, -1.0), (x1, y1, -1.0, -1.0)] {
        let (bx, by) = (px(cx - 6.0 * ax / z, z), px(cy - 6.0 * ay / z, z));
        d.line((bx, by), (bx + 10.0 * ax / z, by), lw, INK_FULL);
        d.line((bx, by), (bx, by + 10.0 * ay / z), lw, INK_FULL);
    }
    // A row of cells is one line of type, its characters set a cell apart by their tracking, each in its
    // cell's paint by its pose: a few lines to lay out, and none again while their characters stay.
    let mut cells = col().key(k(slug, "scan")).width(w).height(h).pass(true);
    let glyphs = scan_glyphs(&sc, hole.as_ref().map(|(b, p, _)| (*b, *p)), fit, &ink, z);
    let (gx, gy) = (((CELL - ADV) * 0.5).round() / z, ((CELL - INK_H) * 0.5).round() / z - if z < 1.0 { 1.0 } else { INK_TOP } / z);
    // A bird whose centre is inside the window is the `#` of the cell that holds it, at full strength
    // (P.2 item 5).
    let birds: Vec<(usize, usize)> = flock_at(s, w, fit, z, mode).map_or(vec![], |(_, _, b)| b.into_iter()
        .filter(|&(bx, by)| bx >= x0 && bx < x1 && by >= y0 && by < y1)
        .map(|(bx, by)| (((bx - x0) / sc.cell) as usize, ((by - y0) / sc.cell) as usize)).collect());
    for j in 0..sc.n.1 {
        let row: Vec<&Glyph> = glyphs.iter().filter(|g| g.j == j).collect();
        if row.is_empty() { continue; }
        let mut line = vec![' '; sc.n.0];
        let mut paints = vec![Paint::INK; sc.n.0];
        for g in &row {
            let bird = birds.contains(&(g.i, j));
            line[g.i] = if bird { '#' } else { RAMP[g.n] };
            paints[g.i] = if !bird { g.paint } else if g.pale { Paint::INK } else { INK_FULL };
        }
        let line: String = line.into_iter().collect();
        let (pitch, left) = (sc.cell * z, x0 + gx);
        let t = text(line, 11.0, Paint { tone: 1.0, accent: 0.0, alpha: 1.0, hue: 0 }).font(Font::Face(amenui::text::MONO)).tracking((pitch - MONO_ADV) / 11.0)
            .pose(move |q| {
                // Each glyph by its column: its cell's paint, and on its cell's pixel column.
                let c = ((q.x - MONO_ADV * 0.5) / pitch).round().clamp(0.0, paints.len() as f32 - 1.0) as usize;
                let p = paints[c];
                Pose { tone: p.tone, accent: p.accent, alpha: p.alpha, hue: p.hue as f32, ..Pose::default() }
            });
        cells = cells.child(at(t.key(k(slug, &format!("scan/row/{j}"))).nowrap().recompose(false).pass(true), left, y0 + j as f32 * sc.cell + gy));
    }
    // The characters are one picture to assistive technology, named as the label names it, not a
    // few hundred lines of punctuation.
    let t = format!("SCAN {}×{}   {} × {} CELLS", (x1 - x0).round(), (y1 - y0).round(), sc.n.0, sc.n.1);
    c = c.child(at(cells.role(Role::Img).label(t.clone()), 0.0, 0.0));
    if mode == Mode::Page {
        c = c.child(at(mono(t, INK_FULL).key(k(slug, "scan-label")).pass(true), x0 - 6.0, y0 - 24.0));
    }
    c = c.child(at(canvas(d).key(k(slug, "scan-marks")).width(w).pass(true), 0.0, 0.0));
    if let (Some((qx, qy)), Some((b, _, human))) = (g.pointer, hole) {
        // The box in the accent for the ground under it, the two crossing by opacity (N.2 item 6).
        let k_dark = dark_at(g, s.clock);
        for (name, paint, op) in [("light", Paint::hue(1), 1.0 - k_dark), ("dark", Paint::ACCENT, k_dark)] {
            if op <= 0.0 { continue; }
            let mut d = Drawing::new(w, h);
            let (cx, cy) = (px(qx, z), px(qy, z));
            d.line((cx - 9.0 / z, cy), (cx + 9.0 / z, cy), lw, paint);
            d.line((cx, cy - 9.0 / z), (cx, cy + 9.0 / z), lw, paint);
            rect(&mut d, b, z, paint);
            let group = col().key(k(slug, &format!("pointer/{name}"))).width(w).height(h).pass(true)
                .child(at(canvas(d).key(k(slug, &format!("pointer/{name}/box"))).width(w).pass(true), 0.0, 0.0));
            // A see-through group only while the two cross: at rest the box is drawn as it is.
            c = c.child(at(if op < 1.0 { group.opacity(op) } else { group }, 0.0, 0.0));
        }
        // The label is not in the accent: the survey ink over a pale ground, the foot's ink over a dark
        // one, the two crossing by opacity (O.2 item 3).
        let k_ink = tone_at(g.label, s.clock);
        for (name, paint, op) in [("survey", INK_FULL, 1.0 - k_ink), ("ink", Paint::INK, k_ink)] {
            if op <= 0.0 { continue; }
            let t = mono(human.clone(), paint).key(k(slug, &format!("human/{name}"))).recompose(false).pass(true);
            c = c.child(at(if op < 1.0 { t.opacity(op) } else { t }, qx.round() + 22.0 / z, qy.round() - 16.0 / z));
        }
    }
    at(c, 0.0, 0.0)
}

/// `/works/` (M.3): the heading and its line, four windows in the day's order, the day's ruler. A
/// window that is not built is its colours, narrow, out of the Tab order and the keys (N.2 item 1).
fn index(s: &Gallery) -> El<Msg> {
    let w = s.width;
    let (pw, ph) = piece_size(s);
    // Window coordinates (the mock-up's) are the canvas's plus the bar.
    let y = |wy: f32| wy - BAR;
    let hh = (ph - 400.0).max(320.0);
    let top = y(208.0);
    // The scale makes the piece a whole number of pixels wide, so its drawings' strokes land on the
    // pixel grid where `px` puts them.
    let kz = zoom_of(s);
    let widths = widths(s, s.clock);
    let mut root = col().key("works/index").width(w).height(s.height).clip()
        .child(at(serif("works", 30.0, 0.3, Paint::INK).key("works/title").heading(1), 32.0, y(98.0)))
        .child(at(mono(amenui::i18n::tr("works.lead"), Paint::DIM).key("works/lead"), 32.0, y(150.0)))
        .child(at_right(mono(amenui::i18n::tr("works.keys"), Paint::DIM).key("works/keys"), 32.0, y(150.0)));
    let mut x: f32 = 32.0;
    for (i, p) in PIECES.iter().enumerate() {
        let ww = widths[i];
        let (wide, live) = (s.works.wide == i, s.works.live == Some(i) && p.built);
        // The piece as it is laid out in the window, scaled to the window's height, cropped about its
        // centre of interest.
        let x0 = (crop_x(s, i, ww, pw, ph) - ww / kz * 0.5).clamp(0.0, (pw - ww / kz).max(0.0));
        let view = (x0, 0.0, x0 + ww / kz, ph);
        let mode = if live { Mode::Live } else { Mode::Still };
        let inner = super::zoomed(picture_layer(s, i, pw, ph, kz, mode, view).rekey("works/window"), kz);
        let mut win = col().key(format!("works/w{i}")).width(ww).height(hh).clip().flow(format!("work/{}", p.slug)).child(at(inner, (-x0 * kz).round(), 0.0));
        if p.built {
            win = win.on_move(move |q| Some(Msg::Works(WMsg::Hover(i, q.map(|(qx, qy)| (x0 + qx / kz, qy / kz))))))
                .on_press(Msg::Works(WMsg::Tap(i))).role(Role::Link).label(p.title).unframed();
        }
        if live { win = win.child(at(live_frame(&format!("works/w{i}/frame"), ww, hh), 0.0, 0.0)); }
        root = root.child(at(win, x.round(), top.round()));
        let yb = top + hh + 18.0;
        let title = if !p.built { unbuilt() } else if wide { Paint::INK } else { Paint::TEXT };
        root = root.child(at(row().key(format!("works/w{i}/meta")).gap(16.0).flow(format!("work/{}/meta", p.slug))
                .child(mono(format!("{:02}", i + 1), if wide && p.built { Paint::ACCENT } else { Paint::DIM }))
                .child(mono(p.hour, if wide && p.built { Paint::INK } else { Paint::DIM })), x, yb))
            .child(at(serif(p.title, if wide { 22.0 } else { 16.0 }, if wide { 0.3 } else { 0.22 }, title)
                .key(format!("works/w{i}/title")).flow(format!("work/{}/title", p.slug)), x, yb + 24.0));
        if live && wide {
            root = root.child(at_right(mono(format!("LIVE · {} · OPEN →", p.live), Paint::ACCENT).key(format!("works/w{i}/live")), w - x - ww, yb));
        }
        x += ww + 16.0;
    }
    root.child(ruler(s, w, y(ph - 84.0)))
}
/// The live window's frame: 1 px in the accent on all four sides, on the pixel grid (N.2 item 8).
fn live_frame(key: &str, ww: f32, hh: f32) -> El<Msg> {
    let mut d = Drawing::new(ww, hh);
    rect(&mut d, (0.0, 0.0, ww.round() - 1.0, hh.round() - 1.0), 1.0, Paint::ACCENT);
    canvas(d).key(key).width(ww).pass(true)
}
/// A ruler's square: filled (the current one in the accent), or a 1 px outline for an unbuilt piece.
fn square(key: String, i: usize, on: bool) -> El<Msg> {
    let p = &PIECES[i];
    if !p.built {
        let mut d = Drawing::new(6.0, 6.0);
        rect(&mut d, (0.0, 0.0, 5.0, 5.0), 1.0, Paint::INK);
        return canvas(d).key(key).width(6.0).pass(true);
    }
    space(6.0, 6.0).key(key).fill(if on { Paint::ACCENT } else { Paint::INK }).on_press(Msg::Works(WMsg::Tap(i))).role(Role::Link).label(p.title).unframed()
}

/// The day's ruler: a hairline from margin to margin, a tick every hour, a square at each piece's hour
/// (the wide one's in the accent); pressing a built piece's square is pressing its window.
fn ruler(s: &Gallery, w: f32, yr: f32) -> El<Msg> {
    let len = w - 64.0;
    let mut d = Drawing::new(len, 12.0);
    d.line((0.0, 0.5), (len, 0.5), 1.0, Paint::FAINT);
    for hr in 0..=24 {
        let x = (len * hr as f32 / 24.0).clamp(0.25, len - 0.25);
        d.line((x, 0.5), (x, if hr % 6 == 0 { 8.5 } else { 4.5 }), 0.5, Paint::DIM);
    }
    let mut r = col().key("works/ruler").width(w).height(40.0).child(at(canvas(d).key("works/ruler/line").width(len).pass(true), 32.0, 0.0));
    for hr in [0, 6, 12, 18, 24] {
        let x = 32.0 + len * hr as f32 / 24.0;
        let t = mono(format!("{hr:02}"), Paint::DIM).key(format!("works/ruler/{hr}"));
        r = r.child(match hr { 0 => at(t, x, 14.0), 24 => at_right(t, 32.0, 14.0), _ => at(t, x - 7.0, 14.0) });
    }
    for (i, p) in PIECES.iter().enumerate() {
        let x = (32.0 + len * p.at / 24.0).round();
        r = r.child(at(square(format!("works/ruler/p{i}"), i, s.works.wide == i), x - 3.0, -2.5));
    }
    at(r, 0.0, yr)
}

/// Where a window of the index centres its crop, in the piece's space: about the piece's centre of
/// interest; W-01's about its mast (x 1,123 of its 1,440) when narrow and x 950 when wide, the band from
/// the mast to its foot (T.5 items 1, 2), moving between the two as the window widens.
fn crop_x(s: &Gallery, i: usize, ww: f32, pw: f32, ph: f32) -> f32 {
    if i != TOUCH_GRASS { return PIECES[i].focus * pw; }
    let total = s.width - 64.0 - 16.0 * (COUNT as f32 - 1.0);
    let narrow = ((total - WIDE) / (COUNT as f32 - 1.0)).max(0.0);
    let k = if WIDE.min(total) > narrow { ((ww - narrow) / (WIDE.min(total) - narrow)).clamp(0.0, 1.0) } else { 1.0 };
    fit_for(s, pw, ph).at(MAST + (STILL_WIDE - MAST) * k, 0.0).0
}
/// W-01's mast and the wide window's centre, in px of its 1440 × 900 picture.
const MAST: f32 = 1123.0;
const STILL_WIDE: f32 = 950.0;

/// The index on a phone: the windows are bands down the page, from x = 32 to the right gutter, 132 px
/// high (48 for an unbuilt piece); a tap widens a built band to 16 : 10 and runs it, a second opens it.
/// Beside them a hairline at x = 16 with a square at each band's centre (N.2 item 3).
fn index_phone(s: &Gallery) -> El<Msg> {
    let w = s.width;
    let bw = w - 48.0;
    let (pw, ph) = (1440.0, 900.0);
    let kz = zoom_of(s);
    let heights: [f32; COUNT] = std::array::from_fn(|i| if !PIECES[i].built { 48.0 } else if s.works.tapped == Some(i) { (bw * 10.0 / 16.0).round() } else { 132.0 });
    // A band, the gap under it, its line of number, hour and title; and the gap to the next.
    const META: f32 = 8.0 + 18.0;
    const GAP: f32 = 16.0;
    let mut bands = col().key("works/bands").width(bw).gap(GAP);
    let mut centres = [0.0; COUNT];
    let mut yb = 0.0;
    for (i, p) in PIECES.iter().enumerate() {
        let open = s.works.tapped == Some(i) && p.built;
        let bh = heights[i];
        centres[i] = yb + bh * 0.5;
        yb += bh + META + GAP;
        // The band covers its width with the picture, cropped about the horizon.
        let horizon = if i == GROUND_TRUTH { SUN.1 as f32 * kz } else { ph * kz * 0.5 };
        let y0 = (horizon - bh * 0.5).clamp(0.0, (ph * kz - bh).max(0.0)).round();
        let mode = if open { Mode::Live } else { Mode::Band };
        let inner = super::zoomed(picture_layer(s, i, pw, ph, kz, mode, (0.0, y0 / kz, pw, (y0 + bh) / kz)).rekey("works/window"), kz);
        let mut band = col().key(format!("works/w{i}")).width(bw).height(bh).clip().flow(format!("work/{}", p.slug)).child(at(inner, 0.0, -y0));
        if p.built { band = band.on_press(Msg::Works(WMsg::Tap(i))).role(Role::Link).label(p.title).unframed(); }
        if open { band = band.child(at(live_frame(&format!("works/w{i}/frame"), bw, bh), 0.0, 0.0)); }
        bands = bands.child(col().key(format!("works/b{i}")).gap(8.0).child(band)
            .child(row().key(format!("works/w{i}/meta")).height(18.0).gap(16.0).align(AlignItems::BASELINE).flow(format!("work/{}/meta", p.slug))
                .child(mono(format!("{:02}", i + 1), if open { Paint::ACCENT } else { Paint::DIM }))
                .child(mono(p.hour, if open { Paint::INK } else { Paint::DIM }))
                .child(serif(p.title, 16.0, 0.22, if !p.built { unbuilt() } else if open { Paint::INK } else { Paint::TEXT }).key(format!("works/w{i}/title")).flow(format!("work/{}/title", p.slug)))));
    }
    let len = centres[COUNT - 1] + heights[COUNT - 1] * 0.5;
    let c = col().key("works/index").fill_width().pad(16.0, 24.0).gap(16.0)
        .child(serif("works", 30.0, 0.3, Paint::INK).key("works/title").heading(1))
        .child(mono(amenui::i18n::tr("works.lead"), Paint::DIM).key("works/lead").width(w - 32.0))
        .child(row().key("works/rows").child(ruler_down(s, len, &centres)).child(bands));
    col().key("works/scroll").fill_width().fill_height().scroll().child(c)
}

/// The ruler on a phone (N.2 item 3): not a time scale. A hairline at x = 16 from the first band's top
/// to the last band's foot, and on it a 6 px square at each band's centre (the current in the accent,
/// an unbuilt piece's an outline). No ticks, no hours. Its column is 16 px, so the bands start at 32.
fn ruler_down(s: &Gallery, len: f32, centres: &[f32; COUNT]) -> El<Msg> {
    let width = 16.0;
    let mut d = Drawing::new(width, len);
    d.line((0.5, 0.0), (0.5, len), 1.0, Paint::FAINT);
    let mut r = col().key("works/ruler").width(width).height(len).child(at(canvas(d).key("works/ruler/line").width(width).pass(true), 0.0, 0.0));
    for (i, &cy) in centres.iter().enumerate() {
        let on = s.works.tapped.unwrap_or(s.works.wide) == i;
        r = r.child(at(square(format!("works/ruler/p{i}"), i, on), -3.0, (cy - 3.0).round()));
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    fn dist(a: R, b: R) -> f32 { (a.0.max(b.0) - a.2.min(b.2)).max(0.0).hypot((a.1.max(b.1) - a.3.min(b.3)).max(0.0)) }
    #[test]
    fn the_flight_keeps_its_boxes_apart() {
        // At full spacing and at a phone's (390 px: the seven within 70 % of the width).
        let raw = FLOCK[BIRDS - 1].0 + (BOX.2 - BOX.0);
        for sx in [1.0, (0.7 * 390.0 / raw).min(1.0)] {
            let boxes: Vec<R> = FLOCK.iter().map(|&(x, y)| (x * sx + BOX.0, y + BOX.1, x * sx + BOX.2, y + BOX.3)).collect();
            for i in 0..BIRDS {
                for j in 0..BIRDS {
                    if i == j { continue; }
                    if j > i { assert!(dist(boxes[i], boxes[j]) >= 12.0, "{sx}: birds {i} and {j}"); }
                    for (back, _) in ECHOES { let e = (boxes[i].0 + back, boxes[i].1, boxes[i].2 + back, boxes[i].3); assert!(!meets(grow(e, 1.0), boxes[j]), "{sx}: an echo of {i} on {j}"); }
                }
                // A bird's echoes do not touch each other.
                assert!(ECHOES.windows(2).all(|e| e[1].0 - e[0].0 > BOX.2 - BOX.0));
            }
        }
    }
    #[test]
    fn human_reads_over_the_sky_and_every_ridge() {
        // O.2 item 3: the survey ink over a pale ground, the foot's ink over a dark one; 4.5 : 1 over the
        // sky and the two near and two far ridges, 3.4 accepted over the middle ridge alone.
        let foot = theme(GROUND_TRUTH).ink;
        let fit = Fit::new(1440.0, 900.0, PIECES[GROUND_TRUTH].focus);
        let measure = |x: f32, y: f32| {
            let li = human_ink(x, y, 1.0);
            let (a, b) = (fit.to_picture(li.0, li.1), fit.to_picture(li.2, li.3));
            let under = mean_at(a.0, a.1, b.0, b.1);
            let ink = if lum(under) < LABEL_BELOW { foot } else { SURVEY };
            contrast(ink, under)
        };
        for x in [120.0, 400.0, 700.0, 1100.0] { for y in [150.0, 300.0, 420.0] { let c = measure(x, y); assert!(c >= 4.5, "sky at {x}, {y}: {c:.2}"); } }
        let mut seen = [0; 5];
        for (r, seen) in seen.iter_mut().enumerate() {
            for x in [150.0f32, 450.0, 800.0, 1200.0] {
                // The label wholly over ridge `r`: between its top and the next one's, at its band's middle.
                let (lx0, lx1) = (x + 22.0, x + 22.0 + 90.0);
                let top = (0..=9).map(|k| ridge_y((lx0 + (lx1 - lx0) * k as f32 / 9.0) as f64, r)).fold(f64::MIN, f64::max) as f32;
                let bottom = if r < 4 { (0..=9).map(|k| ridge_y((lx0 + (lx1 - lx0) * k as f32 / 9.0) as f64, r + 1)).fold(f64::MAX, f64::min) as f32 } else { 900.0 };
                if bottom - top < 14.0 { continue; }
                let ly = (top + bottom) * 0.5;
                let c = measure(x, ly + 16.0 - INK_TOP - INK_H * 0.5);
                let need = if r == 2 { 3.4 } else { 4.5 };
                assert!(c >= need, "ridge {r} at {x}: {c:.2}");
                *seen += 1;
            }
        }
        assert!(seen.iter().all(|&n| n >= 2), "{seen:?}");
    }
    #[test]
    fn a_tone_holds_its_side_until_crossed_by_the_margin() {
        let mut c = retone(UNSET, 0.30, DARK_BELOW, 0.0);
        assert_eq!(c.1, 0.0);
        for l in [0.24, 0.23, 0.226] { c = retone(c, l, DARK_BELOW, 1.0); assert_eq!(c.1, 0.0, "{l}"); }
        c = retone(c, 0.21, DARK_BELOW, 1.0);
        assert_eq!(c.1, 1.0);
        for l in [0.26, 0.275] { c = retone(c, l, DARK_BELOW, 2.0); assert_eq!(c.1, 1.0, "{l}"); }
        c = retone(c, 0.29, DARK_BELOW, 2.0);
        assert_eq!(c.1, 0.0);
    }
    #[test]
    fn the_ramp_has_no_blank() {
        assert!(!RAMP.contains(&' '));
        assert_eq!(RAMP[level(1.0)], '.');
        assert_eq!(RAMP[level(0.8)], '.');
        assert_eq!(RAMP[level(0.7)], ':');
        assert_eq!(RAMP[level(0.0)], '#');
    }
    #[test]
    fn a_paint_comes_near_a_colour_of_its_triangle() {
        let ink = InkSpace { top: [0.5, 0.55, 0.7], bottom: [0.9, 0.8, 0.6], h: 900.0, ink: [0.9, 0.9, 0.9], hues: [ACCENT_DARK, ACCENT_LIGHT, SURVEY] };
        let p = ink.paint(mix3(ink.ground(300.0), SURVEY, 0.45), 300.0);
        assert_eq!(p.hue, 2);
        assert!((p.accent - 0.45).abs() < 0.02 && p.tone < 0.02, "{p:?}");
    }
}
