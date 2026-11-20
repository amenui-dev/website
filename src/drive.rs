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
struct Gallery(crate::gallery::GalleryUi, bool);
impl Driven for Gallery {
    fn now(&self) -> f32 { self.0.now() }
    // The home page's walk hears how ready the page is to draw what comes next (it gives way).
    fn tick(&mut self, t: f32) { let r = self.0.readiness(); self.0.state.body.set_ready(r); self.0.tick(t); }
    fn apply(&mut self, input: &Input) -> Result<(), DriveError> { self.0.apply(input) }
    fn look(&self) -> Look { Driven::look(&self.0) }
    fn tree(&self) -> TreeNode { self.0.export_tree() }
    fn problems(&self) -> Vec<ViewProblem> { self.0.problems() }
    fn needs_frame(&self, t: f32) -> bool { self.0.needs_frame(t) }
    fn draw(&mut self, o: &mut Offscreen, look: &Look) -> Vec<u8> {
        // The offscreen renderer has the framework's sample pictures; the works' pictures are the site's.
        if !self.1 { crate::gallery::works::set_pictures(&mut o.r); self.1 = true; }
        o.render(&mut self.0, look)
    }
    fn recipe(&self) -> Option<String> { Some(self.0.state.recipe()) }
    fn ongoing(&self) -> bool { amenui::drive::Driven::ongoing(&self.0) }
    fn frame_drawn(&mut self) { amenui::drive::Driven::frame_drawn(&mut self.0) }
    fn audit(&mut self, o: &mut Offscreen, look: &Look, px: &[u8]) -> Vec<amenui::audit::Finding> { amenui::drive::Driven::audit(&mut self.0, o, look, px) }
}

/// Pages opened from here are in the light scheme (paper) rather than the dark one: the renderer's
/// `--scheme light`, as a browser whose system is in light mode opens them.
pub static PAPER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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
    // Routes after this one are navigations inside the running app (a flow, a commit), as on the web
    // after the first route.
    ui.state.routed = true;
    // `<path>#stage`: the page's stage alone, as a docs page's live stage shows it.
    if state == "stage" { ui.state.stage_only = true; }
    // `<path>#still`: the same, for its still: a specimen closed at rest is drawn open.
    if state == "still" { (ui.state.stage_only, ui.state.still) = (true, true); gallery::hold_still(&mut ui.state); }
    // `<path>#reduced`: the page as reduced motion draws it, as on the web with the preference set.
    if state == "reduced" { amenui::app::App::set_reduced_motion(&mut ui.state, true); }
    // `body#walk` plays the full intro from the first frame, `body#short` the short path.
    if p == Page::Body && !state.is_empty() { match state { "walk" => ui.state.body.open("", 2), "short" => ui.state.body.open("", 1), _ => ui.state.body.open(state, 0) } }
    // The whole page's material (the look strip's material is a component stage's only).
    if let Some(m) = material { (ui.state.material, ui.state.demo.material) = (m, m); }
    if let Some(t) = theme { gallery::update(&mut ui.state, Msg::Theme(t)); }
    // The scheme (AA.10): the dark one unless the renderer was asked for paper (`render --scheme light`).
    ui.state.set_scheme(PAPER.load(std::sync::atomic::Ordering::Relaxed), None);
    ui.relayout();
    let fps = gallery::works::ambient_fps(ui.state.page);
    ui.ambient_fps(fps);
    Ok(Box::new(Gallery(ui, false)))
}
/// Open, play, draw: the whole `render` tool as a function, for site pages and recipes.
pub fn render(spec: &RenderSpec, faces: Vec<Face>, o: Option<&mut Offscreen>) -> Result<Rendered, DriveError> {
    amenui::drive::render_with(spec, faces, o, &open_in)
}
/// `render` and the audit of the frame drawn (`amenui::audit`), for site pages and recipes.
pub fn audit(spec: &RenderSpec, faces: Vec<Face>, o: Option<&mut Offscreen>) -> Result<(Rendered, Vec<amenui::audit::Finding>), DriveError> {
    amenui::drive::audit_with(spec, faces, o, &open_in)
}
/// `render` with the clock stepped at `fps` frames a second and every frame drawn, as a slow machine
/// draws them (the `render` tool's `--fps`): what is on the screen at `at_ms` shows whether motion keeps
/// the clock's time. The first frame is presented `first` s late (the next one comes then), as a
/// browser that takes that long to make its pipelines. The PNG is the last frame drawn.
pub fn render_paced(spec: &RenderSpec, faces: Vec<Face>, fps: f32, first: f32) -> Result<Rendered, DriveError> {
    let locale = spec.locale.as_deref().and_then(Locale::from_tag);
    let mut d = open_in(&spec.page, faces, (spec.w, spec.h, spec.scale.max(0.25)), (None, None, locale))?;
    let mut o = Offscreen::try_new(spec.w, spec.h)?;
    // Each event's time in s: `at_ms`, or its place on the look's beat (`at`).
    let beat = d.look().beat;
    let mut order = Vec::with_capacity(spec.events.len());
    for (i, e) in spec.events.iter().enumerate() {
        let t = match &e.at {
            Some(at) => beat.parse_at(at).ok_or_else(|| DriveError::AtEvent { index: i, inner: Box::new(DriveError::BadBeatTime { at: at.clone() }) })?,
            None => e.at_ms / 1000.0,
        };
        order.push((i, t, e));
    }
    order.sort_by(|a, b| a.1.total_cmp(&b.1));
    let (end, step) = (spec.at_ms.max(0.0) / 1000.0, 1.0 / fps.max(0.1));
    let (mut next, mut k, mut px) = (0, 0u32, vec![]);
    let first = first.max(step);
    loop {
        let t = if k == 0 { 0.0 } else { (first + (k - 1) as f32 * step).min(end) };
        while next < order.len() && order[next].1 <= t + 1e-6 {
            d.tick(t);
            let (i, _, e) = order[next];
            d.apply(&e.input).map_err(|err| DriveError::AtEvent { index: i, inner: Box::new(err) })?;
            next += 1;
        }
        d.tick(t);
        // A frame is drawn only when one is needed, as the web host does.
        if k == 0 || d.needs_frame(t) { let look = d.look(); px = d.draw(&mut o, &look); d.frame_drawn(); }
        if t >= end { break; }
        k += 1;
    }
    Ok(Rendered { png: amenui::offscreen::png_bytes(spec.w, spec.h, &px), tree: d.tree(), problems: d.problems() })
}
