//! Works (site-design M): `/works/`, four windows onto one day outside, and each piece at
//! `/works/<slug>/`: the picture full bleed under the bar, the title block at the bottom left and the
//! piece strip at the bottom right (M.2). `ground truth` (W-03) is drawn; the other three have their
//! frame over a still of their colours until the framework has what they are drawn with (M.7).
//!
//! Calm when idle (M.2): one piece runs at a time (the piece on its page, or the live window of the
//! index); between events a piece wants no frame; what moves by itself is paced at 30 frames a
//! second, what the hand moves is not; 60 s without input the piece rests after its current event.
use super::{BAR, Gallery, Msg, NARROW, Page, Section};
use amenui::draw::Drawing;
use amenui::el::{El, Font, Paint, Point, Response, Role, canvas, col, picture, row, space, text};
use amenui::morph::FlowSpec;
use amenui::style::{Look, Markr, Material, Theme};
use taffy::AlignItems;

pub const COUNT: usize = 4;

/// A piece: its address, its title (English in both languages), its number and hour, whether its
/// ground is light (the bar's tone), where its centre of interest is (a fraction of its width: the
/// index's crop and a narrow window's), the phrase its live window shows, and whether it is drawn.
pub struct Piece {
    pub slug: &'static str,
    pub title: &'static str,
    pub n: &'static str,
    pub hour: &'static str,
    /// The hour as a number, for the day's ruler.
    pub at: f32,
    pub light: bool,
    /// Whether the sky under the bar is light (the bar's type dark): W-03's dusk sky is, over its dark
    /// foreground.
    pub bar_light: bool,
    pub focus: f32,
    pub live: Option<&'static str>,
    pub built: bool,
}
pub const PIECES: [Piece; COUNT] = [
    Piece { slug: "touch-grass", title: "touch grass", n: "W-01", hour: "12:00", at: 12.0, light: true, bar_light: true, focus: 380.0 / 1440.0, live: Some("WIND OVER A FIELD OF 2,640 STALKS"), built: false },
    Piece { slug: "line-too-long", title: "line too long", n: "W-02", hour: "15:20", at: 15.0 + 20.0 / 60.0, light: false, bar_light: false, focus: 790.0 / 1440.0, live: None, built: false },
    Piece { slug: "ground-truth", title: "ground truth", n: "W-03", hour: "18:40", at: 18.0 + 40.0 / 60.0, light: false, bar_light: true, focus: 870.0 / 1440.0, live: None, built: true },
    Piece { slug: "nightly", title: "nightly", n: "W-04", hour: "23:10", at: 23.0 + 10.0 / 60.0, light: false, bar_light: false, focus: 650.0 / 1440.0, live: None, built: false },
];
pub const GROUND_TRUTH: usize = 2;

/// The pieces' pictures: `PIC + i` (`picture`). W-03's ridges are 960 × 600, shown scaled; the stills
/// of the others are their colours alone.
pub const PIC: u32 = 100;
/// `Markr` slots: a piece's picture as it is, and the scan window's characters.
pub const PIC_SLOT: u8 = 5;
pub const SCAN_SLOT: u8 = 6;

const fn rgb(h: u32) -> [f32; 3] { [((h >> 16) & 255) as f32 / 255.0, ((h >> 8) & 255) as f32 / 255.0, (h & 255) as f32 / 255.0] }
/// W-03's survey ink (`Paint::hue(2)` on its page and on the index).
const SURVEY: [f32; 3] = rgb(0x231f33);
/// Fainter survey ink is the survey ink mixed toward the chrome's pale ink, never see-through: over a
/// picture, ink that is partly transparent would mix its colour code with the picture's colours.
fn survey(a: f32) -> Paint { Paint { tone: 1.0, accent: a, alpha: 1.0, hue: 2 } }
/// A coordinate on the pixel grid for a 1 px line (at any whole scale), in a subtree drawn at `z` times
/// its size (the index's windows); the subtree's origin is on the grid.
fn px(v: f32, z: f32) -> f32 { ((v * z).floor() + 0.5) / z }
/// The chrome over a piece: ink, and the small type at 150 / 255 of it.
fn dim() -> Paint { Paint::INK.alpha(150.0 / 255.0) }

/// A piece's colours: the chrome's ink by its tone, its one accent, and its picture's ink as the third.
pub fn theme(i: usize) -> Theme {
    let p = &PIECES[i];
    let (ink, accent) = if p.light { (rgb(0x17181a), rgb(0xdb4d14)) } else { (rgb(0xe2e3e0), rgb(0xff6b2b)) };
    // W-03's accent is on its light sky: the pointer's box.
    let accent = if i == GROUND_TRUTH { rgb(0xdb4d14) } else { accent };
    Theme { ink, accent, third: SURVEY, ..amenui::style::DARK }
}
/// The bar's tone on a page: 0 none (the site's), 1 light, 2 dark.
pub fn tone(p: Page) -> u8 { match p { Page::Work(i) => if PIECES[i as usize].bar_light { 1 } else { 2 }, _ => 0 } }
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
}
impl Default for Works {
    fn default() -> Self {
        Self { wide: 0, from: [-1.0; COUNT], wide_at: -1.0, live: None, tapped: None, last_input: 0.0, fade: (1.0, 1.0, -1.0), resting: false, held: false, t: 0.0, ground: Ground::default() }
    }
}

/// W-03: the flight of birds (seconds into it; its first frame is the rest pose, halfway), when the
/// next one starts, the scan window (its centre, the pointer, pinned), and how human the pointer looks.
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
}
impl Default for Ground {
    fn default() -> Self { Self { flight: Some(FLIGHT * 0.5), stepped: -1.0, next: f32::INFINITY, scan: None, pointer: None, down: false, pinned: false, human: 0.62, trail: vec![], moved_at: -10.0 } }
}
/// A flight takes 16 s; one starts every 40 s.
const FLIGHT: f32 = 16.0;
const EVERY: f32 = 40.0;
const BIRDS: usize = 7;
/// The scan window, and its cells (`charr`'s 7 × 14).
const SCAN: (f32, f32) = (300.0, 190.0);
const CELL: (f32, f32) = (7.0, 14.0);
/// The chrome fades after this long without input; the piece rests after this.
const IDLE: f32 = 3.0;
const REST: f32 = 60.0;
const FADE: f32 = 0.15;
/// The index's windows widen over this, cubic in and out.
const WIDEN: f32 = 0.32;
const WIDE: f32 = 560.0;
/// Frames for what moves by itself, a second.
const PACE: f32 = 30.0;

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
}

/// Entering the index or a piece: from outside the section, everything is built new.
pub fn enter(s: &mut Gallery, p: Page) {
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
        WMsg::Hover(i, p) => {
            if p.is_some() { widen(s, i); s.works.live = Some(i); } else if s.works.live == Some(i) { s.works.live = None; }
            if i == GROUND_TRUTH { point(s, p, false); }
        }
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
            if running(s, GROUND_TRUTH) { point(s, p.map(|p| (p.x, p.y)), true); }
        }
        WMsg::Recipe(i) => s.navigate = Some(format!("docs/recipes/{}/", PIECES[i].slug)),
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
fn point(s: &mut Gallery, p: Option<(f32, f32)>, _page: bool) {
    let now = s.clock;
    let g = &mut s.works.ground;
    g.pointer = p;
    let Some((x, y)) = p else { g.trail.clear(); return };
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

/// The input that moves the pointer anywhere: the chrome comes back, a resting piece wakes.
pub fn pointer(s: &mut Gallery, _x: f32, _y: f32) -> bool { heard(s) }
/// A press, a wheel turn or a key: the same.
pub fn heard(s: &mut Gallery) -> bool {
    if s.page.section() != Section::Works { return false; }
    let was = s.works.resting || s.works.fade.1 < 1.0;
    s.works.last_input = s.clock;
    was
}

/// Keys no control used. The index: ← → move the wide window, Enter opens it. A piece: ← → the
/// previous and next piece, Esc the index, Space holds and releases the picture.
pub fn key(s: &mut Gallery, k: amenui::el::Key) -> bool {
    use amenui::el::Key;
    let n = COUNT;
    s.works.last_input = s.clock;
    match (s.page, k) {
        (Page::Works, Key::Left) => { let i = (s.works.wide + n - 1) % n; widen(s, i); s.works.live = Some(i); true }
        (Page::Works, Key::Right) => { let i = (s.works.wide + 1) % n; widen(s, i); s.works.live = Some(i); true }
        (Page::Works, Key::Enter) => { go(s, Page::Work(s.works.wide as u8)); true }
        (Page::Work(i), Key::Left) => { go(s, Page::Work(((i as usize + n - 1) % n) as u8)); true }
        (Page::Work(i), Key::Right) => { go(s, Page::Work(((i as usize + 1) % n) as u8)); true }
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
    changed
}
fn ground_tick(s: &mut Gallery, t: f32, dt: f32) -> bool {
    let (w, h) = piece_size(s);
    let rest = rest_place(w, h);
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
    // The scan window: on a spring 120 ms behind the pointer, pinned where a press left it, over the
    // second ridge with no pointer.
    let target = if g.pinned { g.scan.unwrap_or(rest) } else { g.pointer.unwrap_or(rest) };
    let at = g.scan.unwrap_or(rest);
    let next = if still { target } else {
        let k = 1.0 - (-dt / 0.12).exp();
        let n = (at.0 + (target.0 - at.0) * k, at.1 + (target.1 - at.1) * k);
        if (n.0 - target.0).hypot(n.1 - target.1) < 0.3 { target } else { n }
    };
    if g.scan != Some(next) { g.scan = Some(next); changed = true; }
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
    if running(s, GROUND_TRUTH) && !reduced(s) {
        let g = &w.ground;
        if g.flight.is_some() { if !w.held { want(paced); } } else if !w.held && !w.resting { want(g.next); }
        let (pw, ph) = piece_size(s);
        let target = if g.pinned { g.scan } else { Some(g.pointer.unwrap_or(rest_place(pw, ph))) };
        if g.scan.is_none() || target.is_some_and(|p| Some(p) != g.scan) { want(t); }
        if g.pointer.is_some() && (g.human - 0.5).abs() > 0.004 { want(paced.max(g.moved_at + 0.25)); }
    }
    at
}

// ── Look ───────────────────────────────────────────────────────────────────────────────────────────

pub fn look(s: &Gallery, l: &mut Look) {
    if s.page.section() != Section::Works { return; }
    l.pics[PIC_SLOT as usize] = Markr { pitch: 1.0, ..Markr::default() };
    // The scan window: the picture as characters, each in its own colour, on a ground of that colour.
    l.pics[SCAN_SLOT as usize] = Markr { pitch: CELL.0, levels: 10.0, ground: 0.78, gamma: 1.0, ..Markr::with_set(amenui::style::set::CHARS) };
    l.material = Material::Plain;
    // A piece's first frame is its rest pose, whole: nothing condenses.
    l.fx.enter = 0.0;
    match s.page {
        Page::Work(i) => {
            l.theme = theme(i as usize);
            // The finishing pass's fixed grain, and nothing else (M.2).
            l.post = amenui::post::PostSpec { grain: amenui::post::Grain { amount: 0.04, size: 1.0 }, ..amenui::post::PostSpec::default() }.look();
        }
        _ => { l.theme.third = SURVEY; }
    }
}

/// The flow between the index and a piece (M.5): the window grows to the whole window in 420 ms and
/// is not loosened on the way; the rest of the page leaves in 180 ms. With reduced motion, a 120 ms
/// opacity change.
pub fn flow(s: &Gallery, from: Page, to: Page) -> Option<FlowSpec> {
    match (from, to) {
        (Page::Works, Page::Work(_)) | (Page::Work(_), Page::Works) if reduced(s) => Some(FlowSpec { flow_ms: 0.0, flow_loose: 0.0, out_ms: 120.0 }),
        (Page::Works, Page::Work(_)) | (Page::Work(_), Page::Works) => Some(FlowSpec { flow_ms: 420.0, flow_loose: 0.0, out_ms: 180.0 }),
        _ => None,
    }
}

// ── Pictures ───────────────────────────────────────────────────────────────────────────────────────

/// Give a renderer every piece's picture (native hosts and offscreen renders; the web makes each when
/// a page first draws it).
pub fn set_pictures(r: &mut amenui::render::Renderer) {
    for i in 0..COUNT as u32 { if let Some((w, h, px)) = picture_pixels(PIC + i) { r.set_picture(PIC + i, w, h, &px); } }
}
/// The size of piece picture `id` (`PIC + i`).
pub fn picture_size(id: u32) -> Option<(u32, u32)> {
    match id.checked_sub(PIC)? as usize { GROUND_TRUTH => Some((960, 600)), i if i < COUNT => Some((240, 150)), _ => None }
}
/// Piece picture `id`, RGBA8.
pub fn picture_pixels(id: u32) -> Option<(u32, u32, Vec<u8>)> {
    let (w, h) = picture_size(id)?;
    let i = (id - PIC) as usize;
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    let put = |px: &mut Vec<u8>, c: [f64; 3]| { for v in c { px.push((v.clamp(0.0, 255.0)).round() as u8); } px.push(255); };
    if i == GROUND_TRUTH {
        // The ridges' profiles, once per column of each.
        let cols: Vec<[f64; 5]> = (0..w).map(|x| { let dx = (x as f64 + 0.5) * 1.5; std::array::from_fn(|r| ridge_y(dx, r)) }).collect();
        for y in 0..h {
            for x in 0..w { put(&mut px, ground_at((x as f64 + 0.5) * 1.5, (y as f64 + 0.5) * 1.5, &cols[x as usize])); }
        }
    } else {
        let stops: &[(f64, u32)] = match i {
            0 => &[(0.0, 0xbcd3e4), (0.64, 0xeeeee2), (0.64, 0xd9cc92), (1.0, 0xc2a95a)],
            1 => &[(0.0, 0x2b66c4), (0.55, 0x5f9ae0), (1.0, 0xa9cdee)],
            _ => &[(0.0, 0x070b16), (0.60, 0x0c1626), (0.80, 0x12233a), (0.80, 0x0a1320), (1.0, 0x070b14)],
        };
        for y in 0..h {
            let c = ramp(stops, (y as f64 + 0.5) / h as f64);
            for _ in 0..w { put(&mut px, c); }
        }
    }
    Some((w, h, px))
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
const CLOUD: (f64, f64) = (1058.0, 356.0);
fn ground_at(x: f64, y: f64, ridges: &[f64; 5]) -> [f64; 3] {
    let v = y / 900.0;
    let mut c = ramp(&[(0.0, 0x7d8fb4), (0.30, 0xc9b3b4), (0.52, 0xf2c99b), (0.66, 0xf6dcae), (1.0, 0xf6dcae)], v);
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

fn mono<M>(s: impl Into<String>, paint: Paint) -> El<M> { text(s, 11.0, paint).font(Font::Face(amenui::text::MONO)).tracking(0.06) }
fn serif<M>(s: impl Into<String>, size: f32, track: f32, paint: Paint) -> El<M> { text(s, size, paint).font(Font::Whisper).tracking(track) }
fn at<M>(e: El<M>, x: f32, y: f32) -> El<M> { e.absolute(Some(x), Some(y), None, None) }
fn at_right<M>(e: El<M>, right: f32, y: f32) -> El<M> { e.absolute(None, Some(y), Some(right), None) }
fn k(slug: &str, part: &str) -> String { format!("work/{slug}/{part}") }

pub fn page(s: &Gallery) -> El<Msg> {
    match s.page { Page::Work(i) => piece_page(s, i as usize), _ if s.width < NARROW => index_phone(s), _ => index(s) }
}

/// A piece on its page: the picture, and the frame over it.
fn piece_page(s: &Gallery, i: usize) -> El<Msg> {
    let (w, h) = (s.width, s.height);
    let p = &PIECES[i];
    let pic = picture_layer(s, i, w, h, 1.0).flow(format!("work/{}", p.slug)).on_point(|p| Some(Msg::Works(WMsg::Point(p))));
    let page = frame(s, i, w, h, col().key("work/page").width(w).height(h).clip().calm().child(pic));
    if i == GROUND_TRUTH { page.child(scan_and_pointer(s, w, h, &Fit::new(w, h, p.focus), 1.0)) } else { page }
}

/// The title block and the piece strip (M.2): the strip and the small line fade after 3 s without
/// input and return on the next; the title stays at 60 %.
fn frame(s: &Gallery, i: usize, w: f32, h: f32, root: El<Msg>) -> El<Msg> {
    let p = &PIECES[i];
    let op = fade_at(&s.works, s.clock);
    let title_op = 0.6 + 0.4 * op;
    let narrow = w < NARROW;
    let (prev, next) = ((i + COUNT - 1) % COUNT, (i + 1) % COUNT);
    let name = |j: usize| format!("{:02} {}", j + 1, PIECES[j].title).to_uppercase();
    // A link: what it reads, and what it is called (the full name, where a phone shows the number only).
    let link = |key: String, shown: String, label: String, m: WMsg| row().key(key).height(28.0).align(AlignItems::CENTER).on_press(Msg::Works(m)).role(Role::Link).label(label).child(mono(shown, dim()));
    let made = format!("{} ↗", amenui::i18n::tr("works.made")).to_uppercase();
    let (prev_l, next_l) = (format!("← {}", name(prev)), format!("{} →", name(next)));
    let (prev_s, next_s) = if narrow { (format!("← {:02}", prev + 1), format!("{:02} →", next + 1)) } else { (prev_l.clone(), next_l.clone()) };
    let steps = row().key(k(p.slug, "steps")).gap(21.0).align(AlignItems::CENTER)
        .child(link(k(p.slug, "prev"), prev_s, prev_l, WMsg::Open(prev)))
        .child(mono("·", dim()))
        .child(link(k(p.slug, "next"), next_s, next_l, WMsg::Open(next)));
    let strip = if narrow { col().key(k(p.slug, "strip")) } else { row().key(k(p.slug, "strip")).gap(56.0).align(AlignItems::CENTER) }
        .opacity(op).child(link(k(p.slug, "made"), made.clone(), made, WMsg::Recipe(i))).child(steps);
    let n = mono(p.n, Paint::ACCENT).key(k(p.slug, "n")).opacity(title_op);
    let title = serif(p.title, 30.0, 0.3, Paint::INK).key(k(p.slug, "title")).flow(format!("work/{}/title", p.slug)).heading(1).opacity(title_op);
    let small = mono(micro(s, i), dim()).key(k(p.slug, "micro")).flow(format!("work/{}/meta", p.slug)).opacity(op);
    if narrow {
        // On a phone, from the foot up: the strip's two lines (the recipe, then the steps by number),
        // the small line wrapped to the width, the title, the number. Every place is a whole pixel: type
        // over a picture at a fraction of one would be read as the picture's colours at its edges.
        let lines = (micro(s, i).chars().count() as f32 * 7.3 / (w - 64.0)).ceil().max(1.0);
        let y_m = h - 120.0 - 14.0 * lines;
        return root.child(at(n, 32.0, y_m - 76.0)).child(at(title, 32.0, y_m - 48.0)).child(at(small.width(w - 64.0), 32.0, y_m))
            .child(at(strip, 32.0, h - 100.0));
    }
    root.child(at(n, 32.0, h - 132.0))
        .child(at(title, 32.0, h - 104.0))
        .child(at(small, 32.0, h - 56.0))
        .child(at_right(strip, 32.0, h - 64.0))
}
/// The small line: the piece's real state. A piece not drawn yet has only its number and hour.
fn micro(_s: &Gallery, i: usize) -> String {
    let p = &PIECES[i];
    match i {
        GROUND_TRUTH => format!("{} · {} · 5 RIDGES · {BIRDS} TRACKED · THE BOX FOLLOWS WHAT MOVES", p.n, p.hour),
        _ => format!("{} · {}", p.n, p.hour),
    }
}

/// A piece's picture with what is drawn over it, `w × h`, its own space from its top left.
fn picture_layer(s: &Gallery, i: usize, w: f32, h: f32, z: f32) -> El<Msg> {
    let p = &PIECES[i];
    let fit = Fit::new(w, h, p.focus);
    let pic = picture(PIC + i as u32, 600.0 / 960.0).key(k(p.slug, "picture")).markr(PIC_SLOT).respond(Response::Ignore).width(fit.pw);
    let mut layer = col().key(k(p.slug, "layer")).width(w).height(h).child(at(pic, fit.ox, fit.oy));
    // What is drawn over the picture (the survey, the title block and the strip) is drawn after this
    // element in a layer of its own, where its ink replaces the picture's in the near image; in the one
    // layer it would be read as the picture's own colours. A frame has one such layer: the piece on its
    // page, the one window of the index with something drawn over it.
    if i == GROUND_TRUTH || matches!(s.page, Page::Work(_)) {
        let over = amenui::el::Stirr { energy: 0.0, radius: Some(0.0), scope: amenui::el::Scope::Off };
        layer = layer.child(at(space(0.0, 0.0).key(k(p.slug, "over")).fill(Paint::GROUND.alpha(0.0)).stirr(over).pass(true), 0.0, 0.0));
    }
    if i == GROUND_TRUTH { ground_truth(s, layer, w, h, &fit, z) } else { layer }
}

/// How the 1440 × 900 picture lies in a `w × h` piece: cover, about its centre of interest.
struct Fit { s: f32, pw: f32, ox: f32, oy: f32 }
impl Fit {
    fn new(w: f32, h: f32, focus: f32) -> Self {
        let s = (w / 1440.0).max(h / 900.0);
        let pw = 1440.0 * s;
        let ox = (w * 0.5 - focus * pw).clamp(w - pw, 0.0);
        Self { s, pw, ox, oy: (h - 900.0 * s) * 0.5 }
    }
    /// A point of the picture (px of 1440 × 900) in the piece.
    fn at(&self, x: f32, y: f32) -> (f32, f32) { (self.ox + x * self.s, self.oy + y * self.s) }
}
/// Where the scan window rests with no pointer: over the second ridge, 30 % across.
fn rest_place(w: f32, h: f32) -> (f32, f32) { Fit::new(w, h, PIECES[GROUND_TRUTH].focus).at(450.0, 565.0) }

/// A rectangle of four strokes.
fn rect(d: &mut Drawing, x0: f32, y0: f32, x1: f32, y1: f32, z: f32, paint: Paint) {
    let (x0, y0, x1, y1, w) = (px(x0, z), px(y0, z), px(x1, z), px(y1, z), 1.0 / z);
    d.line((x0, y0), (x1, y0), w, paint); d.line((x1, y0), (x1, y1), w, paint);
    d.line((x1, y1), (x0, y1), w, paint); d.line((x0, y1), (x0, y0), w, paint);
}

/// W-03's scan window: the same hills as the system sees them, in characters; `hole` (x0, y0, x1,
/// y1) is left out of it.
fn scan_window(s: &Gallery, w: f32, h: f32, fit: &Fit, hole: Option<(f32, f32, f32, f32)>) -> El<Msg> {
    let slug = PIECES[GROUND_TRUTH].slug;
    let (sx, sy) = s.works.ground.scan.unwrap_or_else(|| rest_place(w, h));
    let (sx, sy) = (sx - SCAN.0 * 0.5, sy - SCAN.1 * 0.5);
    let (x1, y1) = (sx + SCAN.0, sy + SCAN.1);
    // The window as up to four rectangles round the hole.
    let mut parts = vec![];
    match hole.map(|(a, b, c, d)| (a.clamp(sx, x1), b.clamp(sy, y1), c.clamp(sx, x1), d.clamp(sy, y1))) {
        Some((a, b, c, d)) if c > a && d > b => {
            parts.push((sx, sy, x1, b));
            parts.push((sx, d, x1, y1));
            parts.push((sx, b, a, d));
            parts.push((c, b, x1, d));
        }
        _ => parts.push((sx, sy, x1, y1)),
    }
    let mut c = col().key(k(slug, "scan")).width(w).height(h).pass(true);
    for (n, (a, b, cx, d)) in parts.into_iter().enumerate().filter(|(_, r)| r.2 - r.0 >= 1.0 && r.3 - r.1 >= 1.0) {
        let chars = picture(PIC + GROUND_TRUTH as u32, 600.0 / 960.0).key(k(slug, &format!("scan/{n}/picture"))).markr(SCAN_SLOT).loose(1.0).respond(Response::Ignore).width(fit.pw);
        c = c.child(at(col().key(k(slug, &format!("scan/{n}"))).width(cx - a).height(d - b).clip().pass(true).child(at(chars, fit.ox - a, fit.oy - b)), a, b));
    }
    at(c, 0.0, 0.0)
}

/// W-03 `ground truth`: the survey over the ridges, the flight of birds and what tracks them, the
/// scan window and the pointer's box.
fn ground_truth(s: &Gallery, layer: El<Msg>, w: f32, h: f32, fit: &Fit, z: f32) -> El<Msg> {
    let lw = 1.0 / z;
    let g = &s.works.ground;
    let slug = PIECES[GROUND_TRUTH].slug;
    let mut d = Drawing::new(w, h);
    let mut labels: Vec<El<Msg>> = vec![];
    let label = |labels: &mut Vec<El<Msg>>, key: &str, t: String, paint: Paint, x: f32, y: f32| labels.push(at(mono(t, paint).key(k(slug, key)), x, y));
    // The level line at the sun's height.
    let (_, yl) = fit.at(0.0, SUN.1 as f32);
    let yl = px(yl, z);
    let mut x = 32.0;
    while x < w - 32.0 { d.line((x, yl), ((x + 6.0).min(w - 32.0), yl), lw, survey(90.0 / 255.0)); x += 14.0; }
    labels.push(at_right(mono("HORIZON 0.0°", survey(170.0 / 255.0)).key(k(slug, "horizon")), 32.0, yl - 16.0));
    // Spot heights on true summits of the drawn profiles.
    for (n, (x0, r, t, up)) in [(206.0, 0, "▲ 1,731", 40.0), (700.0, 1, "▲ 1,204", 52.0), (1236.0, 2, "▲ 986", 64.0)].into_iter().enumerate() {
        let best = (-60..60).map(|o| x0 + o as f64).min_by(|a, b| ridge_y(*a, r).total_cmp(&ridge_y(*b, r))).unwrap_or(x0);
        let (bx, by) = fit.at(best as f32, ridge_y(best, r) as f32);
        if bx < -40.0 || bx > w + 40.0 { continue; }
        d.line((px(bx, z), by - 3.0), (px(bx, z), by - up), lw, survey(170.0 / 255.0));
        label(&mut labels, &format!("height{n}"), t.into(), survey(235.0 / 255.0), bx + 6.0, by - up - 4.0);
    }
    // The one wrong call: the small pale cloud.
    let (cx, cy) = fit.at(CLOUD.0 as f32, CLOUD.1 as f32);
    rect(&mut d, cx - 48.0, cy - 15.0, cx + 52.0, cy + 11.0, z, survey(140.0 / 255.0));
    label(&mut labels, "cloud", "BIRD 0.03".into(), survey(150.0 / 255.0), cx + 58.0, cy - 14.0);
    // The flight: seven birds in a loose line, right to left, each in its box with three echoes.
    if let Some(f) = g.flight {
        let span = 46.0 * (BIRDS as f32 - 1.0) + 120.0;
        let speed = (w + span + 40.0) / FLIGHT;
        let lead = w + 40.0 - f * speed;
        let (_, y0) = fit.at(0.0, 250.0);
        let echoes = !reduced(s);
        let jit = |n: i64| h01(n) as f32 - 0.5;
        for b in 0..BIRDS {
            let bx = lead + 46.0 * b as f32 + 20.0 * jit(2 * b as i64 + 1);
            let by = y0 - 17.0 * b as f32 + 24.0 * jit(2 * b as i64 + 2) + (b % 3) as f32 * 9.0;
            if bx < -60.0 || bx > w + 60.0 { continue; }
            let span = 5.0 + (b % 3) as f32;
            // Two strokes on the pixel grid, one pixel a column (a stroke antialiased over the picture
            // would mix its colour code with the sky's).
            let n = (span * z).round().max(1.0) as i32;
            for c in -n..=n {
                let rise = if c < 0 { 3.0 } else { 3.5 } * (c as f32 / n as f32).abs();
                let (x, y) = (px(bx + c as f32 / z, z), px(by - rise, z));
                d.line((x, y), (x, y), lw, survey(1.0));
            }
            if echoes {
                for e in 1..=3 {
                    if f - e as f32 * 15.0 / speed < 0.0 { break; }
                    let ex = bx + 15.0 * e as f32;
                    rect(&mut d, ex - 11.0, by - 10.0, ex + 11.0, by + 6.0, z, survey(0.36 * (1.0 - (e - 1) as f32 / 3.0)));
                }
            }
            rect(&mut d, bx - 11.0, by - 10.0, bx + 11.0, by + 6.0, z, survey(235.0 / 255.0));
            if let Some(c) = [Some("0.97"), None, None, Some("0.94"), None, None, Some("0.91")][b] {
                label(&mut labels, &format!("bird{b}"), format!("BIRD {c}"), survey(235.0 / 255.0), bx - 11.0, by + 9.0);
            }
        }
    }
    // The scan window: the same hills as the system sees them, in characters.
    let (sx, sy) = g.scan.unwrap_or_else(|| rest_place(w, h));
    let (sx, sy) = (sx - SCAN.0 * 0.5, sy - SCAN.1 * 0.5);
    for (cx, cy, ax, ay) in [(sx, sy, 1.0, 1.0), (sx + SCAN.0, sy, -1.0, 1.0), (sx, sy + SCAN.1, 1.0, -1.0), (sx + SCAN.0, sy + SCAN.1, -1.0, -1.0)] {
        let (x0, y0) = (px(cx - 6.0 * ax, z), px(cy - 6.0 * ay, z));
        d.line((x0, y0), (x0 + 10.0 * ax, y0), lw, survey(1.0));
        d.line((x0, y0), (x0, y0 + 10.0 * ay), lw, survey(1.0));
    }
    let cells = ((SCAN.0 / CELL.0).ceil(), (SCAN.1 / CELL.1).ceil());
    label(&mut labels, "scan-label", format!("SCAN {}×{}   {} × {} CELLS", SCAN.0, SCAN.1, cells.0, cells.1), survey(230.0 / 255.0), sx - 6.0, sy - 24.0);
    let mut layer = layer.child(at(canvas(d).key(k(slug, "survey")).width(w).pass(true), 0.0, 0.0));
    for l in labels { layer = layer.child(l.pass(true)); }
    // On the index the scan window goes over the survey here; on the piece's page, over the frame too.
    if matches!(s.page, Page::Work(_)) { layer } else { layer.child(scan_and_pointer(s, w, h, fit, z)) }
}

/// W-03's scan window over what is drawn before it (inside it the system's view is all there is), then
/// the pointer, surveyed too. Ink over the window's characters would be read as characters, so the
/// window leaves the pointer's box and its label out. Type set after the characters has its edges read
/// as characters too, so they come last.
fn scan_and_pointer(s: &Gallery, w: f32, h: f32, fit: &Fit, z: f32) -> El<Msg> {
    let g = &s.works.ground;
    let slug = PIECES[GROUND_TRUTH].slug;
    let lw = 1.0 / z;
    let human = format!("HUMAN? {:.2}", g.human);
    let hole = g.pointer.map(|(qx, qy)| (qx - 18.0, qy - 18.0, qx + 22.0 + 7.4 * human.chars().count() as f32 + 2.0, qy + 18.0));
    let mut c = col().key(k(slug, "scan-over")).width(w).height(h).pass(true).child(scan_window(s, w, h, fit, hole));
    if let Some((qx, qy)) = g.pointer {
        let a = Paint::ACCENT;
        let mut d = Drawing::new(w, h);
        let (cx, cy) = (px(qx, z), px(qy, z));
        d.line((cx - 9.0, cy), (cx + 9.0, cy), lw, a);
        d.line((cx, cy - 9.0), (cx, cy + 9.0), lw, a);
        rect(&mut d, qx - 16.0, qy - 16.0, qx + 16.0, qy + 16.0, z, a);
        c = c.child(at(canvas(d).key(k(slug, "pointer")).width(w).pass(true), 0.0, 0.0))
            .child(at(mono(human, a).key(k(slug, "human")).recompose(false).pass(true), qx.round() + 22.0, qy.round() - 16.0));
    }
    at(c, 0.0, 0.0)
}

/// `/works/` (M.3): the heading and its line, four windows in the day's order, the day's ruler.
fn index(s: &Gallery) -> El<Msg> {
    let w = s.width;
    let (pw, ph) = piece_size(s);
    // Window coordinates (the mock-up's) are the canvas's plus the bar.
    let y = |wy: f32| wy - BAR;
    let hh = (ph - 400.0).max(320.0);
    let top = y(208.0);
    let kz = hh / ph;
    let widths = widths(s, s.clock);
    let mut root = col().key("works/index").width(w).height(s.height).clip()
        .child(at(serif("works", 30.0, 0.3, Paint::INK).key("works/title").heading(1), 32.0, y(98.0)))
        .child(at(mono(amenui::i18n::tr("works.lead"), Paint::DIM).key("works/lead"), 32.0, y(150.0)))
        .child(at_right(mono("← →  MOVE     ENTER  OPEN     F2  CONSOLE", Paint::DIM).key("works/keys"), 32.0, y(150.0)));
    let mut x: f32 = 32.0;
    for (i, p) in PIECES.iter().enumerate() {
        let ww = widths[i];
        let (wide, live) = (s.works.wide == i, s.works.live == Some(i));
        // The piece as it is laid out in the window, scaled to the window's height, cropped about its
        // centre of interest.
        let x0 = (p.focus * pw - ww / kz * 0.5).clamp(0.0, (pw - ww / kz).max(0.0));
        let inner = super::zoomed(picture_layer(s, i, pw, ph, kz).rekey("works/window"), kz);
        let mut win = col().key(format!("works/w{i}")).width(ww).height(hh).clip().flow(format!("work/{}", p.slug))
            .on_move(move |q| Some(Msg::Works(WMsg::Hover(i, q.map(|(qx, qy)| (x0 + qx / kz, qy / kz))))))
            .on_press(Msg::Works(WMsg::Tap(i))).role(Role::Link).label(p.title)
            .child(at(inner, (-x0 * kz).round(), 0.0));
        if live { win = win.child(at(space(ww, hh).key(format!("works/w{i}/frame")).border(1.0, Paint::ACCENT).pass(true), 0.0, 0.0)); }
        root = root.child(at(win, x.round(), top.round()));
        let yb = top + hh + 18.0;
        root = root.child(at(row().key(format!("works/w{i}/meta")).gap(16.0).flow(format!("work/{}/meta", p.slug))
                .child(mono(format!("{:02}", i + 1), if wide { Paint::ACCENT } else { Paint::DIM }))
                .child(mono(p.hour, if wide { Paint::INK } else { Paint::DIM })), x, yb))
            .child(at(serif(p.title, if wide { 22.0 } else { 16.0 }, if wide { 0.3 } else { 0.22 }, if wide { Paint::INK } else { Paint::TEXT })
                .key(format!("works/w{i}/title")).flow(format!("work/{}/title", p.slug)), x, yb + 24.0));
        if live && wide {
            let t = match (p.built, p.live) { (true, Some(l)) => format!("LIVE · {l} · OPEN →"), (true, None) => "LIVE · OPEN →".into(), _ => "OPEN →".into() };
            root = root.child(at_right(mono(t, Paint::ACCENT).key(format!("works/w{i}/live")), w - x - ww, yb));
        }
        x += ww + 16.0;
    }
    root.child(ruler(s, w, y(ph - 84.0)))
}

/// The day's ruler: a hairline from margin to margin, a tick every hour, a square at each piece's hour
/// (the wide one's in the accent); pressing a square is pressing its window.
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
        let x = 32.0 + len * p.at / 24.0;
        r = r.child(at(space(6.0, 6.0).key(format!("works/ruler/p{i}")).fill(if s.works.wide == i { Paint::ACCENT } else { Paint::INK })
            .on_press(Msg::Works(WMsg::Tap(i))).role(Role::Link).label(p.title), x - 3.0, -2.5));
    }
    at(r, 0.0, yr)
}

/// The index on a phone: the windows are four bands down the page, full width; a tap widens a band to
/// 16 : 10 and runs it, a second opens it. The day's ruler runs down the left margin beside them.
fn index_phone(s: &Gallery) -> El<Msg> {
    let w = s.width;
    // The ruler's column: the hours' labels, the hairline, its ticks.
    const RULE: f32 = 36.0;
    let bw = w - 32.0 - RULE;
    let (pw, ph) = (1440.0, 900.0);
    let kz = bw / pw;
    let heights: [f32; COUNT] = std::array::from_fn(|i| if s.works.tapped == Some(i) { bw * 10.0 / 16.0 } else { 132.0 });
    // A band, the gap under it, its line of number, hour and title; and the gap to the next.
    const META: f32 = 8.0 + 18.0;
    let len = heights.iter().map(|h| h + META).sum::<f32>() + 16.0 * (COUNT as f32 - 1.0);
    let mut bands = col().key("works/bands").width(bw).gap(16.0);
    for (i, p) in PIECES.iter().enumerate() {
        let open = s.works.tapped == Some(i);
        let bh = heights[i];
        let y0 = ((ph * kz - bh) * 0.5).max(0.0);
        let inner = super::zoomed(picture_layer(s, i, pw, ph, kz).rekey("works/window"), kz);
        let mut band = col().key(format!("works/w{i}")).width(bw).height(bh).clip().flow(format!("work/{}", p.slug))
            .on_press(Msg::Works(WMsg::Tap(i))).role(Role::Link).label(p.title).child(at(inner, 0.0, (-y0).round()));
        if open { band = band.child(at(space(bw, bh).key(format!("works/w{i}/frame")).border(1.0, Paint::ACCENT).pass(true), 0.0, 0.0)); }
        bands = bands.child(col().key(format!("works/b{i}")).gap(8.0).child(band)
            .child(row().key(format!("works/w{i}/meta")).height(18.0).gap(16.0).align(AlignItems::BASELINE).flow(format!("work/{}/meta", p.slug))
                .child(mono(format!("{:02}", i + 1), if open { Paint::ACCENT } else { Paint::DIM }))
                .child(mono(p.hour, if open { Paint::INK } else { Paint::DIM }))
                .child(serif(p.title, 16.0, 0.22, if open { Paint::INK } else { Paint::TEXT }).key(format!("works/w{i}/title")).flow(format!("work/{}/title", p.slug)))));
    }
    let c = col().key("works/index").fill_width().pad(16.0, 24.0).gap(16.0)
        .child(serif("works", 30.0, 0.3, Paint::INK).key("works/title").heading(1))
        .child(mono(amenui::i18n::tr("works.lead"), Paint::DIM).key("works/lead").width(w - 32.0))
        .child(row().key("works/rows").child(ruler_down(s, RULE, len)).child(bands));
    col().key("works/scroll").fill_width().fill_height().scroll().child(c)
}

/// The day's ruler down the left margin (a phone): the hairline from the first band's top to the last
/// one's foot, a tick every hour, `00 06 12 18 24`, and a square at each piece's hour.
fn ruler_down(s: &Gallery, width: f32, len: f32) -> El<Msg> {
    let x = width - 12.0;
    let mut d = Drawing::new(width, len);
    d.line((x, 0.0), (x, len), 1.0, Paint::FAINT);
    for hr in 0..=24 {
        let y = (len * hr as f32 / 24.0).clamp(0.25, len - 0.25);
        d.line((x, y), (x + if hr % 6 == 0 { 8.0 } else { 4.0 }, y), 0.5, Paint::DIM);
    }
    let mut r = col().key("works/ruler").width(width).height(len).child(at(canvas(d).key("works/ruler/line").width(width).pass(true), 0.0, 0.0));
    for hr in [0, 6, 12, 18, 24] {
        let y = (len * hr as f32 / 24.0 - 7.0).clamp(0.0, len - 14.0);
        r = r.child(at(mono(format!("{hr:02}"), Paint::DIM).key(format!("works/ruler/{hr}")), 0.0, y));
    }
    for (i, p) in PIECES.iter().enumerate() {
        let y = len * p.at / 24.0;
        let on = s.works.tapped.unwrap_or(s.works.wide) == i;
        r = r.child(at(space(6.0, 6.0).key(format!("works/ruler/p{i}")).fill(if on { Paint::ACCENT } else { Paint::INK })
            .on_press(Msg::Works(WMsg::Tap(i))).role(Role::Link).label(p.title), x - 3.0, y - 3.0));
    }
    r
}
