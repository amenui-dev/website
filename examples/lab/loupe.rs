//! Lab · loupe: a real magnifier over a real page. This file is set in columns at 2.75 px, too small to
//! read, drawn once and kept (`El::cached`); under the glass the same lines are set again at reading
//! size from their outlines, hard-edged. The glass can show how the renderer sees the page: each
//! pixel's coverage as a grey (`el::cached_view`), or bands of the distance to the ink every 2 px. A
//! press pins the glass and its text selects; a second press frees it; drag the rim to resize it, and
//! `[` `]` change the zoom.
use std::cell::RefCell;
use std::rc::Rc;

use amenui::draw::Drawing;
use amenui::el::{El, Font, GlyphAt, Key, Paint, Point, Pose, Role, cached_view, canvas, col, text, text_lh};
use amenui::experiment::{Experiment, Params, frame, spec};
use amenui::field::{Grid, Scalar};
use amenui::gesture::Spring;
use amenui::pair::Faces;
use amenui::widgets::{Edit, Spec};

/// The page: this file, a text the web module does not carry (`texts`): until the page has given it,
/// the page is empty.
pub fn source() -> &'static str { crate::texts::get(SOURCE).unwrap_or("") }
/// Its name among the site's texts.
pub const SOURCE: &str = "lab/loupe.rs";
/// The face, Departure Mono, by its number; drawn at a quarter of its 11 px grid on the page.
pub const FACE: Font = Font::Face(amenui::text::MONO);
pub const SMALL: f32 = 2.75;
/// The page's line height, px: 1.6 of the size, as the glass's lines are at the zoom.
pub const LINE: f32 = SMALL * 1.6;
/// Departure Mono's advance, ascender and descender, em.
pub const ADV: f32 = 350.0 / 550.0;
const ASC: f32 = 1.0;
const DESC: f32 = 150.0 / 550.0;
/// The page's margin and the gap between its columns, px.
pub const PAD: f32 = 12.0;
/// The page's top, px: under the row of controls (at 10, 18 tall), 12 px clear of it (Z.7).
pub const TOP: f32 = 40.0;
pub const GAP: f32 = 8.0;
/// How far the distance bands reach, px at reading size, and their step.
const REACH: f32 = 8.0;
const BAND: f32 = 2.0;
pub const MODES: [&str; 3] = ["ink", "coverage", "distance"];
static DEFS: [(&str, Spec); 3] = [("zoom", spec(4.0, 12.0, 4.0, 0, "×")), ("radius", spec(64.0, 240.0, 1.0, 0, "px")), ("columns", spec(4.0, 10.0, 1.0, 0, ""))];
/// The glass's follow: L.2's spring.
const SPRING: Spring = Spring { stiffness: 600.0, damping: 44.0, mass: 1.0, rest_distance: 0.1, rest_speed: 2.0 };
/// How far a press may move and still be a press, and how near the rim a press takes it, px.
const SLOP: f32 = 3.0;
const RIM: f32 = 8.0;

/// One row of the page: the source line it is from (from 1), the column of its first character (from
/// 1), and its text.
pub type Row = (usize, usize, String);
/// The rows wrapped to `n` characters, made once for each `n`.
type Wrapped = Rc<RefCell<Option<(usize, Rc<Vec<Row>>)>>>;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Hold { Press { from: (f32, f32), moved: bool }, Rim }

#[derive(Clone, Debug)]
pub struct Loupe {
    pub p: Params,
    pub mode: usize,
    pub grid: bool,
    pub size: (f32, f32),
    /// The glass's centre on the stage, its speed, and where it is going.
    pub at: (f32, f32),
    pub v: (f32, f32),
    pub to: (f32, f32),
    pub moving: bool,
    pub pinned: bool,
    pub t: f32,
    /// The page's faces, for the distance bands (`Experiment::faces`).
    pub faces: Option<Faces>,
    /// Whether the hand set `radius` (the rim, the number); until then it follows the stage (Z.7).
    pub sized: bool,
    hold: Option<Hold>,
    wrapped: Wrapped,
}
impl Default for Loupe {
    fn default() -> Self {
        Self { p: Params::new(&DEFS, &[8.0, 132.0, 8.0]), mode: 0, grid: true, size: (480.0, 360.0), at: (240.0, 180.0), v: (0.0, 0.0), to: (240.0, 180.0), moving: false, pinned: false, t: 0.0, faces: None, sized: false, hold: None, wrapped: Rc::default() }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>, (f32, f32)), Mode(usize), Grid, Key(Key), Num(usize, Edit) }

/// The page laid out: each column's left edge, the characters a row holds, and the rows a column holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sheet { pub cols: usize, pub col_w: f32, pub chars: usize, pub rows: usize }

/// Distances on a grid of `w` × `h` cells to the nearest inked cell, by the exact Euclidean transform
/// (Felzenszwalb and Huttenlocher), in cells.
fn edt(ink: &[bool], w: usize, h: usize) -> Vec<f32> {
    const BIG: f32 = 1e9;
    fn pass(f: &[f32], d: &mut [f32]) {
        let n = f.len();
        let (mut v, mut z) = (vec![0usize; n], vec![0.0f32; n + 1]);
        let mut k = 0;
        z[0] = f32::NEG_INFINITY;
        z[1] = f32::INFINITY;
        for q in 1..n {
            loop {
                let p = v[k];
                let s = ((f[q] + (q * q) as f32) - (f[p] + (p * p) as f32)) / (2.0 * (q - p) as f32);
                if s <= z[k] && k > 0 { k -= 1; continue; }
                if s <= z[k] { v[0] = q; z[0] = f32::NEG_INFINITY; z[1] = f32::INFINITY; break; }
                k += 1;
                v[k] = q;
                z[k] = s;
                z[k + 1] = f32::INFINITY;
                break;
            }
        }
        k = 0;
        for (q, out) in d.iter_mut().enumerate() {
            while z[k + 1] < q as f32 { k += 1; }
            let p = v[k];
            *out = (q as f32 - p as f32).powi(2) + f[p];
        }
    }
    let mut g: Vec<f32> = ink.iter().map(|&i| if i { 0.0 } else { BIG }).collect();
    let (mut col, mut out) = (vec![0.0; h], vec![0.0; h]);
    for x in 0..w {
        for y in 0..h { col[y] = g[y * w + x]; }
        pass(&col, &mut out);
        for y in 0..h { g[y * w + x] = out[y]; }
    }
    let (mut row, mut out) = (vec![0.0; w], vec![0.0; w]);
    for y in 0..h {
        row.copy_from_slice(&g[y * w..(y + 1) * w]);
        pass(&row, &mut out);
        for x in 0..w { g[y * w + x] = out[x].sqrt(); }
    }
    g
}
/// A distance grid read at a point, nearest.
struct Dist<'a> { d: &'a [f32], w: usize, h: usize, step: f32 }
impl Scalar for Dist<'_> {
    fn at(&self, x: f32, y: f32) -> f32 {
        let (i, j) = (((x / self.step) as usize).min(self.w - 1), ((y / self.step) as usize).min(self.h - 1));
        self.d[j * self.w + i] * self.step
    }
}

impl Loupe {
    /// The page's layout on the stage.
    pub fn sheet(&self) -> Sheet {
        let cols = self.p.get(2) as usize;
        let col_w = ((self.size.0 - 2.0 * PAD - (cols - 1) as f32 * GAP) / cols as f32).max(SMALL * ADV);
        Sheet { cols, col_w, chars: ((col_w / (SMALL * ADV)).floor() as usize).max(1), rows: (((self.size.1 - TOP - PAD) / LINE).floor() as usize).max(1) }
    }
    /// The source wrapped to `n` characters a row.
    pub fn rows(&self, n: usize) -> Rc<Vec<Row>> {
        if let Some((m, r)) = self.wrapped.borrow().as_ref() && *m == n && r.len() >= self.sheet().cols * self.sheet().rows { return r.clone(); }
        let mut out = vec![];
        let text = source();
        if text.is_empty() { return Rc::new(out); }
        // The file again after itself until the page is full: every row still names its own line.
        let s = self.sheet();
        while out.len() < s.cols * s.rows { for (l, line) in text.lines().enumerate() {
            let cs: Vec<char> = line.chars().collect();
            if cs.is_empty() { out.push((l + 1, 1, String::new())); continue; }
            for (k, ch) in cs.chunks(n).enumerate() { out.push((l + 1, k * n + 1, ch.iter().collect())); }
        } }
        let r = Rc::new(out);
        *self.wrapped.borrow_mut() = Some((n, r.clone()));
        r
    }
    /// Column `c`'s left edge, px.
    fn col_x(&self, s: &Sheet, c: usize) -> f32 { PAD + c as f32 * (s.col_w + GAP) }
    /// The row under a point on the stage, and the character's index in it: (row, char).
    pub fn under(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        let s = self.sheet();
        // Between two columns it is the left one's last character; past the page, its edge.
        let c = (((x - PAD) / (s.col_w + GAP)).floor().max(0.0) as usize).min(s.cols - 1);
        let r = ((y - TOP) / LINE).floor();
        if r < 0.0 || r as usize >= s.rows { return None; }
        let i = (((x - self.col_x(&s, c)) / (SMALL * ADV)).floor().max(0.0) as usize).min(s.chars - 1);
        Some((c * s.rows + r as usize, i))
    }
    /// What the rim reads: the source line under the glass's centre and its column, `(line, col)`.
    pub fn reading(&self) -> Option<(usize, usize)> {
        let (row, i) = self.under(self.at.0, self.at.1)?;
        let rows = self.rows(self.sheet().chars);
        let (l, c0, s) = rows.get(row)?;
        Some((*l, c0 + i.min(s.chars().count().max(1) - 1)))
    }
    /// The glass's text: for every row it crosses, the characters within its radius of its centre,
    /// with where they start on the stage (x, y of the row's top) and their row.
    pub fn in_glass(&self) -> Vec<(f32, f32, usize, String)> {
        let s = self.sheet();
        let rows = self.rows(s.chars);
        let (gx, gy) = self.at;
        let r = self.p.get(1) / self.p.get(0);
        let ch = SMALL * ADV;
        let mut out = vec![];
        for c in 0..s.cols {
            let x0 = self.col_x(&s, c);
            if x0 > gx + r || x0 + s.col_w < gx - r { continue; }
            let (a, b) = ((((gy - r - TOP) / LINE).floor().max(0.0)) as usize, ((((gy + r - TOP) / LINE).ceil()).max(0.0) as usize).min(s.rows));
            for k in a..b {
                let Some((_, _, t)) = rows.get(c * s.rows + k) else { continue };
                let y = TOP + k as f32 * LINE;
                // Only the characters whose cells reach into the circle.
                let dy = if y + LINE < gy { gy - (y + LINE) } else if y > gy { y - gy } else { 0.0 };
                if dy >= r { continue; }
                let half = (r * r - dy * dy).sqrt();
                let cs: Vec<char> = t.chars().collect();
                let i0 = (((gx - half - x0) / ch).floor().max(0.0)) as usize;
                let i1 = ((((gx + half - x0) / ch).ceil()).max(0.0) as usize).min(cs.len());
                if i0 >= i1 { continue; }
                out.push((x0 + i0 as f32 * ch, y, c * s.rows + k, cs[i0..i1].iter().collect()));
            }
        }
        out
    }
    /// The smallest `radius` on a stage `w` px wide: 64 below 480, else 80 (Z.7).
    pub fn floor(w: f32) -> f32 { if w < 480.0 { 64.0 } else { 80.0 } }
    /// `radius` on this stage: the hand's, or the lesser of 132 and 30 % of the stage's width.
    fn fit_radius(&mut self) {
        let f = Self::floor(self.size.0);
        self.p.v[1] = if self.sized { self.p.v[1].max(f) } else { (132.0f32).min((0.3 * self.size.0).round()).max(f) };
    }
    /// The glass's line slots (Z.7): rows of the page under the glass, `rows` high and `cols` columns
    /// across, each holding one row of the page. How many is fixed by `radius`, `zoom` and `columns`.
    pub fn slots(&self) -> (usize, usize) {
        let s = self.sheet();
        let reach = 2.0 * self.p.get(1) / self.p.get(0);
        (((reach / LINE).ceil() as usize) + 1, ((reach / (s.col_w + GAP)).ceil() as usize + 1).min(s.cols))
    }
    /// Each slot's row of the page: where it starts on the stage (x, y of the row's top), its row (if
    /// the slot is over the page), and its text padded with spaces to the column's width. The slots go
    /// with the glass by whole rows and columns, so a slot's text changes only when its row does.
    pub fn slot_rows(&self) -> Vec<(f32, f32, Option<usize>, String)> {
        let s = self.sheet();
        let rows = self.rows(s.chars);
        let (nr, nc) = self.slots();
        let (gx, gy) = self.at;
        let reach = self.p.get(1) / self.p.get(0);
        let c0 = ((gx - reach - PAD) / (s.col_w + GAP)).floor().max(0.0) as usize;
        let c0 = c0.min(s.cols - nc);
        let r0 = ((gy - reach - TOP) / LINE).floor() as isize;
        // Slot (c mod cols, k mod rows) holds column c's row k: as the glass moves a row, one slot takes
        // the new row and the others keep theirs.
        let mut out = vec![(0.0, 0.0, None, String::new()); nr * nc];
        for c in c0..c0 + nc { for j in 0..nr as isize {
            let k = r0 + j;
            let y = TOP + k as f32 * LINE;
            let row = (k >= 0 && (k as usize) < s.rows).then(|| c * s.rows + k as usize).filter(|&i| i < rows.len());
            let t = row.map_or("", |i| rows[i].2.as_str());
            let pad = s.chars.saturating_sub(t.chars().count());
            out[(c % nc) * nr + k.rem_euclid(nr as isize) as usize] = (self.col_x(&s, c), y, row, format!("{t}{}", " ".repeat(pad)));
        } }
        out
    }
    /// The stage's size, and the glass kept on it.
    fn resize(&mut self, size: (f32, f32)) {
        if size == self.size { return; }
        let k = (size.0 / self.size.0.max(1.0), size.1 / self.size.1.max(1.0));
        self.size = size;
        self.at = (self.at.0 * k.0, self.at.1 * k.1);
        self.to = (self.to.0 * k.0, self.to.1 * k.1);
        self.fit_radius();
    }
    pub fn at_size(&self, size: (f32, f32)) -> Self {
        let mut l = self.clone();
        if l.size != size && !l.moving && l.hold.is_none() {
            // A glass never moved starts at the page's middle.
            let fresh = l.at == Loupe::default().at;
            l.resize(size);
            if fresh { l.at = (size.0 * 0.5, size.1 * 0.5); l.to = l.at; }
        } else { l.resize(size); }
        l.fit_radius();
        l
    }
    fn zoom_by(&mut self, d: f32) { let z = (self.p.get(0) + 4.0 * d).clamp(4.0, 12.0); self.p.v[0] = z; }
    /// The distance bands inside the glass, in its own px: the glass's glyphs at reading size from
    /// their outlines, the distance to their ink on a 4 px grid, traced every 2 px out to 8.
    pub fn bands(&self, text: &[(f32, f32, usize, String)]) -> Vec<(f32, Vec<(f32, f32)>)> {
        let Some(faces) = &self.faces else { return vec![] };
        let (z, r) = (self.p.get(0), self.p.get(1));
        let (px, step) = (SMALL * z, 4.0f32);
        let n = ((2.0 * r / step).ceil() as usize).max(2);
        let mut ink = vec![false; n * n];
        let line = LINE * z;
        let base = (line - (ASC + DESC) * px) * 0.5 + ASC * px;
        for (x, y, _, t) in text {
            let (gx, gy) = ((x - self.at.0) * z + r, (y - self.at.1) * z + r + base);
            for (i, c) in t.chars().enumerate() {
                let Some(g) = faces.glyph(FACE, c, px) else { continue };
                let pen = gx + i as f32 * px * ADV;
                for (k, span) in g.rows.iter().enumerate() {
                    if span[0].is_nan() { continue; }
                    let yy = gy - g.top + k as f32 + 0.5;
                    if yy < 0.0 || yy >= 2.0 * r { continue; }
                    let j = (yy / step) as usize;
                    let (a, b) = (((pen + span[0]) / step).max(0.0) as usize, (((pen + span[1]) / step).max(0.0) as usize).min(n - 1));
                    for q in a..=b { if j < n { ink[j * n + q] = true; } }
                }
            }
        }
        let d = edt(&ink, n, n);
        let grid = Grid::sample(n, n, (n - 1) as f32 * step, (n - 1) as f32 * step, &Dist { d: &d, w: n, h: n, step });
        let mut out = vec![];
        let mut level = BAND;
        while level <= REACH { for l in grid.contours(level) { out.push((level, l)); } level += BAND; }
        out
    }
}

#[inline(never)]
fn button(t: &str, on: bool, key: String, m: Msg) -> El<Msg> {
    text(format!("[ {t} ]"), 11.0, if on { Paint::INK } else { Paint::DIM }).font(Font::Mono).key(key).role(Role::Button).label(t).checked(on).focusable(true).on_press(m)
}

impl Experiment for Loupe {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let kf = |s: &str| format!("{key}/{s}");
        // On the index the cell is the glass alone, its text at 11 px (Z.5): the small page is not drawn.
        let index = key.starts_with("labi/");
        let (w, h) = self.size;
        let size = self.size;
        let (z, r) = if index { (4.0, self.p.get(1)) } else { (self.p.get(0), self.p.get(1)) };
        let me = if index { Loupe { at: (w * 0.5, h * 0.5), p: Params { v: vec![4.0, r, self.p.get(2)], ..self.p.clone() }, ..self.clone() } } else { self.clone() };
        let s = me.sheet();
        let mut piece = col().key(kf("piece")).fill_width().fill_height().clip().role(Role::Group).label("page").focusable(!index);
        if !index {
            piece = piece.on_key(|k, _| Some(Msg::Key(k)));
            // The page: every column one text, drawn once and kept.
            let rows = self.rows(s.chars);
            let mut page = col().key(kf("sheet")).width(w).height(h).absolute(Some(0.0), Some(0.0), None, None);
            for c in 0..s.cols {
                let a = (c * s.rows).min(rows.len());
                let b = ((c + 1) * s.rows).min(rows.len());
                let body: Vec<&str> = rows[a..b].iter().map(|r| r.2.as_str()).collect();
                page = page.child(text_lh(body.join("\n"), SMALL, LINE, Paint::TEXT).font(FACE).nowrap().legible(false).key(kf(&format!("sheet/{c}")))
                    .absolute(Some(self.col_x(&s, c)), Some(TOP), None, None));
            }
            if self.pinned { page = page.on_point(move |p| Some(Msg::Point(p, size))); }
            piece = piece.child(page.cached());
        }
        // The lens: the glass and its rim over the page, the size of the stage. The pointer's messages
        // change it alone: it is made and drawn again by itself, the kept page left as it was (Z.7).
        // Free, it hears the pointer over the whole stage and the glass follows. Pinned, only the page
        // around the glass does: a press on the glass's text selects it, a press beside it frees it.
        // A slot or the rim taking a new row changes its text at once, not glyph by glyph.
        let mut lens = col().key(kf("lens")).width(w).height(h).absolute(Some(0.0), Some(0.0), None, None).recompose(false);
        if !index {
            lens = lens.local_messages();
            // Pinned, the pointer passes through it to the page; the glass's text still takes it.
            lens = if self.pinned { lens.pass(true) } else { lens.on_point(move |p| Some(Msg::Point(p, size))) };
        }
        // The glass.
        let (gx, gy) = me.at;
        let shown = me.in_glass();
        let mut glass = col().key(kf("glass")).width(2.0 * r).height(2.0 * r).radius(r).clip_round(r).fill(Paint::GROUND).border(1.0, Paint::INK)
            .absolute(Some(gx - r), Some(gy - r), None, None);
        // Pinned, the glass holds exactly the text it shows, and it selects. Free, it holds its line
        // slots (Z.7): whole rows of the page, padded to the column's width, each glyph outside the
        // circle left out by its pose, so a moving glass moves its slots and changes no measure.
        let pinned = me.pinned && !index;
        let (cw, lh) = (SMALL * z * ADV, LINE * z);
        let set = |t: &(f32, f32, usize, String), paint: Paint, k: usize| {
            let (ox, oy) = ((t.0 - gx) * z + r, (t.1 - gy) * z + r);
            text_lh(t.3.clone(), SMALL * z, LINE * z, paint).font(FACE).nowrap().key(kf(&format!("glass/line/{k}"))).selectable()
                .absolute(Some(ox), Some(oy), None, None)
        };
        let slot = |t: &(f32, f32, Option<usize>, String), paint: Paint, k: usize, seen: bool| {
            let (ox, oy) = ((t.0 - gx) * z + r, (t.1 - gy) * z + r);
            text_lh(t.3.clone(), SMALL * z, LINE * z, paint).font(FACE).nowrap().key(kf(&format!("glass/line/{k}")))
                .pose(move |g: GlyphAt| {
                    // Drawn when its cell reaches into the circle, as `in_glass` finds the pinned text.
                    let (y0, y1) = (oy, oy + lh);
                    let dy = if y1 < r { r - y1 } else if y0 > r { y0 - r } else { 0.0 };
                    let half = (r * r - dy * dy).max(0.0).sqrt();
                    let x = ox + g.x;
                    let on = seen && dy < r && x + cw * 0.5 > r - half && x - cw * 0.5 < r + half;
                    Pose { alpha: if on { 1.0 } else { 0.0 }, ..Pose::default() }
                })
                .absolute(Some(ox), Some(oy), None, None)
        };
        let slots = if pinned { vec![] } else { me.slot_rows() };
        let lines = |paint: Paint, seen: bool, over: &mut Vec<El<Msg>>| {
            if pinned { for (k, t) in shown.iter().enumerate() { over.push(set(t, paint, k)); } }
            else { for (k, t) in slots.iter().enumerate() { over.push(slot(t, paint, k, seen)); } }
        };
        let mut d = Drawing::new(2.0 * r, 2.0 * r);
        let mut over: Vec<El<Msg>> = vec![];
        match if index { 0 } else { me.mode } {
            1 => {
                let rr = r / z;
                glass = glass.child(cached_view(kf("sheet"), [gx - rr, gy - rr, 2.0 * rr, 2.0 * rr], Paint::INK).width(2.0 * r).height(2.0 * r).absolute(Some(0.0), Some(0.0), None, None));
                if me.grid && z >= 8.0 {
                    // The page's pixel grid (a logical px of the page is `zoom` px here).
                    let off = |c: f32, o: f32| (o - (c - rr) * z).rem_euclid(z);
                    let (ox, oy) = (off(gx, 0.0), off(gy, 0.0));
                    let mut x = ox; while x < 2.0 * r { d.line((x, 0.0), (x, 2.0 * r), 1.0, Paint::tone(0.16)); x += z; }
                    let mut y = oy; while y < 2.0 * r { d.line((0.0, y), (2.0 * r, y), 1.0, Paint::tone(0.16)); y += z; }
                }
                // The slots stay, unseen, so a change of mode lays nothing out.
                lines(Paint::INK, false, &mut over);
            }
            2 => {
                lines(Paint::DIM, true, &mut over);
                for (level, l) in me.bands(&shown) { d.polyline(&l, 1.0, |_, _| Paint::tone(0.9 - 0.06 * level)); }
            }
            _ => lines(Paint::INK, true, &mut over),
        }
        // Four ticks on the rim.
        for (dx, dy) in [(0.0, -1.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)] {
            d.line((r + dx * (r - 1.0), r + dy * (r - 1.0)), (r + dx * (r - 9.0), r + dy * (r - 9.0)), 1.0, Paint::INK);
        }
        // The marks under the text, so a press on the text reaches it.
        glass = glass.child(canvas(d).key(kf("glass/marks")).width(2.0 * r).absolute(Some(0.0), Some(0.0), None, None));
        for e in over { glass = glass.child(e); }
        lens = lens.child(glass);
        if !index {
            // On the rim: the zoom, and the line and column under the centre, in the accent.
            let (l, c) = me.reading().unwrap_or((0, 0));
            // On a plate of the ground, 4 px round it, in figures of one width; near the stage's right
            // or lower edge it goes to the rim's other side (Z.7).
            let rim = format!("×{z:.0} · LINE {l:>4} · COL {c:>3}");
            let (pw, ph) = (rim.chars().count() as f32 * 7.0 + 8.0, 26.0);
            let mut rx = gx + r * 0.71 + 6.0;
            if rx + pw > w - 8.0 { rx = gx - r * 0.71 - 6.0 - pw; }
            let mut ry = gy + r * 0.71 + 2.0;
            if ry + ph > h - 8.0 { ry = gy - r * 0.71 - 2.0 - ph; }
            let (rx, ry) = (rx.clamp(8.0, (w - pw - 8.0).max(8.0)), ry.clamp(8.0, (h - ph - 8.0).max(8.0)));
            let rim = text(rim, 11.0, Paint::ACCENT).font(Font::Mono).key(kf("rim"));
            lens = lens.child(col().key(kf("rim/plate")).pass(true).pad(4.0, 4.0).fill(Paint::GROUND).child(rim).absolute(Some(rx), Some(ry), None, None));
        }
        piece = piece.child(lens);
        if !index {
            // The controls' row, on the ground and over the glass: the glass passes under it (Z.8). The
            // pointer passes through its ground to the lens; the controls take it.
            let mut modes = amenui::el::row().gap(8.0).absolute(Some(12.0), Some(10.0), None, None);
            for (m, name) in MODES.iter().enumerate() { modes = modes.child(button(name, self.mode == m, kf(&format!("mode/{name}")), Msg::Mode(m))); }
            let bar = col().key(kf("controls")).width(w).height(TOP - 12.0).fill(Paint::GROUND).pass(true).absolute(Some(0.0), Some(0.0), None, None)
                .child(modes).child(button("grid", self.grid, kf("grid"), Msg::Grid).absolute(None, Some(10.0), Some(12.0), None));
            piece = piece.child(bar);
        }
        let formula = format!("glass: the page within {:.1} px of its centre, set again at {:.0} × {SMALL} = {:.0} px", r / z, z, SMALL * z);
        // The label's count changes as the glass moves: it changes at once, not glyph by glyph, so a
        // moving glass draws the lens alone.
        frame(key, "loupe", "read the small print", piece, formula, &self.p, Msg::Num, &format!("{} rows at {SMALL} px · {:>2} in the glass", self.rows(s.chars).len().min(s.cols * s.rows), shown.len())).recompose(false)
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Point(p, size) => {
                self.resize(size);
                let r = self.p.get(1);
                match p {
                    Some(q) if q.down => match self.hold {
                        None => {
                            let d = (q.x - self.at.0).hypot(q.y - self.at.1);
                            self.hold = Some(if (d - r).abs() <= RIM { Hold::Rim } else { Hold::Press { from: (q.x, q.y), moved: false } });
                        }
                        Some(Hold::Rim) => { self.p.v[1] = (q.x - self.at.0).hypot(q.y - self.at.1).clamp(Self::floor(self.size.0), 240.0).round(); self.sized = true; }
                        Some(Hold::Press { from, moved }) => {
                            let moved = moved || (q.x - from.0).hypot(q.y - from.1) > SLOP;
                            self.hold = Some(Hold::Press { from, moved });
                            if !self.pinned { self.to = (q.x, q.y); self.moving = true; }
                        }
                    },
                    Some(q) => {
                        // A release: a press without a drag pins the glass or frees it.
                        if let Some(Hold::Press { moved: false, .. }) = self.hold { self.pinned = !self.pinned; }
                        self.hold = None;
                        if !self.pinned { self.to = (q.x, q.y); self.moving = true; }
                    }
                    None => { self.hold = None; }
                }
            }
            Msg::Mode(k) => self.mode = k,
            Msg::Grid => self.grid = !self.grid,
            Msg::Key(k) => match k { Key::Char('[') => self.zoom_by(-1.0), Key::Char(']') => self.zoom_by(1.0), _ => {} },
            Msg::Num(i, e) => {
                self.p.edit(i, e);
                // The zoom is a whole multiple of the face: 4, 8 or 12.
                if i == 0 { self.p.v[0] = ((self.p.v[0] / 4.0).round() * 4.0).clamp(4.0, 12.0); }
                if i == 1 { self.sized = true; self.fit_radius(); }
            }
        }
        let _ = now;
    }
    fn step(&mut self, now: f32) -> bool {
        let dt = (now - self.t).clamp(0.0, 0.05);
        self.t = now;
        if !self.moving { return false; }
        (self.at.0, self.v.0) = SPRING.step(self.at.0, self.v.0, self.to.0, dt);
        (self.at.1, self.v.1) = SPRING.step(self.at.1, self.v.1, self.to.1, dt);
        if (self.at.0 - self.to.0).hypot(self.at.1 - self.to.1) < SPRING.rest_distance && self.v.0.hypot(self.v.1) < SPRING.rest_speed {
            (self.at, self.v, self.moving) = (self.to, (0.0, 0.0), false);
        }
        true
    }
    fn moving(&self) -> bool { self.moving }
    fn faces(&mut self, f: Faces) { self.faces = Some(f); }
}
