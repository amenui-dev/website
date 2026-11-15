//! Text in more than one language (DESIGN.md §1.20): every key is in every shipped catalog (plural
//! keys with every form the locale needs), the film and its widgets show no missing key and no Chinese
//! in English, switching the language is a message that lays everything out again (keyed text
//! recomposes), and a right-to-left row puts its first child on the right.
use lab::el::{El, Paint, row, space};
use lab::gallery::{self, Msg, film};
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
        let missing: Vec<&String> = all.iter().filter(|k| !b.contains_key(*k)).collect();
        assert!(missing.is_empty(), "{} is missing {} keys: {missing:?}", l.tag(), missing.len());
        for (k, forms) in &b {
            if forms.contains("") { continue; }
            for need in l.categories() { assert!(forms.contains(*need), "{}: plural `{k}` has no `{need}` form", l.tag()); }
        }
    }
}

#[test]
fn the_film_in_english_has_every_text_and_no_chinese() {
    let mut ui = gallery::new_ui(fonts(), 1280, 640, 1.0);
    ui.pointer_leave();
    assert!(ui.state.route("film"));
    gallery::update(&mut ui.state, Msg::Locale(Locale::En));
    ui.relayout();
    let _ = i18n::misses();
    let cjk = |s: &str| s.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c));
    for c in 0..film::CHAPTERS {
        film::go(&mut ui.state, c, None);
        ui.relayout();
        let tree = ui.export_tree();
        let mut texts = vec![];
        fn walk(n: &lab::tree::TreeNode, out: &mut Vec<String>) { if let Some(t) = &n.text { out.push(t.clone()); } for c in &n.children { walk(c, out); } }
        walk(&tree, &mut texts);
        let chinese: Vec<&String> = texts.iter().filter(|t| cjk(t) && !t.contains("简体中文")).collect();
        assert!(chinese.is_empty(), "chapter {} in English still shows {chinese:?}", c + 1);
    }
    let missed = i18n::misses();
    assert!(missed.is_empty(), "keys with no text: {missed:?}");
    // The note followed the language; switching back is one message, laid out again.
    assert_eq!(ui.state.film.note.text, Locale::En.tr("film.c01.note-default"));
    let views = ui.counters().views;
    gallery::update(&mut ui.state, Msg::Locale(Locale::ZhHans));
    ui.relayout();
    assert!(ui.counters().views > views);
    assert_eq!(ui.state.film.note.text, Locale::ZhHans.tr("film.c01.note-default"));
    assert!(ui.export_tree().find("film/caption").and_then(|n| n.text.clone()).is_some_and(|t| cjk(&t)));
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
