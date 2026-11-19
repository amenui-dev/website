//! Lab · weld: six chips whose outline is the smooth union of their distance fields. Drag one near
//! another and the outlines neck together, with the gap printed between them; let go inside `join`
//! and the two are one group, in the tree and on screen; pull a member away fast enough and it tears
//! off. A press without a drag turns a chip on. Each chip is a rounded box, its distance
//! `|max(q, 0)| + min(max(qx, qy), 0) − r`; the union is the quadratic smooth minimum over `2k`,
//! sampled on a 2 px grid around the chips and traced by marching squares at 0, 6 and 12 px.
use amenui::draw::Drawing;
use amenui::el::{El, Font, Key, Paint, Phase, Point, Role, canvas, col, text};
use amenui::experiment::{Experiment, Params, frame, spec};
use amenui::field::{Grid, Scalar};
use amenui::gesture::Spring;
use amenui::widgets::{Edit, Spec};

pub const LABELS: [&str; 6] = ["glyph", "layout", "field", "sdf", "rust", "wasm"];
static DEFS: [(&str, Spec); 5] = [("k", spec(0.0, 40.0, 1.0, 0, "px")), ("join", spec(4.0, 32.0, 1.0, 0, "px")), ("tear", spec(24.0, 120.0, 1.0, 0, "px")), ("bands", spec(0.0, 3.0, 1.0, 0, "")), ("radius", spec(0.0, 14.0, 1.0, 0, "px"))];
/// Departure Mono's advance at 11 px, and a chip's padding and height.
pub const ADV: f32 = 69.0 / 9.0;
pub const PAD: f32 = 8.0;
pub const CHIP_H: f32 = 28.0;
/// The grid the union is sampled on, px.
pub const STEP: f32 = 2.0;
/// The bands outside the outline, px.
pub const BANDS: [f32; 3] = [6.0, 12.0, 18.0];
/// How far a press may move and still be a press, px.
const SLOP: f32 = 3.0;
/// The spring a chip goes to its place on: settled in about 320 ms.
const SPRING: Spring = Spring { stiffness: 600.0, damping: 44.0, mass: 1.0, rest_distance: 0.25, rest_speed: 4.0 };
/// Where the chips start, as shares of the stage: two rows of three, apart by more than 2k on a cell.
const HOME: [(f32, f32); 6] = [(0.16, 0.40), (0.50, 0.34), (0.84, 0.42), (0.18, 0.76), (0.52, 0.72), (0.84, 0.78)];

/// A box: its centre and half size, px.
pub type Box2 = ((f32, f32), (f32, f32));
/// The nearest pair across two groups: the chip held's member, the other chip, the gap, and the two
/// nearest points.
pub type Near = (usize, usize, f32, (f32, f32), (f32, f32));
/// A chip's width, px.
pub fn chip_w(i: usize) -> f32 { 2.0 * PAD + LABELS[i].len() as f32 * ADV }

/// One chip: its centre (px), its velocity, where it is going, and whether it is on.
#[derive(Clone, Debug, PartialEq)]
pub struct Chip { pub x: f32, pub y: f32, pub vx: f32, pub vy: f32, pub tx: f32, pub ty: f32, pub moving: bool, pub on: bool, pub group: usize }

/// A drag: the chip held, where on it, where it was pressed, whether it has moved past the slop.
#[derive(Clone, Debug, PartialEq)]
struct Hold { chip: usize, off: (f32, f32), from: (f32, f32), moved: bool, last: (f32, f32, f32) }

#[derive(Clone, Debug, PartialEq)]
pub struct Weld { pub p: Params, pub chips: Vec<Chip>, pub size: (f32, f32), pub t: f32, hold: Option<Hold>, seed: u32, placed: bool }
impl Default for Weld {
    fn default() -> Self {
        let chips = (0..6).map(|i| Chip { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, tx: 0.0, ty: 0.0, moving: false, on: false, group: i }).collect();
        let mut w = Self { p: Params::new(&DEFS, &[18.0, 12.0, 56.0, 2.0, 6.0]), chips, size: (480.0, 360.0), t: 0.0, hold: None, seed: 1, placed: false };
        w.place(w.size);
        w
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>, (f32, f32)), Key(usize, Key), Scatter, Num(usize, Edit) }

/// A rounded box's signed distance: centre `c`, half size `h`, corner radius `r`.
pub fn rounded(x: f32, y: f32, c: (f32, f32), h: (f32, f32), r: f32) -> f32 {
    let r = r.min(h.0).min(h.1);
    let (qx, qy) = ((x - c.0).abs() - (h.0 - r), (y - c.1).abs() - (h.1 - r));
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r
}
/// The quadratic smooth minimum over `k`: `min(a, b)` where they differ by `k` or more.
pub fn smin(a: f32, b: f32, k: f32) -> f32 {
    if k <= 0.0 { return a.min(b); }
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - h * h * k * 0.25
}
/// The union of rounded boxes (centre, half size), smoothed over `k`.
pub struct Union<'a> { pub boxes: &'a [Box2], pub r: f32, pub k: f32 }
impl Scalar for Union<'_> {
    fn at(&self, x: f32, y: f32) -> f32 {
        let mut d = f32::INFINITY;
        for &(c, h) in self.boxes {
            // A box farther than the blend cannot change the union here.
            let b = rounded(x, y, c, h, self.r);
            d = if d.is_infinite() { b } else { smin(d, b, self.k) };
        }
        d
    }
}
/// The union's iso-lines at `levels`, as polylines in stage px: the boxes are taken in clusters (those
/// within `2 × reach` of one another), each sampled on a `STEP` grid over its own bounds grown by
/// `reach`, so the empty stage between them costs nothing; straight runs are kept as their ends.
pub fn trace(boxes: &[Box2], r: f32, k: f32, reach: f32, levels: &[f32]) -> Vec<(f32, Vec<(f32, f32)>)> {
    let n = boxes.len();
    let mut cl: Vec<usize> = (0..n).collect();
    fn root(cl: &mut [usize], i: usize) -> usize { let mut i = i; while cl[i] != i { cl[i] = cl[cl[i]]; i = cl[i]; } i }
    for a in 0..n { for b in a + 1..n { if gap(boxes[a], boxes[b]).0 < 2.0 * reach { let (ra, rb) = (root(&mut cl, a), root(&mut cl, b)); cl[ra] = rb; } } }
    let mut out = vec![];
    for c in 0..n {
        if root(&mut cl, c) != c { continue; }
        let mine: Vec<Box2> = (0..n).filter(|&i| root(&mut cl, i) == c).map(|i| boxes[i]).collect();
        let (bx0, by0, bx1, by1) = mine.iter().fold((f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY), |a, &(c, h)| (a.0.min(c.0 - h.0), a.1.min(c.1 - h.1), a.2.max(c.0 + h.0), a.3.max(c.1 + h.1)));
        let u = Union { boxes: &mine, r, k: 2.0 * k };
        for &l in levels {
            // The outline on the 2 px grid, a band (dim) on a 4 px grid, each out to where the blend
            // can push it: the smooth minimum lowers the field by at most k / 2 (its `2k` over 4).
            let (step, grow) = if l <= 0.0 { (STEP, 0.5 * k + 2.0 * STEP) } else { (2.0 * STEP, l + 0.5 * k + 4.0 * STEP) };
            let (x0, y0) = (((bx0 - grow) / step).floor() * step, ((by0 - grow) / step).floor() * step);
            let (cols, rows) = (((bx1 + grow - x0) / step).ceil() as usize + 1, ((by1 + grow - y0) / step).ceil() as usize + 1);
            let g = Grid::sample(cols, rows, (cols - 1) as f32 * step, (rows - 1) as f32 * step, &|x: f32, y: f32| u.at(x + x0, y + y0));
            for line in g.contours(l) { out.push((l, simplify(line.into_iter().map(|(x, y)| (x + x0, y + y0)).collect()))); }
        }
    }
    out
}
/// A polyline with the points that lie within 0.1 px of the line through their neighbours left out.
fn simplify(p: Vec<(f32, f32)>) -> Vec<(f32, f32)> {
    if p.len() < 3 { return p; }
    let mut out = vec![p[0]];
    for i in 1..p.len() - 1 {
        let (a, b, c) = (*out.last().expect("a point"), p[i], p[i + 1]);
        let (dx, dy) = (c.0 - a.0, c.1 - a.1);
        let len = dx.hypot(dy);
        let off = if len < 1e-6 { 0.0 } else { ((b.0 - a.0) * dy - (b.1 - a.1) * dx).abs() / len };
        if off > 0.1 { out.push(b); }
    }
    out.push(p[p.len() - 1]);
    out
}
/// The gap between two boxes (centre, half size): 0 when they meet; and its two nearest points.
pub fn gap(a: Box2, b: Box2) -> (f32, (f32, f32), (f32, f32)) {
    let span = |ca: f32, ha: f32, cb: f32, hb: f32| -> (f32, f32, f32) {
        let (a0, a1, b0, b1) = (ca - ha, ca + ha, cb - hb, cb + hb);
        if b0 >= a1 { (b0 - a1, a1, b0) } else if a0 >= b1 { (a0 - b1, a0, b1) } else { let m = (a0.max(b0) + a1.min(b1)) * 0.5; (0.0, m, m) }
    };
    let (gx, ax, bx) = span(a.0.0, a.1.0, b.0.0, b.1.0);
    let (gy, ay, by) = span(a.0.1, a.1.1, b.0.1, b.1.1);
    (gx.hypot(gy), (ax, ay), (bx, by))
}
/// Mono 11 px text, selectable, at (x, y) in its parent (one function, not a chain at every call).
#[inline(never)]
fn mono(t: String, paint: Paint, key: String, x: f32, y: f32) -> El<Msg> { at(label(t, paint, key), x, y) }
/// Mono 11 px text, selectable.
#[inline(never)]
fn label(t: String, paint: Paint, key: String) -> El<Msg> { text(t, 11.0, paint).font(Font::Mono).key(key).selectable() }
/// `e` pinned at (x, y) in its parent.
#[inline(never)]
fn at(e: El<Msg>, x: f32, y: f32) -> El<Msg> { e.absolute(Some(x), Some(y), None, None) }
/// A small hash for the scatter.
fn hash(i: u32, k: u32) -> f32 { (i.wrapping_mul(2_654_435_761) ^ k.wrapping_mul(40_503)).wrapping_mul(2_246_822_519) as f32 / u32::MAX as f32 }

impl Weld {
    fn half(&self, i: usize) -> (f32, f32) { (chip_w(i) * 0.5, CHIP_H * 0.5) }
    /// Chip `i`'s box (centre, half size).
    pub fn bx(&self, i: usize) -> Box2 { ((self.chips[i].x, self.chips[i].y), self.half(i)) }
    /// The groups, each its members in order along its row (group ids are chip indices, so in id order).
    pub fn groups(&self) -> Vec<Vec<usize>> {
        (0..self.chips.len()).filter(|&g| self.chips.iter().any(|c| c.group == g)).map(|g| self.of(g)).collect()
    }
    fn members(&self, i: usize) -> Vec<usize> { self.of(self.chips[i].group) }
    /// Group `g`'s members, left to right where they are going (six at most: an insertion sort).
    fn of(&self, g: usize) -> Vec<usize> {
        let mut m: Vec<usize> = Vec::with_capacity(6);
        for i in 0..self.chips.len() {
            if self.chips[i].group != g { continue; }
            let at = m.iter().position(|&q| self.chips[q].tx > self.chips[i].tx).unwrap_or(m.len());
            m.insert(at, i);
        }
        m
    }
    /// Chips at their first places on a stage of `size`.
    fn place(&mut self, size: (f32, f32)) {
        for (i, c) in self.chips.iter_mut().enumerate() { let (u, v) = HOME[i]; (c.x, c.y) = (u * size.0, v * size.1); (c.tx, c.ty) = (c.x, c.y); }
        self.placed = true;
    }
    /// The stage changed size: everything keeps its share of it.
    fn resize(&mut self, size: (f32, f32)) {
        if !self.placed || self.size.0 <= 0.0 { self.place(size); }
        else if size != self.size {
            let (sx, sy) = (size.0 / self.size.0, size.1 / self.size.1);
            for c in &mut self.chips { c.x *= sx; c.y *= sy; c.tx *= sx; c.ty *= sy; }
        }
        self.size = size;
    }
    /// Lay group `members` in a row: the first's left edge at `left`, centred on `y`, abutting.
    fn row(&mut self, members: &[usize], left: f32, y: f32, now: f32) {
        let mut x = left;
        for &m in members {
            let w = chip_w(m);
            let c = &mut self.chips[m];
            (c.tx, c.ty, c.moving) = (x + w * 0.5, y, true);
            x += w;
            let _ = now;
        }
    }
    /// The nearest chip outside `i`'s group and the gap to it.
    pub fn nearest(&self, i: usize) -> Option<Near> {
        let mine = self.members(i);
        let mut best: Option<Near> = None;
        for &a in &mine { for b in 0..self.chips.len() {
            if mine.contains(&b) { continue; }
            let (g, pa, pb) = gap(self.bx(a), self.bx(b));
            if best.is_none_or(|q| g < q.2) { best = Some((a, b, g, pa, pb)); }
        } }
        best
    }
    /// Join `i`'s group to `j`'s: `i`'s members go beside `j`'s row on the side they came from.
    fn join(&mut self, i: usize, j: usize, now: f32) {
        let (mine, theirs) = (self.members(i), self.members(j));
        let g = self.chips[j].group;
        for &m in &mine { self.chips[m].group = g; }
        let (lx, rx) = (theirs.iter().map(|&m| self.chips[m].tx - chip_w(m) * 0.5).fold(f32::INFINITY, f32::min), theirs.iter().map(|&m| self.chips[m].tx + chip_w(m) * 0.5).fold(f32::NEG_INFINITY, f32::max));
        let y = self.chips[theirs[0]].ty;
        let width: f32 = mine.iter().map(|&m| chip_w(m)).sum();
        let right = self.chips[i].x > (lx + rx) * 0.5;
        let all: Vec<usize> = if right { theirs.iter().chain(&mine).copied().collect() } else { mine.iter().chain(&theirs).copied().collect() };
        let left = if right { lx } else { lx - width };
        let left = left.clamp(2.0, (self.size.0 - 2.0 - (rx - lx) - width).max(2.0));
        self.row(&all, left, y, now);
    }
    /// Take `i` out of its group; the rest close up.
    fn part(&mut self, i: usize, now: f32) {
        let rest: Vec<usize> = self.members(i).into_iter().filter(|&m| m != i).collect();
        if rest.is_empty() { return; }
        let free = (0..self.chips.len()).find(|g| self.chips.iter().all(|c| c.group != *g)).unwrap_or(i);
        self.chips[i].group = free;
        let left = rest.iter().map(|&m| self.chips[m].tx - chip_w(m) * 0.5).fold(f32::INFINITY, f32::min);
        let y = self.chips[rest[0]].ty;
        self.row(&rest, left, y, now);
        let c = &mut self.chips[i];
        (c.tx, c.ty, c.moving) = (c.x, c.y, true);
    }
    /// The topmost chip under (x, y).
    fn hit(&self, x: f32, y: f32) -> Option<usize> {
        (0..self.chips.len()).rev().find(|&i| { let (c, h) = self.bx(i); (x - c.0).abs() <= h.0 && (y - c.1).abs() <= h.1 })
    }
    /// New places, apart by more than 2k + 8 where the stage has room.
    fn scatter(&mut self, now: f32) {
        self.seed = self.seed.wrapping_add(1);
        let (w, h) = self.size;
        let k = self.p.get(0);
        for c in self.chips.iter_mut().enumerate() { c.1.group = c.0; }
        let mut placed: Vec<Box2> = vec![];
        for (i, home) in HOME.iter().enumerate() {
            let hf = self.half(i);
            let mut best = (home.0 * w, home.1 * h);
            for n in 0..200u32 {
                let (x, y) = (hf.0 + 8.0 + hash(self.seed * 97 + i as u32, n) * (w - 2.0 * hf.0 - 16.0).max(0.0), 40.0 + hf.1 + hash(self.seed * 131 + i as u32, n + 999) * (h - 2.0 * hf.1 - 48.0).max(0.0));
                if placed.iter().all(|&b| gap(((x, y), hf), b).0 > 2.0 * k + 8.0) { best = (x, y); break; }
            }
            placed.push((best, hf));
            let c = &mut self.chips[i];
            (c.tx, c.ty, c.moving) = (best.0, best.1, true);
            c.on = false;
        }
        let _ = now;
    }
    fn press_move(&mut self, x: f32, y: f32, t: f32) {
        let tear = self.p.get(2);
        let Some(h) = self.hold.as_mut() else { return };
        if !h.moved && (x - h.from.0).hypot(y - h.from.1) >= SLOP { h.moved = true; }
        if !h.moved { return; }
        let i = h.chip;
        let (nx, ny) = (x - h.off.0, y - h.off.1);
        let dt = (t - h.last.2).max(1.0 / 240.0);
        let (vx, vy) = ((nx - h.last.0) / dt, (ny - h.last.1) / dt);
        h.last = (nx, ny, t);
        let c = &mut self.chips[i];
        (c.x, c.y, c.tx, c.ty, c.vx, c.vy, c.moving) = (nx, ny, nx, ny, vx, vy, false);
        // The rest of the group follows on its spring, each to its place beside the one held; the
        // one held tears off when it is more than `tear` from where the group would have it.
        let mine = self.members(i);
        if mine.len() > 1 {
            // A member's centre from its row's left edge.
            let at = |m: usize| -> f32 { let mut x = 0.0; for &q in &mine { if q == m { break; } x += chip_w(q); } x + chip_w(m) * 0.5 };
            let slot_i = at(i);
            let f = mine.iter().copied().find(|&m| m != i).expect("a member");
            let (wx, wy) = (self.chips[f].x + (slot_i - at(f)), self.chips[f].y);
            if (nx - wx).hypot(ny - wy) > tear { self.part(i, t); return; }
            for &m in &mine { if m != i { let sm = at(m); let c = &mut self.chips[m]; (c.tx, c.ty, c.moving) = (nx + (sm - slot_i), ny, true); } }
        }
    }
    fn release(&mut self, t: f32) {
        let Some(h) = self.hold.take() else { return };
        let i = h.chip;
        if !h.moved { self.chips[i].on = !self.chips[i].on; return; }
        let join = self.p.get(1);
        if let Some((_, b, g, _, _)) = self.nearest(i) && g < join { self.join(i, b, t); return; }
        // Not joined: the group closes round the one let go, on its spring.
        let mine = self.members(i);
        if mine.len() > 1 {
            let x0: f32 = { let mut x = self.chips[i].x - chip_w(i) * 0.5; for &q in &mine { if q == i { break; } x -= chip_w(q); } x };
            let y = self.chips[i].y;
            self.row(&mine, x0, y, t);
        }
        let c = &mut self.chips[i];
        (c.tx, c.ty) = (c.x, c.y);
    }
    /// Move `i`'s group by (dx, dy), kept on the stage.
    fn nudge(&mut self, i: usize, dx: f32, dy: f32) {
        let (w, h) = self.size;
        for m in self.members(i) {
            let hf = self.half(m);
            let c = &mut self.chips[m];
            c.tx = (c.tx + dx).clamp(hf.0, (w - hf.0).max(hf.0));
            c.ty = (c.ty + dy).clamp(hf.1, (h - hf.1).max(hf.1));
            c.moving = true;
        }
    }
}

impl Weld {
    /// The piece on a stage of `size` (the host knows the stage; the piece keeps its chips' shares of it).
    pub fn at_size(&self, size: (f32, f32)) -> Self { let mut w = self.clone(); w.resize(size); w }
}

impl Experiment for Weld {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let (w, h) = self.size;
        let kf = |s: &str| format!("{key}/{s}");
        let (k, join, bands, r) = (self.p.get(0), self.p.get(1), self.p.get(3) as usize, self.p.get(4));
        let boxes: Vec<Box2> = (0..self.chips.len()).map(|i| self.bx(i)).collect();
        let mut d = Drawing::new(w.max(1.0), h.max(1.0));
        let reach = if bands > 0 { BANDS[bands.min(3) - 1] } else { 0.0 } + 2.0 * k + 2.0 * STEP;
        let levels: Vec<f32> = BANDS.iter().take(bands).copied().chain([0.0]).collect();
        for (level, line) in trace(&boxes, r, k, reach, &levels) { d.polyline(&line, 1.0, |_, _| if level > 0.0 { Paint::tone(0.32) } else { Paint::tone(0.9) }); }
        // The gap from what is held to the nearest chip outside its group, while it is within 2k.
        let mut dims: Vec<El<Msg>> = vec![];
        if let Some(hd) = self.hold.as_ref().filter(|h| h.moved) && let Some((_, _, gp, a, b)) = self.nearest(hd.chip) && gp < 2.0 * k.max(join) && gp > 0.5 {
            let paint = if gp < join { Paint::ACCENT } else { Paint::DIM };
            d.line(a, b, 1.0, paint);
            let n = ((b.1 - a.1) / gp, -(b.0 - a.0) / gp);
            for p in [a, b] { d.line((p.0 - n.0 * 4.0, p.1 - n.1 * 4.0), (p.0 + n.0 * 4.0, p.1 + n.1 * 4.0), 1.0, paint); }
            // The number above both chips, clear of the outline and its bands.
            let top = self.members(hd.chip).into_iter().chain(self.nearest(hd.chip).map(|q| q.1)).map(|m| self.chips[m].y - CHIP_H * 0.5).fold(f32::INFINITY, f32::min);
            let mx = (a.0 + b.0) * 0.5;
            // Above the bands, which the blend lifts by up to k / 2 over the gap.
            let lift = if bands > 0 { BANDS[bands.min(3) - 1] } else { 0.0 } + 0.5 * k + 2.0;
            let label = format!("{:.0} PX", gp);
            let x = mx - label.len() as f32 * ADV * 0.5;
            dims.push(mono(label, paint, kf("gap"), x, top - 18.0 - lift));
            d.line((mx, top - 4.0 - lift), (mx, a.1), 1.0, paint.alpha(0.5));
        }
        let size = (w, h);
        let mut piece = col().key(kf("piece")).fill_width().fill_height().clip().role(Role::Group).label("chips")
            // A press on the stage is heard, it does not take the focus: the chips take it.
            .focusable(false)
            // Only a press and its drag are heard: a pointer passing over makes no message.
            .on_drag_with(move |d| Some(Msg::Point(Some(Point { x: d.x, y: d.y, down: d.phase != Phase::End, glyph: None, z: 0.0, t: d.t }), size)))
            .child(at(canvas(d).key(kf("canvas")).width(w), 0.0, 0.0));
        // The chips: their fills under the outline, their labels over it; a group is one element.
        for grp in self.groups() {
            let mut g = at(col().key(kf(&format!("group/{}", self.chips[grp[0]].group))).width(w).height(h), 0.0, 0.0);
            if grp.len() > 1 { g = g.role(Role::Group).label(grp.iter().map(|&i| LABELS[i]).collect::<Vec<_>>().join(" + ")); }
            for &i in &grp {
                let c = &self.chips[i];
                let cw = chip_w(i);
                let (fill, ink) = if c.on { (Paint::INK, Paint::GROUND) } else { (Paint::tone(0.08), Paint::TEXT) };
                let ck = kf(&format!("chip/{}", LABELS[i]));
                let name = label(LABELS[i].to_owned(), ink, format!("{ck}/label"));
                g = g.child(at(col().key(ck).width(cw).height(CHIP_H).radius(r.min(CHIP_H * 0.5)).fill(fill).center()
                    .role(Role::Button).label(LABELS[i]).checked(c.on).on_key(move |k, _| Some(Msg::Key(i, k))), c.x - cw * 0.5, c.y - CHIP_H * 0.5)
                    .child(name));
            }
            piece = piece.child(g);
        }
        let groups = self.groups().len();
        piece = piece.child(mono(format!("{} CHIPS · {} GROUPS", self.chips.len(), groups), Paint::DIM, kf("count"), 12.0, 10.0))
            .child(text("[ scatter ]", 11.0, Paint::DIM).font(Font::Mono).key(kf("scatter")).role(Role::Button).label("scatter").on_press(Msg::Scatter).absolute(None, Some(10.0), Some(12.0), None));
        for e in dims { piece = piece.child(e); }
        let formula = format!("d = smin(d_i, 2k)   d_i = |max(q,0)| + min(max(qx,qy),0) − {r:.0}   k = {k:.0}  join < {join:.0}  tear > {:.0} px", self.p.get(2));
        frame(key, "weld", "drag two together", piece, formula, &self.p, Msg::Num, "smooth union · bands at 6, 12 px")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Point(p, size) => {
                self.resize(size);
                match p {
                    Some(q) if q.down => match &self.hold {
                        None => if let Some(i) = self.hit(q.x, q.y) {
                            let c = &self.chips[i];
                            self.hold = Some(Hold { chip: i, off: (q.x - c.x, q.y - c.y), from: (q.x, q.y), moved: false, last: (c.x, c.y, q.t) });
                        },
                        Some(_) => self.press_move(q.x, q.y, q.t),
                    },
                    _ => self.release(now),
                }
            }
            Msg::Key(i, k) => match k {
                Key::Left => self.nudge(i, -8.0, 0.0),
                Key::Right => self.nudge(i, 8.0, 0.0),
                Key::Up => self.nudge(i, 0.0, -8.0),
                Key::Down => self.nudge(i, 0.0, 8.0),
                Key::Enter => if let Some((_, b, g, _, _)) = self.nearest(i) && g < self.p.get(1) { self.join(i, b, now); },
                Key::Backspace => self.part(i, now),
                Key::Char(' ') => self.chips[i].on = !self.chips[i].on,
                _ => {}
            },
            Msg::Scatter => self.scatter(now),
            Msg::Num(i, e) => self.p.edit(i, e),
        }
    }
    fn step(&mut self, now: f32) -> bool {
        let dt = (now - self.t).clamp(0.0, 0.05);
        self.t = now;
        let mut changed = false;
        for c in &mut self.chips {
            if !c.moving { continue; }
            (c.x, c.vx) = SPRING.step(c.x, c.vx, c.tx, dt);
            (c.y, c.vy) = SPRING.step(c.y, c.vy, c.ty, dt);
            if (c.x - c.tx).hypot(c.y - c.ty) < SPRING.rest_distance && c.vx.hypot(c.vy) < SPRING.rest_speed { (c.x, c.y, c.vx, c.vy, c.moving) = (c.tx, c.ty, 0.0, 0.0, false); }
            changed = true;
        }
        changed
    }
    fn moving(&self) -> bool { self.chips.iter().any(|c| c.moving) }
}
