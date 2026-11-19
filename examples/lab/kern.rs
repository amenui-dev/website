//! Lab · kern: one word whose letters are bodies on springs, and the springs' rest lengths are measured
//! between the letters' outlines. `metric` sets each letter where the shaper does (the face's advances,
//! kerning included); `gap` sets every pair `target` px apart at its closest; `area`
//! gives every pair the same white between them. Under each pair the smallest distance between the two
//! outlines, above it the white area. Drag a letter along the baseline: its neighbours keep the rule and
//! are pushed along, touching but never overlapping; let go, they spring back.
use std::cell::RefCell;
use std::rc::Rc;

use amenui::draw::Drawing;
use amenui::el::{El, Font, GlyphAt, Paint, Phase, Point, Pose, Role, canvas, col, text};
use amenui::experiment::{Experiment, Params, frame, spec};
use amenui::gesture::Spring;
use amenui::pair::{self, Faces, Glyph};
use amenui::widgets::{Edit, Spec, TextEdit, input};

/// The face, the site's sans (site-design Z.6 as amended), by its number: the word and its measures
/// name it, so both are set in it whatever face a page's display role resolves to.
pub const FACE: Font = Font::Face(amenui::text::SANS);
/// The smallest drawn size, px.
pub const MIN: f32 = 44.0;
/// How far every glyph stays inside the stage at `target`'s ceiling and in `area`, px.
pub const CLEAR: f32 = 16.0;
/// The word in `metric` is at most this share of the stage.
pub const FIT: f32 = 0.8;
/// The most characters the word takes.
pub const MAX_CHARS: usize = 10;
/// Noto Sans SC's ascender, descender and cap height, em.
pub const ASC: f32 = 1.16;
pub const DESC: f32 = 0.288;
pub const CAP: f32 = 0.733;
/// The clearance at which two letters touch, px: their coverage never shares a pixel.
pub const TOUCH: f32 = 1.0;
/// The figures' advance at 11 px.
const FIG: f32 = 7.0;
pub const MODES: [&str; 3] = ["metric", "gap", "area"];
static DEFS: [(&str, Spec); 4] = [("target", spec(0.0, 40.0, 1.0, 0, "px")), ("size", spec(44.0, 220.0, 1.0, 0, "px")), ("stiffness", spec(100.0, 900.0, 10.0, 0, "")), ("damping", spec(10.0, 60.0, 1.0, 0, ""))];

/// Pair solutions already found: (left, right, size bits, kind, target bits) and pen to pen, px.
type Solved = Rc<RefCell<Vec<((char, char, u32, u8, u32), f32)>>>;
/// The pens the layout set for the word last drawn, as its pose is told them (`GlyphAt::pen`): the
/// word, its size, and each glyph's pen.
type Laid = Rc<RefCell<(String, f32, Vec<f32>)>>;

/// A letter held: which, and where on it (px from its pen).
#[derive(Clone, Debug, PartialEq)]
struct Hold { i: usize, off: f32 }

#[derive(Clone, Debug)]
pub struct Kern {
    pub p: Params,
    pub mode: usize,
    pub dims: bool,
    pub word: TextEdit,
    /// The page's faces, for the outlines (`Experiment::faces`).
    pub faces: Option<Faces>,
    /// The stage, px, and the drawn size there.
    pub size: (f32, f32),
    pub px: f32,
    /// Each letter's pen from the stage's centre, px, its speed, and where it rests.
    pub x: Vec<f32>,
    pub v: Vec<f32>,
    pub home: Vec<f32>,
    pub moving: bool,
    /// `target`'s ceiling on this stage, px.
    pub ceiling: f32,
    pub t: f32,
    hold: Option<Hold>,
    solved: Solved,
    laid: Laid,
    /// The shaper's pens (em, from the first) the rests were found with; empty before the layout's came.
    basis: Vec<f32>,
}
impl Default for Kern {
    fn default() -> Self {
        Self { p: Params::new(&DEFS, &[8.0, 176.0, 600.0, 44.0]), mode: 0, dims: true, word: TextEdit::new("Voyage"), faces: None, size: (480.0, 360.0), px: 0.0, x: vec![], v: vec![], home: vec![], moving: false, ceiling: 40.0, t: 0.0, hold: None, solved: Rc::default(), laid: Rc::default(), basis: vec![] }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Point(Option<Point>, (f32, f32)), Mode(usize, (f32, f32)), Dims, Word(TextEdit, (f32, f32)), Num(usize, Edit) }

/// One pair, measured: its gap (px), its white area (px²), and where its boundary is (px from the
/// stage's centre, between the end of the left ink and the start of the right).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measure { pub gap: f32, pub area: f32, pub at: f32 }

/// `n` with a thin comma every three figures.
fn grouped(n: f32) -> String {
    let s = format!("{:.0}", n.max(0.0));
    let mut out = String::new();
    for (k, c) in s.chars().enumerate() { if k > 0 && (s.len() - k) % 3 == 0 { out.push(','); } out.push(c); }
    out
}

impl Kern {
    /// The page's faces, new or changed: every measure is made again from them.
    pub fn set_faces(&mut self, f: Faces) { self.faces = Some(f); self.solved = Rc::default(); self.laid = Rc::default(); self.px = 0.0; self.home.clear(); }
    pub fn chars(&self) -> Vec<char> { self.word.text.chars().collect() }
    fn glyph(&self, c: char, px: f32) -> Option<Rc<Glyph>> { self.faces.as_ref()?.glyph(FACE, c, px) }
    /// `c`'s advance at `px`.
    fn advance(&self, c: char, px: f32) -> f32 { self.glyph(c, px).map_or(px * 0.55, |g| g.advance) }
    /// The shaper's pens for `cs` (the word or its start), em from the first, once a pose was told them.
    fn laid_em(&self, cs: &[char]) -> Option<Vec<f32>> {
        let s: String = cs.iter().collect();
        let l = self.laid.borrow();
        let (w, lpx, v) = &*l;
        (w.starts_with(&s) && v.len() >= cs.len() && v.len() == w.chars().count() && v.iter().all(|p| p.is_finite())).then(|| v[..cs.len()].iter().map(|p| (p - v[0]) / lpx).collect())
    }
    /// Where the shaper sets each letter's pen, px from the first: the face's advances, kerning
    /// included, as the layout told the word's pose (`GlyphAt::pen`, scaled to `px`). Until it has, the
    /// advances alone.
    pub fn shaped(&self, cs: &[char], px: f32) -> Vec<f32> {
        if let Some(v) = self.laid_em(cs) { return v.iter().map(|p| p * px).collect(); }
        let mut x = 0.0;
        cs.iter().map(|&c| { let p = x; x += self.advance(c, px); p }).collect()
    }
    /// Whether the layout's pens for the word came, or changed, since the rests were found.
    fn stale(&self) -> bool {
        self.laid_em(&self.chars()).is_some_and(|v| v.len() != self.basis.len() || v.iter().zip(&self.basis).any(|(a, b)| (a - b).abs() > 1e-4))
    }
    /// The layout's pens came: the size, the ceiling and the rests again; resting letters go there.
    fn refit(&mut self) {
        let cs = self.chars();
        let n = cs.len();
        self.px = self.fit(self.size.0, &cs);
        self.ceiling = self.target_ceiling();
        self.home = self.homes();
        self.basis = self.laid_em(&cs).unwrap_or_default();
        if (self.hold.is_none() && !self.moving) || self.x.len() != n { self.x = self.home.clone(); self.v = vec![0.0; n]; } else { self.moving = true; }
    }
    /// The word's width in `metric`, px: the last pen plus its advance.
    pub fn metric_width(&self, cs: &[char], px: f32) -> f32 { cs.last().map_or(0.0, |&c| self.shaped(cs, px).last().copied().unwrap_or(0.0) + self.advance(c, px)) }
    /// The drawn size on a stage `w` px wide (Z.6): the largest whole px no more than `size` at which
    /// the word in `metric` is at most 80 % of the stage, never under 44.
    pub fn fit(&self, w: f32, cs: &[char]) -> f32 {
        let per = self.metric_width(cs, 100.0) / 100.0;
        if per <= 0.0 { return self.p.get(1); }
        let mut px = self.p.get(1).min((FIT * w / per).floor()).max(MIN).floor();
        while px > MIN && self.metric_width(cs, px) > FIT * w { px -= 1.0; }
        px
    }
    /// How many of `cs` fit the 80 % at 44 px, at most 10.
    pub fn most_chars(&self, w: f32, cs: &[char]) -> usize {
        (1..=cs.len().min(MAX_CHARS)).take_while(|&n| self.metric_width(&cs[..n], MIN) <= FIT * w).last().unwrap_or(1)
    }
    /// Pen to pen for the pair (a, b) by the outlines (`kind` 1 gap, 2 area, 3 touching), found once.
    fn solve(&self, a: char, b: char, kind: u8, target: f32) -> f32 {
        let px = self.px;
        let key = (a, b, px.to_bits(), kind, target.to_bits());
        if let Some(d) = self.solved.borrow().iter().find(|e| e.0 == key) { return d.1; }
        let (Some(ga), Some(gb)) = (self.glyph(a, px), self.glyph(b, px)) else { return self.advance(a, px) };
        let d = match kind { 2 => pair::spaced_area(&ga, &gb, target), 3 => pair::spaced(&ga, &gb, TOUCH), _ => pair::spaced(&ga, &gb, target) };
        let mut m = self.solved.borrow_mut();
        if m.len() >= 4096 { m.clear(); }
        m.push((key, d));
        d
    }
    /// The white area every pair has in `area`: the mean of the pairs' areas in `metric`.
    fn area_target(&self) -> f32 {
        let cs = self.chars();
        let pens = self.shaped(&cs, self.px);
        let v: Vec<f32> = cs.windows(2).zip(pens.windows(2)).filter_map(|(w, p)| Some(pair::area(&*self.glyph(w[0], self.px)?, &*self.glyph(w[1], self.px)?, p[1] - p[0]))).collect();
        if v.is_empty() { 0.0 } else { v.iter().sum::<f32>() / v.len() as f32 }
    }
    /// Pen to pen for pair `k` (letters k − 1 and k) by the mode, and touching.
    fn rule(&self, cs: &[char], k: usize) -> (f32, f32) {
        let (a, b) = (cs[k - 1], cs[k]);
        let d = match self.mode { 1 => self.solve(a, b, 1, self.p.get(0).min(self.ceiling)), 2 => self.solve(a, b, 2, self.area_target().round()), _ => { let p = self.shaped(cs, self.px); p[k] - p[k - 1] } };
        (d, self.solve(a, b, 3, 0.0))
    }
    /// The ink's span (left, right) from the stage's centre when every pair is `d(k)` apart, centred.
    fn span(&self, cs: &[char], d: impl Fn(usize) -> f32) -> (f32, f32) {
        let mut x = 0.0f32;
        let mut pens = vec![0.0];
        for k in 1..cs.len() { x += d(k); pens.push(x); }
        let ext = |k: usize| self.ext(cs[k]);
        let (l, r) = (pens[0] + ext(0).0, pens[cs.len() - 1] + ext(cs.len() - 1).1);
        (-(r - l) * 0.5, (r - l) * 0.5)
    }
    /// `target`'s ceiling (Z.6 amended): the largest whole px, at most 40, at which the word in `gap`
    /// keeps 16 px clear of each stage edge.
    fn target_ceiling(&self) -> f32 {
        let cs = self.chars();
        if cs.len() < 2 { return 40.0; }
        let room = self.size.0 * 0.5 - CLEAR;
        let fits = |t: f32| self.span(&cs, |k| self.solve(cs[k - 1], cs[k], 1, t)).1 <= room;
        if fits(40.0) { return 40.0; }
        let (mut lo, mut hi) = (0u32, 40u32);
        if !fits(0.0) { return 0.0; }
        while hi - lo > 1 { let m = (lo + hi) / 2; if fits(m as f32) { lo = m; } else { hi = m; } }
        lo as f32
    }
    /// Where the letters rest: by the rule, the ink centred on the stage (in `metric` the shaper's pens
    /// exactly, moved together).
    fn homes(&self) -> Vec<f32> {
        let cs = self.chars();
        if cs.is_empty() { return vec![]; }
        let mut x = vec![0.0f32; cs.len()];
        for k in 1..cs.len() { x[k] = x[k - 1] + self.rule(&cs, k).0; }
        let ext = |k: usize| self.ext(cs[k]);
        let (l, r) = (x[0] + ext(0).0, x[cs.len() - 1] + ext(cs.len() - 1).1);
        let shift = -(l + r) * 0.5;
        x.iter().map(|p| p + shift).collect()
    }
    /// Fit to a stage of `size`: the drawn size again, and where the letters rest. Resting letters go
    /// there, unless `spring`; moving ones (and those of a word that changed) keep their place, scaled
    /// to the new size, and spring there (Z.6: fitted again on the springs).
    fn resize(&mut self, size: (f32, f32), spring: bool) {
        let cs = self.chars();
        let n = cs.len();
        let px = self.fit(size.0, &cs);
        let same = size.0 == self.size.0;
        self.size = size;
        if !spring && same && px == self.px && self.x.len() == n && self.home.len() == n { return; }
        let k = if self.px > 0.0 { px / self.px } else { 1.0 };
        let snap = !spring && self.hold.is_none() && !self.moving && self.x.len() == n;
        self.px = px;
        self.ceiling = self.target_ceiling();
        self.home = self.homes();
        self.basis = self.laid_em(&cs).unwrap_or_default();
        let old = std::mem::replace(&mut self.x, self.home.clone());
        self.v.resize(n, 0.0);
        if snap { self.v.iter_mut().for_each(|v| *v = 0.0); return; }
        for (x, o) in self.x.iter_mut().zip(&old) { *x = o * k; }
        self.moving = true;
    }
    /// After the rule changed: new rests, and the letters spring to them.
    fn rehome(&mut self) { self.home = self.homes(); self.basis = self.laid_em(&self.chars()).unwrap_or_default(); self.moving = true; }
    /// The piece on a stage of `size`.
    pub fn at_size(&self, size: (f32, f32)) -> Self {
        let mut w = self.clone();
        w.resize(size, false);
        // The slider stops at the fit: it shows the drawn size when the ceiling is above it.
        w.p.v[1] = w.p.v[1].min(w.px);
        // `target` stops at its ceiling too, and the panel prints it.
        w.p.v[0] = w.p.v[0].min(w.ceiling);
        w
    }
    /// The baseline, px from the stage's top.
    pub fn baseline(&self) -> f32 { (self.size.1 * 0.5).round() }
    /// Every pair measured where its letters are now.
    pub fn measures(&self) -> Vec<Measure> {
        let cs = self.chars();
        (1..cs.len()).filter_map(|k| {
            let (a, b) = (self.glyph(cs[k - 1], self.px)?, self.glyph(cs[k], self.px)?);
            let dx = self.x[k] - self.x[k - 1];
            let (ar, bl) = (a.extent()?.1, b.extent()?.0);
            Some(Measure { gap: pair::gap(&a, &b, dx), area: pair::area(&a, &b, dx), at: (self.x[k - 1] + ar + self.x[k] + bl) * 0.5 })
        }).collect()
    }
    /// Keep every pair at least touching, outward from the letter held (or from the left).
    fn keep_apart(&mut self) {
        let cs = self.chars();
        let n = cs.len();
        let from = self.hold.as_ref().map_or(0, |h| h.i);
        for k in from + 1..n { let t = self.rule(&cs, k).1; if self.x[k] < self.x[k - 1] + t { self.x[k] = self.x[k - 1] + t; } }
        for k in (0..from).rev() { let t = self.rule(&cs, k + 1).1; if self.x[k] > self.x[k + 1] - t { self.x[k] = self.x[k + 1] - t; } }
    }
    /// Keep the word's ink on the stage: the first letter's left edge and the last's right edge inside
    /// it, the letters between pushed along, touching.
    fn keep_on_stage(&mut self) {
        let cs = self.chars();
        let n = cs.len();
        if n == 0 || self.x.len() != n { return; }
        let ext = |k: usize| self.ext(cs[k]);
        let half = self.size.0 * 0.5 - 1.0;
        let (lo, hi) = (-half - ext(0).0, half - ext(n - 1).1);
        if self.x[0] < lo { self.x[0] = lo; self.v[0] = 0.0; }
        for k in 1..n { let t = self.rule(&cs, k).1; if self.x[k] < self.x[k - 1] + t { self.x[k] = self.x[k - 1] + t; } }
        if self.x[n - 1] > hi { self.x[n - 1] = hi; self.v[n - 1] = 0.0; }
        for k in (0..n - 1).rev() { let t = self.rule(&cs, k + 1).1; if self.x[k] > self.x[k + 1] - t { self.x[k] = self.x[k + 1] - t; } }
    }
    /// Pen `x` for letter `i`, kept where the letters it pushes, touching, stay on the stage.
    fn on_stage(&self, i: usize, x: f32) -> f32 {
        let cs = self.chars();
        let n = cs.len();
        let ext = |k: usize| self.ext(cs[k]);
        // The letters ahead close until they touch, and then the held one stops: no glyph crosses the edge.
        let half = self.size.0 * 0.5 - 1.0;
        let right: f32 = (i + 1..n).map(|k| self.rule(&cs, k).1).sum::<f32>() + ext(n - 1).1;
        let left: f32 = (1..=i).map(|k| self.rule(&cs, k).1).sum::<f32>() - ext(0).0;
        x.clamp(-half + left, (half - right).max(-half + left))
    }
    /// The letter under (x, y) on the stage: the nearest whose ink, widened by 6 px, holds x.
    fn hit(&self, x: f32, y: f32) -> Option<usize> {
        let by = self.baseline();
        if y < by - ASC * self.px - 8.0 || y > by + DESC * self.px + 8.0 { return None; }
        let cx = x - self.size.0 * 0.5;
        self.chars().iter().enumerate().filter_map(|(k, &c)| {
            let (l, r) = self.glyph(c, self.px)?.extent()?;
            let (l, r) = (self.x[k] + l - 6.0, self.x[k] + r + 6.0);
            (cx >= l && cx <= r).then_some((k, (cx - (l + r) * 0.5).abs()))
        }).min_by(|a, b| a.1.total_cmp(&b.1)).map(|q| q.0)
    }
    /// `c`'s ink across at the drawn size, from its pen (its advance when it has none).
    #[inline(never)]
    fn ext(&self, c: char) -> (f32, f32) { self.glyph(c, self.px).and_then(|g| g.extent()).unwrap_or((0.0, self.advance(c, self.px))) }
    fn spring(&self) -> Spring { Spring { stiffness: self.p.get(2), damping: self.p.get(3), mass: 1.0, rest_distance: 0.25, rest_speed: 4.0 } }
}

#[inline(never)]
fn mono(t: String, paint: Paint, key: String, x: f32, y: f32) -> El<Msg> { text(t, 11.0, paint).font(Font::Mono).key(key).selectable().absolute(Some(x), Some(y), None, None) }
#[inline(never)]
fn button(t: &str, on: bool, key: String, m: Msg) -> El<Msg> {
    text(format!("[ {t} ]"), 11.0, if on { Paint::INK } else { Paint::DIM }).font(Font::Mono).key(key).role(Role::Button).label(t).checked(on).focusable(true).on_press(m)
}

impl Experiment for Kern {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        let (w, h) = self.size;
        let kf = |s: &str| format!("{key}/{s}");
        let (cx, by, px) = (w * 0.5, self.baseline(), self.px);
        let cs = self.chars();
        let size = self.size;
        let mut d = Drawing::new(w.max(1.0), h.max(1.0));
        d.line((12.0, by + 0.5), (w - 12.0, by + 0.5), 1.0, Paint::tone(0.3));
        let mut piece = col().key(kf("piece")).fill_width().fill_height().clip().role(Role::Group).label("word")
            .focusable(false)
            .on_drag_with(move |g| Some(Msg::Point(Some(Point { x: g.x, y: g.y, down: g.phase != Phase::End, glyph: None, z: 0.0, t: g.t }), size)));
        let mut labels: Vec<El<Msg>> = vec![];
        if self.dims && cs.len() > 1 && self.faces.is_some() {
            let m = self.measures();
            // The accent marks the tightest pair, or none when the two tightest are within 0.05 px.
            let first = (0..m.len()).min_by(|&a, &b| m[a].gap.total_cmp(&m[b].gap));
            let second = (0..m.len()).filter(|&k| Some(k) != first).map(|k| m[k].gap).fold(f32::INFINITY, f32::min);
            let tight = first.filter(|&a| second - m[a].gap > 0.05);
            let held = self.hold.as_ref().map(|h| h.i);
            // Gap figures under the pairs, in two rows when one row would crowd them.
            let figs: Vec<String> = m.iter().map(|q| format!("{:.1}", q.gap.max(0.0))).collect();
            let widest = figs.iter().map(|f| f.len()).max().unwrap_or(0) as f32 * FIG;
            let pitch = self.x.windows(2).map(|p| p[1] - p[0]).fold(f32::INFINITY, f32::min);
            let two = widest + 6.0 > pitch;
            let (ly, gy) = (by + DESC * px + 8.0, by + DESC * px + 12.0);
            let ay = by - CAP * px - 24.0;
            for (k, q) in m.iter().enumerate() {
                let paint = if Some(k) == tight { Paint::ACCENT } else { Paint::DIM };
                let x = cx + q.at;
                let g = q.gap.max(0.0);
                d.line((x - g * 0.5, ly), (x + g * 0.5, ly), 1.0, paint);
                for e in [x - g * 0.5, x + g * 0.5] { d.line((e, ly - 3.0), (e, ly + 3.0), 1.0, paint); }
                let row = if two && k % 2 == 1 { 14.0 } else { 0.0 };
                labels.push(mono(figs[k].clone(), paint, kf(&format!("gap/{k}")), (x - figs[k].len() as f32 * FIG * 0.5).round(), gy + row));
                // Below 66 px only the tightest pair's and the held pair's areas (Z.6).
                let shown = px >= 66.0 || Some(k) == tight || held.is_some_and(|i| i == k || i == k + 1);
                if shown {
                    let a = grouped(q.area);
                    labels.push(mono(a.clone(), if Some(k) == tight { Paint::ACCENT } else { Paint::FAINT }, kf(&format!("area/{k}")), (x - a.len() as f32 * FIG * 0.5).round(), ay));
                }
            }
        }
        piece = piece.child(canvas(d).key(kf("canvas")).width(w).absolute(Some(0.0), Some(0.0), None, None));
        // The word, one text: each glyph moved from its pen as laid out to its letter's pen; the pens
        // laid out are kept, for `metric` and the measures (`shaped`).
        let (rec, said, n) = (self.laid.clone(), self.word.text.clone(), cs.len());
        let line = (px * 1.6).round();
        let left = cx + self.x.first().copied().unwrap_or(0.0);
        let top = by - ((line - (ASC + DESC) * px) * 0.5 + ASC * px);
        let pens: Rc<Vec<f32>> = Rc::new(self.x.iter().map(|x| cx + x - left).collect());
        let word = text(self.word.text.clone(), px, Paint::INK).font(FACE).nowrap().selectable().key(kf("word"))
            .pose(move |g: GlyphAt| {
                if g.n == n && g.i < n {
                    // Kept from the first size the word is drawn at: the rests are found from one set of
                    // pens, so a size they choose is not chosen again from another's.
                    let mut l = rec.borrow_mut();
                    if l.0 != said || l.2.len() != n { *l = (said.clone(), px, vec![f32::NAN; n]); }
                    if l.1 == px { l.2[g.i] = g.pen; }
                }
                Pose { dx: pens.get(g.i).map_or(0.0, |p| p - g.pen), ..Pose::default() }
            })
            .absolute(Some(left), Some(top), None, None);
        piece = piece.child(word);
        for l in labels { piece = piece.child(l); }
        let mut modes = amenui::el::row().gap(8.0).absolute(Some(12.0), Some(10.0), None, None);
        for (m, name) in MODES.iter().enumerate() { modes = modes.child(button(name, self.mode == m, kf(&format!("mode/{name}")), Msg::Mode(m, size))); }
        piece = piece.child(modes)
            .child(button("dims", self.dims, kf("dims"), Msg::Dims).absolute(None, None, Some(12.0), Some(16.0)))
            .child(input(&kf("field"), &self.word, "word", 120.0, MAX_CHARS, move |e| Msg::Word(e, size)).absolute(Some(12.0), None, None, Some(10.0)));
        let rule = match self.mode {
            1 => format!("x_i = x_(i−1) + min dx : d(A, B + dx) ≥ {:.0} px", self.p.get(0)),
            2 => format!("x_i = x_(i−1) + dx : white(A, B + dx) = {} px²", grouped(self.area_target().round())),
            _ => "x_i = x_(i−1) + the face's advance, kerning included".to_owned(),
        };
        let formula = format!("{rule}   k = {:.0}  c = {:.0}   drawn {px:.0} px", self.p.get(2), self.p.get(3));
        frame(key, "kern", "pull a letter", piece, formula, &self.p, Msg::Num, &format!("pair distances · drawn {px:.0} px"))
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Point(p, size) => {
                self.resize(size, false);
                match p {
                    Some(q) if q.down => match &self.hold {
                        None => if let Some(i) = self.hit(q.x, q.y) { self.hold = Some(Hold { i, off: q.x - size.0 * 0.5 - self.x[i] }); self.moving = true; },
                        Some(h) => {
                            let i = h.i;
                            self.x[i] = self.on_stage(i, q.x - size.0 * 0.5 - h.off);
                            self.v[i] = 0.0;
                            self.keep_apart();
                            self.keep_on_stage();
                            self.moving = true;
                        }
                    },
                    _ => { self.hold = None; self.moving = true; }
                }
            }
            Msg::Mode(k, size) => { self.resize(size, false); self.mode = k; self.rehome(); }
            Msg::Dims => self.dims = !self.dims,
            Msg::Word(mut e, size) => {
                // Only characters the face has, with ink, and only as many as fit at 44 px.
                let cs: Vec<char> = e.text.chars().filter(|c| !c.is_whitespace() && self.glyph(*c, MIN).is_some_and(|g| g.extent().is_some())).collect();
                let ok: String = cs[..self.most_chars(size.0, &cs).min(cs.len())].iter().collect();
                if ok != e.text { e = TextEdit::new(&ok); }
                let changed = e.text != self.word.text;
                self.word = e;
                if changed { self.hold = None; self.resize(size, true); }
            }
            Msg::Num(i, e) => {
                self.p.edit(i, e);
                match i { 0 => self.rehome(), 1 => { let size = self.size; self.resize(size, true); } _ => {} }
            }
        }
        let _ = now;
    }
    fn step(&mut self, now: f32) -> bool {
        let dt = (now - self.t).clamp(0.0, 0.05);
        self.t = now;
        if self.stale() { self.refit(); return true; }
        if !self.moving { return false; }
        let cs = self.chars();
        let n = cs.len();
        let sp = self.spring();
        let held = self.hold.as_ref().map(|h| h.i);
        // Held: each letter springs to its rule's distance from its neighbour toward the hand (a train
        // that follows); let go, every letter springs home.
        let goal: Vec<f32> = (0..n).map(|k| match held {
            Some(i) if k > i => self.x[k - 1] + self.rule(&cs, k).0,
            Some(i) if k < i => self.x[k + 1] - self.rule(&cs, k + 1).0,
            _ => self.home[k],
        }).collect();
        let mut rest = true;
        for (k, g) in goal.into_iter().enumerate() {
            if Some(k) == held { continue; }
            (self.x[k], self.v[k]) = sp.step(self.x[k], self.v[k], g, dt);
            if (self.x[k] - g).abs() >= sp.rest_distance || self.v[k].abs() >= sp.rest_speed { rest = false; }
        }
        self.keep_apart();
        self.keep_on_stage();
        if rest {
            if held.is_none() { self.x = self.home.clone(); }
            self.v = vec![0.0; n];
            self.moving = false;
        }
        true
    }
    fn moving(&self) -> bool { self.moving || self.stale() }
    fn faces(&mut self, f: Faces) { self.set_faces(f); }
}
