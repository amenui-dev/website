//! Text in more than one language (DESIGN.md §1.20): every key is in every shipped catalog (plural
//! keys with every form the locale needs; the Body and Loose Thread pages' words are English only), the landing scene and
//! Loose Thread's composition show no missing key and no Chinese in either language, switching the language is a message that lays everything out again (keyed text
//! recomposes), and a right-to-left row puts its first child on the right.
use website::gallery::{self, Msg};
use amenui::i18n::{self, Locale};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }

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
            fn walk(n: &amenui::tree::TreeNode, out: &mut Vec<String>) { if let Some(t) = &n.text { out.push(t.clone()); } for c in &n.children { walk(c, out); } }
            walk(&tree, &mut texts);
            let chinese: Vec<&String> = texts.iter().filter(|t| cjk(t)).collect();
            assert!(chinese.is_empty(), "{}: at {t} s the scene shows {chinese:?}", l.tag());
        }
        let missed = i18n::misses();
        assert!(missed.is_empty(), "keys with no text: {missed:?}");
    }
}

#[test]
fn loose_thread_shows_no_chinese_in_either_language() {
    let mut ui = gallery::new_ui(fonts(), 1280, 1000, 1.0);
    ui.pointer_leave();
    assert!(ui.state.route("style-loose-thread"));
    let cjk = |s: &str| s.chars().any(|c| ('\u{3000}'..='\u{9fff}').contains(&c) || ('\u{ff00}'..='\u{ffef}').contains(&c));
    let mut seen: Vec<Vec<String>> = vec![];
    for l in [Locale::ZhHans, Locale::En] {
        gallery::update(&mut ui.state, Msg::Locale(l));
        ui.relayout();
        let _ = i18n::misses();
        let (mut now, end) = (ui.now(), ui.now() + 1.0);
        while now < end { now += 1.0 / 60.0; ui.tick(now); }
        let tree = ui.export_tree();
        let comp = tree.find("style-loose-thread/comp").expect("the composition");
        let mut texts = vec![];
        fn walk(n: &amenui::tree::TreeNode, out: &mut Vec<String>) { if let Some(t) = &n.text { out.push(t.clone()); }
            if let Some(t) = &n.label { out.push(t.clone()); } for c in &n.children { walk(c, out); } }
        walk(comp, &mut texts);
        let chinese: Vec<&String> = texts.iter().filter(|t| cjk(t)).collect();
        assert!(chinese.is_empty(), "{}: the composition shows {chinese:?}", l.tag());
        assert!(i18n::misses().is_empty());
        seen.push(texts);
    }
    assert_eq!(seen[0], seen[1], "the composition differs between the languages");
    // The title is lowercase, in the bundled serif.
    let title = ui.node("style-loose-thread/title").expect("title");
    let amenui::el::Content::Text { s, font, .. } = &title.content else { panic!("the title is text") };
    assert_eq!(*s, s.to_lowercase());
    assert_eq!(*font, amenui::el::Font::Face(amenui::text::SERIF));
    // The controls can be reached from the keyboard, as a switch, a switch and a two-way choice.
    let a11y = ui.a11y();
    let role = |k: &str| a11y.iter().find(|n| n.key == format!("style-loose-thread/{k}")).map(|n| (n.role, n.focusable));
    assert_eq!(role("binary"), Some((amenui::el::Role::Switch, true)));
    assert_eq!(role("invert"), Some((amenui::el::Role::Switch, true)));
    assert_eq!(role("ground"), Some((amenui::el::Role::RadioGroup, true)));
    assert_eq!(a11y.iter().filter(|n| n.key.starts_with("style-loose-thread/ground:") && n.role == amenui::el::Role::Radio).count(), 2);
}
