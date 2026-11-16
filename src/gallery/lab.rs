//! The Lab (DESIGN.md §5.3): a grid of small live cells, one experiment each (`examples/lab/`), in the
//! lab's look (near-black ground, grey-white hairlines and dots, one blue accent where the pointer
//! reaches). A cell steps only while it can be seen; an experiment's own page shows it larger with its
//! documentation (`docs/site/<language>/lab-<name>.md`).
use super::{Gallery, Msg};
use crate::el::{El, Font, Paint, col, text};
use crate::experiment::Experiment;
use crate::recipes::lab as x;
use crate::style::{LAB, Look, Material};
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
        pub struct Lab { pub seen: [bool; COUNT], pub until: f32, /** The viewer asked for reduced motion: cells rest after `IDLE_S`. */ pub reduced: bool, $( pub $f: $t ),* }
        impl Lab {
            /// Cell `i`, keyed under `key`, telling when it comes into view.
            pub fn cell(&self, i: usize, key: &str) -> El<LMsg> {
                let c = match i { $( $i => self.$f.view(key).map(Rc::new(LMsg::$v)), )* _ => col() };
                c.on_visible(move |v| Some(LMsg::Seen(i, v)))
            }
            pub fn update(&mut self, m: LMsg, now: f32) {
                if !matches!(m, LMsg::Seen(..)) { self.until = now + crate::experiment::IDLE_S; }
                match m { LMsg::Seen(i, v) => self.seen[i] = v, $( LMsg::$v(m) => self.$f.update(m, now), )* }
            }
            /// Step what can be seen, every frame (the living pieces); with motion reduced only for
            /// `IDLE_S` after input and while settling. Whether the view changed.
            pub fn step(&mut self, now: f32) -> bool { let live = !self.reduced || now < self.until; let mut c = false; $( if self.seen[$i] && (live || self.$f.moving()) { c |= self.$f.step(now); } )* c }
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
pub fn enter(s: &mut Gallery) { s.lab.until = s.clock + crate::experiment::IDLE_S; }
fn on_lab(s: &Gallery) -> bool { matches!(s.page, super::Page::Lab | super::Page::Experiment(_)) }

/// The lab's look on its pages: its theme, plain material, nothing else.
pub fn look(s: &Gallery, l: &mut Look) {
    if !on_lab(s) { return; }
    l.theme = LAB;
    l.material = Material::Plain;
    l.post = Look::default().post;
}

/// The grid: as many columns of at least 340 px as fit (one on a phone).
pub fn page(s: &Gallery) -> El<Msg> {
    let tr = crate::i18n::tr;
    let mut grid = col().key("lab/grid").fill_width().min_width(0.0);
    grid.style.display = taffy::Display::Grid;
    // As many columns of at least 340 px as the page has room for; one on a phone.
    let room = s.width - if s.width < super::NARROW { 28.0 } else { 300.0 };
    let cols = ((room + 14.0) / (340.0 + 14.0)).floor().max(1.0) as usize;
    grid.style.grid_template_columns = vec![taffy::prelude::minmax(taffy::prelude::length(0.0), taffy::prelude::fr(1.0)); cols];
    grid.style.gap = taffy::prelude::Size { width: taffy::prelude::length(14.0), height: taffy::prelude::length(14.0) };
    for (i, n) in NAMES.iter().enumerate() { grid = grid.child(s.lab.cell(i, &format!("lab/{n}")).map(Rc::new(Msg::Lab))); }
    col().key("lab").fill_width().min_width(0.0).gap(18.0)
        .child(text(tr("lab.title"), 13.0, Paint::TEXT).font(Font::Mono).key("lab/title"))
        .child(text(tr("lab.intro"), 11.0, Paint::DIM).font(Font::Mono).key("lab/intro"))
        .child(grid)
}

/// An experiment's page: its documentation, then the cell.
pub fn experiment(s: &Gallery, i: usize) -> El<Msg> {
    let name = NAMES[i.min(COUNT - 1)];
    let src = doc(name, s.locale);
    let w = (s.width - if s.width < super::NARROW { 28.0 } else { 300.0 }).clamp(200.0, 720.0);
    let (body, _) = crate::markdown::render(src, &format!("lab/{name}/doc"), w, &|t| super::docs::link(t));
    col().key(format!("lab/{name}/page")).fill_width().min_width(0.0).max_width(w).gap(18.0)
        .child(s.lab.cell(i, &format!("lab/{name}")).map(Rc::new(Msg::Lab)))
        .child(body)
}

/// An experiment's documentation in a language.
pub fn doc(name: &str, l: crate::i18n::Locale) -> &'static str {
    let en = l == crate::i18n::Locale::En;
    macro_rules! pick { ($($n:literal),*) => { match name { $( $n => if en { include_str!(concat!("../../docs/site/en/lab-", $n, ".md")) } else { include_str!(concat!("../../docs/site/zh-Hans/lab-", $n, ".md")) }, )* _ => "" } } }
    pick!("lumen", "contours", "silk", "grown", "mercury", "flow", "loupe", "orbit", "spring", "stack")
}
