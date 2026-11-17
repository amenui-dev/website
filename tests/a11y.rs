//! The accessibility mirror (DESIGN.md §1.14), without a GPU: roles, names and states from the widgets;
//! a dialog alone while it is open; activation routed to the same messages; focus from assistive
//! technology; the keyed diff sends only what changed.
use amenui::el::Role;
use amenui::gallery::{self, GalleryUi, Msg, Page};
use amenui::ui::{A11yNode, A11yOp};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }
fn at(p: &str) -> GalleryUi {
    let mut ui = gallery::new_ui(fonts(), 1280, 1800, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash(p).unwrap()));
    ui.relayout();
    ui.tick(1.0);
    ui
}
fn find<'a>(t: &'a [A11yNode], role: Role, name: &str) -> Option<&'a A11yNode> { t.iter().find(|n| n.role == role && n.name == name) }

#[test]
fn buttons_and_activation() {
    let mut ui = at("button");
    let t = ui.a11y();
    let b = find(&t, Role::Button, "按下").expect("the live button");
    assert!(b.focusable && !b.disabled);
    assert!(find(&t, Role::Button, "不可用").expect("a disabled one").disabled);
    assert!(find(&t, Role::Heading, "按钮").is_some_and(|h| h.level == 1), "the page title is a heading");
    assert!(t.iter().any(|n| n.role == Role::Text && n.name == "按下了 0 次"), "text outside controls");
    // Text inside a button is its name, not a separate element.
    assert!(!t.iter().any(|n| n.role == Role::Text && n.name == "按下"));
    let key = b.key.clone();
    let first = ui.a11y_diff();
    assert!(first.len() > 20);
    assert!(ui.a11y_diff().is_empty(), "nothing changed");
    assert!(ui.activate(&key));
    assert_eq!(ui.state.presses, 1, "the same message as a press");
    assert_eq!(ui.focus(), Some(key.as_str()));
    let ops = ui.a11y_diff();
    assert!(ops.iter().any(|o| matches!(o, A11yOp::Set(n) if n.name == "按下了 1 次")), "the count changed: {ops:?}");
    assert!(ops.iter().any(|o| matches!(o, A11yOp::Focus(Some(k)) if *k == key)));
    assert!(ops.len() < 6, "only what changed: {}", ops.len());
}

#[test]
fn a_dialog_alone_while_open() {
    let mut ui = at("dialog");
    assert!(ui.activate("dialog/open"));
    let t = ui.a11y();
    let d = t.iter().find(|n| n.role == Role::Dialog).expect("a dialog");
    assert!(d.modal);
    assert!(!d.name.is_empty());
    assert!(t.iter().all(|n| n.role == Role::Dialog || n.parent.is_some()), "everything else is inside it");
    assert!(find(&t, Role::Button, "关闭").is_some());
    assert!(t.iter().any(|n| n.role == Role::TextBox), "its field");
    assert!(!t.iter().any(|n| n.role == Role::Heading && n.level == 1), "the page beneath is not there");
}

#[test]
fn states_of_the_widgets() {
    let ui = at("slider");
    let t = ui.a11y();
    let s = find(&t, Role::Slider, "数值").expect("the slider");
    let (v, lo, hi) = s.value.expect("a value");
    assert!(lo < hi && v >= lo && v <= hi);
    let ui = at("switch");
    assert!(ui.a11y().iter().any(|n| n.role == Role::Switch && n.checked.is_some()));
    let ui = at("tabs");
    let t = ui.a11y();
    assert!(t.iter().any(|n| n.role == Role::TabList));
    let tabs: Vec<_> = t.iter().filter(|n| n.role == Role::Tab).collect();
    assert!(tabs.len() >= 2);
    for list in t.iter().filter(|n| n.role == Role::TabList) {
        let mine: Vec<_> = tabs.iter().filter(|n| n.parent.as_deref() == Some(list.key.as_str())).collect();
        assert_eq!(mine.iter().filter(|n| n.selected == Some(true)).count(), 1, "one selected tab in {}", list.key);
    }
    let ui = at("list");
    let t = ui.a11y();
    assert!(t.iter().any(|n| n.role == Role::List));
    assert!(t.iter().filter(|n| n.role == Role::ListItem).any(|n| n.selected == Some(true)));
    let mut ui = at("input");
    gallery::update(&mut ui.state, Msg::Text(amenui::widgets::TextEdit::new("你好")));
    ui.relayout();
    assert!(ui.a11y().iter().any(|n| n.role == Role::TextBox && n.text.as_deref() == Some("你好")));
    let ui = at("image");
    assert!(ui.a11y().iter().any(|n| n.role == Role::Img && !n.name.is_empty()));
}

#[test]
fn focus_from_assistive_technology() {
    let mut ui = at("switch");
    let k = ui.a11y().iter().find(|n| n.role == Role::Switch).unwrap().key.clone();
    let before = ui.state.on;
    ui.focus_key(&k);
    assert_eq!(ui.focus(), Some(k.as_str()));
    ui.key(amenui::el::Key::Char(' '), false);
    assert_ne!(ui.state.on, before, "Space on the focused switch flips it");
    ui.activate(&k);
    assert_eq!(ui.state.on, before);
}
