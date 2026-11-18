//! The Lab (DESIGN.md §5.3): a grid of small live cells, one experiment each (`examples/lab/`), in the
//! lab's look (near-black ground, grey-white hairlines and dots, one blue accent where the pointer
//! reaches). A cell steps only while it can be seen; an experiment's own page shows it larger with its
//! documentation (`docs/site/<language>/lab-<name>.md`).
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
    ($( $i:literal => $f:ident : $t:ty , $v:ident );* $(;)?) => {
        #[derive(Clone, Debug, PartialEq)]
        pub enum LMsg { Seen(usize, bool), $( $v(<$t as Experiment>::Msg) ),* }
        /// Every experiment's state, and which can be seen.
        #[derive(Clone, Default)]
        pub struct Lab { pub seen: [bool; COUNT], pub until: f32, /** Whose turn it is when only some cells step. */ pub turn: usize, /** The viewer asked for reduced motion: cells rest after `IDLE_S`. */ pub reduced: bool, $( pub $f: $t ),* }
        impl Lab {
            /// Cell `i`, keyed under `key`, telling when it comes into view, `width` px wide (Lumen's
            /// text fills its stage in whole lines of that width); on the index (`index`) only its name,
            /// its hint and the piece (the formula and the label are on its page).
            pub fn cell(&self, i: usize, key: &str, width: f32, index: bool) -> El<LMsg> {
                let lumen = x::lumen::Lumen { rows: lumen_rows(width), ..self.lumen.clone() };
                let stack = if index && self.stack.deck.under().is_none() && !self.stack.deck.moving() { x::stack::Stack { deck: fanned(&self.stack.deck), quiet: true, ..self.stack.clone() } } else { self.stack.clone() };
                let (lumen, stack) = (&lumen, &stack);
                let mut c = match i { $( $i => cells!(@view self, $f, lumen, stack).view(key).map(Rc::new(LMsg::$v)), )* _ => col() };
                if index { c.children.truncate(2); }
                // A living piece moves on purpose while it is seen.
                if !self.reduced { c = c.ongoing(); }
                // The same cell on the index and on its page: it flies from one to the other.
                c.on_visible(move |v| Some(LMsg::Seen(i, v))).flow(format!("lab-cell-{i}"))
            }
            pub fn update(&mut self, m: LMsg, now: f32) {
                if !matches!(m, LMsg::Seen(..)) { self.until = now + amenui::experiment::IDLE_S; }
                match m { LMsg::Seen(i, v) => self.seen[i] = v, $( LMsg::$v(m) => self.$f.update(m, now), )* }
            }
            /// Step what can be seen, every frame (the living pieces); with motion reduced only for
            /// `IDLE_S` after input and while settling. Whether the view changed.
            /// The pieces draw from the frame budget: when frames run over it (`budget::usage().scale` < 1),
            /// only that share of the seen cells steps each frame, in turn, so the page's cost falls with it.
            pub fn step(&mut self, now: f32) -> bool {
                let live = !self.reduced || now < self.until;
                let seen = self.seen.iter().filter(|&&v| v).count();
                let share = ((seen as f32 * amenui::budget::usage().scale).ceil() as usize).clamp(1, COUNT);
                let (mut c, mut k) = (false, 0usize);
                self.turn = (self.turn + 1) % COUNT;
                $( if self.seen[$i] && (live || self.$f.moving()) {
                    k += 1;
                    let mine = share >= seen || ((k - 1 + self.turn) % seen.max(1)) < share;
                    if mine { c |= self.$f.step(now); }
                } )*
                c
            }
            pub fn moving(&self, now: f32) -> bool { let live = !self.reduced || now < self.until; false $( || (self.seen[$i] && (live || self.$f.moving())) )* }
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

/// How many whole lines of Lumen's text fill a cell's stage `width` px wide (the cell's padding and
/// border off, the stage `STAGE` as tall as it is wide).
fn lumen_rows(width: f32) -> usize { (((width - 26.0) * amenui::experiment::STAGE) / x::lumen::LINE).floor().max(1.0) as usize }
/// The stack on the index at rest: fanned under its top card, settled (a hover fans it live).
pub fn fanned(d: &amenui::widgets::Stack) -> amenui::widgets::Stack {
    let mut f = d.clone();
    f.update(amenui::widgets::StackMsg::Press(Some(0)), 0.0);
    let mut t = 0.0;
    while f.moving() && t < 5.0 { t += 1.0 / 60.0; f.step(t); }
    f
}

pub fn update(s: &mut Gallery, m: LMsg) { let now = s.clock; s.lab.update(m, now); }
/// Step the cells that can be seen, on the Lab and on an experiment's page.
pub fn tick(s: &mut Gallery, t: f32) -> bool { on_lab(s) && s.lab.step(t) }
pub fn wake(s: &Gallery, t: f32) -> Option<f32> { (on_lab(s) && s.lab.moving(t)).then_some(t) }
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

/// The Experiments group: the ten cells, then the field map and the reader.
pub fn experiments() -> Vec<super::Page> {
    use super::Page;
    (0..COUNT as u8).map(Page::Experiment).chain([Page::Stirr, Page::Reader]).collect()
}

/// The Experiments grid's columns: at least this wide (four at 1440, three at 1146, one on a phone).
const CELL_MIN: f32 = 300.0;

/// `/lab/`: three grids. Experiments: twelve cells of one size, each its name, its hint and its
/// piece (the ten live ones, then the field map and the reader as a still in the same frame), with its
/// name under it; Looks and Motion: a card per page, every name on one baseline.
pub fn page(s: &Gallery) -> El<Msg> {
    use super::{Page, index};
    use amenui::el::row;
    let tr = amenui::i18n::tr;
    let groups = super::lab_groups();
    let room = index::room(s);
    let cols = index::columns(room, CELL_MIN) as f32;
    let width = (room - 16.0 * (cols - 1.0)) / cols;
    let mut cells = index::grid("lab/grid", room, CELL_MIN);
    let name_row = |k: String, n: usize, p: Page| row().key(k).gap(10.0).on_press(Msg::Go(p)).role(amenui::el::Role::Link)
        .child(text(format!("{n:02}"), 11.0, Paint::DIM).font(Font::Mono))
        .child(text(format!("{} →", p.name()), 11.0, Paint::TEXT).font(Font::Mono));
    for (k, p) in groups[0].1.iter().enumerate() {
        let (key, piece) = match *p {
            Page::Experiment(i) => (NAMES[i as usize].to_owned(), s.lab.cell(i as usize, &format!("labi/{}", NAMES[i as usize]), width, true).map(Rc::new(Msg::Lab))),
            other => (other.hash(), still_cell(&format!("labi/{}", other.hash()), other, width)),
        };
        cells = cells.child(col().key(format!("labx/{key}")).fill_width().min_width(0.0).gap(8.0).child(piece).child(name_row(format!("labx/{key}/name"), k + 1, *p)));
    }
    let looks = {
        let mut g = index::grid("labx/looks", room, 240.0);
        let marks: Vec<Page> = groups[1].1.iter().copied().filter(|p| matches!(p, Page::Marks(_))).collect();
        let mats: Vec<Page> = groups[1].1.iter().copied().filter(|p| matches!(p, Page::Material(_))).collect();
        let styles: Vec<Page> = groups[1].1.iter().copied().filter(|p| matches!(p, Page::Style(_))).collect();
        g = g.child(index::card_links("labx/look0", 1, Page::Presets.name(), styles, Page::Presets))
            .child(index::card_links("labx/look1", 2, tr("nav.marks"), marks.clone(), marks[0]))
            .child(index::card_links("labx/look2", 3, tr("nav.materials"), mats.clone(), mats[0]))
            .child(index::card_named("labx/look3", 4, Page::Gaze.name(), still(Page::Gaze), Page::Gaze))
            .child(index::card_named("labx/look4", 5, Page::Type.name(), still(Page::Type), Page::Type));
        g
    };
    let mut motion = index::grid("labx/motion", room, 240.0);
    for (k, p) in groups[2].1.iter().enumerate() { motion = motion.child(index::card(&format!("labx/motion/{}", p.hash()), k + 1, p.name(), String::new(), still(*p), *p)); }
    // The scenes: demonstrations of mechanisms, the Lab's fourth grid (M.1).
    let mut scenes = index::grid("labx/scenes", room, 240.0);
    for (k, sc) in super::SCENES.iter().enumerate() {
        let p = Page::Scene(k as u8);
        scenes = scenes.child(index::card(&format!("labx/scenes/{}", p.hash()), k + 1, p.name(), sc.2.to_owned(), Some(super::scenes::thumb(k, 200.0)), p));
    }
    let sec = |k: &str, n: usize, name: &str, body: El<Msg>| col().key(format!("labx/{k}")).fill_width().min_width(0.0).gap(16.0).child(index::header(&format!("labx/{k}/head"), n, name)).child(body);
    col().key("lab").fill_width().min_width(0.0).gap(28.0)
        .child(text(tr("nav.lab"), 34.0, Paint::INK).font(Font::Display).heading(1).key("labx/title").flow("page-title"))
        .child(sec("s1", 1, &groups[0].0, cells))
        .child(sec("s2", 2, &groups[1].0, looks))
        .child(sec("s3", 3, &groups[2].0, motion))
        .child(sec("s4", 4, &tr("nav.scenes"), scenes))
}

/// A page with no live cell, in an Experiments cell's frame: `[ NAME ]` and the hint over its still,
/// drawn at the stage's size; pressing it opens the page.
fn still_cell(key: &str, p: super::Page, width: f32) -> El<Msg> {
    use super::Page;
    use amenui::draw::Drawing;
    use amenui::el::{canvas, row, space, text_lh};
    let (w, h) = (width - 26.0, ((width - 26.0) * amenui::experiment::STAGE).floor());
    let (name, hint, piece) = match p {
        Page::Stirr => {
            let mut d = Drawing::new(w, h);
            let f = |x: f32, y: f32| amenui::field::noise(x / 48.0, y / 48.0);
            d.dots(&f, 7.0, |v| (0.4 + 2.6 * v).clamp(0.0, 2.8), |_, _, _| Paint::INK);
            ("field map", "drag the window", canvas(d).width(w).key(format!("{key}/still")))
        }
        _ => ("reader", "open to read", text_lh(READER_STILL, 11.0, 18.0, Paint::TEXT).font(Font::Mono).width(w).height(((h / 18.0).floor() * 18.0).min(18.0 * 6.0)).key(format!("{key}/still"))),
    };
    let mut stage = col().key(format!("{key}/stage")).fill_width().min_width(0.0).clip().child(piece);
    stage.style.aspect_ratio = Some(1.0 / amenui::experiment::STAGE);
    col().key(key.to_owned()).fill_width().min_width(0.0).pad(12.0, 10.0).gap(8.0).border(1.0, Paint::FAINT).on_press(Msg::Go(p)).role(amenui::el::Role::Link).label(p.name())
        .child(row().fill_width().gap(8.0)
            .child(text(format!("[ {} ]", name.to_uppercase()), 11.0, Paint::TEXT).font(Font::Mono).key(format!("{key}/name")))
            .child(space(0.0, 0.0).grow(1.0))
            .child(text(hint, 11.0, Paint::DIM).font(Font::Mono).key(format!("{key}/hint"))))
        .child(stage)
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
        .child(s.lab.cell(i, &format!("lab/{name}"), w, false).map(Rc::new(Msg::Lab)))
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
