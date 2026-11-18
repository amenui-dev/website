//! W-01 `touch grass` (site-design M.4, R.2): a straw field seen side on under a pale sky, 1,570 stalks
//! at 1440 × 900 in 15 rows drawn as the framework's blades (`Mark::Blade`), bent on the GPU by the field map.
//! Wind comes as gusts with stillness between them (`Cmd::Gust`, laid over the map and gone without a
//! trace); the pointer walks a path through the field that closes in 6 s (`El::map_life`). Between
//! gusts nothing moves but the countdown, once a second.
use super::works::{Fit, at, mono};
use super::{Gallery, Msg, NARROW, Page};
use amenui::draw::{Blade, Drawing, Head};
use amenui::el::{El, Font, Paint, Response, Scope, canvas, col};
use amenui::fieldmap::Gust;
use amenui::morph::{Cmd, StirAt};

/// W-01's place in the day.
pub const TOUCH_GRASS: usize = 0;
/// The seed the field and the gusts' times are drawn from (the small line says it).
const SEED: u32 = 412;
/// The horizon, a share of the picture's height down (on a phone's page 70 %).
pub const HORIZON: f32 = 0.64;
pub const HORIZON_PHONE: f32 = 0.70;
/// Rows of stalks, and the most stalks the field has (on a phone).
const ROWS: usize = 15;
const CAP: usize = 3000;
const CAP_PHONE: usize = 900;
/// Each stalk's length is its row's × (0.80 + 0.35 h), and it leans downwind at rest by 0.06 ± 0.06 rad,
/// h and the ± by hash (R.2 items 2, 3).
const LEN_FROM: f32 = 0.80;
const LEN_SPREAD: f32 = 0.35;
const STANDING: f32 = 0.06;
/// The punctuation the heads are (M.4), and the depth from which a stalk carries one.
const HEADS: [char; 8] = ['\'', '"', ';', ':', '!', 'i', 'l', ','];
const HEAD_FROM: f32 = 0.55;
/// A blade's lean at most, and the gusts' strength: 0.79 leans it 57°, a strong one 72°.
const MAX_LEAN: f32 = 72.0;
const NORMAL: f32 = 0.79;
const STRONG: f32 = 1.0;
/// The band across the field (M.4): 330 px wide at 240 px/s, the near rows 500 px behind the far ones
/// along a power of 0.53 of the depth (the rows are at the depth's 1.9th power).
const BAND: f32 = 330.0;
const SPEED: f32 = 240.0;
const SLANT: f32 = 500.0;
const CURVE: f32 = 0.53;
/// How long the path takes to close (s), and the first gust after the piece opens (s).
pub const CLOSES: f32 = 6.0;
const FIRST: f32 = 6.0;
/// A pointer faster than this (px a second) pushes a small gust ahead of itself, at most once a second.
const FAST: f32 = 1400.0;
/// The piece's frame time, s: what the pointer stirs goes in at most this often.
const PACE: f32 = 1.0 / 30.0;

/// The colours (M.4, R.2 item 5): the notes' ink; the stalk upright, mixed toward the ground in the far
/// rows; the stalk leaning, at full strength at every depth; the heads.
pub const INK: [f32; 3] = super::works::rgb(0x2a2b2d);
pub const STALK: [f32; 3] = super::works::rgb(0x7b6a2c);
pub const STALK_LEAN: [f32; 3] = super::works::rgb(0xfff6c8);
pub const HEAD: [f32; 3] = super::works::rgb(0x5d4f1e);
/// The ground's two ends (the picture's `#d9cc92` far, `#c2a95a` near).
const FAR: u32 = 0xd9cc92;
const NEAR: u32 = 0xc2a95a;
/// Clear ground round the type (R.2 item 4): no stalk within this of the number, the title or the
/// small line, no head within `HEAD_CLEAR` of the strip, the status line or the callout.
const TYPE_CLEAR: f32 = 24.0;
const HEAD_CLEAR: f32 = 8.0;
/// The windsock (R.2 item 7): its length, its width at the mast and at its tip, and how far below the
/// horizontal it hangs with no wind.
const SOCK: f32 = 22.0;
const SOCK_MOUTH: f32 = 8.0;
const SOCK_TIP: f32 = 4.0;
const SOCK_HANG: f32 = 55.0;

/// W-01's state: the gusts (how many so far, when the next comes, when the last one started), the
/// pointer's last place and time, the last fast push, and how many stalks the field has.
#[derive(Clone, Debug)]
pub struct Grass {
    pub gusts: u32,
    pub next: f32,
    pub last: Option<f32>,
    pub pointer: Option<(f32, f32, f32)>,
    pub pushed: f32,
    /// When the pointer last laid stalks down (the path closes 6 s after).
    pub walked: f32,
    /// The countdown's seconds as last drawn.
    pub shown: u32,
    /// When the last layout took what the pointer stirred.
    pub flushed: f32,
    /// The gust under way: its number and when it started.
    pub current: Option<(u32, f32)>,
    /// The windsock's angle below the horizontal as last drawn, whole degrees.
    pub sock: i32,
}
impl Default for Grass {
    fn default() -> Self { Self { gusts: 0, next: f32::INFINITY, last: None, pointer: None, pushed: f32::NEG_INFINITY, walked: f32::NEG_INFINITY, shown: 0, flushed: f32::NEG_INFINITY, current: None, sock: SOCK_HANG as i32 } }
}

fn hash(a: u32, b: u32, c: u32) -> u32 {
    let mut h = a.wrapping_mul(0x9e37_79b9) ^ b.wrapping_mul(0x85eb_ca6b) ^ c.wrapping_mul(0xc2b2_ae35) ^ SEED.wrapping_mul(0x27d4_eb2f);
    h ^= h >> 15; h = h.wrapping_mul(0x2c1b_3c6d); h ^= h >> 12; h = h.wrapping_mul(0x297a_2d39); h ^= h >> 15;
    h
}
fn unit(h: u32) -> f32 { (h >> 8) as f32 / (1u32 << 24) as f32 }

/// The time between gust `n` and the next: 9 to 14 s, by hash of the seed.
fn interval(n: u32) -> f32 { 9.0 + 5.0 * unit(hash(n, 7, 0x51)) }
/// How long a gust takes from the field's left edge until its band and wake have left the near row (s).
fn crossing(w: f32) -> f32 { (40.0 + w + SLANT + BAND + 0.5 * SPEED) / SPEED }
/// Every fourth gust is strong.
fn strong(n: u32) -> bool { n % 4 == 3 }

/// The horizon on screen in a piece `w × h` laid out by `fit`.
pub fn horizon(fit: &Fit, w: f32) -> f32 { fit.at(0.0, 900.0 * if w < NARROW { HORIZON_PHONE } else { HORIZON }).1 }

/// One stalk: its root, length, depth (0 far, 1 near), lean at rest and head.
pub(crate) struct Stalk { x: f32, y: f32, len: f32, t: f32, lean: f32, head: Option<(char, f32)>, width: f32 }
impl Stalk {
    /// Its upright extent, root to tip with its head's box (x0, y0, x1, y1), and the head's box alone;
    /// widened downwind by what its lean at rest moves the tip.
    pub(crate) fn extent(&self) -> ([f32; 4], Option<[f32; 4]>) {
        let tip = self.y - self.len;
        let drift = self.len * self.lean.sin();
        let head = self.head.map(|(_, size)| [self.x - 0.35 * size, tip - 0.8 * size, self.x + 0.35 * size + drift, tip + 1.0]);
        let w = self.width * 0.5 + 0.5;
        let mut e = [self.x - w, tip, self.x + w + drift, self.y];
        if let Some(b) = head { e = [e[0].min(b[0]), e[1].min(b[1]), e[2].max(b[2]), e[3].max(b[3])]; }
        (e, head)
    }
}
/// The field for a `w × h` piece (M.4, R.2 items 1 to 3): row k at depth t = k / 14, y = horizon + 6 +
/// (field + 30) · t^1.9, the row's length 5 + 78 · t^1.5, stalks 5 + 24 · t^1.4 apart, each moved up to
/// 45 % of that by hash, one in six left out; spaced wider only where that would pass the cap. Each
/// stalk's own length and lean at rest by hash.
fn stalks(w: f32, h: f32, hz: f32) -> Vec<Stalk> {
    let field = h - hz;
    let phone = w < NARROW;
    let cap = if phone { CAP_PHONE } else { CAP };
    let count = |k: f32| -> usize { (0..ROWS).map(|r| { let t = r as f32 / (ROWS - 1) as f32; ((w / ((5.0 + 24.0 * t.powf(1.4)) * k)).ceil() as usize + 1) * 5 / 6 }).sum() };
    let mut k = 1.0;
    while count(k) > cap { k *= 1.02; }
    let mut out = vec![];
    for r in 0..ROWS {
        let t = r as f32 / (ROWS - 1) as f32;
        let y = (hz + 6.0 + (field + 30.0) * t.powf(1.9)).round();
        let row_len = 5.0 + 78.0 * t.powf(1.5);
        let sp = (5.0 + 24.0 * t.powf(1.4)) * k;
        let n = (w / sp).ceil() as u32 + 1;
        // On a phone the two nearest rows carry 16 px heads, not 22 (R.2 item 4).
        let size = if r >= ROWS - 2 { if phone { 16.0 } else { 22.0 } } else { 11.0 };
        for i in 0..n {
            let hh = hash(r as u32, i, 1);
            if hh.is_multiple_of(6) { continue; }
            let x = (i as f32 + 0.5) * sp + (unit(hash(r as u32, i, 2)) * 2.0 - 1.0) * 0.45 * sp;
            let head = (t > HEAD_FROM).then(|| (HEADS[(hash(r as u32, i, 3) % HEADS.len() as u32) as usize], size));
            let len = row_len * (LEN_FROM + LEN_SPREAD * unit(hash(r as u32, i, 4)));
            let lean = STANDING + STANDING * (unit(hash(r as u32, i, 5)) * 2.0 - 1.0);
            out.push(Stalk { x, y, len, t, lean, head, width: 0.75 + 0.75 * t });
        }
    }
    out
}
/// The stalks planted at this size: the field less those that come too near the type (R.2 item 4),
/// decided once per layout from the text boxes as laid out. The small line states the count, so its
/// box is taken at the count it ends up with.
pub(crate) fn planted(s: &Gallery, w: f32, h: f32, fit: &Fit) -> Vec<Stalk> {
    let hz = horizon(fit, w);
    let all = stalks(w, h, hz);
    let keep = |n: usize| -> Vec<bool> {
        let (types, heads) = clear_boxes(s, w, h, fit, &micro_of(n));
        all.iter().map(|st| {
            let (e, head) = st.extent();
            !types.iter().any(|b| meets(e, *b)) && !head.is_some_and(|hb| heads.iter().any(|b| meets(hb, *b)))
        }).collect()
    };
    let count = |k: &[bool]| k.iter().filter(|k| **k).count();
    let mut kept = keep(all.len());
    // Once more when the count as built is written with fewer characters.
    if thousands(count(&kept)).len() != thousands(all.len()).len() { kept = keep(count(&kept)); }
    all.into_iter().zip(kept).filter_map(|(st, k)| k.then_some(st)).collect()
}
fn meets(a: [f32; 4], b: [f32; 4]) -> bool { a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3] }
/// What no stalk may come near (the number, the title and the small line, grown by 24 px) and what no
/// head may (the strip, the status line and the callout, grown by 8).
fn clear_boxes(s: &Gallery, w: f32, h: f32, fit: &Fit, small: &str) -> (Vec<[f32; 4]>, Vec<[f32; 4]>) {
    let grow = |b: [f32; 4], d: f32| [b[0] - d, b[1] - d, b[2] + d, b[3] + d];
    let (types, mut heads) = super::works::touch_text_boxes(s, w, h, small);
    heads.extend(callout_boxes(w, horizon(fit, w)));
    (types.into_iter().map(|b| grow(b, TYPE_CLEAR)).collect(), heads.into_iter().map(|b| grow(b, HEAD_CLEAR)).collect())
}
/// How many stalks the field has at this size.
pub fn count(s: &Gallery, w: f32, h: f32, fit: &Fit) -> usize { planted(s, w, h, fit).len() }

/// The upright stalk's paint in a row whose ground is `ground`: the stalk at `alpha` over it, as one
/// colour at full strength (`tone` toward the ink, `accent` toward the stalk; the theme's ground is the
/// leaning colour). A blade's leaning paint keeps its upright paint's alpha, so the far rows' mix is
/// made in the colour and both ends of the lean are opaque (R.2 item 5).
fn upright(alpha: f32, ground: [f32; 3]) -> Paint {
    let want: [f32; 3] = std::array::from_fn(|k| alpha * STALK[k] + (1.0 - alpha) * ground[k]);
    let (g, i, hc) = (STALK_LEAN, INK, STALK);
    let mut best = (f32::INFINITY, 0.0, 1.0);
    for a in 0..=200 {
        let ac = a as f32 / 200.0;
        // For this accent the colour is linear in the tone: base + tone · dir.
        let base: [f32; 3] = std::array::from_fn(|k| (1.0 - ac) * g[k] + ac * hc[k]);
        let dir: [f32; 3] = std::array::from_fn(|k| (1.0 - ac) * (i[k] - g[k]));
        let dd: f32 = dir.iter().map(|d| d * d).sum();
        let tone = if dd > 1e-9 { ((0..3).map(|k| (want[k] - base[k]) * dir[k]).sum::<f32>() / dd).clamp(0.0, 1.0) } else { 0.0 };
        let e: f32 = (0..3).map(|k| (base[k] + tone * dir[k] - want[k]).powi(2)).sum();
        if e < best.0 { best = (e, tone, ac); }
    }
    Paint { tone: best.1, accent: best.2, alpha: 1.0, hue: 2 }
}

/// Gust `n` across a field `w` wide under a horizon at `hz`. The first starts with its band a third of
/// the way across the far row: the piece opens on its rest pose (M.4), which then moves on.
fn gust(n: u32, w: f32, hz: f32) -> Gust {
    let from = if n == 0 { w / 3.0 + BAND * 0.5 } else { -40.0 };
    Gust { from: [from, hz], dir: [1.0, 0.0], speed: SPEED, width: BAND, slant: SLANT, curve: CURVE, strength: if strong(n) { STRONG } else { NORMAL }, ..Gust::default() }
}
/// The field's box in the piece, which the gusts cross.
fn scope_box(w: f32, h: f32, hz: f32) -> [f32; 4] { [0.0, hz.floor(), w, h] }
/// The rest pose's lean at a point: the first gust held with its band a third of the way across the
/// far row (with reduced motion the map has no flow, so this is drawn as each blade's own lean).
fn rest_lean(x: f32, y: f32, w: f32, h: f32, hz: f32) -> f32 {
    let (_, f) = gust(0, w, hz).at([x, y], 0.0, scope_box(w, h, hz));
    MAX_LEAN.to_radians() * f[0].clamp(-1.0, 1.0)
}

/// The field as a drawing over the picture: every stalk a blade, its head a glyph; the mast on the
/// horizon 78 % across (its windsock is `sock`).
pub fn field(s: &Gallery, w: f32, h: f32, fit: &Fit) -> El<Msg> {
    let hz = horizon(fit, w);
    let still = s.body.reduced;
    let foot = fit.at(0.0, 900.0).1;
    let mut d = Drawing::new(w, h);
    let mut rows: Vec<(f32, Paint)> = vec![];
    for st in planted(s, w, h, fit) {
        // Far rows are mixed 55 % toward the ground; leaning, every row is the pale side at full strength.
        let paint = match rows.iter().find(|r| r.0 == st.t) { Some(r) => r.1, None => {
            let alpha = 1.0 - 0.55 * (1.0 - st.t);
            let depth = ((st.y - hz) / (foot - hz).max(1.0)).clamp(0.0, 1.0).powf(0.8);
            let p = upright(alpha, std::array::from_fn(|k| { let (a, b) = (super::works::rgb(FAR)[k], super::works::rgb(NEAR)[k]); a + (b - a) * depth }));
            rows.push((st.t, p));
            p
        } };
        let lean_paint = Paint { tone: 0.0, accent: 0.0, alpha: 1.0, hue: 2 };
        let head = st.head.map(|(ch, size)| Head { ch, size, font: Font::Face(amenui::text::MONO), paint: Paint::hue(1) });
        let lean = if still { (st.lean + rest_lean(st.x, st.y, w, h, hz)).min(MAX_LEAN.to_radians()) } else { st.lean };
        d.blade(Blade { root: (st.x, st.y), len: st.len, width: st.width, lean, max_lean: MAX_LEAN.to_radians(), paint, lean_paint, head, ..Blade::default() });
    }
    // The mast: 1 px, 46 px tall, on the pixel grid.
    let mx = (w * 0.78).round() + 0.5;
    d.line((mx, hz - 46.0), (mx, hz), 1.0, Paint::INK);
    let k = |p: &str| format!("work/touch-grass/{p}");
    canvas(d).key(k("field")).width(w).absolute(Some(0.0), Some(0.0), None, None).respond(Response::Ignore).pass(true)
}

/// The windsock's angle below the horizontal now, degrees (R.2 item 7): 55° with no wind, lifted to
/// 55° · (1 − f) while a band is over the mast, f the gust's own value at the mast's foot (0.79 in a
/// normal gust, 1 in a strong one). With reduced motion, at the held band's.
fn sock_angle(s: &Gallery, w: f32, h: f32, hz: f32) -> f32 {
    let mx = (w * 0.78).round() + 0.5;
    let at = |n: u32, t: f32| gust(n, w, hz).at([mx, hz.floor() + 0.5], t, scope_box(w, h, hz)).1[0].clamp(0.0, 1.0);
    let f = if s.body.reduced { at(0, 0.0) } else { s.works.grass.current.map_or(0.0, |(n, start)| at(n, s.clock - start)) };
    SOCK_HANG * (1.0 - f)
}
/// The windsock: 22 px from the mast's top, 8 px wide there tapering to 4, in the accent; its own small
/// drawing, redrawn only on frames the gust already draws.
fn sock(s: &Gallery, w: f32, hz: f32) -> El<Msg> {
    let mx = (w * 0.78).round() + 0.5;
    let a = (s.works.grass.sock as f32).to_radians();
    let (dir, across) = ((a.cos(), a.sin()), (-a.sin(), a.cos()));
    let mut d = Drawing::new(w, hz);
    // Eight strokes along its length, the mouth's half width to the tip's; the top one hangs from the
    // mast's top.
    let (lw, n) = (1.2f32, 8);
    let mouth = (mx + 0.5, hz - 46.0 + SOCK_MOUTH * 0.5);
    let span = |half: f32| half - lw * 0.5;
    for k in 0..n {
        let u = k as f32 / (n - 1) as f32 * 2.0 - 1.0;
        let (o0, o1) = (u * span(SOCK_MOUTH * 0.5), u * span(SOCK_TIP * 0.5));
        let p0 = (mouth.0 + dir.0 * lw * 0.5 + across.0 * o0, mouth.1 + dir.1 * lw * 0.5 + across.1 * o0);
        let l = SOCK - lw;
        let p1 = (mouth.0 + dir.0 * (lw * 0.5 + l) + across.0 * o1, mouth.1 + dir.1 * (lw * 0.5 + l) + across.1 * o1);
        d.line(p0, p1, lw, Paint::ACCENT);
    }
    canvas(d).key("work/touch-grass/sock").width(w).absolute(Some(0.0), Some(0.0), None, None).respond(Response::Ignore).pass(true)
}
/// The callout's ink boxes (the wind, the countdown and the leader), for the clear ground.
fn callout_boxes(w: f32, hz: f32) -> Vec<[f32; 4]> {
    let (lx, ly, right, mx) = callout_place(w, hz);
    let tw = 18.0 * super::works::ADV;
    let (ax, bx) = (mx + if right { 26.0 } else { -4.0 }, if right { lx - 6.0 } else { lx + tw + 6.0 });
    vec![[lx, ly, lx + tw, ly + 18.0], [lx, ly + 16.0, lx + tw, ly + 34.0], [ax.min(bx), ly + 7.0, ax.max(bx), hz - 52.0]]
}
/// Where the callout's lines go (their top left), whether it is right of the mast, and the mast's x.
fn callout_place(w: f32, hz: f32) -> (f32, f32, bool, f32) {
    let mx = (w * 0.78).round() + 0.5;
    // Up and to the right of the sock; on a phone, where that runs out of the window, to its left.
    let tw = 18.0 * super::works::ADV;
    let right = mx + 76.0 + tw <= w - 16.0;
    let (lx, ly) = if right { (mx + 76.0, hz - 84.0) } else { (mx - 76.0 - tw, hz - 84.0) };
    (lx, ly, right, mx)
}

/// The callout at the mast: the wind and the countdown to the next gust (one small redraw a second).
pub fn callout(s: &Gallery, w: f32, fit: &Fit) -> El<Msg> {
    let hz = horizon(fit, w);
    let g = &s.works.grass;
    let left = (g.next - s.clock).max(0.0).ceil() as u32;
    let wind = "WIND 284°  3.1 M/S".to_owned();
    let next = format!("GUST IN {:02}:{:02}", left / 60, left % 60);
    let (lx, ly, right, mx) = callout_place(w, hz);
    let tw = 18.0 * super::works::ADV;
    let mut d = Drawing::new(w, hz);
    let (ax, ay) = (mx + if right { 26.0 } else { -4.0 }, hz - 52.0);
    let (bx, by) = (if right { lx - 6.0 } else { lx + tw + 6.0 }, ly + 7.5);
    d.line((ax, ay), (bx - if right { 14.0 } else { -14.0 }, by), 1.0, Paint::INK);
    d.line((bx - if right { 14.0 } else { -14.0 }, by), (bx, by), 1.0, Paint::INK);
    let k = |p: &str| format!("work/touch-grass/{p}");
    col().key(k("callout")).width(w).height(hz).absolute(Some(0.0), Some(0.0), None, None).respond(Response::Ignore).pass(true)
        .child(sock(s, w, hz))
        .child(canvas(d).key(k("leader")).width(w).absolute(Some(0.0), Some(0.0), None, None).pass(true))
        .child(at(mono(wind, Paint::INK).key(k("wind")), lx.round(), ly.round()))
        .child(at(mono(next, Paint::INK).key(k("countdown")).recompose(false), lx.round(), (ly + 16.0).round()))
}

/// The small line: the piece's real state.
pub fn micro(s: &Gallery, w: f32, h: f32, fit: &Fit) -> String { micro_of(count(s, w, h, fit)) }
fn micro_of(n: usize) -> String { format!("W-01 · 12:00 · SEED {SEED:04} · {} STALKS · PATH CLOSES IN 6 S", thousands(n)) }

/// A count with a comma between thousands.
fn thousands(n: usize) -> String { let d = n.to_string(); let mut o = String::new(); for (k, c) in d.chars().enumerate() { if k > 0 && (d.len() - k).is_multiple_of(3) { o.push(','); } o.push(c); } o }

/// Entering the piece: the first gust is under way on its first frame, a third of the way across.
pub fn enter(s: &mut Gallery) { s.works.grass = Grass { next: s.clock, ..Grass::default() }; }

/// A tick on the piece's page: the next gust when it is due (none while the piece rests or is held,
/// nor with reduced motion, whose rest pose is drawn); whether the view changed (the countdown's
/// second).
pub fn tick(s: &mut Gallery, t: f32, w: f32, fit: &Fit) -> bool {
    let hz = horizon(fit, w);
    let (held, resting, still) = (s.works.held, s.works.resting, s.body.reduced);
    let g = &mut s.works.grass;
    let mut changed = false;
    if t >= g.next {
        if !held && !resting && !still {
            s.cmds.push(Cmd::Gust { gust: gust(g.gusts, w, hz), scope: Scope::Container(Some(AREA.into())) });
            g.last = Some(t);
            g.current = Some((g.gusts, t));
        }
        // The stillness between gusts is counted from when the band and its wake have left the field.
        let crossing = if g.gusts == 0 { 0.0 } else { crossing(w) };
        g.next = t + crossing + if g.gusts == 0 { FIRST } else { interval(g.gusts) };
        g.gusts += 1;
        changed = true;
    }
    let left = (g.next - t).max(0.0).ceil() as u32;
    if left != g.shown { g.shown = left; changed = true; }
    // The windsock follows the gust on the frames the gust draws, to the whole degree.
    let sock = sock_angle(s, w, s.height, hz).round() as i32;
    let g = &mut s.works.grass;
    if sock != g.sock { g.sock = sock; changed = true; }
    // What the pointer stirred goes in with a layout, at the piece's 30 frames a second at most.
    let stirred = !s.cmds.is_empty() && t - g.flushed >= PACE - 1e-3;
    if changed || stirred { g.flushed = t; }
    changed || stirred
}
/// When the piece next wants a frame: the countdown's next second (none while it rests).
pub fn wake(s: &Gallery, t: f32) -> Option<f32> {
    let g = &s.works.grass;
    if !s.cmds.is_empty() { return Some(g.flushed + PACE); }
    if g.next.is_infinite() || s.works.resting { return None; }
    let left = g.next - t;
    Some(if left <= 0.0 { t } else { t + left - (left.ceil() - 1.0) })
}

/// The pointer over the field (the piece's own space): stalks within 10 + 60 · t px of where it has
/// been lie down away from it, in a path that closes in 6 s; a fast pointer pushes a small gust ahead.
pub fn point(s: &mut Gallery, p: Option<(f32, f32)>, w: f32, fit: &Fit) {
    let now = s.clock;
    let hz = horizon(fit, w);
    let h = s.height;
    let g = &mut s.works.grass;
    let Some((x, y)) = p else { g.pointer = None; return };
    // A mouse dragging is heard twice (its move and its drag): once is enough. After a pause the path
    // starts again where the pointer is.
    if g.pointer.is_some_and(|q| q.0 == x && q.1 == y) { return; }
    let last = g.pointer.filter(|q| now - q.2 < 0.25);
    let from = last.map_or((x, y), |(a, b, _)| (a, b));
    let dt = last.map_or(1.0, |q| (now - q.2).max(1e-3));
    g.pointer = Some((x, y, now));
    if y < hz - 4.0 { return; }
    let depth = |y: f32| (((y - hz - 6.0) / (h - hz + 30.0)).clamp(0.0, 1.0)).powf(1.0 / 1.9);
    let (dx, dy) = (x - from.0, y - from.1);
    let dist = dx.hypot(dy);
    let steps = (dist / 8.0).ceil().max(1.0) as usize;
    let area = || Scope::Container(Some(AREA.into()));
    for k in 1..=steps {
        let f = k as f32 / steps as f32;
        let (px, py) = (from.0 + dx * f, from.1 + dy * f);
        let r = 10.0 + 60.0 * depth(py);
        // Each side of the pointer lies away from it.
        for side in [-1.0f32, 1.0] {
            s.cmds.push(Cmd::Stir { at: StirAt::Point([px + side * r * 0.5, py]), v: [side, 0.0], energy: 0.4, radius: r * 0.7, scope: area() });
        }
    }
    g.walked = now;
    let speed = dist / dt;
    if speed > FAST && now - g.pushed > 1.0 && dx.abs() > dy.abs() {
        g.pushed = now;
        let dir = dx.signum();
        let push = Gust { from: [x + dir * 20.0, y], dir: [dir, 0.0], speed: 480.0, width: 120.0, strength: 0.4, wake: 0.0, ..Gust::default() };
        s.cmds.push(Cmd::Gust { gust: push, scope: area() });
    }
}

/// The piece's picture, 960 × 600: the sky from `#bcd3e4` at the top to `#eeeee2` at the horizon, one
/// thin high cloud of `#f7f6ee` at 0.6, the field's ground from `#d9cc92` far to `#c2a95a` near, and the
/// piece's grain. On a phone's page the horizon is lower (`phone`).
pub fn pixels(phone: bool) -> Vec<u8> {
    let (w, h) = (960usize, 600usize);
    let hz = (if phone { HORIZON_PHONE } else { HORIZON }) * h as f32;
    let c = |v: u32| [((v >> 16) & 255) as f32, ((v >> 8) & 255) as f32, (v & 255) as f32];
    let mix = |a: [f32; 3], b: [f32; 3], t: f32| -> [f32; 3] { std::array::from_fn(|k| a[k] + (b[k] - a[k]) * t) };
    let (top, low, cloud, far, near) = (c(0xbcd3e4), c(0xeeeee2), c(0xf7f6ee), c(0xd9cc92), c(0xc2a95a));
    let mut px = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        let yf = y as f32 + 0.5;
        for x in 0..w {
            let xf = x as f32 + 0.5;
            let mut col = if yf < hz {
                let sky = mix(top, low, (yf / hz).powf(1.3));
                // The cloud: a long thin streak high on the left, frayed along its length.
                let (u, v) = ((xf - 150.0) / 170.0, (yf - 105.0) / 9.0);
                let fray = 0.65 + 0.35 * (xf * 0.031).sin() * (xf * 0.013 + 1.0).cos();
                let a = 0.6 * (-(u * u) - v * v).exp() * fray;
                let (u2, v2) = ((xf - 470.0) / 90.0, (yf - 140.0) / 5.0);
                let a2 = 0.35 * (-(u2 * u2) - v2 * v2).exp();
                mix(sky, cloud, (a + a2).min(0.6))
            } else {
                let g = mix(far, near, ((yf - hz) / (h as f32 - hz)).clamp(0.0, 1.0).powf(0.8));
                let n = streaks(xf * 1.5, (yf - hz) * 1.5, (h as f32 - hz) * 1.5);
                g.map(|v| v + n)
            };
            let n = (unit(hash(x as u32, y as u32, 9)) - 0.5) * 0.04 * 255.0;
            for v in &mut col { *v = (*v + n).clamp(0.0, 255.0); }
            px.extend([col[0].round() as u8, col[1].round() as u8, col[2].round() as u8, 255]);
        }
    }
    px
}

/// The ground's streaks (R.2 item 6) at (x, y) in 1440-wide px, y down from the horizon over a field
/// `field` deep: value noise of four octaves, ± 18 of 255, stretched along x. At depth d (0 at the
/// horizon, 1 at the foot) the first octave's period is (30 + 300 d) px wide and (3 + 22 d) px high; the
/// height follows d smoothly, so the lattice's rows are evenly spaced in ln(3 + 22 d).
fn streaks(x: f32, y: f32, field: f32) -> f32 {
    let d = (y / field).clamp(0.0, 1.0);
    let (u, v) = (x / (30.0 + 300.0 * d), field / 22.0 * ((3.0 + 22.0 * d) / 3.0).ln());
    let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
    let lattice = |i: i32, j: i32, o: u32| unit(hash(i as u32, j as u32, 20 + o)) * 2.0 - 1.0;
    let (mut sum, mut amp, mut norm) = (0.0, 1.0, 0.0);
    for o in 0..4u32 {
        let k = (1u32 << o) as f32;
        let (pu, pv) = (u * k + o as f32 * 17.3, v * k + o as f32 * 5.1);
        let (i, j) = (pu.floor(), pv.floor());
        let (fu, fv) = (smooth(pu - i), smooth(pv - j));
        let (i, j) = (i as i32, j as i32);
        let a = lattice(i, j, o) + (lattice(i + 1, j, o) - lattice(i, j, o)) * fu;
        let b = lattice(i, j + 1, o) + (lattice(i + 1, j + 1, o) - lattice(i, j + 1, o)) * fu;
        sum += amp * (a + (b - a) * fv);
        norm += amp;
        amp *= 0.5;
    }
    18.0 * sum / norm
}

/// The field's page area: the drawing under the map's 6 s lives, the scope the gusts cross.
/// It is the field from the horizon down, so a gust's lag runs from the far row (0) to the near (1);
/// the drawing inside it is the whole piece's, from its top left.
pub fn area(field: El<Msg>, w: f32, h: f32, fit: &Fit) -> El<Msg> {
    let hz = horizon(fit, w).floor();
    col().key(AREA).width(w).height(h - hz).absolute(Some(0.0), Some(hz), None, None).map_life(CLOSES, CLOSES).respond(Response::Ignore).pass(true)
        .child(field.absolute(Some(0.0), Some(-hz), None, None))
}

/// The element whose box the gusts cross and the path's lives hold in.
const AREA: &str = "work/touch-grass/area";

/// Whether a page is W-01's.
pub fn on(p: Page) -> bool { p == Page::Work(TOUCH_GRASS as u8) }
