//! The amenui website: the landing page (the Body scene), the Lab, the showcase scenes, a page for every
//! component, material, mark set and preset, and the documentation, all drawn by amenui itself. The same
//! code runs in a native window (`window`) and on the web (`web/`, built to one canvas).
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

/// The renderer's pictures, before the first frame.
pub fn setup(r: &mut amenui::render::Renderer) { for (id, w, h, px) in demo::pictures() { r.set_picture(id, w, h, &px); } }

/// The site in one `Ui`. The Body scene is the landing page; any other page is one hash away (`#gallery`
/// is the gallery's first, `#lab` the Lab).
pub fn make(faces: Vec<amenui::text::Face>, w: u32, h: u32, scale: f32) -> gallery::GalleryUi {
    let mut ui = gallery::new_ui(faces, w, h, scale);
    if ui.state.route("body") { ui.relayout(); }
    ui
}

// Web entry: the page (`web/page.html`) is a canvas that forwards events to the exports `web_app!` makes.
#[cfg(target_arch = "wasm32")]
amenui::web_app!(gallery::Gallery, gallery::Msg, setup, make);

/// The readout's choice: 0 hidden, 1 shown, 2 with more (the page keeps it in localStorage).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn readout() -> u8 { host(|s| s.ui.state.overlay).unwrap_or(0) }
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn set_readout(v: u8) { host(|s| { s.ui.state.overlay = v.min(2); s.ui.relayout(); }); }
