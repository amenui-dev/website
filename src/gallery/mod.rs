//! The gallery: the main application, written with the framework only. A left rail lists the
//! components, the mark sets and the type; each component has a page with a live preview, every
//! variant, its own numbers and its source. The top bar switches theme and material; the switch is
//! committed from the control that was pressed. The old skeleton demo is one of the pages.
pub mod body;
pub mod craft;
pub mod lab;
pub mod decode;
pub mod stirr;
pub mod docs;
pub mod mechanism;
mod looks;
mod pages;
mod scenes;
mod styles;
pub mod three;
pub mod works;
mod touch;
pub mod nightly;
pub mod particles;
pub mod presets;
pub mod routes;
pub mod english;
mod index;

use crate::demo::{self, Demo};
use amenui::el::{El, Paint, col, row, space, text};
use amenui::morph::{Cmd, Ease, FlowSpec, MorphSpec};
use amenui::style::{GAZE_MODES, Gaze, GazeMode, Look, MATERIAL_COUNT, MATERIALS, Material, MaterialParams, Markr, THEMES, Theme, gaze, set, styles};
use amenui::text::Face;
use amenui::widgets::{Edit, SELECTED, Spec, TextEdit, Variant, button_as, divider, menu, skip_link, vdivider};
use std::rc::Rc;
use taffy::AlignItems;

pub use scenes::{SCENES, SMsg, Scenes};
pub use routes::Section;
/// The component indices in `COMPONENTS`.
pub mod comp { pub use super::pages::{BUTTON, CARD, CHOICE, DIALOG, DIVIDER, IMAGE, INPUT, LIST, NUMBER, SCROLL, SLIDER, SWITCH, TABS}; }
pub use pages::{COMPONENTS, Comp, FLOW_PARAMS, MORPH_NAMES, MORPH_PARAMS, ParamDef, flow_slot, hold_still, morph_slot};

/// Below this width (logical px) the rail becomes a row of tabs under the top bar.
pub const NARROW: f32 = 720.0;
pub const RAIL: f32 = 232.0;
/// The height of the page's HTML navigation bar, which a scene's canvas lies under.
pub const BAR: f32 = 48.0;
/// A change of page (AA.11): the old content fades out over this (ms), the new in over this, and what
/// is the same object on both pages travels over this.
pub const PAGE_OUT: f32 = 90.0;
pub const PAGE_IN: f32 = 140.0;
pub const PAGE_TRAVEL: f32 = 240.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page { Components, Widget(u8), Comp(usize), LongList, Marks(u8), Material(u8), Gaze, Style(u8), Morph, Particles, Flow, Three, Presets, Type, Demo, Scenes, Scene(u8), Body, Lab, Experiment(u8), Reader, Decode, Stirr, Works, Work(u8) }

pub const MARK_HASH: [&str; set::COUNT] = ["chars", "dither", "pixel", "mosaic", "brick", "dots", "cross", "diagonal", "diamond", "lines", "glow", "stroke", "led", "braille", "cubes", "hex", "stipple", "hatch", "truchet", "topo"];

impl Page {
    /// Every page, in rail order.
    pub fn all() -> Vec<Page> {
        let mut v: Vec<Page> = vec![Page::Body, Page::Components];
        v.extend(comp_pages());
        v.extend((0..docs::widget_names().len() as u8).map(Page::Widget));
        v.push(Page::Lab);
        v.extend(lab::experiments());
        v.extend(set::FAMILIES.iter().flat_map(|f| f.1.iter().copied()).map(Page::Marks));
        v.extend((0..MATERIAL_COUNT as u8).map(Page::Material));
        v.push(Page::Gaze);
        v.extend((0..styles().len() as u8).map(Page::Style));
        v.push(Page::Presets);
        v.extend([Page::Morph, Page::Particles, Page::Flow, Page::Three, Page::Decode]);
        v.push(Page::Type);
        v.push(Page::Demo);
        v.extend((0..scenes::COUNT as u8).map(Page::Scene));
        v.push(Page::Works);
        // A piece that has no page yet (N.2 item 1): its address goes on to the index.
        v.extend((0..works::COUNT as u8).filter(|&i| works::PIECES[i as usize].page).map(Page::Work));
        v
    }
    /// The page's address on the web: plain ASCII words, so a page can be linked.
    pub fn hash(self) -> String {
        match self {
            Page::Components => "components".into(),
            Page::Widget(i) => docs::page_hash(&docs::widget_names()[i as usize]),
            Page::Comp(i) => COMPONENTS[i].hash.to_owned(),
            Page::LongList => "list-10k".into(),
            Page::Marks(s) => format!("marks-{}", MARK_HASH[s as usize]),
            Page::Material(m) => format!("material-{}", Material::from_id(m).name()),
            Page::Gaze => "gaze".into(),
            Page::Style(i) => format!("style-{}", styles()[i as usize].hash),
            Page::Morph => "morph".into(),
            Page::Particles => "particles".into(),
            Page::Flow => "flow".into(),
            Page::Three => "three".into(),
            Page::Presets => "styles".into(),
            Page::Type => "type".into(),
            Page::Demo => "demo".into(),
            Page::Scenes => "scenes".into(),
            Page::Body => "body".into(),
            Page::Scene(i) => scenes::hash(i as usize),
            Page::Lab => "lab".into(),
            Page::Experiment(i) => format!("lab-{}", lab::NAMES[i as usize]),
            Page::Reader => "lab-reader".into(),
            Page::Decode => "decode".into(),
            Page::Stirr => "stirr".into(),
            Page::Works => "works".into(),
            Page::Work(i) => format!("work-{}", works::PIECES[i as usize].slug),
        }
    }
    /// From `#name` or `name`; an empty hash is the first page.
    pub fn from_hash(h: &str) -> Option<Page> {
        let h = h.trim_start_matches('#');
        if h.is_empty() || h == "gallery" { return Some(Page::Components); }
        Page::all().into_iter().find(|p| p.hash() == h).or_else(|| {
            // A Lab piece's hash before it was re-made under a new name.
            let new = lab::RENAMED.iter().find(|(old, _)| h.strip_prefix("lab-") == Some(*old))?.1;
            lab::NAMES.iter().position(|n| *n == new).map(|i| Page::Experiment(i as u8))
        })
    }
    /// The page's name in the current language.
    pub fn name(self) -> String {
        let l = amenui::i18n::current();
        let en = l == amenui::i18n::Locale::En;
        let zh = self.name_zh();
        match self {
            Page::Lab => amenui::i18n::tr("nav.lab"),
            Page::Body => amenui::i18n::tr("nav.body"),
            Page::Experiment(i) => { let n = lab::NAMES[i as usize]; n[..1].to_uppercase() + &n[1..] }
            Page::Decode => amenui::i18n::tr("nav.decode"),
            Page::Stirr => amenui::i18n::tr("nav.stirr"),
            Page::Components => amenui::i18n::tr("nav.components"),
            Page::Works => amenui::i18n::tr("nav.works"),
            // A piece's title is English in both languages (M.1).
            Page::Work(i) => works::PIECES[i as usize].title.to_owned(),
            Page::Reader => amenui::i18n::tr("nav.reader"),
            Page::Widget(i) => { let n = docs::widget_names()[i as usize].clone(); if en { n.replace('_', " ") } else { docs::items().into_iter().find(|it| it.name == n).map_or(n, |it| it.title) } }
            Page::Comp(i) if en => docs::COMP_NAMES[i].replace('-', " "),
            _ if en => match self {
                Page::LongList => "10,000 rows".into(), Page::Gaze => "Annotation".into(), Page::Morph => "Morph".into(), Page::Particles => "Particles".into(),
                Page::Flow => "Page change".into(), Page::Three => "3D".into(), Page::Presets => "Switching styles".into(), Page::Type => "Type".into(),
                Page::Demo => "Skeleton demo".into(), Page::Scenes => "All scenes".into(), _ => zh.to_owned(),
            },
            _ => zh.to_owned(),
        }
    }
    fn name_zh(self) -> &'static str {
        match self {
            Page::Components | Page::Reader | Page::Widget(_) | Page::Lab | Page::Experiment(_) | Page::Decode | Page::Stirr | Page::Body | Page::Works | Page::Work(_) => "",
            Page::Comp(i) => COMPONENTS[i].title,
            Page::LongList => "一万行",
            Page::Marks(s) => set::NAMES[s as usize],
            Page::Material(m) => Material::from_id(m).name(),
            Page::Gaze => "标注层",
            Page::Style(i) => styles()[i as usize].name,
            Page::Morph => "变形",
            Page::Particles => "粒子",
            Page::Flow => "换页",
            Page::Three => "三维",
            Page::Presets => "风格切换",
            Page::Type => "字号与字体",
            Page::Demo => "骨架演示",
            Page::Scenes => "全部场景",
            Page::Scene(i) => SCENES[i as usize].1,
        }
    }
}

/// A number on some page, for routing its edits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// Parameter `i` of component `c`.
    Param(usize, usize),
    /// `Markr` field `i` of mark set `s` (see `demo::PIC_PARAMS`).
    Pic(u8, usize),
    Wipe,
    /// The live number on the number page.
    Value,
    /// Morph parameter `i` (see `MORPH_PARAMS`), flow parameter `i` (`FLOW_PARAMS`).
    Morph(usize),
    Flow(usize),
    /// Number `i` of the 3D page (`three::PARAMS`), of the particle page (`particles::PARAMS`).
    Three(usize),
    Parts(usize),
    /// Number `i` of the target preset on the 风格 page (`presets::editable`).
    Preset(usize),
    /// Number `i` of material `m` (see `style::MAT_PARAMS`), and the looseness the material pages hold
    /// their specimen at.
    Mat(u8, usize),
    Loose,
    /// The annotation page's density and draw-on time.
    Density,
    DrawMs,
    /// The readouts' numbers: waveform height and base line, the level bar's full mark.
    OscAmp,
    OscY,
    LevelFull,
    /// A number on a style page (its own meaning per page).
    StyleNum(usize),
    /// Number `i` of scene `s` (see `scenes::PARAMS`), and the number on the opening scene's page.
    Scene(usize, usize),
    OpValue,
    /// Number `i` of the button page's ripple section (`pages::RIPPLE_PARAMS`).
    Rip(usize),
    /// Number `i` of the list page's scroll section (`pages::SCROLL_PARAMS`).
    Scroll(usize),
    /// Number `i` of interaction preview `p` (`craft`).
    Craft(usize, usize),
    /// A variant: shown, but its value does not change.
    Shown,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Msg {
    Go(Page),
    /// Open or close a group of the Components rail; open or close the rail's sheet on a phone.
    RailGroup(usize),
    Sheet(bool),
    /// Switch the language (DESIGN.md §1.20): everything is laid out again; keyed text recomposes.
    Locale(amenui::i18n::Locale),
    /// A component page's 预览 / 代码 tab; an outline entry (scroll to that key); a copy button (its
    /// text for the clipboard).
    DocTab(usize),
    Outline(String),
    Copy(String),
    Theme(usize),
    Material(usize),
    Reset,
    Num(Target, Edit),
    Nothing,
    Press,
    Text(TextEdit),
    /// The multi-line field on the input page.
    Notes(TextEdit),
    /// The ten thousand rows: select row `i`; flip row `i`'s switch.
    LongPick(usize),
    LongToggle(usize),
    DialogText(TextEdit),
    Switch,
    Level(f32),
    Tab(usize),
    Pick(usize),
    Row(usize),
    Dialog(u8),
    Show(u32),
    Wipe(f32),
    Demo(demo::Msg),
    /// Morph what the stage shows into candidate `i`; back into what it was; the curve of the flight.
    MorphTo(usize),
    MorphBack,
    Ease(usize),
    /// Open a card of the page-change page, or go back to the list.
    Open(Option<usize>),
    /// Open the look strip's material menu (2), or close it (0).
    Menu(u8),
    /// Apply style preset `i` (theme, material, its numbers, annotation, type).
    Style(usize),
    /// The annotation page: mode, and one kind on or off.
    GazeMode(usize),
    GazeKind(u16),
    /// A choice on a style page: slot, index.
    Choose(usize, usize),
    /// A toggle on a style page.
    Toggle(usize),
    /// The finishing pass's black-and-white switch (0) or its inversion (1), on the Loose Thread page.
    PostFlag(u8),
    /// The pointer over the Loose Thread composition (logical px from its top left), or gone.
    ThreadAt(Option<(f32, f32)>),
    /// The launcher's query (cyberminimalism).
    Query(TextEdit),
    /// A slider on a style page.
    Slide(usize, f32),
    /// Something on a scene page.
    Scene(SMsg),
    /// The ripple section: the counter, its scope, and ten fast presses of it.
    RipCount,
    RipScope(usize),
    RipBurst,
    /// Something on the 3D page, on the particle page.
    Three(three::TMsg),
    Parts(particles::PMsg),
    /// Something on the 风格 page.
    Preset(presets::PreMsg),
    /// Something in an interaction preview (`craft`).
    Craft(craft::CMsg),
    /// Something in a Lab cell.
    Lab(lab::LMsg),
    Body(crate::showcase::scenes::body::Msg),
    /// Something on the decode transition's page.
    Decode(decode::DMsg),
    Stirr(stirr::SMsg),
    /// Something on the Works index or in a piece.
    Works(works::WMsg),
}

pub struct Gallery {
    pub page: Page,
    pub theme: usize,
    pub material: Material,
    /// Each component's own numbers, in the order of its `params`.
    pub params: Vec<Vec<f32>>,
    pub edit: Option<(Target, String)>,
    pub presses: u32,
    pub value: f32,
    pub text: TextEdit,
    pub dialog_text: TextEdit,
    pub on: bool,
    pub level: f32,
    pub tab: usize,
    pub pick: usize,
    pub row: Option<usize>,
    /// Which dialog is open (0 = none).
    pub dialog: u8,
    /// One set of mark parameters per mark page, the picture they show, and the compare line.
    pub marks: [Markr; set::COUNT],
    pub shown: u32,
    pub wipe: f32,
    pub demo: Demo,
    /// The window's logical width, kept up to date by the runtime (`Ui::on_resize`).
    pub width: f32,
    pub height: f32,
    /// The morph page: what the stage shows, what it showed before, how morphs go.
    pub shape: usize,
    pub shape_before: Option<usize>,
    pub morph: MorphSpec,
    /// The page-change page: which card is open, and how shared parts flow (on every page change).
    pub opened: Option<usize>,
    pub flow: FlowSpec,
    /// Commands for the runtime, taken after every update.
    pub cmds: Vec<Cmd>,
    /// The language everything is shown in.
    pub locale: amenui::i18n::Locale,
    /// The documentation: the component page's tab, the outline entry in view, and text a copy button
    /// asked the host to put on the clipboard.
    pub doc_tab: usize,
    pub outline_at: usize,
    /// The Components rail's open group when one was opened or closed by hand on this page (else the
    /// page's own), and whether its sheet is open on a phone.
    pub rail_open: Option<Option<usize>>,
    pub sheet: bool,
    /// The page's stage alone, for a live stage on a docs page (`<path>#stage`).
    pub stage_only: bool,
    /// The stage drawn for its still (`<path>#still`): a specimen that is closed at rest is shown in
    /// the state that shows it (R.2 item 8); the live stage starts closed.
    pub still: bool,
    /// The page before this one; where the pointer pressed the link of an in-app navigation (CSS px
    /// on the canvas); whether the app has been routed once (the first route is the page loading, not
    /// a navigation).
    pub from: Page,
    pub pressed_at: Option<(f32, f32)>,
    pub routed: bool,
    pub clipboard: Option<String>,
    /// Which of the look strip's menus is open (0 = none).
    pub menu: u8,
    /// The material the look strip gives the specimen's stage (and only it).
    pub stage_material: Material,
    /// Every material's numbers; the looseness the material pages hold their specimen at.
    pub mats: [MaterialParams; MATERIAL_COUNT],
    pub loose: f32,
    /// The style applied last, if any, and what it set: annotation, looseness scale, picture marks.
    pub style: Option<usize>,
    pub gaze: Gaze,
    pub loose_scale: f32,
    pub style_pic: Markr,
    /// The annotation page's own settings.
    pub gaze_page: Gaze,
    /// What the pointer is over (kept by the runtime), and the clock (for things that tick).
    pub hovered: Option<String>,
    pub clock: f32,
    /// State of the style pages' controls.
    pub choices: [usize; 8],
    pub toggles: [bool; 8],
    pub slides: [f32; 8],
    pub query: TextEdit,
    /// The showcase scenes' state.
    pub sc: Scenes,
    /// The performance readout (F2, or the status line): 0 off, 1 one line, 2 every number; and the
    /// material resolution (0 = automatic).
    pub overlay: u8,
    pub quality: f32,
    /// The page around the canvas has the HTML status line, which carries the one-line readout's
    /// numbers: the canvas draws the full readout only (AA.6).
    pub status_line: bool,
    /// The palette: 0 the site's, 1 and 2 the proposed dark and light (AA.9, AA.10), for review only.
    pub palette: u8,
    /// The input page's multi-line field.
    pub notes: TextEdit,
    /// The ten thousand rows: the selected one, and which switches were flipped from where they start.
    pub long_pick: Option<usize>,
    pub long_on: Vec<bool>,
    /// The ripple section of the button page (DESIGN.md §1.15, §1.16): what the counter's presses reach
    /// (0 own, 1 the row, 2 the window), the legibility cap, the looseness clamp, how glyphs recompose,
    /// and the second counter's value.
    pub rip_scope: usize,
    pub legible_cap: f32,
    pub loose_max: f32,
    pub recompose: amenui::morph::Recompose,
    /// How scrolling looks (the 滚动 section of the list page; a preset sets it).
    pub scroll: amenui::style::Scroll,
    /// The finishing pass and depth marks of the style in use (DESIGN.md §1.21).
    pub post: amenui::post::Post,
    pub depth_marks: amenui::style::Marks,
    /// The field map's numbers (DESIGN.md §1.29): the applied preset's, edited on `#stirr`.
    pub map: amenui::fieldmap::MapFx,
    pub beat: amenui::beat::Beat,
    pub rip_count: u32,
    /// Ten fast presses of the counter in progress: when they started (state clock) and how many are done.
    pub rip_burst: Option<(f32, u32)>,
    /// The 3D page.
    pub three: three::Three,
    /// When morphs go to the GPU and the forces on its particles (the 粒子 page sets them).
    pub particles: amenui::morph::Particles,
    /// The particle page.
    pub parts: particles::Parts,
    /// The presets as data (DESIGN.md §4.1), and what the one applied last set beyond the look's choices:
    /// its faces, press burst, default responses and its transition.
    pub presets: Vec<amenui::preset::Preset>,
    pub faces: Option<[u8; 3]>,
    pub press_ms: f32,
    pub roles: [amenui::el::Response; 2],
    pub transition: amenui::preset::Transition,
    /// The 风格 page: from and to, the JSON field, and what reading it said.
    pub pre_from: usize,
    pub pre_to: usize,
    pub pre_json: TextEdit,
    pub pre_note: String,
    /// The interaction previews (sheet, floating box, menus, loading, stack, chart, fields).
    pub craft: craft::Craft,
    pub lab: lab::Lab,
    /// The Body scene (`#body`).
    pub body: crate::showcase::scenes::body::Body,
    pub decode: decode::DecodePage,
    pub stirr: stirr::StirrPage,
    /// The Works index and its pieces (M).
    pub works: works::Works,
    /// A docs page the canvas asked the host to open (its path under the site's base).
    pub navigate: Option<String>,
    /// Where the pointer is over the Loose Thread composition: a point there stirs the field map.
    pub thread_at: Option<(f32, f32)>,
    /// When the Loose Thread page was entered (state clock): its tangle draws on from then.
    pub thread_drawn: Option<f32>,
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            page: Page::Comp(0), theme: 0, material: Material::Dottr,
            params: COMPONENTS.iter().map(|c| c.params.iter().map(|p| p.default).collect()).collect(),
            edit: None, presses: 0, value: 42.0, text: TextEdit::default(), dialog_text: TextEdit::default(), on: true, level: 40.0,
            tab: 0, pick: 1, row: Some(2), dialog: 0, marks: std::array::from_fn(|s| mark_defaults(s as u8)), shown: demo::TREE, wipe: 0.3,
            demo: Demo::default(), width: 1280.0, height: 800.0,
            shape: 0, shape_before: None, morph: MorphSpec::default(), opened: None, flow: FlowSpec::default(), cmds: vec![], locale: amenui::i18n::Locale::default(), doc_tab: 0, outline_at: 0, rail_open: None, sheet: false, stage_only: false, still: false, from: Page::Components, pressed_at: None, routed: false, clipboard: None,
            menu: 0, stage_material: Material::Dottr, mats: MaterialParams::all(), loose: 0.0, style: None, gaze: Gaze::default(), loose_scale: 1.0, style_pic: Markr::default(),
            gaze_page: Gaze { mode: GazeMode::All, kinds: gaze::FRAME | gaze::INDEX | gaze::SIZE, density: 12.0, ..Gaze::default() },
            hovered: None, clock: 0.0, choices: [0, 0, 0, 0, 0, 0, 0, 0], toggles: [true, false, true, false, true, false, false, false],
            slides: [0.35, 0.4, 0.6, 14.0, 0.5, 2.0, 0.0, 0.5], query: TextEdit::new("lay"), sc: Scenes::default(), overlay: 0, quality: 1.0, status_line: false, palette: 0, long_pick: None, long_on: vec![],
            rip_scope: 1, legible_cap: amenui::style::LEGIBLE_CAP, loose_max: 1.0, recompose: amenui::morph::Recompose::default(), scroll: amenui::style::Scroll::default(), post: amenui::post::Post::default(), depth_marks: amenui::style::Marks::default(), map: amenui::fieldmap::MapFx::default(), beat: amenui::beat::Beat::default(), rip_count: 1, rip_burst: None, three: three::Three::default(), particles: amenui::morph::Particles::default(), parts: particles::Parts::default(), presets: amenui::preset::presets(), faces: None, press_ms: 350.0, roles: [amenui::el::Response::Nudge, amenui::el::Response::Loosen], transition: Look::default().transition, pre_from: 6, pre_to: 1, pre_json: TextEdit::default(), pre_note: String::new(), craft: craft::Craft::default(), lab: lab::Lab::default(), body: Default::default(), decode: decode::DecodePage::default(), stirr: stirr::StirrPage::default(), works: works::Works::default(), navigate: None, thread_at: None, thread_drawn: None, notes: TextEdit::new("First line\nSecond line, a longer one that wraps at the width of the box."),
        }
    }
}

/// Values that suit each mark set (the same ones the contact sheets use).
pub fn mark_defaults(s: u8) -> Markr {
    let base = Markr::with_set(s);
    match s {
        set::CHARS => Markr { pitch: 9.0, levels: 16.0, ground: 0.0, gamma: 1.1, ..base },
        set::DITHER => Markr { pitch: 3.0, levels: 16.0, ground: 0.0, ..base },
        set::PIXEL => Markr { pitch: 6.0, ..base },
        set::MOSAIC | set::BRICK => Markr { pitch: 10.0, relief: 1.0, ground: 0.2, ..base },
        set::GLOW => Markr { pitch: 8.0, levels: 12.0, ground: 0.0, glow: 1.0, gamma: 0.8, ..base },
        set::STROKE => Markr { pitch: 9.0, levels: 8.0, ground: 0.15, follow: 1.0, ..base },
        set::LED => Markr { pitch: 7.0, levels: 8.0, ground: 0.0, glow: 0.6, ..base },
        set::BRAILLE => Markr { pitch: 8.0, levels: 9.0, ground: 0.0, ..base },
        set::CUBES => Markr { pitch: 12.0, levels: 8.0, ground: 0.1, relief: 1.0, ..base },
        set::HATCH => Markr { pitch: 9.0, levels: 5.0, ground: 0.0, ..base },
        set::TOPO => Markr { pitch: 12.0, levels: 8.0, ground: 0.0, ..base },
        _ => Markr { pitch: 8.0, levels: 8.0, ground: 0.15, ..base },
    }
}

/// `Markr` slots: the demo uses 0 and 1; the image page 2; the mark pages 3; the style pages 4.
pub const IMAGE_SLOT: u8 = 2;
pub const MARK_SLOT: u8 = 3;
pub const STYLE_SLOT: u8 = 4;
pub const WIPE_SPEC: Spec = Spec { min: 0.0, max: 1.0, step: 0.01, decimals: 2, unit: "" };

impl Gallery {
    pub fn theme(&self) -> Theme { THEMES[self.theme.min(THEMES.len() - 1)] }
    pub fn param(&self, c: usize, i: usize) -> f32 { self.params[c][i] }
    pub fn look(&self) -> Look {
        let mut pics = self.demo.pics;
        let img = pages::IMAGE;
        pics[IMAGE_SLOT as usize] = Markr { pitch: self.param(img, 0), levels: self.param(img, 1), ..Markr::with_set(set::DOTS) };
        if let Page::Marks(s) = self.page { pics[MARK_SLOT as usize] = self.marks[s as usize]; }
        // The Marks cell on the Lab's index shows the member it is at.
        if self.page == Page::Lab && let Page::Marks(s) = lab::family(1)[(self.lab.fam_t[1] / lab::FAMILY_S) as usize % lab::MARK_ORDER.len()] { pics[MARK_SLOT as usize] = self.marks[s as usize]; }
        pics[STYLE_SLOT as usize] = self.style_pic;
        let gaze = if self.page == Page::Gaze { self.gaze_page } else { self.gaze };
        let mut l = Look { fx: self.demo.fx, theme: self.theme(), material: self.material, pics, params: self.mats, gaze, loose_scale: self.loose_scale, quality: self.quality, overlay: if self.status_line && self.overlay == 1 { 0 } else { self.overlay }, overlay_at: amenui::style::Corner::BottomRight, overlay_inset: [12.0, 12.0], overlay_keep: false,
                          legible_cap: self.legible_cap, loose_max: self.loose_max, recompose: self.recompose, particles: self.particles,
                          press_ms: self.press_ms, roles: self.roles, transition: self.transition, scroll: self.scroll, post: self.post, marks: self.depth_marks, map: self.map, beat: self.beat, off: amenui::style::Off::default(), select: amenui::style::Select::default() };
        if let Some(f) = self.faces { l.theme = Theme { display: f[0], body: f[1], mono: f[2], ..l.theme }; }
        // The proposed palette, behind its switch: the ground, the ink and the one signal.
        if self.palette > 0 {
            let (ground, ink, accent, second) = crate::showcase::scenes::body::palette(self.palette);
            l.theme = Theme { ground, ground2: ground, ink, accent, second, ..l.theme };
        }
        scenes::look(self, &mut l);
        lab::look(self, &mut l);
        body::look(self, &mut l);
        works::look(self, &mut l);
        // With reduced motion nothing condenses from dots: a page is at rest from its first frame.
        if self.body.reduced { l.fx.enter = 0.0; }
        l
    }
    /// The title's flow id on a page: from the Components index, the pressed card's name (so it flies
    /// to the title); otherwise the one id every page title shares (it flies from page to page).
    pub fn title_flow(&self) -> String { if self.from == Page::Components { format!("card-name-{}", self.page.hash()) } else { "page-title".into() } }
    /// What a navigation inside the running app does on the way from `from` to `to` (AA.11): one quiet
    /// change everywhere. The old page fades out and the new one in at its place, with no looseness,
    /// no flight and no front; only what is literally the same object on both pages (a Works window
    /// and its piece, a Lab cell and its experiment, a Components card and its stage) travels, 240 ms.
    /// With reduced motion it is a cut.
    fn transition(&mut self, from: Page, to: Page) {
        self.pressed_at = None;
        if from == to { return; }
        if self.body.reduced { self.cmds.push(Cmd::Flow(FlowSpec { flow_ms: 0.0, flow_loose: 0.0, out_ms: 0.0, fade_ms: 1.0 })); return; }
        let travels = matches!((from, to), (Page::Works, Page::Work(_)) | (Page::Work(_), Page::Works) | (Page::Lab, Page::Experiment(_)) | (Page::Experiment(_), Page::Lab))
            || (from == Page::Components && matches!(to, Page::Comp(_) | Page::Widget(_)));
        let flow_ms = if travels { PAGE_TRAVEL } else { 0.0 };
        self.cmds.push(Cmd::Flow(FlowSpec { flow_ms, flow_loose: 0.0, out_ms: PAGE_OUT, fade_ms: PAGE_IN }));
    }
    /// Apply style preset `i`: everything at once; the theme and material switches still work after.
    pub fn apply_style(&mut self, i: usize) {
        let i = i.min(self.presets.len().saturating_sub(1));
        let Some(st) = self.presets.get(i).cloned() else { return };
        self.style = Some(i);
        let th = amenui::style::theme_index(&st.theme);
        self.theme = th; self.demo.theme = th;
        self.faces = Some(st.faces);
        self.material = st.material; self.demo.material = st.material;
        self.mats = MaterialParams::all();
        self.mats[st.material.id() as usize] = MaterialParams(st.params);
        self.gaze = st.gaze;
        self.loose_scale = st.loose_scale;
        self.demo.fx.pitch = st.pitch;
        self.style_pic = st.pic;
        let it = st.interaction;
        (self.demo.fx.wave_speed, self.demo.fx.wave_width, self.demo.fx.wave_life) = (it.front_speed, it.front_width, it.front_life);
        self.recompose = it.recompose;
        self.press_ms = it.press_ms;
        self.roles = [it.text, it.decoration];
        self.scroll = it.scroll;
        self.transition = st.transition;
        self.post = st.post.look();
        self.depth_marks = st.marks;
        self.map = it.map;
        self.beat = it.beat;
    }
    /// Opening a page that has a style of its own applies it: a style page its preset, a scene its
    /// default.
    fn enter(&mut self, p: Page) {
        member_in_view(self, p);
        match p {
            Page::Style(i) => {
                self.apply_style(i as usize);
                self.thread_drawn = (styles::is_loose_thread(i as usize)).then_some(self.clock);
            }
            Page::Scene(i) => { let st = SCENES[i as usize].3; self.sc.look = st; self.apply_style(st); }
            // The 风格 page starts from its "from".
            Page::Presets => self.apply_style(self.pre_from),
            Page::Lab | Page::Experiment(_) => lab::enter(self),
            Page::Body => self.body = body::fresh(self),
            Page::Works | Page::Work(_) => works::enter(self, p),
            // Every other page is in the site's own look, whatever a preset or scene set before.
            _ if self.style.is_some() => self.site_look(),
            _ => {}
        }
    }
    /// The site's own look: what `apply_style` sets, back to the defaults.
    fn site_look(&mut self) {
        let d = Gallery::default();
        self.style = None;
        (self.theme, self.demo.theme, self.faces, self.material, self.demo.material) = (d.theme, d.demo.theme, d.faces, d.material, d.demo.material);
        (self.mats, self.gaze, self.loose_scale, self.demo.fx, self.style_pic) = (d.mats, d.gaze, d.loose_scale, d.demo.fx, d.style_pic);
        (self.recompose, self.press_ms, self.roles, self.scroll, self.transition) = (d.recompose, d.press_ms, d.roles, d.scroll, d.transition);
        (self.post, self.depth_marks, self.map, self.beat) = (d.post, d.depth_marks, d.map, d.beat);
    }
    /// The key of morph candidate `i` on the morph page.
    pub fn shape_key(i: usize) -> String { format!("morph/c{i}") }
    fn morph_to(&mut self, i: usize) {
        if i == self.shape || i >= MORPH_NAMES.len() { return; }
        // The tiles are the material's lattice cells.
        let spec = MorphSpec { lattice: self.demo.fx.pitch, ..self.morph.clone() };
        self.cmds.push(Cmd::Morph { from: Self::shape_key(self.shape), to: Self::shape_key(i), spec });
        self.shape_before = Some(self.shape);
        self.shape = i;
    }
    pub fn draft(&self, t: Target) -> Option<&str> { self.edit.as_ref().filter(|(k, _)| *k == t).map(|(_, d)| d.as_str()) }
    /// The look as a recipe: the active preset (the first when none is chosen) as `Preset::recipe`.
    pub fn recipe(&self) -> String { self.presets[self.style.unwrap_or(0).min(self.presets.len().saturating_sub(1))].recipe() }
    /// Take a recipe (`look:v3:…`): its preset replaces the built-in one it names, and becomes the look.
    pub fn take_recipe(&mut self, r: &str) -> Result<(), amenui::error::PresetError> {
        let p = amenui::preset::Preset::from_recipe(r)?;
        let i = self.presets.iter().position(|q| q.hash == p.hash).unwrap_or(0);
        self.presets[i] = p;
        self.apply_style(i);
        Ok(())
    }
    /// Go to the page an address names: a hash name (`#button`, `button`), a path (`components/button/`,
    /// with a slash in it, or `/` for home) or a look recipe (the look changes, the page stays).
    /// A page by its path (`components/button/`), its older hash name (`button`) or a recipe; the home
    /// page's state may follow after `#` (`/#source`, `body#body/cloud/08`), opened at rest.
    pub fn route(&mut self, hash: &str) -> bool {
        if hash.trim_start_matches('#').starts_with(amenui::preset::RECIPE_PREFIX) { return self.take_recipe(hash).is_ok(); }
        let (hash, state) = hash.trim_start_matches('#').split_once('#').unwrap_or((hash, ""));
        let page = if hash.contains('/') { Page::from_path(hash) } else { Page::from_hash(hash) };
        match page {
            Some(p) => {
                if self.routed && p != self.page { self.transition(self.page, p); self.from = self.page; }
                self.routed = true;
                (self.rail_open, self.sheet, self.dialog) = (None, false, 0);
                self.page = p; self.edit = None; self.enter(p);
                if state == "stage" { self.stage_only = true; }
                else if p == Page::Body && !state.is_empty() { self.body.open(state, 0); }
                else if p == Page::Lab && let Some(i) = lab::STEPS.iter().position(|k| *k == state) { lab::at_step(self, i); }
                true
            }
            None => false,
        }
    }
}

pub fn update(s: &mut Gallery, m: Msg) {
    match m {
        Msg::Go(p) => {
            // Parts the two pages share (the page title, the rail's selection) move rather than vanish.
            if p != s.page { let from = s.page; s.transition(from, p); s.from = from; }
            s.page = p; s.edit = None; s.dialog = 0; s.rail_open = None; s.sheet = false;
            // A style page shows its style, a scene its own.
            s.enter(p);
        }
        Msg::RailGroup(g) => { let open = s.rail_open.unwrap_or_else(|| comp_group_of(s.page)); s.rail_open = Some(if open == Some(g) { None } else { Some(g) }); }
        Msg::Sheet(on) => s.sheet = on,
        Msg::Menu(k) => s.menu = k,
        Msg::Style(i) => { s.apply_style(i); s.menu = 0; }
        Msg::GazeMode(i) => s.gaze_page.mode = GAZE_MODES[i.min(GAZE_MODES.len() - 1)].0,
        Msg::GazeKind(k) => s.gaze_page.kinds ^= k,
        Msg::Choose(slot, i) => s.choices[slot.min(7)] = i,
        Msg::Toggle(slot) => s.toggles[slot.min(7)] = !s.toggles[slot.min(7)],
        Msg::ThreadAt(p) => s.thread_at = p,
        Msg::PostFlag(f) => {
            let (binary, invert) = (s.post.binary ^ (f == 0), s.post.invert ^ (f == 1));
            s.post = amenui::post::PostSpec { binary, invert, ..amenui::post::PostSpec::default() }.look();
        }
        Msg::Query(t) => s.query = t,
        Msg::Scene(m) => scenes::update(s, m),
        Msg::Slide(slot, v) => s.slides[slot.min(7)] = v.clamp(0.0, 1.0),
        Msg::MorphTo(i) => s.morph_to(i),
        Msg::MorphBack => if let Some(b) = s.shape_before { s.morph_to(b); },
        Msg::Ease(i) => s.morph.ease = Ease::from_index(i),
        Msg::Open(o) => { if o != s.opened { s.cmds.push(Cmd::Flow(s.flow)); } s.opened = o; }
        Msg::Locale(l) => {
            s.locale = l;
            amenui::i18n::set_current(l);
            s.menu = 0;
        }
        Msg::DocTab(i) => s.doc_tab = i.min(1),
        Msg::Outline(k) => docs::outline_go(s, k),
        Msg::Copy(t) => s.clipboard = Some(t),
        Msg::Theme(i) => { s.theme = i.min(THEMES.len() - 1); s.demo.theme = s.theme; s.faces = None; s.menu = 0; }
        Msg::Material(i) => { s.stage_material = MATERIALS[i.min(MATERIALS.len() - 1)].0; s.demo.material = s.stage_material; s.menu = 0; }
        Msg::Reset => {
            let d = Gallery::default();
            s.mats = d.mats; s.loose = d.loose; s.style = None; s.gaze = d.gaze; s.loose_scale = d.loose_scale; s.style_pic = d.style_pic; s.gaze_page = d.gaze_page;
            s.params = d.params; s.marks = d.marks; s.wipe = d.wipe; s.edit = None;
            s.value = d.value; s.level = d.level;
            s.demo = Demo { theme: s.theme, material: s.material, ..Demo::default() };
            s.morph = d.morph; s.flow = d.flow; s.three = d.three; s.presets = d.presets; s.faces = None; s.press_ms = d.press_ms; s.roles = d.roles; s.transition = d.transition; s.parts = d.parts; s.particles = d.particles;
            s.sc = Scenes::default();
            s.stage_material = d.stage_material;
        }
        // Variants are shown as they are: typing into one does not change it.
        Msg::Num(Target::Shown, _) => {}
        Msg::Num(t, Edit::Draft(d)) => s.edit = Some((t, d)),
        Msg::Num(t, Edit::Set(v)) => {
            match t {
                Target::Param(c, i) => s.params[c][i] = v,
                Target::Pic(k, i) => { let (f, scale) = demo::pic_slot(&mut s.marks[k as usize], i); *f = v / scale; }
                Target::Wipe => s.wipe = v,
                Target::Value => s.value = v,
                Target::Morph(i) => *morph_slot(&mut s.morph, i) = v,
                Target::Flow(i) => *flow_slot(&mut s.flow, i) = v,
                Target::Three(i) if i >= three::PARAMS.len() => s.three.mech.explode_to = v.clamp(0.0, 1.0),
                Target::Three(i) => *three::slot(&mut s.three, i) = v,
                Target::Parts(i) => *particles::slot(&mut s.parts, &mut s.particles, i) = v,
                Target::Craft(p, i) => craft::set(s, p, i, v),
                Target::Preset(i) => presets::set(s, i, v),
                Target::Mat(m, i) => s.mats[m as usize].0[i] = v,
                Target::Loose => s.loose = v,
                Target::Density => s.gaze_page.density = v,
                Target::DrawMs => s.gaze_page.draw_ms = v,
                Target::OscAmp => s.gaze_page.osc_amp = v,
                Target::OscY => s.gaze_page.osc_y = v,
                Target::LevelFull => s.gaze_page.level_full = v,
                Target::StyleNum(i) => s.slides[i.min(7)] = v,
                Target::Scene(k, i) => scenes::set(s, k, i, v),
                Target::OpValue => s.sc.op_value = v,
                Target::Rip(i) => match i {
                    0 => s.legible_cap = v, 1 => s.loose_max = v, 2 => s.recompose.duration_ms = v,
                    3 => s.recompose.stagger = v, 4 => s.recompose.count_max = v, 5 => s.recompose.slide_ms = v,
                    6 => s.recompose.min_interval_ms = v, 7 => s.recompose.max_tiles = v, _ => s.recompose.tile = v,
                },
                Target::Scroll(i) => *pages::scroll_slot(&mut s.scroll, i) = pages::SCROLL_PARAMS[i.min(7)].1.clamp(v),
                Target::Shown => {}
            }
            s.edit = None;
        }
        Msg::Num(_, Edit::Cancel) => s.edit = None,
        Msg::Nothing => {}
        Msg::Press => s.presses += 1,
        Msg::RipCount => s.rip_count += 1,
        Msg::RipScope(i) => s.rip_scope = i.min(2),
        Msg::Three(m) => three::update(s, m),
        Msg::Parts(m) => particles::update(s, m),
        Msg::Preset(m) => presets::update(s, m),
        Msg::Craft(m) => craft::update(s, m),
        Msg::Lab(m) => lab::update(s, m),
        Msg::Body(m) => body::update(s, m),
        Msg::Decode(m) => decode::update(s, m),
        Msg::Stirr(m) => stirr::update(s, m),
        Msg::Works(m) => works::update(s, m),
        Msg::RipBurst => { s.rip_burst = Some((s.clock, 0)); burst_step(s, s.clock); }
        Msg::Text(t) => s.text = t,
        Msg::Notes(t) => s.notes = t,
        Msg::LongPick(i) => s.long_pick = Some(i),
        Msg::LongToggle(i) => { if s.long_on.len() <= i { s.long_on.resize(i + 1, false); } s.long_on[i] = !s.long_on[i]; }
        Msg::DialogText(t) => s.dialog_text = t,
        Msg::Switch => s.on = !s.on,
        Msg::Level(v) => s.level = v,
        Msg::Tab(i) => s.tab = i,
        Msg::Pick(i) => s.pick = i,
        Msg::Row(i) => s.row = Some(i),
        Msg::Dialog(k) => s.dialog = k,
        Msg::Show(id) => s.shown = id,
        Msg::Wipe(x) => s.wipe = WIPE_SPEC.clamp(x),
        Msg::Demo(m) => {
            // The demo's own theme and material controls are the gallery's.
            match m {
                demo::Msg::Theme(i) => return update(s, Msg::Theme(i)),
                demo::Msg::Material(i) => return update(s, Msg::Material(i)),
                _ => {}
            }
            demo::update(&mut s.demo, m);
        }
    }
}

/// The interface. The demos are in English whatever the language: in English all of it, otherwise the
/// page under the chrome except the documentation's chrome and prose, whose demos (a component's
/// previews and examples) are in English too; the framework's own files shown as they are (agents,
/// llms, release notes, the registry) are left alone.
pub fn view(s: &Gallery) -> El<Msg> {
    let e = view_in(s);
    match s.page {
        _ if s.locale == amenui::i18n::Locale::En => english::tree(e, Rc::new(|_| false)),
        // The page under the chrome; inside it, the documentation's chrome and prose (`doc/…`) stay as
        // written except a component's previews and example boxes.
        _ => english::demos(e, Rc::new(|k| {
            // (A component's preview column holds the preview and its parameters; its tabs are chrome.)
            if works::english(k) { english::Part::Demo }
            else if k.starts_with("page/") || k.ends_with("/preview-col") { english::Part::Page }
            // A page's title and its description (`button/title`) are the page's name, in its language.
            else if (k.ends_with("/title") || k.ends_with("/desc")) && k.matches('/').count() == 1 { english::Part::Chrome }
            else if k.starts_with("doc/") { if k.ends_with("/preview") || k.ends_with("/box") || k.ends_with("/source") { english::Part::Demo } else { english::Part::Chrome } }
            // A scene's name on the list is the one its own page flies it to (scenes keep the
            // interface language in Chinese: their type is set to fit it).
            else if k.starts_with("scenes/title") { english::Part::Chrome }
            // The index pages' cards are chrome (their names and lines follow the language); what a card
            // shows of its page is a demo.
            else if k.starts_with("scenes/t") && k.ends_with("/box") || k.starts_with("index/") && k.ends_with("/preview") { english::Part::Demo }
            else if k.starts_with("scenes/card") || k == "scenes/body" || k == "scenes/head" || k.starts_with("index/") || k.starts_with("labx/") { english::Part::Chrome }
            else { english::Part::Same }
        })),
    }
}
fn view_in(s: &Gallery) -> El<Msg> {
    let narrow = s.width < NARROW;
    let th = s.theme();
    // A stage on a docs page (`<path>#stage`): the page's stage alone, filling the box, on the raised
    // ground (D.2) in plain, so no ambient dots cross it, `STAGE_INSET` inside the box. A specimen is
    // fitted by the framework (`El::fit`): a component or a widget reflows (`FitMode::Reflow`: laid
    // out at the box's width, zoomed down only if it still does not fit); a lab experiment is a scene
    // of fixed proportions, scaled whole and centred (`FitMode::Contain`). A scene fills the box.
    if s.stage_only {
        let stage = col().key("gallery").fonts(th.roles()).fill_width().fill_height().center().pad(STAGE_INSET, STAGE_INSET).fill(RAISED).material(Material::Plain.id());
        return match s.page {
            Page::Scene(_) => stage.child(stage_view(s, narrow).fill_width().align(AlignItems::CENTER)),
            Page::Experiment(_) => stage.fit(amenui::el::Fit { mode: amenui::el::FitMode::Contain, ..Default::default() }).child(stage_view(s, narrow)),
            _ => stage.fit(amenui::el::Fit { mode: amenui::el::FitMode::Reflow, ..Default::default() }).child(stage_view(s, narrow).fill_width().align(AlignItems::CENTER)),
        };
    }
    // The Body scene is the whole window, full-bleed, no chrome: its own markings lead elsewhere.
    if s.page == Page::Body {
        return col().key("gallery").fonts(amenui::style::LAB.roles()).fill_width().fill_height().child(body::page(s));
    }
    // Works: the index under the bar, a piece the whole window behind it (M.2, M.3).
    if s.page.section() == Section::Works {
        return col().key("gallery").fonts(th.roles()).fill_width().fill_height().material(Material::Plain.id())
            .child(col().key(format!("main/{}", s.page.hash())).grow(1.0).fill_width().min_height(0.0).child(works::page(s)));
    }
    if let Page::Scene(i) = s.page {
        // The canvas lies under the page's navigation bar here, so the scene starts below it.
        return col().key("gallery").fonts(th.roles()).fill_width().fill_height().child(space(0.0, BAR).key("bar-room"))
            .child(lab_strip(s)).child(divider(th.hairline).key("strip-rule"))
            .child(col().key(format!("main/{}", s.page.hash())).grow(1.0).fill_width().child(scenes::page(s, i as usize)));
    }
    // The first Tab stop: past the bar and the rail to the page.
    let skip = skip_link("skip", &amenui::i18n::tr("shell.skip"), &format!("main/{}", s.page.hash()));
    let mut root = col().key("gallery").fonts(th.roles()).fill_width().fill_height().child(skip);
    // The Components index and the component pages are on a flat ground: plain, so no ambient dots
    // run through the gutters, under the labels or the source. (Home and Scenes keep them.)
    if s.page.section() == Section::Components { root = root.fill(Paint::GROUND).material(Material::Plain.id()); }
    let page = main_page(s, narrow);
    let main = |page: El<Msg>| col().key(format!("main/{}", s.page.hash())).scroll().bar().grow(1.0).fill_width().min_height(0.0)
        .child(if narrow { col().key("narrow").width(s.width).min_width(0.0).child(page) } else { page }).shrink();
    match s.page.section() {
        // Components: the rail of six groups beside the page; on a phone the strip that opens it as a sheet.
        Section::Components if narrow => {
            root = root.child(comp_strip(s)).child(divider(th.hairline).key("rail-rule"));
            root = if s.sheet { root.child(col().key("main/sheet").scroll().bar().grow(1.0).fill_width().min_height(0.0).child(rail_items(s, col().key("rail").fill_width().pad(4.0, 8.0).gap(2.0)))) } else { root.child(main(page)) };
        }
        Section::Components => {
            // `min_height(0)`: the row is as tall as the window allows, not as tall as the rail's
            // contents (which made the page as tall as the rail, so it never scrolled).
            root = root.child(row().key("body").grow(1.0).min_height(0.0).fill_width().align(AlignItems::STRETCH)
                .child(rail(s)).child(vdivider(th.hairline).key("rail-rule")).child(main(page)));
        }
        // The Lab: no rail; a page inside it has the path strip under the bar.
        Section::Lab if s.page != Page::Lab => { root = root.child(lab_strip(s)).child(divider(th.hairline).key("strip-rule")).child(main(page)); }
        // The index: the path line stays under the bar while it scrolls.
        Section::Lab => { root = root.child(lab::path_line(s)).child(main(page)); }
        _ => { root = root.child(main(page)); }
    }
    root
}

/// How far a stage's specimen keeps from the stage's box.
pub const STAGE_INSET: f32 = 16.0;

/// `El::zoom` (which scales the offsets a widget places its parts by, `El::shift`, too: a floating
/// box's corner, a chart's dots).
pub(crate) fn zoomed(e: El<Msg>, k: f32) -> El<Msg> { if (k - 1.0).abs() < 1e-4 { e } else { e.zoom(k) } }

/// What a docs page's live stage shows of a route: a component's preview, an experiment's cell, a
/// scene; any other page as it is, without the rail.
fn stage_view(s: &Gallery, narrow: bool) -> El<Msg> {
    match s.page {
        Page::Comp(c) => col().key(format!("stage/{}", s.page.hash())).material(s.stage_material.id()).child(pages::live(s, c)),
        Page::Widget(i) => {
            let n = &docs::widget_names()[i as usize];
            // The tip and the toast as the index draws them: placed from their boxes' near edges, so
            // they stay on the stage (the examples place them from the window's far edges).
            let near = matches!(n.as_str(), "tooltip" | "toast").then(|| index::widget_preview(n, &format!("stage/{}/w", s.page.hash()))).flatten();
            // The specimen alone (I.2 item 7): a crafted demo without its formula caption and its
            // number controls (the component's own page keeps them).
            let bare = |e: El<Msg>| if e.key.ends_with("/cap/with") { e.children.into_iter().next().unwrap_or_else(|| space(0.0, 0.0)) } else { e };
            let e = craft::live(s, n).map(bare).or(near).or_else(|| still_specimen(s, n)).or_else(|| docs::first_example(n)).unwrap_or_else(|| space(0.0, 0.0));
            col().key(format!("stage/{}", s.page.hash())).material(s.stage_material.id()).child(e)
        }
        Page::Experiment(i) => col().key(format!("stage/{}", s.page.hash())).width((s.width - 32.0).clamp(200.0, 760.0)).child(lab::on_page(s, i as usize)),
        Page::Scene(i) => col().key(format!("main/{}", s.page.hash())).fill_width().fill_height().child(scenes::page(s, i as usize)),
        _ => col().key(format!("main/{}", s.page.hash())).scroll().fill_width().fill_height().child(main_page(s, narrow)),
    }
}

/// For a stage's still, the specimens that are closed at rest in the state that shows them: the context
/// menu open at its anchor and the sheet at its resting detent, as the index draws them, and the skip
/// link as itself (the still's script focuses it, web/build.py); and the checkbox checked, its example
/// that shows most ink (S.2 item 8; the switch, the slider: `pages::hold_still`).
fn still_specimen(s: &Gallery, n: &str) -> Option<El<Msg>> {
    if !s.still { return None; }
    let k = format!("stage/{}/w", s.page.hash());
    match n {
        "context_menu" | "sheet" => index::widget_preview(n, &k),
        "checkbox" => docs::example(n, 1),
        "skip-link" | "skip_link" => Some(col().key(format!("{k}/box")).width(160.0).height(48.0)
            .child(amenui::widgets::skip_link(&k, &amenui::i18n::tr("shell.skip"), &format!("stage/{}", s.page.hash())))),
        _ => None,
    }
}

/// The page under the top bar, padded.
fn main_page(s: &Gallery, narrow: bool) -> El<Msg> {
    let main = col().key(format!("page/{}", s.page.hash())).fill_width().min_width(0.0);
    let pad = |m: El<Msg>| m.pad(if narrow { 18.0 } else { 40.0 }, 32.0);
    match s.page {
        Page::Demo => main.child(demo::view(&s.demo).map(Rc::new(Msg::Demo))),
        Page::Lab => main.pad(if narrow { 16.0 } else { 32.0 }, 28.0).child(lab::page(s)),
        Page::Components => pad(main).child(index::components(s)),
        Page::Reader => pad(main).child(docs::reader(s)),
        Page::Decode => pad(main).child(decode::page(s)),
        Page::Stirr => pad(main).child(stirr::page(s)),
        Page::Experiment(i) => main.pad(if narrow { 14.0 } else { 40.0 }, 32.0).child(lab::experiment(s, i as usize)),
        Page::Material(_) | Page::Gaze => pad(main).child(looks::page(s)),
        Page::Style(i) => main.child(styles::page(s, i as usize)),
        Page::Three => pad(main).child(three::page(s)),
        Page::Particles => pad(main).child(particles::page(s)),
        Page::Presets => pad(main).child(presets::page(s)),
        Page::Widget(i) => { let n = &docs::widget_names()[i as usize]; pad(main).child(docs::component(s, n, craft::live(s, n), None)) }
        _ => pad(main).child(pages::page(s)),
    }
}

/// The width the rail takes beside the page (none off the Components section and on a phone).
pub fn side(s: &Gallery) -> f32 { if s.page.section() == Section::Components && s.width >= NARROW { RAIL } else { 0.0 } }

/// The look strip at the top of a specimen's stage: the stage's material and `reset`. It is chrome:
/// calm, and in the plain material whatever the stage is in. (The theme and the whole looks join it
/// once a theme can be set on a subtree.)
pub fn look_strip(s: &Gallery, key: &str) -> El<Msg> {
    let tr = amenui::i18n::tr;
    let names: Vec<&str> = MATERIALS.iter().map(|m| m.1).collect();
    let now = MATERIALS.iter().position(|m| m.0 == s.stage_material);
    row().key(key).gap(16.0).wrap().align(AlignItems::CENTER).calm().material(Material::Plain.id())
        .child(menu(&format!("{key}/material"), &tr("shell.material"), &names, now, s.menu == 2, false, Msg::Menu(if s.menu == 2 { 0 } else { 2 }), Msg::Menu(0), Msg::Material))
        .child(button_as(format!("{key}/reset"), &tr("shell.reset"), Variant::Default, Msg::Reset))
}

/// The component pages: one per component, and the list's ten thousand rows (by path only, from the
/// list's page) right after the list.
fn comp_pages() -> Vec<Page> {
    let mut v: Vec<Page> = (0..COMPONENTS.len()).map(Page::Comp).collect();
    v.insert(pages::LIST + 1, Page::LongList);
    v
}

/// The Components rail's six groups (registry.json's `group`, shadcn's way) and their components.
pub fn comp_groups() -> Vec<(String, Vec<Page>)> {
    let tr = amenui::i18n::tr;
    let comps: Vec<Page> = (0..COMPONENTS.len()).map(Page::Comp).chain((0..docs::widget_names().len() as u8).map(Page::Widget)).collect();
    let name_of = |p: &Page| match *p { Page::Comp(c) => docs::COMP_NAMES[c].to_owned(), Page::Widget(i) => docs::widget_names()[i as usize].clone(), _ => "list".into() };
    docs::GROUPS.iter().map(|(g, key)| (tr(key), comps.iter().filter(|p| docs::group_of(&name_of(p)) == *g).copied().collect())).collect()
}
/// The Lab's three groups and their pages, in the order the path strip steps through them. The
/// skeleton demo is in none (it is reached by its path only).
/// The group of the rail that holds the page.
fn comp_group_of(p: Page) -> Option<usize> {
    let p = if p == Page::LongList { Page::Comp(pages::LIST) } else { p };
    comp_groups().iter().position(|g| g.1.contains(&p))
}
fn rail_item(s: &Gallery, p: Page, h: f32) -> El<Msg> {
    let on = s.page == p || (p == Page::Comp(pages::LIST) && s.page == Page::LongList);
    let mut item = row().key(format!("nav/{}", p.hash())).height(h).pad(12.0, 0.0).radius(4.0).align(AlignItems::CENTER).on_press(Msg::Go(p)).ripplr(amenui::el::Scope::Global)
        .child(text(p.name(), 14.0, if on { Paint::ACCENT } else { Paint::TEXT }));
    // The selection is one thing that moves from item to item when the page changes.
    if on { item = item.fill(SELECTED).flow("rail-selection"); }
    item
}
/// The rail's groups into `r`: each group's name, which opens or closes it; only one is open, the
/// current page's unless another was opened.
fn rail_items(s: &Gallery, mut r: El<Msg>) -> El<Msg> {
    let open = s.rail_open.unwrap_or_else(|| comp_group_of(s.page));
    for (gi, (name, pages)) in comp_groups().into_iter().enumerate() {
        let is = open == Some(gi);
        r = r.child(row().key(format!("rail/group/{gi}")).height(30.0).pad(12.0, 0.0).gap(8.0).align(AlignItems::CENTER).radius(4.0)
            .on_press(Msg::RailGroup(gi)).role(amenui::el::Role::Button)
            .child(text(name, 12.0, if is { Paint::INK } else { Paint::DIM }).font(amenui::el::Font::Mono))
            .child(space(0.0, 0.0).grow(1.0))
            .child(text(format!("{}", pages.len()), 11.0, Paint::DIM).font(amenui::el::Font::Mono)));
        if is { for p in pages { r = r.child(rail_item(s, p, 30.0)); } }
    }
    r
}
fn rail(s: &Gallery) -> El<Msg> {
    rail_items(s, col().key("rail").width(RAIL).fill(Paint::GROUND).scroll().bar().pad(12.0, 16.0).gap(2.0).calm().material(Material::Plain.id()))
}

/// A link in a strip: dim, ink when it is where one is.
fn strip_link(key: String, label: String, here: bool, m: Msg) -> El<Msg> {
    // A style page changes the whole look: the change spreads from the link pressed.
    let commit = matches!(m, Msg::Go(Page::Style(_) | Page::Scene(_)));
    let l = row().key(key).height(36.0).align(AlignItems::CENTER).on_press(m).role(amenui::el::Role::Link)
        .child(text(label, 11.0, if here { Paint::INK } else { Paint::TEXT }).font(amenui::el::Font::Mono));
    if commit { l.commit() } else { l }
}
fn sep(t: &str) -> El<Msg> { text(t.to_owned(), 11.0, Paint::DIM).font(amenui::el::Font::Mono) }
/// The Lab's path strip (Z.5): `lab / ink / kern` on the left (the step goes to its header on the
/// index), `← 02.2 take apart · 02.4 weld →` on the right, along the whole path. A family's page has
/// all its members under it: on lines that break only between the marks' runs, and below 720 on one
/// line that scrolls sideways, kept with the page's member in view (`member_in_view`).
/// A family member's page on a phone: its link in the members' line is scrolled into view (the
/// focus taken there and given up again, which scrolls the line and leaves nothing focused).
fn member_in_view(s: &mut Gallery, p: Page) {
    if s.width >= NARROW || lab::family_of(p).is_none() || p == Page::Presets { return; }
    s.cmds.push(Cmd::Focus { key: format!("strip/m/{}", p.hash()) });
    s.cmds.push(Cmd::ClearFocus);
}
fn lab_strip(s: &Gallery) -> El<Msg> {
    let tr = amenui::i18n::tr;
    let low = |t: String| t.to_lowercase();
    let narrow = s.width < NARROW;
    let pad = if narrow { 16.0 } else { 32.0 };
    let mut left = row().key("strip/path").gap(8.0).align(AlignItems::CENTER)
        .child(strip_link("strip/lab".into(), low(tr("nav.lab")), false, Msg::Go(Page::Lab)));
    let mut right = row().key("strip/steps").gap(8.0).align(AlignItems::CENTER);
    if let Some((st, _)) = lab::place(s.page) {
        left = left.child(sep("/")).child(strip_link("strip/group".into(), low(lab::step_name(st)), false, Msg::Lab(lab::LMsg::GoStep(st))));
    }
    if let Some((prev, (next, nl))) = lab::neighbours(s.page) {
        if let Some((p, l)) = prev { right = right.child(strip_link("strip/prev".into(), l, false, Msg::Go(p))).child(sep("·")); }
        right = right.child(strip_link("strip/next".into(), nl, false, Msg::Go(next)));
    }
    // Where one is: the one part of the strip that changes, and moves, from page to page.
    left = left.child(sep("/")).child(text(low(lab::page_name(s.page)), 11.0, Paint::INK).font(amenui::el::Font::Mono).key("strip/here").flow("strip-here"));
    let top = row().key("strip").fill_width().height(36.0).pad(pad, 0.0).gap(16.0).align(AlignItems::CENTER).calm().material(Material::Plain.id())
        .child(left).child(space(0.0, 0.0).grow(1.0)).child(if narrow { space(0.0, 0.0) } else { right });
    let Some(f) = lab::family_of(s.page) else { return top };
    // The family's members: the marks in their four runs, 26 px apart, a run never broken across lines.
    let members = lab::family(f);
    let link = |p: Page| { let here = p == s.page; strip_link(format!("strip/m/{}", p.hash()), if here { format!("[ {} ]", low(lab::page_name(p))) } else { low(lab::page_name(p)) }, here, Msg::Go(p)).height(24.0) };
    let runs: Vec<&[Page]> = if f == 1 { lab::MARK_RUNS.iter().zip(lab::MARK_RUNS.iter().skip(1).chain([&members.len()])).map(|(&a, &b)| &members[a..b]).collect() } else { members.chunks(1).collect() };
    let mut m = row().key("strip/members").align(AlignItems::CENTER);
    let between = if f == 1 { 26.0 } else { 10.0 };
    if narrow { m = m.gap(between).scroll_x().fill_width().min_width(0.0); } else {
        m = m.wrap().fill_width().min_width(0.0);
        m.style.gap = taffy::prelude::Size { width: taffy::prelude::length(between), height: taffy::prelude::length(0.0) };
    }
    for (r, run) in runs.into_iter().enumerate() {
        let mut g = row().key(format!("strip/run{r}")).gap(10.0).align(AlignItems::CENTER);
        for &p in run { g = g.child(link(p)); }
        m = m.child(g);
    }
    let row2 = row().key("strip/family").fill_width().min_height(28.0).pad(pad, 2.0).align(AlignItems::CENTER).calm().material(Material::Plain.id()).child(m);
    col().key("strip/both").fill_width().child(top).child(row2)
}
/// The Components strip on a phone: `components / inputs / button ▾`, which opens the rail as a sheet.
fn comp_strip(s: &Gallery) -> El<Msg> {
    let tr = amenui::i18n::tr;
    let low = |t: String| t.to_lowercase();
    let mut path = row().key("strip/path").gap(8.0).align(AlignItems::CENTER)
        .child(strip_link("strip/components".into(), low(tr("nav.components")), s.page == Page::Components, Msg::Go(Page::Components)));
    if let Some(g) = comp_group_of(s.page) { path = path.child(sep("/")).child(text(low(comp_groups()[g].0.clone()), 11.0, Paint::TEXT).font(amenui::el::Font::Mono)); }
    if s.page != Page::Components { path = path.child(sep("/")).child(text(low(s.page.name()), 11.0, Paint::INK).font(amenui::el::Font::Mono).key("strip/here")); }
    row().key("strip").fill_width().height(36.0).pad(16.0, 0.0).gap(8.0).align(AlignItems::CENTER).calm().material(Material::Plain.id())
        .child(path).child(space(0.0, 0.0).grow(1.0))
        .child(strip_link("strip/sheet".into(), if s.sheet { "▴".into() } else { "▾".into() }, s.sheet, Msg::Sheet(!s.sheet)))
}

pub type GalleryUi = amenui::ui::Ui<Gallery, Msg>;
/// D.2's raised colour (code blocks, stages, sheets): `#131517` on the dark ground.
pub const RAISED: Paint = Paint::tone(0.03);

/// Ten presses of the ripple counter in 300 ms: the gap between two.
pub const BURST_GAP: f32 = 0.3 / 9.0;
/// Press the ripple counter for every press of the burst that is due at `t`; returns whether any was.
fn burst_step(s: &mut Gallery, t: f32) -> bool {
    let Some((start, done)) = s.rip_burst else { return false };
    let mut n = done;
    while n < 10 && t >= start + n as f32 * BURST_GAP - 1e-4 { s.cmds.push(Cmd::Press { key: "ripple/counter".into() }); n += 1; }
    s.rip_burst = (n < 10).then_some((start, n));
    n > done
}

pub fn new_ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> GalleryUi {
    crate::install_catalogs();
    let state = Gallery { width: w as f32 / scale, height: h as f32 / scale, ..Gallery::default() };
    // Tools (reel, scroll, pages) can ask for a language: AMENUI_LOCALE=en.
    #[cfg(not(target_arch = "wasm32"))]
    let state = { let mut s = state; if let Some(l) = std::env::var("AMENUI_LOCALE").ok().and_then(|t| amenui::i18n::Locale::from_tag(&t)) { update(&mut s, Msg::Locale(l)); } s };
    let mut ui = amenui::ui::Ui::new(state, view, update, faces, w, h, scale);
    // The mechanism's meshes (gears, a cam, plates…), after the built-ins, in the order `mechanism`
    // names them.
    for m in mechanism::meshes() { ui.add_mesh(m); }
    // Kern and Loupe measure the outlines of the page's own faces (`Experiment::faces`).
    ui = ui.on_faces(|s, f| s.lab.faces(f));
    ui.on_resize(|s, w, h| { s.width = w; s.height = h; s.body.resize(w, h); }).on_commands(|s| std::mem::take(&mut s.cmds))
        .on_gaze(|s| s.look().gaze)
        .on_hover(|s, k| {
            let k = k.map(str::to_owned);
            if k == s.hovered { return false; }
            // Only the pages that show what is hovered need a new view.
            let shows = matches!(s.page, Page::Style(_));
            s.hovered = k;
            shows
        })
        .on_tick(|s, t| {
            // The departure board ticks once a second; scenes have their own clocks.
            let changed = s.page == Page::Style(1) && t.floor() != s.clock.floor();
            s.clock = t;
            let burst = burst_step(s, t);
            // The 3D page's camera coasts after a fast drag.
            let coast = s.page == Page::Three && s.three.mech.step(t);
            let craft = craft::tick(s, t);
            let lab = body::tick(s, t) | works::tick(s, t) | (s.page == Page::Decode && decode::tick(s, t)) | (s.page == Page::Stirr && stirr::tick(s, t));
            let thread = matches!(s.page, Page::Style(i) if styles::is_loose_thread(i as usize)) && styles::drawing(s, t);
            // The hosts coalesce input (`Ui::coalesce`): what a tick changes is laid out once, before the frame.
            scenes::tick(s, t) || changed || burst || coast || craft || lab || thread
        })
        // The Lab's cells step on their own: only the stages that changed are made and drawn again.
        .on_step(lab::step)
        .stage_view(lab::stage_of)
        .on_focus(lab::focus)
        .on_look(|s| s.look())
        .on_locale(|s| s.locale)
        .on_wake(|s, t| {
            // The departure board wants the next whole second; scenes say for themselves.
            let board = (s.page == Page::Style(1)).then(|| t.floor() + 1.0);
            let burst = s.rip_burst.map(|(start, n)| start + n as f32 * BURST_GAP);
            let burst = if s.page == Page::Three && s.three.mech.moving(t) { Some(t) } else { burst };
            let board = match (board, burst) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) };
            let board = match (board, craft::wake(s, t)) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) };
            let board = match (board, if s.page == Page::Decode { decode::wake(s, t) } else { None }) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) };
            let board = match (board, if s.page == Page::Stirr { stirr::wake(s, t) } else { None }) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) };
            let board = match (board, (matches!(s.page, Page::Style(i) if styles::is_loose_thread(i as usize)) && styles::drawing(s, t)).then_some(t)) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) };
            let board = match (board, lab::wake(s, t)) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) };
            let board = match (board, body::wake(s, t)) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) };
            let board = match (board, works::wake(s, t)) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) };
            match (board, scenes::wake(s, t)) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) }
        })
        .on_unused_key(|s, k, shift| {
            // F2 shows or hides the frame-time readout anywhere.
            if k == amenui::el::Key::F(2) { s.overlay = (s.overlay + 1) % 3; return true; }
            body::key(s, k) || scenes::key(s, k, shift) || works::key(s, k) || lab::key(s, k)
        })
        .on_input(|g| scenes::heard(g) | body::heard(g) | works::heard(g))
        .on_pointer(works::pointer)
        .on_key_up(scenes::key_up)
        .on_scrolled(docs::scrolled)
}
