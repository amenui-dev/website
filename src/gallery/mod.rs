//! The gallery: the main application, written with the framework only. A left rail lists the
//! components, the mark sets and the type; each component has a page with a live preview, every
//! variant, its own numbers and its source. The top bar switches theme and material; the switch is
//! committed from the control that was pressed. The old skeleton demo is one of the pages.
pub mod craft;
pub mod docs;
pub mod film;
pub mod mechanism;
mod looks;
mod pages;
mod scenes;
mod styles;
pub mod three;
pub mod particles;
pub mod presets;

use crate::demo::{self, Demo};
use crate::el::{El, Font, Paint, col, row, space, text};
use crate::morph::{Cmd, Ease, FlowSpec, MorphSpec};
use crate::style::{GAZE_MODES, Gaze, GazeMode, Look, MATERIAL_COUNT, MATERIALS, Material, MaterialParams, PicFx, THEMES, Theme, gaze, set, styles};
use crate::text::Face;
use crate::widgets::{Edit, SELECTED, Spec, TextEdit, Variant, button_as, divider, menu, vdivider};
use std::rc::Rc;
use taffy::AlignItems;

pub use scenes::{SCENES, SMsg, Scenes};
/// The component indices in `COMPONENTS`.
pub mod comp { pub use super::pages::{BUTTON, CARD, CHOICE, DIALOG, DIVIDER, IMAGE, INPUT, LIST, NUMBER, SCROLL, SLIDER, SWITCH, TABS}; }
pub use pages::{COMPONENTS, Comp, FLOW_PARAMS, MORPH_NAMES, MORPH_PARAMS, ParamDef, flow_slot, morph_slot};

/// Below this width (logical px) the rail becomes a row of tabs under the top bar.
pub const NARROW: f32 = 720.0;
pub const RAIL: f32 = 220.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page { Film, Doc(u8), Ai(u8), Registry, Widget(u8), Comp(usize), LongList, Marks(u8), Material(u8), Gaze, Style(u8), Morph, Particles, Flow, Three, Presets, Type, Demo, Scenes, Scene(u8) }

pub const MARK_HASH: [&str; set::COUNT] = ["chars", "dither", "pixel", "mosaic", "brick", "dots", "cross", "diagonal", "diamond", "lines", "glow", "stroke", "led", "braille", "cubes", "hex", "stipple", "hatch", "truchet", "topo"];

impl Page {
    /// Every page, in rail order.
    pub fn all() -> Vec<Page> {
        let mut v: Vec<Page> = vec![Page::Film];
        v.extend((0..docs::DOCS.len() as u8).map(Page::Doc));
        v.extend(comp_pages());
        v.extend((0..docs::widget_names().len() as u8).map(Page::Widget));
        v.extend(set::FAMILIES.iter().flat_map(|f| f.1.iter().copied()).map(Page::Marks));
        v.extend((0..MATERIAL_COUNT as u8).map(Page::Material));
        v.push(Page::Gaze);
        v.extend((0..12u8).map(Page::Style));
        v.push(Page::Presets);
        v.extend([Page::Morph, Page::Particles, Page::Flow, Page::Three]);
        v.push(Page::Type);
        v.push(Page::Demo);
        v.push(Page::Scenes);
        v.extend((0..scenes::COUNT as u8).map(Page::Scene));
        v.extend([Page::Ai(0), Page::Ai(1), Page::Registry, Page::Ai(2)]);
        v
    }
    /// The page's address on the web: plain ASCII words, so a page can be linked.
    pub fn hash(self) -> String {
        match self {
            Page::Film => "film".into(),
            Page::Doc(i) => format!("docs-{}", docs::DOCS[i as usize]),
            Page::Ai(i) => docs::AI[i as usize].0.into(),
            Page::Registry => "ai-registry".into(),
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
            Page::Scene(i) => scenes::hash(i as usize),
        }
    }
    /// From `#name` or `name`; an empty hash is the first page.
    pub fn from_hash(h: &str) -> Option<Page> {
        let h = h.trim_start_matches('#');
        if h.is_empty() || h == "gallery" { return Some(Page::Comp(0)); }
        Page::all().into_iter().find(|p| p.hash() == h)
    }
    /// The page's name in the current language.
    pub fn name(self) -> String {
        let l = crate::i18n::current();
        let en = l == crate::i18n::Locale::En;
        let zh = self.name_zh();
        match self {
            Page::Film => crate::i18n::tr("shell.film"),
            Page::Doc(i) => docs::title(i as usize, l),
            Page::Ai(i) => crate::i18n::tr(docs::AI[i as usize].1),
            Page::Registry => crate::i18n::tr("docs.ai.registry"),
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
            Page::Film | Page::Doc(_) | Page::Ai(_) | Page::Registry | Page::Widget(_) => "",
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
    /// `PicFx` field `i` of mark set `s` (see `demo::PIC_PARAMS`).
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
    /// Switch the language (DESIGN.md §1.20): everything is laid out again; keyed text recomposes.
    Locale(crate::i18n::Locale),
    /// A component page's 预览 / 代码 tab; an outline entry (scroll to that key); the search over every
    /// page (open, its query, run row `i`); a copy button (its text for the clipboard).
    DocTab(usize),
    Outline(String),
    Search(bool),
    SearchQuery(TextEdit),
    SearchRun(usize),
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
    /// Open the theme (1), material (2), style (3) or performance (4) menu, or close it (0).
    Menu(u8),
    /// The performance menu's item `i`: the readout on or off (0), a quality (1: automatic, then the
    /// levels of `perf::LEVELS`).
    Perf(usize),
    /// Apply style preset `i` (theme, material, its numbers, annotation, type).
    Style(usize),
    /// The annotation page: mode, and one kind on or off.
    GazeMode(usize),
    GazeKind(u16),
    /// A choice on a style page: slot, index.
    Choose(usize, usize),
    /// A toggle on a style page.
    Toggle(usize),
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
    /// Something in the film.
    Film(film::FMsg),
    /// Something in an interaction preview (`craft`).
    Craft(craft::CMsg),
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
    pub marks: [PicFx; set::COUNT],
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
    pub locale: crate::i18n::Locale,
    /// The documentation: the component page's tab, the outline entry in view, the search, and text
    /// a copy button asked the host to put on the clipboard.
    pub doc_tab: usize,
    pub outline_at: usize,
    pub search: bool,
    pub search_q: TextEdit,
    pub search_active: usize,
    pub clipboard: Option<String>,
    /// Which top-bar menu is open (0 = none).
    pub menu: u8,
    /// Every material's numbers; the looseness the material pages hold their specimen at.
    pub mats: [MaterialParams; MATERIAL_COUNT],
    pub loose: f32,
    /// The style applied last, if any, and what it set: annotation, looseness scale, picture marks.
    pub style: Option<usize>,
    pub gaze: Gaze,
    pub loose_scale: f32,
    pub style_pic: PicFx,
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
    /// The frame-time readout (F2, or the 性能 menu) and the material resolution (0 = automatic).
    /// The performance readout: 0 off, 1 one line, 2 every number.
    pub overlay: u8,
    pub quality: f32,
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
    pub recompose: crate::morph::Recompose,
    /// How scrolling looks (the 滚动 section of the list page; a preset sets it).
    pub scroll: crate::style::Scroll,
    /// The finishing pass and depth marks of the style in use (DESIGN.md §1.21).
    pub post: crate::post::Post,
    pub depth_marks: crate::style::Marks,
    pub rip_count: u32,
    /// Ten fast presses of the counter in progress: when they started (state clock) and how many are done.
    pub rip_burst: Option<(f32, u32)>,
    /// The 3D page.
    pub three: three::Three,
    /// When morphs go to the GPU and the forces on its particles (the 粒子 page sets them).
    pub particles: crate::morph::Particles,
    /// The particle page.
    pub parts: particles::Parts,
    /// The presets as data (DESIGN.md §4.1), and what the one applied last set beyond the look's choices:
    /// its faces, press burst, default responses and its transition.
    pub presets: Vec<crate::preset::Preset>,
    pub faces: Option<[u8; 3]>,
    pub press_ms: f32,
    pub roles: [crate::el::Response; 2],
    pub transition: crate::preset::Transition,
    /// The 风格 page: from and to, the JSON field, and what reading it said.
    pub pre_from: usize,
    pub pre_to: usize,
    pub pre_json: TextEdit,
    pub pre_note: String,
    /// The film (`#film`).
    pub film: film::Film,
    /// The interaction previews (sheet, floating box, menus, loading, stack, chart, fields).
    pub craft: craft::Craft,
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            page: Page::Comp(0), theme: 0, material: Material::Dot,
            params: COMPONENTS.iter().map(|c| c.params.iter().map(|p| p.default).collect()).collect(),
            edit: None, presses: 0, value: 42.0, text: TextEdit::default(), dialog_text: TextEdit::default(), on: true, level: 40.0,
            tab: 0, pick: 1, row: Some(2), dialog: 0, marks: std::array::from_fn(|s| mark_defaults(s as u8)), shown: demo::BUST, wipe: 0.3,
            demo: Demo::default(), width: 1280.0, height: 800.0,
            shape: 0, shape_before: None, morph: MorphSpec::default(), opened: None, flow: FlowSpec::default(), cmds: vec![], locale: crate::i18n::Locale::default(), doc_tab: 0, outline_at: 0, search: false, search_q: TextEdit::default(), search_active: 0, clipboard: None,
            menu: 0, mats: MaterialParams::all(), loose: 0.0, style: None, gaze: Gaze::default(), loose_scale: 1.0, style_pic: PicFx::default(),
            gaze_page: Gaze { mode: GazeMode::All, kinds: gaze::FRAME | gaze::INDEX | gaze::SIZE, density: 12.0, ..Gaze::default() },
            hovered: None, clock: 0.0, choices: [0, 0, 0, 0, 0, 0, 0, 0], toggles: [true, false, true, false, true, false, false, false],
            slides: [0.35, 0.4, 0.6, 14.0, 0.5, 2.0, 0.0, 0.5], query: TextEdit::new("lay"), sc: Scenes::default(), overlay: 0, quality: 1.0, long_pick: None, long_on: vec![],
            rip_scope: 1, legible_cap: crate::style::LEGIBLE_CAP, loose_max: 1.0, recompose: crate::morph::Recompose::default(), scroll: crate::style::Scroll::default(), post: crate::post::Post::default(), depth_marks: crate::style::Marks::default(), rip_count: 1, rip_burst: None, three: three::Three::default(), particles: crate::morph::Particles::default(), parts: particles::Parts::default(), presets: crate::preset::presets(), faces: None, press_ms: 350.0, roles: [crate::el::Response::Nudge, crate::el::Response::Loosen], transition: Look::default().transition, pre_from: 7, pre_to: 1, pre_json: TextEdit::default(), pre_note: String::new(), film: film::Film::default(), craft: craft::Craft::default(), notes: TextEdit::new("第一行\n第二行，长一些的一行会在框的宽度里折行。"),
        }
    }
}

/// Values that suit each mark set (the same ones the contact sheets use).
pub fn mark_defaults(s: u8) -> PicFx {
    let base = PicFx::with_set(s);
    match s {
        set::CHARS => PicFx { pitch: 9.0, levels: 16.0, ground: 0.0, gamma: 1.1, ..base },
        set::DITHER => PicFx { pitch: 3.0, levels: 16.0, ground: 0.0, ..base },
        set::PIXEL => PicFx { pitch: 6.0, ..base },
        set::MOSAIC | set::BRICK => PicFx { pitch: 10.0, relief: 1.0, ground: 0.2, ..base },
        set::GLOW => PicFx { pitch: 8.0, levels: 12.0, ground: 0.0, glow: 1.0, gamma: 0.8, ..base },
        set::STROKE => PicFx { pitch: 9.0, levels: 8.0, ground: 0.15, follow: 1.0, ..base },
        set::LED => PicFx { pitch: 7.0, levels: 8.0, ground: 0.0, glow: 0.6, ..base },
        set::BRAILLE => PicFx { pitch: 8.0, levels: 9.0, ground: 0.0, ..base },
        set::CUBES => PicFx { pitch: 12.0, levels: 8.0, ground: 0.1, relief: 1.0, ..base },
        set::HATCH => PicFx { pitch: 9.0, levels: 5.0, ground: 0.0, ..base },
        set::TOPO => PicFx { pitch: 12.0, levels: 8.0, ground: 0.0, ..base },
        _ => PicFx { pitch: 8.0, levels: 8.0, ground: 0.15, ..base },
    }
}

/// `PicFx` slots: the demo uses 0 and 1; the image page 2; the mark pages 3; the style pages 4.
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
        pics[IMAGE_SLOT as usize] = PicFx { pitch: self.param(img, 0), levels: self.param(img, 1), ..PicFx::with_set(set::DOTS) };
        if let Page::Marks(s) = self.page { pics[MARK_SLOT as usize] = self.marks[s as usize]; }
        pics[STYLE_SLOT as usize] = self.style_pic;
        let gaze = if self.page == Page::Gaze { self.gaze_page } else { self.gaze };
        let mut l = Look { fx: self.demo.fx, theme: self.theme(), material: self.material, pics, params: self.mats, gaze, loose_scale: self.loose_scale, quality: self.quality, overlay: self.overlay,
                          legible_cap: self.legible_cap, loose_max: self.loose_max, recompose: self.recompose, particles: self.particles,
                          press_ms: self.press_ms, roles: self.roles, transition: self.transition, scroll: self.scroll, post: self.post, marks: self.depth_marks };
        if let Some(f) = self.faces { l.theme = Theme { display: f[0], body: f[1], mono: f[2], ..l.theme }; }
        scenes::look(self, &mut l);
        film::look(self, &mut l);
        l
    }
    /// Apply style preset `i`: everything at once; the theme and material switches still work after.
    pub fn apply_style(&mut self, i: usize) {
        let i = i.min(self.presets.len().saturating_sub(1));
        let Some(st) = self.presets.get(i).cloned() else { return };
        self.style = Some(i);
        let th = crate::style::theme_index(&st.theme);
        self.theme = th; self.demo.theme = th;
        self.faces = Some(st.faces);
        self.material = st.material; self.demo.material = st.material;
        self.mats = MaterialParams::all();
        self.mats[st.material as usize] = MaterialParams(st.params);
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
    }
    /// Opening a page that has a style of its own applies it: a style page its preset, a scene its
    /// default.
    fn enter(&mut self, p: Page) {
        match p {
            Page::Style(i) => self.apply_style(i as usize),
            Page::Scene(i) => { let st = SCENES[i as usize].3; self.sc.look = st; self.apply_style(st); }
            // The 风格 page starts from its "from".
            Page::Presets => self.apply_style(self.pre_from),
            // The film opens on its chapter, in its preset.
            Page::Film => { self.film.entered = self.clock; self.film.ghost = 0; self.apply_style(film::CH[self.film.ch].style); }
            _ => {}
        }
    }
    /// The key of morph candidate `i` on the morph page.
    pub fn shape_key(i: usize) -> String { format!("morph/c{i}") }
    fn morph_to(&mut self, i: usize) {
        if i == self.shape || i >= MORPH_NAMES.len() { return; }
        // The tiles are the material's lattice cells.
        let spec = MorphSpec { lattice: self.demo.fx.pitch, ..self.morph };
        self.cmds.push(Cmd::Morph { from: Self::shape_key(self.shape), to: Self::shape_key(i), spec });
        self.shape_before = Some(self.shape);
        self.shape = i;
    }
    pub fn draft(&self, t: Target) -> Option<&str> { self.edit.as_ref().filter(|(k, _)| *k == t).map(|(_, d)| d.as_str()) }
    /// Go to the page a hash names; returns whether it names one.
    /// The look as a recipe: the active preset (the first when none is chosen) as `Preset::recipe`.
    pub fn recipe(&self) -> String { self.presets[self.style.unwrap_or(0).min(self.presets.len().saturating_sub(1))].recipe() }
    /// Take a recipe (`look:v1:…`): its preset replaces the built-in one it names, and becomes the look.
    pub fn take_recipe(&mut self, r: &str) -> Result<(), crate::error::PresetError> {
        let p = crate::preset::Preset::from_recipe(r)?;
        let i = self.presets.iter().position(|q| q.hash == p.hash).unwrap_or(0);
        self.presets[i] = p;
        self.apply_style(i);
        Ok(())
    }
    pub fn route(&mut self, hash: &str) -> bool {
        // A recipe in the address: the look changes, the page stays.
        if hash.trim_start_matches('#').starts_with(crate::preset::RECIPE_PREFIX) { return self.take_recipe(hash).is_ok(); }
        match Page::from_hash(hash) {
            Some(p) => { self.page = p; self.edit = None; self.enter(p); true }
            None => false,
        }
    }
}

pub fn update(s: &mut Gallery, m: Msg) {
    match m {
        Msg::Go(p) => {
            // Parts the two pages share (the page title, the rail's selection) move rather than vanish.
            if p != s.page { s.cmds.push(Cmd::Flow(s.flow)); }
            s.page = p; s.edit = None; s.dialog = 0;
            // A style page shows its style, a scene its own.
            s.enter(p);
        }
        Msg::Menu(k) => s.menu = k,
        Msg::Perf(0) => { s.overlay = (s.overlay + 1) % 3; s.menu = 0; }
        Msg::Perf(i) => { s.quality = if i == 1 { 0.0 } else { crate::perf::LEVELS[(i - 2).min(2)] }; s.menu = 0; }
        Msg::Style(i) => { s.apply_style(i); s.menu = 0; }
        Msg::GazeMode(i) => s.gaze_page.mode = GAZE_MODES[i.min(GAZE_MODES.len() - 1)].0,
        Msg::GazeKind(k) => s.gaze_page.kinds ^= k,
        Msg::Choose(slot, i) => s.choices[slot.min(7)] = i,
        Msg::Toggle(slot) => s.toggles[slot.min(7)] = !s.toggles[slot.min(7)],
        Msg::Query(t) => s.query = t,
        Msg::Scene(m) => scenes::update(s, m),
        Msg::Slide(slot, v) => s.slides[slot.min(7)] = v.clamp(0.0, 1.0),
        Msg::MorphTo(i) => s.morph_to(i),
        Msg::MorphBack => if let Some(b) = s.shape_before { s.morph_to(b); },
        Msg::Ease(i) => s.morph.ease = Ease::from_index(i),
        Msg::Open(o) => { if o != s.opened { s.cmds.push(Cmd::Flow(s.flow)); } s.opened = o; }
        Msg::Locale(l) => {
            // The film's sentence follows the language while nobody has changed it.
            if s.film.note.text == s.locale.tr("film.c01.note-default") { s.film.note = TextEdit::new(&l.tr("film.c01.note-default")); }
            s.locale = l;
            crate::i18n::set_current(l);
            s.menu = 0;
        }
        Msg::DocTab(i) => s.doc_tab = i.min(1),
        Msg::Outline(k) => docs::outline_go(s, k),
        Msg::Search(on) => { s.search = on; s.search_active = 0; if on { s.cmds.push(Cmd::Focus { key: "search/palette/query".into() }); } }
        Msg::SearchQuery(q) => { s.search_q = q; s.search_active = 0; }
        Msg::SearchRun(i) => docs::search_run(s, i),
        Msg::Copy(t) => s.clipboard = Some(t),
        Msg::Theme(i) => { s.theme = i.min(THEMES.len() - 1); s.demo.theme = s.theme; s.faces = None; s.menu = 0; }
        Msg::Material(i) => { s.material = MATERIALS[i.min(MATERIALS.len() - 1)].0; s.demo.material = s.material; s.menu = 0; }
        Msg::Reset => {
            let d = Gallery::default();
            s.mats = d.mats; s.loose = d.loose; s.style = None; s.gaze = d.gaze; s.loose_scale = d.loose_scale; s.style_pic = d.style_pic; s.gaze_page = d.gaze_page;
            s.params = d.params; s.marks = d.marks; s.wipe = d.wipe; s.edit = None;
            s.value = d.value; s.level = d.level;
            s.demo = Demo { theme: s.theme, material: s.material, ..Demo::default() };
            s.morph = d.morph; s.flow = d.flow; s.three = d.three; s.presets = d.presets; s.faces = None; s.press_ms = d.press_ms; s.roles = d.roles; s.transition = d.transition; s.parts = d.parts; s.particles = d.particles;
            s.sc = Scenes::default();
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
        Msg::Film(m) => film::update(s, m),
        Msg::Craft(m) => craft::update(s, m),
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

pub fn view(s: &Gallery) -> El<Msg> {
    let narrow = s.width < NARROW;
    let th = s.theme();
    if s.page == Page::Film { return film::view(s); }
    // A scene has the whole window under the top bar.
    if let Page::Scene(i) = s.page {
        // On a scene the top bar is chrome, like the strip: fronts the scene makes pass it by.
        return col().key("gallery").fonts(th.roles()).fill_width().fill_height().child(top_bar(s).calm()).child(divider(th.hairline).key("top-rule"))
            .child(col().key(format!("main/{}", s.page.hash())).grow(1.0).fill_width().child(scenes::page(s, i as usize)));
    }
    let mut root = col().key("gallery").fonts(th.roles()).fill_width().fill_height().child(top_bar(s)).child(divider(th.hairline).key("top-rule"));
    let page = main_page(s, narrow);
    if narrow {
        // A phone's width: the page is set in the window's width, so its text wraps there; anything
        // wider (an example, a code line) scrolls sideways in its own box.
        let main = col().key(format!("main/{}", s.page.hash())).scroll().bar().grow(1.0).fill_width()
            .child(col().key("narrow").width(s.width).min_width(0.0).child(page));
        root = root.child(rail_row(s)).child(divider(th.hairline).key("rail-rule")).child(main);
    } else {
        // `min_height(0)`: the row is as tall as the window allows, not as tall as the rail's contents (which
        // made the page itself as tall as the rail, so the page never scrolled once the rail was longer
        // than the window).
        root = root.child(row().key("body").grow(1.0).min_height(0.0).fill_width().align(AlignItems::STRETCH)
            .child(rail(s)).child(vdivider(th.hairline).key("rail-rule")).child(col().key(format!("main/{}", s.page.hash())).scroll().bar().grow(1.0).fill_width().child(page).shrink()));
    }
    root.child(docs::search(s))
}

/// The page under the top bar, padded.
fn main_page(s: &Gallery, narrow: bool) -> El<Msg> {
    let main = col().key(format!("page/{}", s.page.hash())).fill_width().min_width(0.0);
    match s.page {
        Page::Demo => main.child(demo::view(&s.demo).map(Rc::new(Msg::Demo))),
        Page::Material(_) | Page::Gaze => main.pad(if narrow { 18.0 } else { 40.0 }, 32.0).child(looks::page(s)),
        Page::Style(i) => main.child(styles::page(s, i as usize)),
        Page::Three => main.pad(if narrow { 18.0 } else { 40.0 }, 32.0).child(three::page(s)),
        Page::Particles => main.pad(if narrow { 18.0 } else { 40.0 }, 32.0).child(particles::page(s)),
        Page::Presets => main.pad(if narrow { 18.0 } else { 40.0 }, 32.0).child(presets::page(s)),
        Page::Doc(i) => main.pad(if narrow { 18.0 } else { 40.0 }, 32.0).child(docs::prose(s, i as usize)),
        Page::Ai(i) => main.pad(if narrow { 18.0 } else { 40.0 }, 32.0).child(docs::ai(s, i as usize)),
        Page::Registry => main.pad(if narrow { 18.0 } else { 40.0 }, 32.0).child(docs::registry(s)),
        Page::Widget(i) => { let n = &docs::widget_names()[i as usize]; main.pad(if narrow { 18.0 } else { 40.0 }, 32.0).child(docs::component(s, n, craft::live(s, n), None)) }
        Page::Scenes | Page::Scene(_) => main.pad(if narrow { 18.0 } else { 40.0 }, 32.0).child(scenes::list(s)),
        _ => main.pad(if narrow { 18.0 } else { 40.0 }, 32.0).child(pages::page(s)),
    }
}

/// The performance menu: the readout (its label says what pressing it does), then the qualities.
const PERF_ITEMS: [&str; 5] = ["", "shell.quality.auto", "shell.quality.1", "shell.quality.075", "shell.quality.05"];
fn perf_item(s: &Gallery) -> usize {
    if s.quality == 0.0 { 1 } else { 2 + crate::perf::LEVELS.iter().position(|&l| l == s.quality).unwrap_or(0) }
}

fn top_bar(s: &Gallery) -> El<Msg> {
    let theme_names: Vec<&str> = THEMES.iter().map(|t| t.name).collect();
    let material_names: Vec<&str> = MATERIALS.iter().map(|m| m.1).collect();
    let style_names: Vec<&str> = styles().iter().map(|t| t.name).collect();
    let material_now = MATERIALS.iter().position(|m| m.0 == s.material);
    let tr = crate::i18n::tr;
    let name = if matches!(s.page, Page::Scene(_)) {
        button_as("top/back", &tr("shell.back-to-scenes"), Variant::Quiet, Msg::Go(Page::Scenes))
    } else { text("gui-lab", 17.0, Paint::INK).font(Font::Mono).key("top/name") };
    let perf: Vec<String> = PERF_ITEMS.iter().map(|i| if i.is_empty() { tr(if s.overlay == 2 { "shell.readout.hide" } else if s.overlay == 1 { "shell.readout.more" } else { "shell.readout.show" }) } else { tr(i) }).collect();
    let perf: Vec<&str> = perf.iter().map(String::as_str).collect();
    let langs: Vec<&str> = crate::i18n::Locale::ALL.iter().map(|l| l.name()).collect();
    let lang_now = crate::i18n::Locale::ALL.iter().position(|l| *l == s.locale);
    row().key("top").fill_width().wrap().gap(12.0).pad(20.0, 10.0).align(AlignItems::CENTER)
        .child(name)
        .child(space(0.0, 0.0).grow(1.0))
        // On a phone's width the controls stay on one line that scrolls sideways; wider, they wrap.
        .child({ let r = row().key("top/controls").gap(16.0).align(AlignItems::CENTER); if s.width < NARROW { r.scroll_x().fill_width().min_width(0.0) } else { r.wrap().shrink().min_width(0.0) } }
            .child(menu("perf", &tr("shell.perf"), &perf,
                Some(perf_item(s)), s.menu == 4, false, Msg::Menu(if s.menu == 4 { 0 } else { 4 }), Msg::Menu(0), Msg::Perf))
            .child(button_as("top/film", &tr("shell.film"), Variant::Quiet, Msg::Go(Page::Film)))
            .child(button_as("top/scenes", &tr("shell.scenes"), Variant::Quiet, Msg::Go(Page::Scenes)))
            .child(menu("lang", &tr("shell.language"), &langs, lang_now, s.menu == 5, false, Msg::Menu(if s.menu == 5 { 0 } else { 5 }), Msg::Menu(0), |i| Msg::Locale(crate::i18n::Locale::ALL[i.min(1)])))
            .child(menu("theme", &tr("shell.theme"), &theme_names, Some(s.theme), s.menu == 1, true, Msg::Menu(if s.menu == 1 { 0 } else { 1 }), Msg::Menu(0), Msg::Theme))
            .child(menu("material", &tr("shell.material"), &material_names, material_now, s.menu == 2, true, Msg::Menu(if s.menu == 2 { 0 } else { 2 }), Msg::Menu(0), Msg::Material))
            .child(menu("style", &tr("shell.style"), &style_names, s.style, s.menu == 3, true, Msg::Menu(if s.menu == 3 { 0 } else { 3 }), Msg::Menu(0), Msg::Style))
            .child(button_as("reset", &tr("shell.reset"), Variant::Default, Msg::Reset)))
}

/// The component pages: one per component, and the list's ten thousand rows right after the list.
fn comp_pages() -> Vec<Page> {
    let mut v: Vec<Page> = (0..COMPONENTS.len()).map(Page::Comp).collect();
    v.insert(pages::LIST + 1, Page::LongList);
    v
}

/// The groups and their pages.
fn groups() -> Vec<(String, Vec<Page>)> {
    let tr = crate::i18n::tr;
    // The components in six groups (registry.json's `group`), shadcn's way; the long list with the list.
    let comps: Vec<Page> = comp_pages().into_iter().chain((0..docs::widget_names().len() as u8).map(Page::Widget)).collect();
    let name_of = |p: &Page| match *p { Page::Comp(c) => docs::COMP_NAMES[c].to_owned(), Page::Widget(i) => docs::widget_names()[i as usize].clone(), _ => "list".into() };
    let mut v = vec![
        (tr("nav.start"), (0..docs::START as u8).map(Page::Doc).collect()),
        (tr("nav.concepts"), (docs::START as u8..docs::DOCS.len() as u8).map(Page::Doc).collect::<Vec<_>>()),
    ];
    for (g, key) in docs::GROUPS { v.push((format!("{} · {}", tr("nav.components"), tr(key)), comps.iter().filter(|p| docs::group_of(&name_of(p)) == g).copied().collect())); }
    v.extend([
        (tr("nav.styles"), std::iter::once(Page::Presets).chain((0..12u8).map(Page::Style)).collect()),
        (tr("nav.ai"), vec![Page::Ai(0), Page::Ai(1), Page::Registry]),
        (tr("nav.changelog"), vec![Page::Ai(2)]),
        (tr("nav.marks"), set::FAMILIES.iter().flat_map(|f| f.1.iter().copied()).map(Page::Marks).collect()),
        (tr("nav.materials"), (0..MATERIAL_COUNT as u8).map(Page::Material).collect()),
        (tr("nav.annotation"), vec![Page::Gaze]),
        (tr("nav.transforms"), vec![Page::Morph, Page::Particles, Page::Flow, Page::Three]),
        (tr("nav.type"), vec![Page::Type, Page::Demo]),
        (tr("nav.scenes"), std::iter::once(Page::Scenes).chain((0..scenes::COUNT as u8).map(Page::Scene)).collect()),
    ]);
    v
}

fn rail_item(s: &Gallery, p: Page, h: f32) -> El<Msg> {
    let on = s.page == p;
    let mut item = row().key(format!("nav/{}", p.hash())).height(h).pad(12.0, 0.0).radius(4.0).align(AlignItems::CENTER).on_press(Msg::Go(p)).ripple(crate::el::Scope::Global)
        .child(text(p.name(), 14.0, if on { Paint::ACCENT } else { Paint::TEXT }));
    // A style page changes the whole look: the change spreads from the item pressed.
    if matches!(p, Page::Style(_) | Page::Scene(_)) { item = item.commit(); }
    // The selection is one thing that moves from item to item when the page changes.
    if on { item = item.fill(SELECTED).flow("rail-selection"); }
    item
}

fn rail(s: &Gallery) -> El<Msg> {
    let mut r = col().key("rail").width(RAIL).scroll().bar().pad(12.0, 16.0).gap(2.0);
    for (gi, (name, pages)) in groups().into_iter().enumerate() {
        if gi > 0 { r = r.child(space(1.0, 14.0)); }
        r = r.child(row().key(format!("rail/group/{gi}")).pad(12.0, 4.0).child(text(name, 12.0, Paint::DIM)));
        for p in pages { r = r.child(rail_item(s, p, 30.0)); }
    }
    r
}

/// The rail on narrow windows: one row that scrolls sideways, the group names as dim labels.
fn rail_row(s: &Gallery) -> El<Msg> {
    let mut r = row().key("rail").fill_width().scroll_x().bar().pad(12.0, 6.0).gap(4.0).align(AlignItems::CENTER);
    for (gi, (name, pages)) in groups().into_iter().enumerate() {
        r = r.child(row().key(format!("rail/group/{gi}")).pad(if gi > 0 { 12.0 } else { 4.0 }, 0.0).child(text(name, 12.0, Paint::DIM)));
        for p in pages { r = r.child(rail_item(s, p, 32.0)); }
    }
    r
}

pub type GalleryUi = crate::ui::Ui<Gallery, Msg>;

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
    let state = Gallery { width: w as f32 / scale, height: h as f32 / scale, ..Gallery::default() };
    // Tools (reel, scroll, pages) can ask for a language: LAB_LOCALE=en.
    #[cfg(not(target_arch = "wasm32"))]
    let state = { let mut s = state; if let Some(l) = std::env::var("LAB_LOCALE").ok().and_then(|t| crate::i18n::Locale::from_tag(&t)) { update(&mut s, Msg::Locale(l)); } s };
    let mut ui = crate::ui::Ui::new(state, view, update, faces, w, h, scale);
    // The mechanism's meshes (gears, a cam, plates…), after the built-ins, in the order `mechanism`
    // names them.
    for m in mechanism::meshes() { ui.add_mesh(m); }
    ui.on_resize(|s, w, h| { s.width = w; s.height = h; }).on_commands(|s| std::mem::take(&mut s.cmds))
        .on_gaze(|s| s.look().gaze)
        .on_hover(|s, k| {
            let k = k.map(str::to_owned);
            if k == s.hovered { return false; }
            // Only the pages that show what is hovered need a new view.
            let shows = matches!(s.page, Page::Style(_));
            let film = film::hovered(s, k.as_deref());
            s.hovered = k;
            shows || film
        })
        .on_tick(|s, t| {
            // The departure board ticks once a second; scenes have their own clocks.
            let changed = s.page == Page::Style(1) && t.floor() != s.clock.floor();
            s.clock = t;
            let burst = burst_step(s, t);
            // The 3D page's camera coasts after a fast drag.
            let coast = s.page == Page::Three && s.three.mech.step(t);
            let film = film::tick(s, t);
            let craft = craft::tick(s, t);
            scenes::tick(s, t) || changed || burst || coast || film || craft
        })
        .on_look(|s| s.look())
        .on_locale(|s| s.locale)
        .on_wake(|s, t| {
            // The departure board wants the next whole second; scenes say for themselves.
            let board = (s.page == Page::Style(1)).then(|| t.floor() + 1.0);
            let burst = s.rip_burst.map(|(start, n)| start + n as f32 * BURST_GAP);
            let burst = if s.page == Page::Three && s.three.mech.moving(t) { Some(t) } else { burst };
            let board = match (board, burst) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) };
            let board = match (board, film::wake(s, t)) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) };
            let board = match (board, craft::wake(s, t)) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) };
            match (board, scenes::wake(s, t)) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) }
        })
        .on_unused_key(|s, k, shift| {
            // F2 shows or hides the frame-time readout anywhere.
            if k == crate::el::Key::F(2) { s.overlay = (s.overlay + 1) % 3; return true; }
            if docs::search_key(s, k) { return true; }
            if k == crate::el::Key::Char('/') && s.page != Page::Film { update(s, Msg::Search(true)); return true; }
            film::key(s, k, shift) || scenes::key(s, k, shift)
        })
        .on_input(|s| film::heard(s) | scenes::heard(s))
        .on_pointer(film::pointer)
        .on_key_up(scenes::key_up)
        .on_shortcut(|s, c| { if c == 'k' { update(s, Msg::Search(!s.search)); true } else { false } })
        .on_scrolled(docs::scrolled)
}
