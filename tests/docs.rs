//! The documentation site (DESIGN.md §5): every component has a page with all seven sections, non-empty,
//! in both languages; every code sample on it is compiled (the examples are modules of the library,
//! and each `ex_*` is the preview it sits under); every internal link resolves to a page.
use lab::gallery::{self, Msg, Page, docs};
use lab::i18n::Locale;

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }
fn texts(n: &lab::tree::TreeNode, out: &mut Vec<String>) { if let Some(t) = &n.text { out.push(t.clone()); } for c in &n.children { texts(c, out); } }

#[test]
fn every_component_has_a_full_page_in_both_languages() {
    let mut ui = gallery::new_ui(fonts(), 1400, 900, 1.0);
    for l in Locale::ALL {
        gallery::update(&mut ui.state, Msg::Locale(l));
        for it in docs::items() {
            let h = docs::page_hash(&it.name);
            let p = Page::from_hash(&h).unwrap_or_else(|| panic!("no page for {}", it.name));
            gallery::update(&mut ui.state, Msg::Go(p));
            ui.relayout();
            let tree = ui.export_tree();
            for i in 0..7 {
                let sec = tree.find(&format!("doc/{h}/sec{i}")).unwrap_or_else(|| panic!("{} ({}): no section {i}", it.name, l.tag()));
                let mut t = vec![];
                texts(sec, &mut t);
                assert!(t.len() >= 2, "{} ({}): section {i} is empty: {t:?}", it.name, l.tag());
            }
            assert!(ui.problems().is_empty(), "{}: {:?}", it.name, ui.problems());
        }
    }
    assert!(lab::i18n::misses().is_empty(), "{:?}", lab::i18n::misses());
}

#[test]
fn every_code_sample_is_compiled_and_shown_with_its_preview() {
    for it in docs::items() {
        let Some(src) = docs::example_source(&it.name) else { continue };
        let ex = docs::examples_of(src);
        let n = lab::recipes::components::examples(match it.name.as_str() { "scroll-area" => "scroll", n => n }).len();
        assert!(!ex.is_empty() && ex.len() == n, "{}: {} samples, {} previews", it.name, ex.len(), n);
    }
    let with: Vec<String> = docs::items().into_iter().filter(|i| docs::example_source(&i.name).is_some()).map(|i| i.name).collect();
    assert!(with.len() >= 31, "examples for {with:?}");
}

#[test]
fn every_internal_link_resolves() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut n = 0;
    for dir in ["docs/site/zh-Hans", "docs/site/en"] {
        for e in std::fs::read_dir(format!("{root}/{dir}")).unwrap() {
            let p = e.unwrap().path();
            for l in lab::markdown::links(&std::fs::read_to_string(&p).unwrap()) {
                if l.starts_with('#') { assert!(Page::from_hash(&l).is_some(), "{}: {l} is no page", p.display()); n += 1; }
            }
        }
    }
    assert!(n > 4);
    for (i, _) in docs::DOCS.iter().enumerate() { for l in Locale::ALL { assert!(!docs::title(i, l).is_empty()); } }
}

#[test]
fn llms_lists_every_page_that_has_markdown() {
    let llms = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/llms.txt")).expect("llms.txt");
    for p in lab::gallery::Page::all() {
        if lab::gallery::docs::markdown(p, lab::i18n::Locale::En).is_some() {
            assert!(llms.contains(&format!("(md/en/{}.md): ", p.hash())), "llms.txt does not list {}", p.hash());
        }
    }
}
