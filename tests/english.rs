//! Demos are in English whatever the interface language: with the language set to English no page's
//! tree holds a CJK character (but the framework's own files shown as they are: the agents' guide,
//! llms.txt, the release notes, the registry); in Simplified Chinese CJK appears only in the site's
//! chrome (the rail, the top bar, the search) and in the documentation's prose.
use website::gallery::{self, Msg, Page};
use amenui::i18n::Locale;
use amenui::tree::TreeNode;

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }
fn cjk(s: &str) -> bool { s.chars().any(|c| ('\u{3000}'..='\u{9fff}').contains(&c) || ('\u{ff00}'..='\u{ffef}').contains(&c)) }
/// What a key makes of the subtree under it: as its parent, to be checked, prose or chrome (not
/// checked, but a demo inside it is), or a demo (checked, all of it).
#[derive(Clone, Copy, PartialEq)]
enum Part { Same, Page, Prose, Demo }
/// Every text and label with CJK in it, with its key, where `part` says English is due.
fn walk(n: &TreeNode, on: u8, part: &dyn Fn(&str) -> Part, out: &mut Vec<String>) {
    let key = n.key.clone();
    let on = if on == 2 { 2 } else { match part(&key) { Part::Same => on, Part::Page => 1, Part::Prose => 0, Part::Demo => 2 } };
    if on > 0 { for t in [&n.text, &n.label].into_iter().flatten() { if cjk(t) { out.push(format!("{key}: {t}")); } } }
    for c in &n.children { walk(c, on, part, out); }
}
fn scan(l: Locale, part: &dyn Fn(&str) -> Part) -> Vec<String> { scan_pages(l, &|_| true, part) }
fn scan_pages(l: Locale, pages: &dyn Fn(Page) -> bool, part: &dyn Fn(&str) -> Part) -> Vec<String> {
    let mut ui = gallery::new_ui(fonts(), 1440, 900, 1.0);
    ui.pointer_leave();
    gallery::update(&mut ui.state, Msg::Locale(l));
    let mut bad = vec![];
    for p in Page::all().into_iter().filter(|&p| pages(p)) {
        gallery::update(&mut ui.state, Msg::Go(p));
        ui.relayout();
        let mut now = ui.now();
        let end = now + 0.5;
        while now < end { now += 1.0 / 30.0; ui.tick(now); }
        let mut found = vec![];
        let t = ui.export_tree();
        // In Chinese only the page under the chrome counts.
        let root = if l == Locale::En { Some(&t) } else { t.find(&format!("page/{}", p.hash())) };
        if let Some(r) = root { walk(r, 1, part, &mut found); }
        for f in found.into_iter() { bad.push(format!("{}: {f}", p.hash())); }
    }
    bad
}

#[test]
fn no_page_shows_cjk_in_english() {
    let bad = scan(Locale::En, &|_| Part::Same);
    assert!(bad.is_empty(), "{} places:\n{}", bad.len(), bad.join("\n"));
}

#[test]
fn the_demos_stay_english_in_chinese() {
    // The documentation's chrome and prose (`doc/…`) is in the interface language, but for a
    // component's previews, parameters and examples; so are an experiment's write-up and a page's
    // name and summary. The skeleton demo's theme choice names the theme by the theme's own name (the
    // framework's data).
    let part = |k: &str| {
        if k.ends_with("/preview-col") { Part::Page }
        // The index pages' cards are chrome; what a card shows of its page is a demo.
        else if k.starts_with("scenes/t") && k.ends_with("/box") || k.starts_with("index/") && k.ends_with("/preview") { Part::Demo }
        // (A scene's name on the list is the one its own page, in the interface language, flies it to.)
        else if k.starts_with("index/") || k.starts_with("labx/") || k == "scenes/head" || k.starts_with("scenes/title") || k.starts_with("scenes/card") || k == "scenes/body" || k == "theme" || k.contains("/doc/") || k.ends_with("/title") || k.ends_with("/desc") { Part::Prose }
        else if k.starts_with("doc/") { if k.ends_with("/preview") || k.ends_with("/box") || k.ends_with("/source") { Part::Demo } else { Part::Prose } }
        else { Part::Same }
    };
    let bad = scan_pages(Locale::ZhHans, &|p| !matches!(p, Page::Body), &part);
    assert!(bad.is_empty(), "{} places:\n{}", bad.len(), bad.join("\n"));
}
