//! What the web build needs from the site as JSON on stdout: the site's languages, every page's path,
//! section and title in each of them, the older hash names and the path each now has, and the home page's listing (the
//! first lines of the text the body is made of, which the page shows while the module loads).
//! `web/build.py` writes one HTML page per path from it.
use serde_json::{Value, json};
use website::gallery::{Msg, Page, routes};
use website::showcase::scenes::body;

fn main() {
    website::install_catalogs();
    let mut g = website::gallery::Gallery::default();
    let mut titles = |p: Page| -> Value {
        let mut o = serde_json::Map::new();
        for l in website::LOCALES {
            website::gallery::update(&mut g, Msg::Locale(l));
            o.insert(l.tag().into(), p.name().into());
        }
        Value::Object(o)
    };
    let mut seen = std::collections::HashSet::new();
    let mut pages = vec![];
    for p in Page::all() {
        if !seen.insert(p.path()) { continue; }
        pages.push(json!({ "path": p.path(), "hash": p.hash(), "section": p.section().name(), "title": titles(p) }));
    }
    for (path, p) in routes::aliases() {
        pages.push(json!({ "path": path, "hash": p.hash(), "section": p.section().name(), "title": titles(p), "alias": true }));
    }
    let mut redirects = serde_json::Map::new();
    for p in Page::all() { redirects.insert(p.hash(), p.path().into()); }
    redirects.insert("gallery".into(), "components/".into());
    let b = body::Body::default();
    let listing = b.listing();
    let locales: Vec<Value> = website::LOCALES.iter().map(|l| json!({ "tag": l.tag(), "name": l.name() })).collect();
    let out = json!({ "locales": locales, "pages": pages, "redirects": redirects, "listing": listing, "glyphs": b.glyphs() });
    println!("{}", serde_json::to_string(&out).expect("json"));
}
