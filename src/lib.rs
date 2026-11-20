//! The amenui website: the home page (the Body scene), the Lab, the showcase scenes, a page for every
//! component, material, mark set and preset, and the documentation, all drawn by amenui itself. The same
//! code runs in a native window (`window`) and on the web (`web/`: a small HTML page per address with
//! the navigation in HTML, and the canvas under it).
pub mod demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod drive;
pub mod gallery;
pub mod showcase;
pub mod texts;
pub mod tokens;

/// The site's message catalogs (`assets/i18n/`), installed over amenui's own.
pub const CATALOGS: [(amenui::i18n::Locale, &str, &str); 2] = [
    (amenui::i18n::Locale::ZhHans, "assets/i18n/zh-Hans.txt", include_str!("../assets/i18n/zh-Hans.txt")),
    (amenui::i18n::Locale::En, "assets/i18n/en.txt", include_str!("../assets/i18n/en.txt")),
];
/// The languages the site is written in (amenui itself ships more).
pub const LOCALES: [amenui::i18n::Locale; 2] = [amenui::i18n::Locale::ZhHans, amenui::i18n::Locale::En];

/// Install the site's catalogs, once (`gallery::new_ui` and the Body scene call it).
pub fn install_catalogs() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| amenui::i18n::install(&CATALOGS).unwrap_or_else(|e| panic!("{e}")));
}

/// What the shells ask of the site beyond its view: routes are page hashes, a copy button's text goes to
/// the clipboard, and the host can set the look, the language and reduced motion.
impl amenui::app::App for gallery::Gallery {
    fn route(&mut self, hash: &str) -> bool { gallery::Gallery::route(self, hash) }
    fn hash(&self) -> String { self.page.hash() }
    fn take_clipboard(&mut self) -> Option<String> { self.clipboard.take() }
    fn set_locale(&mut self, locale: amenui::i18n::Locale) -> bool { gallery::update(self, gallery::Msg::Locale(locale)); true }
    fn set_look(&mut self, theme: &str, material: &str) -> bool {
        let mut changed = false;
        if let Ok(t) = amenui::style::theme_named(theme) { gallery::update(self, gallery::Msg::Theme(t)); changed = true; }
        if let Ok(m) = amenui::style::material_named(material) { gallery::update(self, gallery::Msg::Material(m.id() as usize)); changed = true; }
        changed
    }
    fn set_reduced_motion(&mut self, on: bool) { self.lab.reduced = on; self.body.reduced = on; }
}

/// The renderer's pictures (photographs drawn in marks), before the first frame.
pub fn setup(r: &mut amenui::render::Renderer) {
    for (id, w, h, px) in demo::pictures() { r.set_picture(id, w, h, &px); }
    gallery::works::set_pictures(r);
}
/// One of the pictures, by id (`demo::pictures` makes them all): its width, height and RGBA8.
pub fn picture(id: u32) -> Option<(u32, u32, Vec<u8>)> {
    if let Some(p) = gallery::works::picture_pixels(id) { return Some(p); }
    let f: fn(u32, u32) -> Vec<u8> = match id {
        demo::SPHERE => demo::picture_pixels, demo::SKY => demo::sky_pixels, demo::TERRAIN => demo::terrain_pixels, demo::TREE => demo::tree_pixels,
        demo::CUTOUT => demo::cutout_pixels, demo::DUSK => demo::dusk_pixels, demo::PLANET => demo::planet_pixels, demo::FLOWER => demo::flower_pixels,
        _ => return None,
    };
    let (w, h) = *demo::PICTURE_SIZES.get(id as usize)?;
    Some((w, h, f(w, h)))
}
/// On the web each picture waits until a page draws it (`pictures`): home shows none, most pages one
/// or two, and making them all is most of what starting a page would cost.
#[cfg(target_arch = "wasm32")]
fn web_setup(_: &mut amenui::render::Renderer) {}

/// The serif (Source Serif 4 at 8 pt, Basic Latin, its one weight; OFL, `assets/fonts/SourceSerif4-OFL.txt`),
/// compiled into the web module.
#[cfg(target_arch = "wasm32")]
const SERIF: &[u8] = include_bytes!("../assets/fonts/SourceSerif4-8pt-subset.ttf");

/// The site in one `Ui`. The Body scene is the landing page; any other page is one hash away (`#gallery`
/// is the gallery's first, `#lab` the Lab).
pub fn make(faces: Vec<amenui::text::Face>, w: u32, h: u32, scale: f32) -> gallery::GalleryUi {
    let mut ui = gallery::new_ui(faces, w, h, scale);
    // On the web the module carries one face beyond the three the page passes in: the serif, which
    // titles across the site are set in. The five Geist Pixel faces are fetched when a page uses one.
    #[cfg(target_arch = "wasm32")]
    if let Err(e) = ui.add_face(amenui::text::SERIF_FILE, SERIF.to_vec()) { panic!("{e}"); }
    if ui.state.route("body") { ui.relayout(); }
    // The first layouts are of the gallery's first page and of home, before the page's own route: what
    // they asked for is not wanted.
    #[cfg(target_arch = "wasm32")]
    texts::forget();
    ui
}

// Web entry: the page (`web/shell.html`, `web/app.js`) is a canvas that forwards events to the exports
// `web_app!` makes. The page passes the address's path (`components/button/`, empty for home) or an older
// hash name as the route.
#[cfg(target_arch = "wasm32")]
amenui::web_app!(gallery::Gallery, gallery::Msg, web_setup, make);

/// The readout's choice: 0 hidden, 1 shown, 2 with more (the page keeps it in localStorage).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn readout() -> u8 { host(|s| s.ui.state.overlay).unwrap_or(0) }
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_readout(v: u8) { host(|s| { s.ui.state.overlay = v.min(2); s.ui.relayout(); }); }
/// The material resolution the readout's quality choice sets: 0 automatic, else a level of `perf::LEVELS`.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn quality() -> f32 { host(|s| s.ui.state.quality).unwrap_or(1.0) }
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_quality(q: f32) { host(|s| { s.ui.state.quality = if q <= 0.0 { 0.0 } else { q.min(1.0) }; s.ui.relayout(); }); }

#[cfg(target_arch = "wasm32")]
mod site_exports {
    use super::{gallery, host};
    use crate::showcase::scenes::body;
    use wasm_bindgen::prelude::wasm_bindgen;

    /// Make the pictures the page's scene draws that the renderer does not have yet, each once. The
    /// page calls it after a route and after every frame; when it made one it returns true and the
    /// page draws that frame again at once, each new picture where it is, so no picture is ever seen
    /// blank.
    #[wasm_bindgen]
    pub fn pictures() -> bool {
        thread_local! { static MADE: std::cell::RefCell<Vec<u32>> = const { std::cell::RefCell::new(Vec::new()) }; }
        host(|s| {
            let mut want: Vec<u32> = s.ui.scene().instances.iter().filter(|i| i.paint[3] == amenui::scene::PICTURE).map(|i| i.extra[1] as u32).collect();
            want.sort_unstable();
            want.dedup();
            MADE.with(|m| {
                let mut m = m.borrow_mut();
                want.retain(|id| !m.contains(id));
                for &id in &want {
                    if let Some((w, h, px)) = super::picture(id) { s.r.set_picture(id, w, h, &px); }
                    m.push(id);
                }
            });
            if want.is_empty() { return false; }
            // The next frame draws each new picture again where it is (`Renderer::set_picture`).
            s.ui.invalidate();
            true
        }).unwrap_or(false)
    }
    /// A text the module does not carry (`texts`), given by the page: the view is laid out again.
    #[wasm_bindgen]
    pub fn put_text(name: &str, body: String) { crate::texts::put(name, body); host(|s| s.ui.relayout()); }
    /// The texts the module has asked for since the last call, one a line, for the page to fetch.
    #[wasm_bindgen]
    pub fn wanted_texts() -> String { crate::texts::wanted().join("\n") }
    /// Whether the pointer can hover: without (touch), the Lab index runs its centred cell.
    #[wasm_bindgen]
    pub fn set_hover(on: bool) { host(|s| s.ui.state.lab.coarse = !on); }
    /// The current page's path under the site's base (`components/button/`; empty for home).
    /// A piece of Works, for the page around the canvas: [the bar's tone (0 none, 1 light, 2 dark),
    /// the chrome faded (1), the piece resting (1), the status line's tone].
    #[wasm_bindgen]
    pub fn works_state() -> Vec<u8> {
        host(|s| { let g = &s.ui.state; vec![gallery::works::tone(g.page), (g.works.fade.1 < 1.0) as u8, g.works.resting as u8, gallery::works::foot_tone(g.page)] }).unwrap_or_default()
    }
    /// Pace what moves by itself on the current page (`Ui::ambient_fps`): 30 frames a second on a
    /// page of Works (M.7), no pacing elsewhere. The page calls it when its path changes.
    #[wasm_bindgen]
    pub fn pace() { host(|s| { let n = gallery::works::ambient_fps(s.ui.state.page); s.ui.ambient_fps(n); }); }
    /// An HTML page a press asked for (a piece's recipe), once.
    #[wasm_bindgen]
    pub fn take_navigate() -> String { host(|s| s.ui.state.navigate.take()).flatten().unwrap_or_default() }
    #[wasm_bindgen]
    pub fn path() -> String { host(|s| s.ui.state.page.path()).unwrap_or_default() }
    /// The current page's name, for the document's title.
    #[wasm_bindgen]
    pub fn title() -> String { host(|s| s.ui.state.page.name()).unwrap_or_default() }
    /// The stages' scale while it is below the device's (`stage 1.5x`), empty at the device's own.
    #[wasm_bindgen]
    pub fn stage() -> String { host(|s| s.ui.perf().stage_label()).flatten().unwrap_or_default() }
    /// A new route: the stages' scale steps from the device's own again.
    #[wasm_bindgen]
    pub fn reset_stage() { host(|s| s.ui.reset_stage_scale()); }
    /// How the last frame was drawn (`perf::FramePath::line`): whether it moved the last frame and, if
    /// not, why; whether the material pass was the specialised one; the share shaded; the scales.
    #[wasm_bindgen]
    pub fn frame_path() -> String { host(|s| s.ui.perf().path.line()).unwrap_or_default() }
    /// Force the material pass made for the page's material on (true) or off (false), or leave it to
    /// the renderer (null), from the next frame (`Renderer::spec_force`), for an A/B on one page.
    #[wasm_bindgen]
    pub fn spec_force(on: Option<bool>) { host(|s| { s.r.spec_force = on; s.ui.invalidate(); }); }
    /// The graphics backend in use: `webgpu` or `webgl2`.
    #[wasm_bindgen]
    pub fn backend() -> String { host(|s| s.ui.perf().backend.to_lowercase()).unwrap_or_default() }
    /// How many glyphs the page draws (on home, the body is made of).
    #[wasm_bindgen]
    pub fn glyphs() -> u32 {
        host(|s| if s.ui.state.page == gallery::Page::Body { s.ui.state.body.glyphs() as u32 } else { s.ui.scene().instances.iter().filter(|i| i.paint[3] < 0.5).count() as u32 }).unwrap_or(0)
    }
    /// The home page's stage (CSS px from the canvas's top left), laid out by the page, and the optical
    /// centre of its text block (vertical, same space; negative for none); a zero-sized stage gives the
    /// scene's own.
    #[wasm_bindgen]
    pub fn set_stage(x0: f32, y0: f32, x1: f32, y1: f32, centre: f32) {
        host(|s| { s.ui.state.body.set_stage(Some([x0, y0, x1, y1])); s.ui.state.body.set_text_centre((centre >= 0.0).then_some(centre)); s.ui.relayout(); });
    }
    /// The home page's state for the address (`source`, `body/cloud/08`, empty for the defaults).
    #[wasm_bindgen]
    pub fn hero() -> String { host(|s| s.ui.state.body.state()).unwrap_or_default() }
    /// Open a state from the address at rest; an empty one plays the short path when `intro` is set.
    #[wasm_bindgen]
    pub fn set_hero(state: &str, walk: u8) { host(|s| { s.ui.state.body.open(state, walk); s.ui.relayout(); }); }
    /// The rail row: the four states, `play`, `skip`, `seed+`, `seed-`; and `hidden` / `shown` when the
    /// tab or the stage goes out of view and comes back (nothing turns by itself out of view).
    #[wasm_bindgen]
    pub fn hero_press(what: &str, t: f32) {
        let m = match what {
            "source" => body::Msg::Show(1), "interface" => body::Msg::Show(2), "material" => body::Msg::Show(3), "body" => body::Msg::Show(4),
            "seed+" => body::Msg::Seed(1), "seed-" => body::Msg::Seed(-1), "play" => body::Msg::Play, "skip" => body::Msg::Skip,
            "hidden" => body::Msg::Away(true), "shown" => body::Msg::Away(false), _ => return,
        };
        host(|s| { s.ui.tick(t); gallery::update(&mut s.ui.state, gallery::Msg::Body(m)); s.ui.relayout(); });
    }
    /// The walk (0 none, 1 playing, 2 stopped by a person), whether it has been seen through, and the
    /// state on the stage (1 to 4).
    #[wasm_bindgen]
    pub fn hero_walk() -> Vec<u32> { host(|s| { let b = &s.ui.state.body; vec![b.walking() as u32, b.seen as u32, b.at as u32] }).unwrap_or_default() }
    /// Where the next in-app navigation was pressed (CSS px on the canvas; a link's centre for a
    /// keyboard activation): its front starts there.
    #[wasm_bindgen]
    pub fn pressed_at(x: f32, y: f32) { host(|s| s.ui.state.pressed_at = Some((x, y))); }
    /// The home page's seed, for the rail row on a phone.
    #[wasm_bindgen]
    pub fn hero_seed() -> u32 { host(|s| s.ui.state.body.seed).unwrap_or(0) }
    /// The page has the HTML status line: it carries the one-line readout's numbers (AA.6).
    #[wasm_bindgen]
    pub fn set_status_line(on: bool) { host(|s| s.ui.state.status_line = on); }
    /// The scheme the page is in (AA.10): paper (light) or graphite (dark). A change made with the bar's
    /// switch comes with where it was pressed (CSS px on the canvas) and is a commit from there; one
    /// made by the system (NaN) is a cut. Either way only the look's colours change.
    #[wasm_bindgen]
    pub fn set_scheme(paper: bool, x: f32, y: f32) {
        host(|s| { let from = (x.is_finite() && y.is_finite()).then_some((x, y)); s.ui.state.set_scheme(paper, from); s.ui.relayout(); });
    }
    /// The body's circle on the canvas (centre x, y and radius, CSS px), empty while it is the listing:
    /// a touch that starts inside it turns the body instead of scrolling the page.
    #[wasm_bindgen]
    pub fn hero_circle() -> Vec<f32> {
        host(|s| { let b = &s.ui.state.body; if s.ui.state.page != gallery::Page::Body || b.target() < 0.5 { vec![] } else { let (x, y, r) = b.placement(); vec![x, y, r] } }).unwrap_or_default()
    }
}
