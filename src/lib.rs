//! The amenui website: the home page (the Body scene), the Lab, the showcase scenes, a page for every
//! component, material, mark set and preset, and the documentation, all drawn by amenui itself. The same
//! code runs in a native window (`window`) and on the web (`web/`: a small HTML page per address with
//! the navigation in HTML, and the canvas under it).
pub mod demo;
#[cfg(not(target_arch = "wasm32"))]
pub mod drive;
pub mod gallery;
pub mod showcase;

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
pub fn setup(r: &mut amenui::render::Renderer) { for (id, w, h, px) in demo::pictures() { r.set_picture(id, w, h, &px); } }
/// One of the pictures, by id (`demo::pictures` makes them all): its width, height and RGBA8.
pub fn picture(id: u32) -> Option<(u32, u32, Vec<u8>)> {
    let f: fn(u32, u32) -> Vec<u8> = match id {
        demo::SPHERE => demo::picture_pixels, demo::SKY => demo::sky_pixels, demo::TERRAIN => demo::terrain_pixels, demo::BUST => demo::bust_pixels,
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

/// The site in one `Ui`. The Body scene is the landing page; any other page is one hash away (`#gallery`
/// is the gallery's first, `#lab` the Lab).
pub fn make(faces: Vec<amenui::text::Face>, w: u32, h: u32, scale: f32) -> gallery::GalleryUi {
    let mut ui = gallery::new_ui(faces, w, h, scale);
    if ui.state.route("body") { ui.relayout(); }
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
    /// page draws that frame again at once, whole, so no picture is ever seen blank.
    #[wasm_bindgen]
    pub fn pictures() -> bool {
        thread_local! { static MADE: std::cell::RefCell<Vec<u32>> = const { std::cell::RefCell::new(Vec::new()) }; }
        host(|s| {
            // A frame drawn whole for a new picture has been drawn: frames draw only what changed again.
            s.r.partial = true;
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
            // The next frame is drawn whole: what changed in the scene is not where the new picture is.
            s.r.partial = false;
            s.ui.invalidate();
            true
        }).unwrap_or(false)
    }
    /// The next tick is the frame's, at `t` (the page calls it just before `frame`): a tick an input
    /// brings in between steps the clocks and leaves the laying out to that input and to the frame.
    #[wasm_bindgen]
    pub fn framing(t: f32) { host(|s| s.ui.state.frame_at = Some(t)); }
    /// On a stage route, draw the specimen at the scale that keeps it inside the box (the page calls
    /// it after every frame; it lays out again only when the scale changes).
    #[wasm_bindgen]
    pub fn fit_stage() -> bool { host(|s| { let changed = gallery::fit_stage(&mut s.ui); if changed { s.ui.invalidate(); } changed }).unwrap_or(false) }
    /// The current page's path under the site's base (`components/button/`; empty for home).
    #[wasm_bindgen]
    pub fn path() -> String { host(|s| s.ui.state.page.path()).unwrap_or_default() }
    /// The current page's name, for the document's title.
    #[wasm_bindgen]
    pub fn title() -> String { host(|s| s.ui.state.page.name()).unwrap_or_default() }
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
    /// The rail row: `source`, `body`, `form`, `seed+`, `seed-`.
    #[wasm_bindgen]
    pub fn hero_press(what: &str, t: f32) {
        let m = match what {
            "source" => body::Msg::Show(1), "interface" => body::Msg::Show(2), "material" => body::Msg::Show(3), "body" => body::Msg::Show(4),
            "form" => body::Msg::Form, "seed+" => body::Msg::Seed(1), "seed-" => body::Msg::Seed(-1), "play" => body::Msg::Play, "skip" => body::Msg::Skip, _ => return,
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
    /// The home page's form (0 rock, 1 cloud) and seed, for the rail row.
    #[wasm_bindgen]
    pub fn hero_form() -> Vec<u32> { host(|s| vec![s.ui.state.body.form, s.ui.state.body.seed]).unwrap_or_default() }
    /// The body's circle on the canvas (centre x, y and radius, CSS px), empty while it is the listing:
    /// a touch that starts inside it turns the body instead of scrolling the page.
    #[wasm_bindgen]
    pub fn hero_circle() -> Vec<f32> {
        host(|s| { let b = &s.ui.state.body; if s.ui.state.page != gallery::Page::Body || b.target() < 0.5 { vec![] } else { let (x, y, r) = b.placement(); vec![x, y, r] } }).unwrap_or_default()
    }
}
