//! Addresses: every page has a path under the site's base (`components/button/`, `lab/lumen/`,
//! `docs/concepts/layout/`, `works/ground-truth/`; the home page is the empty path) and belongs to one
//! of the four sections. The older hash names (`#button`) still name the same pages, and the scenes'
//! older paths (`scenes/reading/`, now `lab/scenes/reading/`) theirs; the web shell redirects both.
use super::{MARK_HASH, Page, docs, lab, scenes, works};
use amenui::style::{Material, styles};

/// The four sections of the site, in the navigation bar's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section { Home, Docs, Components, Lab, Works }
impl Section {
    pub fn name(self) -> &'static str {
        match self { Section::Home => "home", Section::Docs => "docs", Section::Components => "components", Section::Lab => "lab", Section::Works => "works" }
    }
}

/// The docs groups (`docs/<group>/<slug>/`): which group each prose page is in.
pub fn doc_group(slug: &str) -> &'static str {
    match slug {
        "introduction" | "installation" | "first-view" | "project-structure" => "start",
        "forms" | "theming" | "i18n" | "rtl" | "accessibility" | "testing" | "performance" => "guides",
        "cli" | "registry-schema" => "reference",
        _ => "concepts",
    }
}
/// The docs groups in their order, with their title keys.
pub const DOC_GROUPS: [(&str, &str); 4] = [("start", "nav.start"), ("guides", "nav.guides"), ("concepts", "nav.concepts"), ("reference", "nav.reference")];

impl Page {
    /// The page's path under the site's base: no leading slash, a trailing one; empty for home.
    pub fn path(self) -> String {
        match self {
            Page::Body => String::new(),
            Page::Components => "components/".into(),
            Page::Comp(_) | Page::Widget(_) => format!("components/{}/", self.hash()),
            Page::LongList => "components/list/10k/".into(),
            Page::Lab => "lab/".into(),
            Page::Experiment(i) => format!("lab/{}/", lab::NAMES[i as usize]),
            Page::Reader => "lab/reader/".into(),
            Page::Decode => "lab/decode/".into(),
            Page::Stirr => "lab/field-map/".into(),
            Page::Presets => "lab/looks/presets/".into(),
            Page::Style(i) => format!("lab/looks/presets/{}/", styles()[i as usize].hash),
            Page::Marks(s) => format!("lab/looks/marks/{}/", MARK_HASH[s as usize]),
            Page::Material(m) => format!("lab/looks/materials/{}/", Material::from_id(m).name()),
            Page::Gaze => "lab/looks/annotation/".into(),
            Page::Type => "lab/looks/type/".into(),
            Page::Morph => "lab/motion/morph/".into(),
            Page::Particles => "lab/motion/particles/".into(),
            Page::Flow => "lab/motion/flow/".into(),
            Page::Three => "lab/motion/three/".into(),
            Page::Demo => "lab/skeleton/".into(),
            Page::Scenes => "lab/scenes/".into(),
            Page::Scene(i) => format!("lab/scenes/{}/", scenes::SCENES[i as usize].0),
            Page::Works => "works/".into(),
            Page::Work(i) => format!("works/{}/", works::PIECES[i as usize].slug),
        }
    }
    /// The page a path names (with or without its slashes). `None` for a path that names none (the
    /// docs are HTML pages, not routes of the canvas application).
    pub fn from_path(p: &str) -> Option<Page> {
        let p = p.trim_matches('/');
        if p.is_empty() { return Some(Page::Body); }
        // The scenes were a section of their own before they joined the Lab.
        let want = if p == "scenes" || p.starts_with("scenes/") { format!("lab/{p}/") } else { format!("{p}/") };
        Page::all().into_iter().find(|q| q.path() == want)
    }
    pub fn section(self) -> Section {
        match self {
            Page::Body => Section::Home,
            Page::Components | Page::Comp(_) | Page::Widget(_) | Page::LongList => Section::Components,
            Page::Works | Page::Work(_) => Section::Works,
            _ => Section::Lab,
        }
    }
}

/// The aliases the build writes a page for besides every page's own path: (path, page it shows).
pub fn aliases() -> Vec<(&'static str, Page)> { vec![] }
/// The paths pages had before they moved, and the page each leads to: the scenes, which were a
/// section of their own (`scenes/`, `scenes/<name>/`) and are now in the Lab.
pub fn moved() -> Vec<(String, Page)> {
    std::iter::once(Page::Scenes).chain((0..scenes::COUNT as u8).map(Page::Scene)).map(|p| (p.path()["lab/".len()..].to_owned(), p)).collect()
}
/// The older hash names of the docs pages, which are HTML now, and the path each has: the prose pages
/// (`#docs-layout`), the For-AI pages, the registry and the changelog.
pub fn doc_redirects() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = docs::DOCS.iter().map(|s| (format!("docs-{s}"), format!("docs/{}/{s}/", doc_group(s)))).collect();
    v.extend([("ai-agents", "docs/reference/agents/"), ("ai-llms", "docs/reference/llms/"), ("ai-registry", "docs/reference/registry/"), ("changelog", "docs/release-notes/")].map(|(h, p)| (h.to_owned(), p.to_owned())));
    v
}
