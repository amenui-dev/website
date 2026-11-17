//! The site's pages, driven from outside like amenui's recipes (amenui's `drive`): `open_in` opens a page
//! by its hash (`button`, `material-dot`, `scene-morph`…) and otherwise a recipe, and `render` is the
//! `render` tool as a function.
use amenui::drive::{Driven, Input, RenderSpec, Rendered};
use amenui::error::{DriveError, LookError, ViewProblem};
use amenui::i18n::Locale;
use amenui::offscreen::Offscreen;
use amenui::style::{Look, Material};
use amenui::text::Face;
use amenui::tree::TreeNode;

/// A site page: driven as any page, and its preset is its recipe.
struct Gallery(crate::gallery::GalleryUi);
impl Driven for Gallery {
    fn now(&self) -> f32 { self.0.now() }
    // The home page's walk hears how ready the page is to draw what comes next (it gives way).
    fn tick(&mut self, t: f32) { let r = self.0.readiness(); self.0.state.body.set_ready(r); self.0.tick(t) }
    fn apply(&mut self, input: &Input) -> Result<(), DriveError> { self.0.apply(input) }
    fn look(&self) -> Look { Driven::look(&self.0) }
    fn tree(&self) -> TreeNode { self.0.export_tree() }
    fn problems(&self) -> Vec<ViewProblem> { self.0.problems() }
    fn needs_frame(&self, t: f32) -> bool { self.0.needs_frame(t) }
    fn draw(&mut self, o: &mut Offscreen, look: &Look) -> Vec<u8> { o.render(&mut self.0, look) }
    fn recipe(&self) -> Option<String> { Some(self.0.state.recipe()) }
    fn ongoing(&self) -> bool { amenui::drive::Driven::ongoing(&self.0) }
    fn frame_drawn(&mut self) { amenui::drive::Driven::frame_drawn(&mut self.0) }
    fn audit(&mut self, o: &mut Offscreen, look: &Look, px: &[u8]) -> Vec<amenui::audit::Finding> { amenui::drive::Driven::audit(&mut self.0, o, look, px) }
}

/// Open a site page by its hash or path (the home page's state may follow after `#`: `body#source`), or
/// a recipe (`recipe-form`, or just `form`), at `w` × `h` device px and
/// `scale`, with the material and theme given (else the page's).
pub fn open(page: &str, faces: Vec<Face>, w: u32, h: u32, scale: f32, material: Option<Material>, theme: Option<usize>) -> Result<Box<dyn Driven>, LookError> {
    open_in(page, faces, (w, h, scale), (material, theme, None))
}
/// `open` with the look and the language given together: (material, theme, locale). Recipes keep
/// their own language.
pub fn open_in(page: &str, faces: Vec<Face>, (w, h, scale): (u32, u32, f32), (material, theme, locale): (Option<Material>, Option<usize>, Option<Locale>)) -> Result<Box<dyn Driven>, LookError> {
    use crate::gallery::{self, Msg, Page};
    let (name, state) = page.split_once('#').unwrap_or((page, ""));
    let found = if name.contains('/') { Page::from_path(name) } else { Page::from_hash(name) };
    let Some(p) = found else { return amenui::drive::open_in(page, faces, (w, h, scale), (material, theme, locale)) };
    let mut ui = gallery::new_ui(faces, w, h, scale);
    ui.pointer_leave();
    if let Some(l) = locale { gallery::update(&mut ui.state, Msg::Locale(l)); }
    gallery::update(&mut ui.state, Msg::Go(p));
    // `<path>#stage`: the page's stage alone, as a docs page's live stage shows it.
    if state == "stage" { ui.state.stage_only = true; }
    // `body#walk` plays the full intro from the first frame, `body#short` the short path.
    if p == Page::Body && !state.is_empty() { match state { "walk" => ui.state.body.open("", 2), "short" => ui.state.body.open("", 1), _ => ui.state.body.open(state, 0) } }
    // The whole page's material (the look strip's material is a component stage's only).
    if let Some(m) = material { (ui.state.material, ui.state.demo.material) = (m, m); }
    if let Some(t) = theme { gallery::update(&mut ui.state, Msg::Theme(t)); }
    ui.relayout();
    Ok(Box::new(Gallery(ui)))
}
/// Open, play, draw: the whole `render` tool as a function, for site pages and recipes.
pub fn render(spec: &RenderSpec, faces: Vec<Face>, o: Option<&mut Offscreen>) -> Result<Rendered, DriveError> {
    amenui::drive::render_with(spec, faces, o, &open_in)
}
/// `render` and the audit of the frame drawn (`amenui::audit`), for site pages and recipes.
pub fn audit(spec: &RenderSpec, faces: Vec<Face>, o: Option<&mut Offscreen>) -> Result<(Rendered, Vec<amenui::audit::Finding>), DriveError> {
    amenui::drive::audit_with(spec, faces, o, &open_in)
}
