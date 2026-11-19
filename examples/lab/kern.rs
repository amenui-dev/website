//! Lab · kern: one word whose letters are bodies on springs, and the springs' rest lengths are measured
//! between the letters' outlines. `metric` sets each letter after the last by the face's advance (a
//! monospace face: the box model itself); `gap` sets every pair `target` px apart at its closest; `area`
//! gives every pair the same white between them. Under each pair the smallest distance between the two
//! outlines, above it the white area. Drag a letter along the baseline: its neighbours keep the rule and
//! are pushed along, touching but never overlapping; let go, they spring back.
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use amenui::draw::Drawing;
use amenui::el::{El, Font, GlyphAt, Paint, Phase, Point, Pose, Role, canvas, col, text};
use amenui::experiment::{Experiment, Params, frame, spec};
use amenui::gesture::Spring;
use amenui::pair::{self, Faces, Glyph};
use amenui::widgets::{Edit, Spec, TextEdit, input};

/// The face, Departure Mono (site-design Z.6), by its number: the word and its measures name it, so
/// both are set in it whatever face a page's display role resolves to.
pub const FACE: Font = Font::Face(amenui::text::MONO);
/// The face is drawn on an 11-unit grid: the drawn size is a multiple of this, at least `MIN`.
pub const GRID: f32 = 11.0;
pub const MIN: f32 = 44.0;
/// The word in `metric` is at most this share of the stage.
pub const FIT: f32 = 0.8;
/// The most characters the word takes.
pub const MAX_CHARS: usize = 10;
/// Departure Mono's ascender, descender and cap height, em.
pub const ASC: f32 = 1.0;
pub const DESC: f32 = 150.0 / 550.0;
pub const CAP: f32 = 400.0 / 550.0;
/// The clearance at which two letters touch, px: their coverage never shares a pixel.
pub const TOUCH: f32 = 1.0;
/// The figures' advance at 11 px.
const FIG: f32 = 7.0;
pub const MODES: [&str; 3] = ["metric", "gap", "area"];
static DEFS: [(&str, Spec); 4] = [("target", spec(0.0, 40.0, 1.0, 0, "px")), ("size", spec(44.0, 220.0, 11.0, 0, "px")), ("stiffness", spec(100.0, 900.0, 10.0, 0, "")), ("damping", spec(10.0, 60.0, 1.0, 0, ""))];

/// Pair solutions already found: (left, right, size bits, kind, target bits) → pen to pen, px.
type Solved = Rc<RefCell<HashMap<(char, char, u32, u8, u32), f32>>>;

/// A letter held: which, and where on it (px from its pen).
#[derive(Clone, Debug, PartialEq)]
struct Hold { i: usize, off: f32 }

#[derive(Clone, Debug)]
pub struct Kern {
    pub p: Params,
    pub mode: usize,
    pub dims: bool,
    pub word: TextEdit,
    /// The page's faces, for the outlines (`Ui::on_faces`).
    pub faces: Option<Faces>,
    /// The stage, px, and the drawn size there.
    pub size: (f32, f32),
    pub px: f32,
    /// Each letter's pen from the stage's centre, px, its speed, and where it rests.
    pub x: Vec<f32>,
    pub v: Vec<f32>,
    pub home: Vec<f32>,
    pub moving: bool,
    pub t: f32,
    hold: Option<Hold>,
    solved: Solved,
}
impl Default for Kern {
    fn default() -> Self {
        Self { p: Params::new(&DEFS, &[8.0, 176.0, 600.0, 44.0]), mode: 0, dims: true, word: TextEdit::new("Voyage"), faces: None, size: (480.0, 360.0), px: 0.0, x: vec![], v: vec![], home: vec![], moving: false, t: 0.0, hold: None, solved: Rc::default() }
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
    pub fn set_faces(&mut self, f: Faces) { self.faces = Some(f); self.solved = Rc::default(); self.px = 0.0; self.home.clear(); }
    pub fn chars(&self) -> Vec<char> { self.word.text.chars().collect() }
    fn glyph(&self, c: char, px: f32) -> Option<Rc<Glyph>> { self.faces.as_ref()?.glyph(FACE, c, px) }
    /// The face's advance at `px`.
    fn advance(&self, px: f32) -> f32 { self.glyph('0', px).map_or(px * 350.0 / 550.0, |g| g.advance) }
    /// The drawn size on a stage `w` px wide for `n` characters (Z.6): the largest multiple of 11 no more
    /// than `size` at which the word in `metric` is at most 80 % of the stage, never under 44.
    pub fn fit(&self, w: f32, n: usize) -> f32 {
        let a11 = self.advance(GRID) / GRID;
        let most = FIT * w / (n.max(1) as f32 * a11);
        ((self.p.get(1).min(most) / GRID).floor() * GRID).max(MIN)
    }
    /// How many characters fit the 80 % at 44 px, at most 10.
    pub fn most_chars(&self, w: f32) -> usize { ((FIT * w / self.advance(MIN)).floor() as usize).clamp(1, MAX_CHARS) }
    /// Pen to pen for the pair (a, b) by the rule (`kind` 0 metric, 1 gap, 2 area, 3 touching), found once.
    fn solve(&self, a: char, b: char, kind: u8, target: f32) -> f32 {
        let px = self.px;
        if kind == 0 { return self.advance(px); }
        let key = (a, b, px.to_bits(), kind, target.to_bits());
        if let Some(d) = self.solved.borrow().get(&key) { return *d; }
        let (Some(ga), Some(gb)) = (self.glyph(a, px), self.glyph(b, px)) else { return self.advance(px) };
        let d = match kind { 2 => pair::spaced_area(&ga, &gb, target), 3 => pair::spaced(&ga, &gb, TOUCH), _ => pair::spaced(&ga, &gb, target) };
        self.solved.borrow_mut().insert(key, d);
        d
    }
    /// The white area every pair has in `area`: the mean of the pairs' areas in `metric`.
    fn area_target(&self) -> f32 {
        let cs = self.chars();
        let adv = self.advance(self.px);
        let v: Vec<f32> = cs.windows(2).filter_map(|w| Some(pair::area(&*self.glyph(w[0], self.px)?, &*self.glyph(w[1], self.px)?, adv))).collect();
        if v.is_empty() { 0.0 } else { v.iter().sum::<f32>() / v.len() as f32 }
    }
    /// Pen to pen for pair `k` (letters k − 1 and k) by the mode, and touching.
    fn rule(&self, cs: &[char], k: usize) -> (f32, f32) {
        let (a, b) = (cs[k - 1], cs[k]);
        let d = match self.mode { 1 => self.solve(a, b, 1, self.p.get(0)), 2 => self.solve(a, b, 2, self.area_target().round()), _ => self.solve(a, b, 0, 0.0) };
        (d, self.solve(a, b, 3, 0.0))
    }
    /// Where the letters rest: by the rule, the ink centred on the stage, every pen on a whole px.
    fn homes(&self) -> Vec<f32> {
        let cs = self.chars();
        if cs.is_empty() { return vec![]; }
        let mut x = vec![0.0f32; cs.len()];
        for k in 1..cs.len() { x[k] = x[k - 1] + self.rule(&cs, k).0; }
        let ext = |k: usize| self.glyph(cs[k], self.px).and_then(|g| g.extent()).unwrap_or((0.0, self.advance(self.px)));
        let (l, r) = (x[0] + ext(0).0, x[cs.len() - 1] + ext(cs.len() - 1).1);
        let shift = -(l + r) * 0.5;
        x.iter().map(|p| (p + shift).round()).collect()
    }
    /// Fit to a stage of `size`: the drawn size again, and where the letters rest. Resting letters go
    /// there, unless `spring`; moving ones (and those of a word that changed) keep their place, scaled
    /// to the new size, and spring there (Z.6: fitted again on the springs).
    fn resize(&mut self, size: (f32, f32), spring: bool) {
        let n = self.chars().len();
        let px = self.fit(size.0, n);
        self.size = size;
        if !spring && px == self.px && self.x.len() == n && self.home.len() == n { return; }
        let k = if self.px > 0.0 { px / self.px } else { 1.0 };
        let snap = !spring && self.hold.is_none() && !self.moving && self.x.len() == n;
        self.px = px;
        self.home = self.homes();
        let old = std::mem::replace(&mut self.x, self.home.clone());
        self.v.resize(n, 0.0);
        if snap { self.v.iter_mut().for_each(|v| *v = 0.0); return; }
        for (x, o) in self.x.iter_mut().zip(&old) { *x = o * k; }
        self.moving = true;
    }
    /// After the rule changed: new rests, and the letters spring to them.
    fn rehome(&mut self) { self.home = self.homes(); self.moving = true; }
    /// The piece on a stage of `size`.
    pub fn at_size(&self, size: (f32, f32)) -> Self {
        let mut w = self.clone();
        w.resize(size, false);
        // The slider stops at the fit: it shows the drawn size when the ceiling is above it.
        w.p.v[1] = w.p.v[1].min(w.px);
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
    /// Pen `x` for letter `i`, kept where the letters it pushes, touching, stay on the stage.
    fn on_stage(&self, i: usize, x: f32) -> f32 {
        let cs = self.chars();
        let n = cs.len();
        let ext = |k: usize| self.glyph(cs[k], self.px).and_then(|g| g.extent()).unwrap_or((0.0, self.advance(self.px)));
        let half = self.size.0 * 0.5 - 12.0;
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
            let tight = (0..m.len()).min_by(|a, b| m[*a].gap.total_cmp(&m[*b].gap));
            let held = self.hold.as_ref().map(|h| h.i);
            // Gap figures under the pairs, in two rows when one row would crowd them.
            let figs: Vec<String> = m.iter().map(|q| format!("{:.1}", q.gap.max(0.0))).collect();
            let widest = figs.iter().map(|f| f.len()).max().unwrap_or(0) as f32 * FIG;
            let pitch = self.advance(px);
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
        // The word, one text: each glyph moved from where the layout set it (`i` advances from its left)
        // to its letter's pen.
        let adv = self.advance(px);
        let line = (px * 1.6).round();
        let left = (cx + self.x.first().copied().unwrap_or(0.0)).round();
        let top = by - ((line - (ASC + DESC) * px) * 0.5 + ASC * px);
        let pens: Rc<Vec<f32>> = Rc::new(self.x.iter().map(|x| cx + x - left).collect());
        let word = text(self.word.text.clone(), px, Paint::INK).font(FACE).nowrap().selectable().key(kf("word"))
            .pose(move |g: GlyphAt| Pose { dx: pens.get(g.i).map_or(0.0, |p| p - g.i as f32 * adv), ..Pose::default() })
            .absolute(Some(left), Some(top), None, None);
        piece = piece.child(word);
        for l in labels { piece = piece.child(l); }
        let mut modes = amenui::el::row().gap(8.0).absolute(Some(12.0), Some(10.0), None, None);
        for (m, name) in MODES.iter().enumerate() { modes = modes.child(button(name, self.mode == m, kf(&format!("mode/{name}")), Msg::Mode(m, size))); }
        piece = piece.child(modes)
            .child(button("dims", self.dims, kf("dims"), Msg::Dims).absolute(None, None, Some(12.0), Some(16.0)))
            .child(input(&kf("field"), &self.word, "word", 120.0, self.most_chars(w), move |e| Msg::Word(e, size)).absolute(Some(12.0), None, None, Some(10.0)));
        let rule = match self.mode {
            1 => format!("x_i = x_(i−1) + min dx : d(A, B + dx) ≥ {:.0} px", self.p.get(0)),
            2 => format!("x_i = x_(i−1) + dx : white(A, B + dx) = {} px²", grouped(self.area_target().round())),
            _ => format!("x_i = x_(i−1) + {adv:.0} px, the face's advance"),
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
                let most = self.most_chars(size.0);
                let ok: String = e.text.chars().filter(|c| !c.is_whitespace() && self.glyph(*c, GRID).is_some_and(|g| g.extent().is_some())).take(most).collect();
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
        if rest {
            if held.is_none() { self.x = self.home.clone(); }
            self.v = vec![0.0; n];
            self.moving = false;
        }
        true
    }
    fn moving(&self) -> bool { self.moving }
}
