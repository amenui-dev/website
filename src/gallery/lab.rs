//! The Lab (DESIGN.md §5.3, site-design Z.5): one reading path in five steps, in the order amenui
//! draws a page (set, ink, material, field, change), 29 cells in one card. The ten live experiments
//! (`examples/lab/`) are in the lab's look (near-black ground, grey-white hairlines and dots, one blue
//! accent where the pointer reaches). On the index each cell rests as a still and one runs at a time;
//! an experiment's own page shows it larger with its documentation (`docs/site/<language>/lab-<name>.md`).
use super::{Gallery, Msg};
use amenui::el::{El, Font, Paint, col, text};
use amenui::experiment::Experiment;
use crate::showcase::lab as x;
use amenui::style::{LAB, Look, Material};
use std::rc::Rc;

pub const COUNT: usize = 10;
/// The cells in page order (their files in `examples/lab/` and their pages `#lab-<name>`).
pub const NAMES: [&str; COUNT] = ["lumen", "contours", "silk", "grown", "mercury", "flow", "loupe", "orbit", "spring", "stack"];

macro_rules! cells {
    (@view $s:ident, lumen, $l:ident, $k:ident) => { $l };
    (@view $s:ident, stack, $l:ident, $k:ident) => { $k };
    (@view $s:ident, $f:ident, $l:ident, $k:ident) => { (&$s.$f) };
    // Whether a piece's message says the pointer came onto it (`Some(true)`) or left it.
    (@over stack, $m:ident) => { match &$m { x::stack::Msg::Deck(amenui::widgets::StackMsg::Hover(h)) => Some(h.is_some()), _ => None } };
    (@over $f:ident, $m:ident) => { match &$m { x::$f::Msg::Point(p) => Some(p.is_some()), _ => None } };
    ($( $i:literal => $f:ident : $t:ty , $v:ident );* $(;)?) => {
        #[derive(Clone, Debug, PartialEq)]
        pub enum LMsg { Seen(usize, bool), /** The pointer is over cell `i`'s frame (outside its piece). */ Frame(usize, bool), /** Step `i`'s section is in the window. */ Step(usize, bool), /** The pointer is over family `f`'s cell. */ Fam(usize, bool), /** To step `i`'s header on the index. */ GoStep(usize), $( $v(<$t as Experiment>::Msg) ),* }
        /// Every experiment's state, its own clock, and which cell can be seen and which the pointer is on.
        #[derive(Clone, Default)]
        pub struct Lab {
            pub seen: [bool; COUNT], pub until: f32,
            /// The viewer asked for reduced motion: cells rest after `IDLE_S`.
            pub reduced: bool,
            /// Each piece's own time: it advances only while the piece runs, so a still stays where it stopped.
            pub local: [f32; COUNT], pub last: f32,
            /// The pointer is on the cell's frame, on its piece.
            pub frame: [bool; COUNT], pub over: [bool; COUNT],
            /// No hover (touch): the cell nearest the window's centre runs, `centred`, once scrolling has
            /// rested `SETTLE_S` after `seen_t`.
            pub coarse: bool, pub centred: Option<usize>, pub seen_t: f32,
            /// Which steps' sections are in the window (the path line marks the first).
            pub steps: [bool; 5],
            /// A family's cell under the pointer steps through its members, `FAMILY_S` each: how long it has.
            pub fam_on: [bool; 3], pub fam_t: [f32; 3],
            $( pub $f: $t ),*
        }
        impl Lab {
            /// Cell `i`, keyed under `key`, telling when it comes into view, `width` px wide (Lumen's
            /// text fills its stage in whole lines of that width); on the index (`index`) only its name,
            /// its hint and the piece (the formula and the label are on its page).
            /// `stage` px tall on the index (a wide cell keeps its row's height).
            pub fn cell(&self, i: usize, key: &str, width: f32, stage: f32, index: bool) -> El<LMsg> {
                let lumen = x::lumen::Lumen { rows: ((stage / x::lumen::LINE).floor().max(1.0)) as usize, ..self.lumen.clone() };
                let stack = if index && self.stack.deck.under().is_none() && !self.stack.deck.moving() { x::stack::Stack { deck: fanned(&self.stack.deck), quiet: true, ..self.stack.clone() } } else { self.stack.clone() };
                let (lumen, stack) = (&lumen, &stack);
                let mut c = match i { $( $i => cells!(@view self, $f, lumen, stack).view(key).map(Rc::new(LMsg::$v)), )* _ => col() };
                // The piece's box is the cell's stage (`El::stage`): when it is slow to draw it holds its
                // last image on frames that answer input, so the parameters beside it answer in the frame.
                if let Some(st) = c.children.get_mut(1) { st.stage = true; }
                if index {
                    c.children.truncate(2);
                    if let Some(st) = c.children.get_mut(1) { st.style.aspect_ratio = Some((width - 26.0) / stage); }
                }
                // A living piece moves on purpose while it runs.
                if !self.reduced { c = c.ongoing(); }
                c = c.on_move(move |p| Some(LMsg::Frame(i, p.is_some())));
                // At least half of it is in the window: a point at the middle of its stage (the cell's
                // padding, its name's line and the gap above the stage) tells when it is seen.
                let mid = amenui::el::space(1.0, 1.0).key(format!("{key}/seen")).absolute(Some(width * 0.5), Some(10.0 + 14.0 + 8.0 + stage * 0.5), None, None)
                    .on_visible(move |v| Some(LMsg::Seen(i, v)));
                // The same cell on the index and on its page: it flies from one to the other.
                c.child(mid).flow(format!("lab-cell-{i}"))
            }
            pub fn update(&mut self, m: LMsg, now: f32) {
                if !matches!(m, LMsg::Seen(..) | LMsg::Frame(..) | LMsg::Step(..)) { self.until = now + amenui::experiment::IDLE_S; }
                match m {
                    LMsg::Seen(i, v) => if self.seen[i] != v { self.seen[i] = v; self.seen_t = now; },
                    LMsg::Frame(i, v) => self.frame[i] = v,
                    LMsg::Step(i, v) => self.steps[i] = v,
                    LMsg::Fam(f, v) => { self.fam_on[f] = v; self.last = now; }
                    LMsg::GoStep(_) => {}
                    $( LMsg::$v(m) => { if let Some(o) = cells!(@over $f, m) { self.over[$i] = o; } self.$f.update(m, self.local[$i]) } )*
                }
            }
            /// The one cell that runs: the page's own, else the one under the pointer, else (touch) the
            /// centred one.
            fn running(&self, page: Option<usize>) -> Option<usize> {
                page.or_else(|| (0..COUNT).find(|&i| self.frame[i] || self.over[i])).or(if self.coarse { self.centred } else { None })
            }
            /// The middle one of the cells in view, in page order (the one nearest the window's centre).
            fn middle(&self) -> Option<usize> {
                let seen: Vec<usize> = (0..COUNT).filter(|&i| self.seen[i]).collect();
                seen.get(seen.len() / 2).copied()
            }
            fn ready(&self, i: usize) -> bool { i != 3 || self.grown.mask.is_some() }
            /// Each frame: on the index, bring every piece to its still (`STILL_S` of its own time) once;
            /// then step the one cell that runs (with motion reduced only for `IDLE_S` after input), and
            /// any piece still settling. Whether the view changed.
            pub fn step(&mut self, now: f32, page: Option<usize>, index: bool) -> bool {
                let gap = (now - self.last).max(0.0);
                let dt = gap.min(0.1);
                self.last = now;
                let mut c = false;
                let live = !self.reduced || now < self.until;
                for f in 0..3 {
                    if !(self.fam_on[f] && live) { continue; }
                    let was = (self.fam_t[f] / FAMILY_S) as usize;
                    self.fam_t[f] += gap.min(FAMILY_S);
                    c |= (self.fam_t[f] / FAMILY_S) as usize != was;
                }
                // Grown's reaction-diffusion costs a few ms a step: its still comes over a second.
                $( if index && self.ready($i) && self.local[$i] < STILL_S {
                    let mut n = if $i == 3 { 4 } else { 240 };
                    while self.local[$i] < STILL_S && n > 0 { self.local[$i] += 1.0 / 60.0; self.$f.step(self.local[$i]); n -= 1; }
                    c = true;
                } )*
                if self.coarse && now - self.seen_t >= SETTLE_S { self.centred = self.middle(); }
                let run = self.running(page).filter(|_| !self.reduced || now < self.until);
                $( if run == Some($i) || self.$f.moving() { self.local[$i] += dt; c |= self.$f.step(self.local[$i]); } )*
                c
            }
            /// When the cells next need a frame.
            pub fn wake(&self, now: f32, page: Option<usize>, index: bool) -> Option<f32> {
                let live = !self.reduced || now < self.until;
                let busy = (live && self.running(page).is_some()) || (index && (0..COUNT).any(|i| self.ready(i) && self.local[i] < STILL_S)) $( || self.$f.moving() )*;
                let fam = (0..3).filter(|&f| self.fam_on[f] && live).map(|f| now + FAMILY_S - self.fam_t[f] % FAMILY_S).reduce(f32::min);
                if busy { Some(now) } else if let Some(t) = fam { Some(t) } else if self.coarse && self.middle() != self.centred { Some(self.seen_t + SETTLE_S) } else { None }
            }
        }
    };
}
cells! {
    0 => lumen: x::lumen::Lumen, Lumen;
    1 => contours: x::contours::Contours, Contours;
    2 => silk: x::silk::Silk, Silk;
    3 => grown: x::grown::Grown, Grown;
    4 => mercury: x::mercury::Mercury, Mercury;
    5 => flow: x::flow::Flow, Flow;
    6 => loupe: x::loupe::Loupe, Loupe;
    7 => orbit: x::orbit::Orbit, Orbit;
    8 => spring: x::spring::Spring, Spring;
    9 => stack: x::stack::Stack, Stack;
}

/// The stack on the index at rest: fanned under its top card, settled (a hover fans it live).
pub fn fanned(d: &amenui::widgets::Stack) -> amenui::widgets::Stack {
    let mut f = d.clone();
    f.update(amenui::widgets::StackMsg::Press(Some(0)), 0.0);
    let mut t = 0.0;
    while f.moving() && t < 5.0 { t += 1.0 / 60.0; f.step(t); }
    f
}

pub fn update(s: &mut Gallery, m: LMsg) {
    if let LMsg::GoStep(i) = m {
        super::update(s, Msg::Go(super::Page::Lab));
        s.cmds.push(amenui::morph::Cmd::ScrollTo { key: format!("labx/{}/head", STEPS[i]) });
        return;
    }
    let now = s.clock;
    s.lab.update(m, now);
}
/// A piece's time at which the index shows it still (the frame the audit uses).
pub const STILL_S: f32 = 4.0;
/// With no hover, how long scrolling rests before the centred cell takes over.
pub const SETTLE_S: f32 = 0.2;
fn page_cell(s: &Gallery) -> Option<usize> { if let super::Page::Experiment(i) = s.page { Some(i as usize) } else { None } }
/// Step the one cell that runs (Y.7): on an experiment's page its own; on the index the one under the
/// pointer, or with no hover the one nearest the window's centre; every other cell is a still.
pub fn tick(s: &mut Gallery, t: f32) -> bool { let (p, i) = (page_cell(s), s.page == super::Page::Lab); on_lab(s) && s.lab.step(t, p, i) }
pub fn wake(s: &Gallery, t: f32) -> Option<f32> { if on_lab(s) { s.lab.wake(t, page_cell(s), s.page == super::Page::Lab) } else { None } }
/// Arriving on a Lab page starts the idle motion of what can be seen.
pub fn enter(s: &mut Gallery) { s.lab.until = s.clock + amenui::experiment::IDLE_S; }
fn on_lab(s: &Gallery) -> bool { matches!(s.page, super::Page::Lab | super::Page::Experiment(_)) }

/// The lab's look on its pages: its theme, plain material, nothing else.
pub fn look(s: &Gallery, l: &mut Look) {
    if !on_lab(s) { return; }
    l.theme = LAB;
    l.material = Material::Plain;
    l.post = Look::default().post;
}

/// The ten experiments, then the field map and the reader (their pages, for the route table).
pub fn experiments() -> Vec<super::Page> {
    use super::Page;
    (0..COUNT as u8).map(Page::Experiment).chain([Page::Stirr, Page::Reader]).collect()
}

// ── The path (Z.5) ───────────────────────────────────────────────────────────────────────────────

/// The five steps, in the order amenui draws a page: their keys (`lab.step.<key>` names them,
/// `lab.step.<key>.line` is the sentence under the header; `/lab/#<key>`).
pub const STEPS: [&str; 5] = ["set", "ink", "material", "field", "change"];
/// The materials in the order their page strip shows them: the control, the reference, then the
/// lattices, cut and print, line, signal.
pub const MATERIAL_ORDER: [u8; 9] = [0, 7, 1, 2, 3, 5, 4, 8, 6];
/// The marks by what they are made of: characters, dots, lines, tiles (`MARK_RUNS` starts each run).
pub const MARK_ORDER: [u8; 20] = [0, 13, 5, 16, 12, 10, 1, 9, 17, 11, 7, 6, 19, 2, 3, 4, 14, 15, 8, 18];
pub const MARK_RUNS: [usize; 4] = [0, 2, 7, 13];
/// How long a family's cell shows each member while the pointer is on it.
pub const FAMILY_S: f32 = 1.2;

/// The presets in the order of the material each uses.
pub fn preset_order() -> Vec<u8> {
    let st = amenui::style::styles();
    let at = |m: Material| MATERIAL_ORDER.iter().position(|&k| k == m.id()).unwrap_or(MATERIAL_ORDER.len());
    let mut v: Vec<u8> = (0..st.len() as u8).collect();
    v.sort_by_key(|&i| at(st[i as usize].material));
    v
}
/// The members of family `f` (materials, marks, presets), in their strip's order.
pub fn family(f: usize) -> Vec<super::Page> {
    use super::Page;
    match f { 0 => MATERIAL_ORDER.map(Page::Material).to_vec(), 1 => MARK_ORDER.map(Page::Marks).to_vec(), _ => preset_order().into_iter().map(Page::Style).collect() }
}
/// The family a page belongs to (the presets' own page is the family's).
pub fn family_of(p: super::Page) -> Option<usize> {
    use super::Page;
    match p { Page::Material(_) => Some(0), Page::Marks(_) => Some(1), Page::Style(_) | Page::Presets => Some(2), _ => None }
}
/// The 29 cells, step by step, in teaching order. Until a piece of Z.3 is built its old piece holds
/// the place: Lumen Gnomon's, Silk Lee's, Mercury Weld's, Flow Trace's, Orbit Kern's, Spring Throw's,
/// Stack Sheaf's, the sphere (`loupe`) Loupe's.
pub fn path() -> [Vec<super::Page>; 5] {
    use super::Page::*;
    let e = Experiment;
    [
        vec![Reader, Scene(1), e(5), e(1), e(9), Type, Gaze],
        vec![e(6), Scene(4), e(7), e(4), e(0), e(3)],
        vec![Material(MATERIAL_ORDER[0]), Marks(MARK_ORDER[0]), Scene(2), Presets, Scene(6), Three],
        vec![Stirr, e(2), Scene(5)],
        vec![e(8), Morph, Particles, Flow, Decode, Scene(0), Scene(3)],
    ]
}
/// Where a page is on the path, (step, place): a family's member at its family's cell.
pub fn place(p: super::Page) -> Option<(usize, usize)> {
    use super::Page;
    let q = match p { Page::Material(_) => Page::Material(MATERIAL_ORDER[0]), Page::Marks(_) => Page::Marks(MARK_ORDER[0]), Page::Style(_) => Page::Presets, q => q };
    path().iter().enumerate().find_map(|(s, v)| v.iter().position(|&x| x == q).map(|i| (s, i)))
}
/// A step's name in the current language.
pub fn step_name(s: usize) -> String { amenui::i18n::tr(&format!("lab.step.{}", STEPS[s])) }
/// A cell's name on the index: a family by its family's name.
pub fn cell_name(p: super::Page) -> String {
    use super::Page;
    let tr = amenui::i18n::tr;
    match p { Page::Material(_) => tr("nav.materials"), Page::Marks(_) => tr("nav.marks"), Page::Presets => tr("lab.presets"), p => page_name(p) }
}
/// A page's name as shown: the names written in Chinese are put in English on the way to the screen;
/// here already, so they can be set in capitals and lower case.
pub fn page_name(p: super::Page) -> String {
    let n = p.name();
    if amenui::i18n::current() == amenui::i18n::Locale::En { super::english::english(&n) } else { n }
}
/// A cell's number, `02.3`.
pub fn number(s: usize, i: usize) -> String { format!("{:02}.{}", s + 1, i + 1) }
/// A page with the label that leads to it.
pub type Link = (super::Page, String);
/// The pages before and after `p` along the whole path, with their labels (`← 02.2 take apart`,
/// `02.4 weld →`; into the next step `03 material: materials →`; after the last, `lab ↑`).
pub fn neighbours(p: super::Page) -> Option<(Option<Link>, Link)> {
    let (s, i) = place(p)?;
    let flat: Vec<(usize, usize, super::Page)> = path().iter().enumerate().flat_map(|(s, v)| v.iter().enumerate().map(move |(i, &p)| (s, i, p))).collect();
    let k = flat.iter().position(|&(a, b, _)| (a, b) == (s, i))?;
    let low = |p: super::Page| cell_name(p).to_lowercase();
    let prev = k.checked_sub(1).map(|j| { let (a, b, q) = flat[j]; (q, format!("← {} {}", number(a, b), low(q))) });
    let next = match flat.get(k + 1) {
        Some(&(a, _, q)) if a != s => (q, format!("{:02} {}: {} →", a + 1, step_name(a).to_lowercase(), low(q))),
        Some(&(a, b, q)) => (q, format!("{} {} →", number(a, b), low(q))),
        None => (super::Page::Lab, format!("{} ↑", amenui::i18n::tr("nav.lab").to_lowercase())),
    };
    Some((prev, next))
}
/// `1` to `5` on the index: to that step.
pub fn key(s: &mut Gallery, k: amenui::el::Key) -> bool {
    let amenui::el::Key::Char(c @ '1'..='5') = k else { return false };
    if s.page != super::Page::Lab { return false; }
    s.cmds.push(amenui::morph::Cmd::ScrollTo { key: format!("labx/{}/head", STEPS[c as usize - '1' as usize]) });
    true
}
/// The step whose section is first in the window (the path line's "you are here").
pub fn here(s: &Gallery) -> usize { s.lab.steps.iter().position(|&v| v).unwrap_or(0) }

/// The path line under the bar on the index: the five steps, each a link to its header; the one the
/// window is in, in the site's brackets and the accent. On a phone the numbers and that one's name.
pub fn path_line(s: &Gallery) -> El<Msg> {
    use amenui::el::row;
    let narrow = s.width < super::NARROW;
    let at = here(s);
    let mut r = row().key("lab/path").fill_width().height(28.0).pad(if narrow { 16.0 } else { 32.0 }, 0.0).gap(24.0).align(taffy::AlignItems::CENTER)
        .fill(Paint::GROUND).calm().material(Material::Plain.id());
    for (i, k) in STEPS.iter().enumerate() {
        let name = step_name(i).to_lowercase();
        let label = if i == at { format!("[ {:02} {name} ]", i + 1) } else if narrow { format!("{:02}", i + 1) } else { format!("{:02} {name}", i + 1) };
        r = r.child(row().key(format!("lab/path/{k}")).height(28.0).align(taffy::AlignItems::CENTER).on_press(Msg::Outline(format!("labx/{k}/head"))).role(amenui::el::Role::Link)
            .child(text(label, 11.0, if i == at { Paint::ACCENT } else { Paint::TEXT }).font(Font::Mono)));
    }
    r
}

/// The index's columns by the window: four at 1280 and over, three from 1080, two from 720, one below.
fn columns(w: f32) -> usize { if w >= 1280.0 { 4 } else if w >= 1080.0 { 3 } else if w >= 720.0 { 2 } else { 1 } }

/// `/lab/`: the title, one line, then the five steps, each its header with the count, its sentence
/// and its cells: one card for all 29 (`[ NAME ]`, the hint, the still; under the frame `02.3  Kern →`).
/// No cell stands alone in a row: when a step leaves one over, its first cell is two columns wide.
pub fn page(s: &Gallery) -> El<Msg> {
    use super::{Page, index};
    use amenui::el::{row, text_lh};
    let tr = amenui::i18n::tr;
    // The index's padding (`main_page`) off the window.
    let room = s.width - if s.width < super::NARROW { 32.0 } else { 64.0 };
    let cols = columns(s.width);
    let width = ((room - 16.0 * (cols as f32 - 1.0)) / cols as f32).floor();
    let stage = ((width - 26.0) * amenui::experiment::STAGE).floor();
    let mut page = col().key("lab").fill_width().min_width(0.0).gap(48.0)
        .child(col().key("labx/top").fill_width().min_width(0.0).gap(12.0)
            .child(text(tr("nav.lab"), 34.0, Paint::INK).font(Font::Display).heading(1).key("labx/title").flow("page-title"))
            .child(text_lh(tr("lab.path.intro"), 14.0, 22.0, Paint::TEXT).key("labx/intro").selectable()));
    for (si, pages) in path().into_iter().enumerate() {
        let k = STEPS[si];
        let mut cells = vec![];
        let wide = cols > 1 && pages.len() % cols == 1;
        for (i, p) in pages.iter().copied().enumerate() {
            let w = if wide && i == 0 { width * 2.0 + 16.0 } else { width };
            let piece = match p {
                Page::Experiment(e) => s.lab.cell(e as usize, &format!("labi/{}", p.hash()), w, stage, true).map(Rc::new(Msg::Lab)),
                other => still_cell(s, &format!("labi/{}", other.hash()), other, w, stage),
            };
            let name = row().key(format!("labx/{}/name", p.hash())).gap(10.0).on_press(Msg::Go(p)).role(amenui::el::Role::Link)
                .child(text(number(si, i), 11.0, Paint::DIM).font(Font::Mono))
                .child(text(format!("{} →", cell_name(p)), 11.0, Paint::TEXT).font(Font::Mono));
            let c = col().key(format!("labx/{}", p.hash())).fill_width().min_width(0.0).gap(8.0).child(piece).child(name);
            cells.push((c, if wide && i == 0 { 2 } else { 1 }));
        }
        let grid = index::rows(&format!("labx/{k}/grid"), cols, cells);
        let head = row().key(format!("labx/{k}/headrow")).fill_width().gap(10.0).align(taffy::AlignItems::CENTER)
            .child(index::header(&format!("labx/{k}/head"), si + 1, &step_name(si)).grow(1.0))
            .child(text(pages.len().to_string(), 11.0, Paint::DIM).font(Font::Mono));
        page = page.child(col().key(format!("labx/{k}")).fill_width().min_width(0.0).gap(16.0)
            .on_visible(move |v| Some(Msg::Lab(LMsg::Step(si, v))))
            .child(head)
            .child(text_lh(tr(&format!("lab.step.{k}.line")), 14.0, 22.0, Paint::TEXT).key(format!("labx/{k}/line")).selectable())
            .child(grid));
    }
    page
}

/// A cell's hint, top right: what to do with it, or a family's count.
fn hint(p: super::Page) -> &'static str {
    use super::Page;
    match p {
        Page::Reader => "open to read",
        Page::Stirr => "drag the window",
        Page::Material(_) => "9 materials",
        Page::Marks(_) => "20 marks",
        Page::Presets => "10 presets",
        Page::Type => "three sizes",
        Page::Gaze => "open to measure",
        Page::Three => "drag to turn",
        Page::Morph => "open to morph",
        Page::Particles => "move to scatter",
        Page::Flow => "open to turn",
        Page::Decode => "open to decode",
        Page::Scene(i) => ["press to run", "drag the picture", "move to reveal", "press to start", "hold space", "press to play", "press to switch"][i as usize % 7],
        _ => "",
    }
}

/// A page with no live cell, in a live cell's frame: `[ NAME ]` and the hint over its still, centred
/// on a stage `stage` px tall and cut at it; pressing it opens the page. A family's cell under the
/// pointer steps through its members.
fn still_cell(s: &Gallery, key: &str, p: super::Page, width: f32, stage: f32) -> El<Msg> {
    use super::Page;
    use amenui::draw::Drawing;
    use amenui::el::{canvas, picture, row, space, text_lh};
    let (w, h) = (width - 26.0, stage);
    let member = |f: usize| { let m = family(f); m[(s.lab.fam_t[f] / FAMILY_S) as usize % m.len()] };
    // The materials' and the presets' still: a specimen in the material, its name under it.
    let specimen = |m: u8, name: String| col().key(format!("{key}/specimen")).gap(8.0).align(taffy::AlignItems::START)
        .child(row().gap(14.0).align(taffy::AlignItems::CENTER).respond(amenui::el::Response::Loosen).legible(false).material(m).loosen(s.loose.max(0.35))
            .child(text("Aa", 44.0, Paint::INK).font(Font::Display).weight(1.5))
            .child(picture(crate::demo::SKY, super::pages::ratio(crate::demo::SKY)).markr(super::IMAGE_SLOT).width(120.0)))
        .child(text(name, 11.0, Paint::DIM).font(Font::Mono));
    let piece = match p {
        Page::Stirr => {
            let mut d = Drawing::new(w, h);
            let f = |x: f32, y: f32| amenui::field::noise(x / 48.0, y / 48.0);
            d.dots(&f, 7.0, |v| (0.4 + 2.6 * v).clamp(0.0, 2.8), |_, _, _| Paint::INK);
            canvas(d).width(w)
        }
        Page::Reader => text_lh(READER_STILL, 11.0, 18.0, Paint::TEXT).font(Font::Mono).width(w.min(420.0)).height(((h / 18.0).floor() * 18.0).min(18.0 * 6.0)),
        Page::Scene(i) => super::scenes::thumb(i as usize, 200.0),
        Page::Material(_) => { let Page::Material(m) = member(0) else { return space(0.0, 0.0) }; specimen(m, Material::from_id(m).name().to_owned()) }
        Page::Presets => { let Page::Style(i) = member(2) else { return space(0.0, 0.0) }; let st = &amenui::style::styles()[i as usize]; specimen(st.material.id(), st.name.to_owned()) }
        Page::Marks(_) => { let Page::Marks(k) = member(1) else { return space(0.0, 0.0) };
            col().gap(8.0).child(picture(crate::demo::SKY, super::pages::ratio(crate::demo::SKY)).markr(super::MARK_SLOT).loose(1.0).width(w.min(240.0)))
                .child(text(super::MARK_HASH[k as usize], 11.0, Paint::DIM).font(Font::Mono)) }
        other => still(other).unwrap_or_else(|| space(0.0, 0.0)),
    };
    // A hint that would not fit beside the name on a narrow cell is left out (11 px mono: about 6.6 px a Latin character, 11 a CJK one).
    let name = format!("[ {} ]", cell_name(p).to_uppercase());
    let wide = |t: &str| t.chars().map(|c| if (c as u32) >= 0x2E80 { 11.0 } else { 6.6 }).sum::<f32>();
    let hint = Some(hint(p)).filter(|h| wide(&name) + wide(h) + 8.0 <= w);
    let stage_el = col().key(format!("{key}/stage")).fill_width().min_width(0.0).height(h).clip().center().child(piece);
    let mut c = col().key(key.to_owned()).fill_width().min_width(0.0).pad(12.0, 10.0).gap(8.0).border(1.0, Paint::FAINT).on_press(Msg::Go(p)).role(amenui::el::Role::Link).label(cell_name(p))
        .child(row().fill_width().gap(8.0)
            .child(text(name, 11.0, Paint::TEXT).font(Font::Mono).key(format!("{key}/name")))
            .child(space(0.0, 0.0).grow(1.0))
            .child(text(hint.unwrap_or_default(), 11.0, Paint::DIM).font(Font::Mono).key(format!("{key}/hint"))))
        .child(stage_el);
    if let Some(f) = family_of(p) { c = c.on_move(move |q| Some(Msg::Lab(LMsg::Fam(f, q.is_some())))); }
    c
}
/// The reader's still: the start of a page set in one face at one measure.
const READER_STILL: &str = "The reader sets a page in one face at one measure. Lines break by whole words, never inside one; a long page turns instead of scrolling, and the outline beside it keeps your place.";

/// An experiment's page: its documentation, then the cell.
pub fn experiment(s: &Gallery, i: usize) -> El<Msg> {
    let name = NAMES[i.min(COUNT - 1)];
    let src = doc(name, s.locale);
    let w = (s.width - if s.width < super::NARROW { 28.0 } else { 80.0 }).clamp(200.0, 720.0);
    let (body, _) = amenui::markdown::render(src, &format!("lab/{name}/doc"), w, &|t| super::docs::link(t));
    col().key(format!("lab/{name}/page")).fill_width().min_width(0.0).max_width(w).gap(18.0)
        .child(s.lab.cell(i, &format!("lab/{name}"), w, ((w - 26.0) * amenui::experiment::STAGE).floor(), false).map(Rc::new(Msg::Lab)))
        .child(body)
}

/// An experiment's documentation in a language.
/// (`docs/site/<lang>/lab-<name>.md`, which the web module fetches: `texts`).
pub fn doc(name: &str, l: amenui::i18n::Locale) -> &'static str {
    let lang = if l == amenui::i18n::Locale::En { "en" } else { "zh-Hans" };
    crate::texts::get(&format!("{lang}/lab-{name}.md")).unwrap_or("")
}

/// What a Lab card shows of its page, drawn small and still in ink: the decode's blocks half way
/// over the grown cell's outline, a morph's tiles in flight from a block to a ring, a burst of
/// particles, one page sliding over another, a cube; a dimension line with its number (annotation),
/// one word at three sizes (type).
fn still(p: super::Page) -> Option<El<Msg>> {
    use super::Page;
    use amenui::draw::Drawing;
    use amenui::el::{canvas, layers, row};
    let (w, h) = (200.0, 96.0);
    let hash = |i: u32, k: u32| (i.wrapping_mul(2_654_435_761) ^ k.wrapping_mul(40_503)).wrapping_mul(2_246_822_519) as f32 / u32::MAX as f32;
    Some(match p {
        Page::Decode => {
            // "Aa" as the grown cell draws it, half decoded: its right half still in blocks of the
            // decode's tiers (24, 12, 6 px), each the tone of what it covers.
            let mut d = Drawing::new(w, h);
            let (x0, x1, y0, y1) = (w * 0.5, w * 0.5 + 54.0, 18.0, 78.0);
            let mut y = y0;
            while y < y1 {
                let mut x = x0;
                while x < x1 {
                    let k = hash((x as u32) * 31 + y as u32, 9);
                    let size: f32 = if k < 0.25 { 24.0 } else if k < 0.7 { 12.0 } else { 6.0 };
                    let size = size.min(y1 - y).min(x1 - x).max(6.0);
                    for r in 0..(size / 2.0) as u32 { let ly = y + 1.0 + r as f32 * 2.0; d.line((x, ly), (x + size, ly), 2.0, Paint::GROUND); }
                    let tone = 0.25 + 0.6 * hash((x as u32) * 7 + y as u32, 11);
                    for r in 0..((size - 2.0) / 2.0).max(1.0) as u32 { let ly = y + 2.0 + r as f32 * 2.0; d.line((x + 1.0, ly), (x + size - 1.0, ly), 2.0, Paint::tone(tone)); }
                    x += size;
                }
                y += 12.0;
            }
            layers([
                col().width(w).height(h).center().child(text("Aa", 76.0, Paint::TEXT).font(Font::Face(0)).weight(2.6)),
                canvas(d).width(w).key("labx/decode/still"),
            ])
        }
        Page::Morph => {
            // Tiles leaving a block on the left for a ring on the right, part of the way there.
            let mut d = Drawing::new(w, h);
            let n = 120u32;
            for i in 0..n {
                let (gx, gy) = ((i % 10) as f32, (i / 10) as f32);
                let from = (18.0 + gx * 5.0, 18.0 + gy * 5.0);
                let a = i as f32 / n as f32 * std::f32::consts::TAU;
                let to = (156.0 + 30.0 * a.cos(), 48.0 + 30.0 * a.sin());
                let u = (hash(i, 1) * 1.3 - 0.15).clamp(0.0, 1.0);
                let e = u * u * (3.0 - 2.0 * u);
                let (x, y) = (from.0 + (to.0 - from.0) * e, from.1 + (to.1 - from.1) * e - 14.0 * (std::f32::consts::PI * e).sin());
                d.line((x - 1.5, y), (x + 1.5, y), 3.0, Paint::tone(0.55 + 0.45 * (1.0 - (2.0 * e - 1.0).abs())));
            }
            canvas(d).width(w).key("labx/morph/still")
        }
        Page::Particles => {
            // A burst from a point: farther out, smaller and fainter.
            let mut d = Drawing::new(w, h);
            for i in 0..160u32 {
                let a = hash(i, 3) * std::f32::consts::TAU;
                let r = 4.0 + 44.0 * hash(i, 5).sqrt();
                let k = 1.0 - r / 52.0;
                d.dot(100.0 + r * 1.6 * a.cos(), 48.0 + r * a.sin(), 0.6 + 1.6 * k, Paint::tone(0.35 + 0.65 * k));
            }
            canvas(d).width(w).key("labx/particles/still")
        }
        Page::Flow => {
            // One page sliding over another: their frames, a title and lines of text.
            let mut d = Drawing::new(w, h);
            let page = |d: &mut Drawing, x: f32, y: f32, t: f32| {
                let (pw, ph) = (92.0, 76.0);
                for (a, b) in [((x, y), (x + pw, y)), ((x + pw, y), (x + pw, y + ph)), ((x + pw, y + ph), (x, y + ph)), ((x, y + ph), (x, y))] { d.line(a, b, 1.0, Paint::tone(t)); }
                d.line((x + 8.0, y + 12.0), (x + 50.0, y + 12.0), 3.0, Paint::tone(t));
                for k in 0..5 { let ly = y + 26.0 + k as f32 * 9.0; d.line((x + 8.0, ly), (x + pw - 8.0 - (k % 2) as f32 * 18.0, ly), 1.0, Paint::tone(t * 0.8)); }
            };
            page(&mut d, 30.0, 14.0, 0.35);
            page(&mut d, 78.0, 6.0, 1.0);
            canvas(d).width(w).key("labx/flow/still")
        }
        Page::Three => {
            // A cube, its hidden edges fainter.
            let mut d = Drawing::new(w, h);
            let (cx, cy, s) = (100.0, 50.0, 30.0);
            let p = |x: f32, y: f32, z: f32| (cx + (x - z) * s * 0.87, cy + (x + z) * s * 0.5 - y * s);
            let v = [(-1.0, -1.0, -1.0), (1.0, -1.0, -1.0), (1.0, 1.0, -1.0), (-1.0, 1.0, -1.0), (-1.0, -1.0, 1.0), (1.0, -1.0, 1.0), (1.0, 1.0, 1.0), (-1.0, 1.0, 1.0)].map(|(x, y, z): (f32, f32, f32)| p(x * 0.5, y * 0.5, z * 0.5));
            for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4), (0, 4), (1, 5), (2, 6), (3, 7)] {
                let hidden = a == 0 || b == 0;
                d.line(v[a], v[b], 1.2, if hidden { Paint::tone(0.3) } else { Paint::INK });
            }
            canvas(d).width(w).key("labx/three/still")
        }
        Page::Gaze => {
            // A dimension line: the extent between two ticks, its number above.
            let mut d = Drawing::new(w, 14.0);
            d.line((1.0, 7.0), (w - 1.0, 7.0), 1.0, Paint::TEXT);
            for x in [1.0, w - 1.0] { d.line((x, 1.0), (x, 13.0), 1.0, Paint::TEXT); }
            col().key("labx/gaze/still").width(w).gap(6.0).align(taffy::AlignItems::CENTER)
                .child(text("200", 11.0, Paint::TEXT).font(Font::Mono))
                .child(canvas(d).width(w))
        }
        Page::Type => row().key("labx/type/still").gap(14.0).align(taffy::AlignItems::BASELINE)
            .child(text("Type", 12.0, Paint::TEXT).font(Font::Display))
            .child(text("Type", 20.0, Paint::TEXT).font(Font::Display))
            .child(text("Type", 34.0, Paint::INK).font(Font::Display)),
        _ => return None,
    })
}
