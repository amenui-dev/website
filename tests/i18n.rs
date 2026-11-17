//! Text in more than one language (DESIGN.md §1.20): every key is in every shipped catalog (plural
//! keys with every form the locale needs; the Body page's words are English only), the landing scene shows no
//! missing key and no Chinese in either language, switching the language is a message that lays everything out again (keyed text
//! recomposes), and a right-to-left row puts its first child on the right.
use lab::el::{El, Paint, row, space};
use lab::gallery::{self, Msg};
use lab::i18n::{self, Locale};
use std::collections::BTreeSet;

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }
/// A catalog's messages by their base key (plural forms folded in), with the forms each gives.
fn bases(l: Locale) -> std::collections::BTreeMap<String, BTreeSet<String>> {
    let mut m = std::collections::BTreeMap::new();
    for k in i18n::catalog(l).keys() {
        let (base, form) = match k.rsplit_once('.') {
            Some((b, f)) if ["zero", "one", "two", "few", "many", "other"].contains(&f) => (b.to_owned(), f.to_owned()),
            _ => (k.clone(), String::new()),
        };
        m.entry(base).or_insert_with(BTreeSet::new).insert(form);
    }
    m
}

#[test]
fn every_key_is_in_every_locale() {
    let all: BTreeSet<String> = Locale::ALL.iter().flat_map(|l| bases(*l).into_keys()).collect();
    for l in Locale::ALL {
        let b = bases(l);
        // The Body page is English whatever the language (its words exist only in English).
        let english_only = |k: &str| l != Locale::En && k.starts_with("body.") && k != "body.label";
        let missing: Vec<&String> = all.iter().filter(|k| !b.contains_key(*k) && !english_only(k)).collect();
        assert!(missing.is_empty(), "{} is missing {} keys: {missing:?}", l.tag(), missing.len());
        for (k, forms) in &b {
            if forms.contains("") { continue; }
            for need in l.categories() { assert!(forms.contains(*need), "{}: plural `{k}` has no `{need}` form", l.tag()); }
        }
    }
}

#[test]
fn the_landing_scene_shows_no_chinese_in_either_language() {
    let mut ui = gallery::new_ui(fonts(), 1280, 800, 1.0);
    ui.pointer_leave();
    assert!(ui.state.route("body"));
    let cjk = |s: &str| s.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
    for l in [Locale::En, Locale::ZhHans] {
        gallery::update(&mut ui.state, Msg::Locale(l));
        ui.relayout();
        let _ = i18n::misses();
        // On the page and in the body (its markings shown).
        for t in [1.0f32, 6.0] {
            let mut now = ui.now();
            while now < t { now += 1.0 / 60.0; ui.tick(now); }
            let tree = ui.export_tree();
            let mut texts = vec![];
            fn walk(n: &lab::tree::TreeNode, out: &mut Vec<String>) { if let Some(t) = &n.text { out.push(t.clone()); } for c in &n.children { walk(c, out); } }
            walk(&tree, &mut texts);
            let chinese: Vec<&String> = texts.iter().filter(|t| cjk(t)).collect();
            assert!(chinese.is_empty(), "{}: at {t} s the scene shows {chinese:?}", l.tag());
        }
        let missed = i18n::misses();
        assert!(missed.is_empty(), "keys with no text: {missed:?}");
        // The language marking names the language the rest of the gallery is in.
        let lang = ui.export_tree().find("body/mark/lang").and_then(|n| n.text.clone()).unwrap_or_default();
        assert!(lang.ends_with(if l == Locale::En { "EN" } else { "ZH" }), "{lang}");
    }
}

#[test]
fn a_right_to_left_row_starts_at_the_right() {
    fn view(rtl: &bool) -> El<()> {
        row().key("row").width(300.0).rtl(*rtl).child(space(50.0, 20.0).fill(Paint::INK).key("first")).child(space(50.0, 20.0).fill(Paint::INK).key("second"))
    }
    let mut ui = lab::ui::Ui::new(false, view, |s: &mut bool, _: ()| { *s = !*s; }, fonts(), 400, 200, 1.0);
    let (a, b) = (ui.node("first").unwrap().rect.x, ui.node("second").unwrap().rect.x);
    assert!(a < b, "left to right: {a} {b}");
    ui.state = true;
    ui.relayout();
    let (a, b) = (ui.node("first").unwrap().rect.x, ui.node("second").unwrap().rect.x);
    assert!(a > b, "right to left, the first child is on the right: {a} {b}");
    assert!(a + 50.0 > 299.0, "and starts at the row's right edge: {a}");
}
