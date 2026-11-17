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
    ($( $i:literal => $f:ident : $t:ty , $v:ident );* $(;)?) => {
        #[derive(Clone, Debug, PartialEq)]
        pub enum LMsg { Seen(usize, bool), $( $v(<$t as Experiment>::Msg) ),* }
        /// Every experiment's state, and which can be seen.
        #[derive(Clone, Default)]
        pub struct Lab { pub seen: [bool; COUNT], pub until: f32, /** Whose turn it is when only some cells step. */ pub turn: usize, /** The viewer asked for reduced motion: cells rest after `IDLE_S`. */ pub reduced: bool, $( pub $f: $t ),* }
        impl Lab {
            /// Cell `i`, keyed under `key`, telling when it comes into view.
            pub fn cell(&self, i: usize, key: &str) -> El<LMsg> {
                let c = match i { $( $i => self.$f.view(key).map(Rc::new(LMsg::$v)), )* _ => col() };
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

/// The grid: as many columns of at least 340 px as fit (one on a phone).
/// The Experiments group: the ten cells, then the reader, the decode transition and the field map.
pub fn experiments() -> Vec<super::Page> {
    use super::Page;
    (0..COUNT as u8).map(Page::Experiment).chain([Page::Reader, Page::Decode, Page::Stirr]).collect()
}

/// `/lab/`: three grids, Experiments (the live cells, each with its name under it), Looks and Motion
/// (a card per page).
pub fn page(s: &Gallery) -> El<Msg> {
    use super::{Page, index};
    let tr = amenui::i18n::tr;
    let groups = super::lab_groups();
    let room = index::room(s);
    let mut cells = index::grid("lab/grid", room, 340.0);
    for (i, n) in NAMES.iter().enumerate() {
        let p = Page::Experiment(i as u8);
        cells = cells.child(col().key(format!("labx/{n}")).fill_width().min_width(0.0).gap(8.0)
            .child(s.lab.cell(i, &format!("labi/{n}")).map(Rc::new(Msg::Lab)))
            .child(amenui::el::row().key(format!("labx/{n}/name")).gap(10.0).on_press(Msg::Go(p)).role(amenui::el::Role::Link)
                .child(text(format!("{:02}", i + 1), 11.0, Paint::DIM).font(Font::Mono))
                .child(text(format!("{} →", p.name()), 11.0, Paint::TEXT).font(Font::Mono))));
    }
    // The other experiments, the Looks and the Motion pages as cards.
    let mut more = index::grid("labx/more", room, 240.0);
    for (k, p) in groups[0].1.iter().skip(COUNT).enumerate() { more = more.child(index::card(&format!("labx/{}", p.hash()), COUNT + k + 1, p.name(), String::new(), None, *p)); }
    let looks = |s: &Gallery| {
        let mut g = index::grid("labx/looks", room, 240.0);
        let marks: Vec<Page> = groups[1].1.iter().copied().filter(|p| matches!(p, Page::Marks(_))).collect();
        let mats: Vec<Page> = groups[1].1.iter().copied().filter(|p| matches!(p, Page::Material(_))).collect();
        let styles: Vec<Page> = groups[1].1.iter().copied().filter(|p| matches!(p, Page::Style(_))).collect();
        g = g.child(index::card_links("labx/look0", 1, Page::Presets.name(), styles, Page::Presets))
            .child(index::card_links("labx/look1", 2, tr("nav.marks"), marks.clone(), marks[0]))
            .child(index::card_links("labx/look2", 3, tr("nav.materials"), mats.clone(), mats[0]))
            .child(index::card("labx/look3", 4, Page::Gaze.name(), String::new(), None, Page::Gaze))
            .child(index::card("labx/look4", 5, Page::Type.name(), String::new(), None, Page::Type));
        let _ = s;
        g
    };
    let mut motion = index::grid("labx/motion", room, 240.0);
    for (k, p) in groups[2].1.iter().enumerate() { motion = motion.child(index::card(&format!("labx/{}", p.hash()), k + 1, p.name(), String::new(), None, *p)); }
    let sec = |k: &str, n: usize, name: &str, body: El<Msg>| col().key(format!("labx/{k}")).fill_width().min_width(0.0).gap(16.0).child(index::header(&format!("labx/{k}/head"), n, name)).child(body);
    col().key("lab").fill_width().min_width(0.0).gap(28.0)
        .child(text(tr("nav.lab"), 34.0, Paint::INK).font(Font::Display).heading(1).key("labx/title").flow("page-title"))
        .child(sec("s1", 1, &groups[0].0, col().key("labx/s1/body").fill_width().min_width(0.0).gap(16.0).child(cells).child(more)))
        .child(sec("s2", 2, &groups[1].0, looks(s)))
        .child(sec("s3", 3, &groups[2].0, motion))
}

/// An experiment's page: its documentation, then the cell.
pub fn experiment(s: &Gallery, i: usize) -> El<Msg> {
    let name = NAMES[i.min(COUNT - 1)];
    let src = doc(name, s.locale);
    let w = (s.width - if s.width < super::NARROW { 28.0 } else { 80.0 }).clamp(200.0, 720.0);
    let (body, _) = amenui::markdown::render(src, &format!("lab/{name}/doc"), w, &|t| super::docs::link(t));
    col().key(format!("lab/{name}/page")).fill_width().min_width(0.0).max_width(w).gap(18.0)
        .child(s.lab.cell(i, &format!("lab/{name}")).map(Rc::new(Msg::Lab)))
        .child(body)
}

/// An experiment's documentation in a language.
pub fn doc(name: &str, l: amenui::i18n::Locale) -> &'static str {
    let en = l == amenui::i18n::Locale::En;
    macro_rules! pick { ($($n:literal),*) => { match name { $( $n => if en { include_str!(concat!("../../docs/site/en/lab-", $n, ".md")) } else { include_str!(concat!("../../docs/site/zh-Hans/lab-", $n, ".md")) }, )* _ => "" } } }
    pick!("lumen", "contours", "silk", "grown", "mercury", "flow", "loupe", "orbit", "spring", "stack")
}
