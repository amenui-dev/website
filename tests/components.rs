//! Acceptance for step 3, without a GPU: drag capture, text editing, the modal layer (focus trap,
//! dismissal, beneath held loose), tabs / list / slider / number scrub, disabled controls, horizontal
//! scroll, commits, and the gallery's hash routes.
use lab::el::{Drag, El, Key, Paint, Phase, col, space};
use lab::gallery::{self, COMPONENTS, GalleryUi, Msg, Page};
use lab::ui::Ui;
use lab::widgets::TextEdit;

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }

fn gallery_at(p: Page) -> GalleryUi {
    let mut ui = gallery::new_ui(fonts(), 1280, 1800, 1.0);
    go(&mut ui, p);
    ui
}
fn go(ui: &mut GalleryUi, p: Page) { gallery::update(&mut ui.state, Msg::Go(p)); ui.relayout(); }
fn centre(ui: &GalleryUi, k: &str) -> (f32, f32) { ui.node(k).unwrap_or_else(|| panic!("no node {k}")).rect.center() }
fn click(ui: &mut GalleryUi, k: &str) {
    // The rail scrolls: bring a rail item into its view first.
    for _ in 0..60 {
        let n = ui.node(k).unwrap_or_else(|| panic!("no node {k}"));
        let (c, r) = (n.clip, n.rect);
        if !k.starts_with("nav/") || (r.y >= c.y && r.bottom() <= c.bottom()) { break; }
        let rail = ui.node("rail").expect("rail").rect;
        ui.wheel(rail.center().0, rail.center().1, if r.y < c.y { -120.0 } else { 120.0 });
    }
    let (x, y) = centre(ui, k);
    ui.press(x, y);
    ui.release(x, y);
}
fn comp(hash: &str) -> Page { Page::Comp(COMPONENTS.iter().position(|c| c.hash == hash).expect("component")) }

// ── Drag capture ─────────────────────────────────────────────────────────────────────────────────

#[derive(Default)]
struct Drags { seen: Vec<Drag> }
fn drags(_: &Drags) -> El<Drag> {
    col().key("root").pad(20.0, 20.0)
        .child(space(100.0, 40.0).key("handle").fill(Paint::INK).on_drag_with(Some))
        .child(space(100.0, 40.0).key("other").on_press(Drag { phase: Phase::End, x: -1.0, y: -1.0, w: 0.0, h: 0.0, dx: 0.0, dy: 0.0 }))
}

#[test]
fn drag_is_captured_until_release() {
    let mut ui = Ui::new(Drags::default(), drags, |s: &mut Drags, d| s.seen.push(d), fonts(), 400, 300, 1.0);
    ui.press(30.0, 30.0);
    assert_eq!(ui.dragging(), Some("handle"));
    // Far outside the element, and over another pressable one: the drag keeps going.
    ui.pointer_move(300.0, 80.0);
    ui.release(300.0, 80.0);
    ui.pointer_move(10.0, 10.0);
    let s = &ui.state.seen;
    assert_eq!(s.iter().map(|d| d.phase).collect::<Vec<_>>(), [Phase::Start, Phase::Move, Phase::End]);
    assert_eq!((s[0].x, s[0].y, s[0].w, s[0].h), (10.0, 10.0, 100.0, 40.0));
    assert_eq!((s[1].dx, s[1].dy), (270.0, 50.0));
    assert_eq!(s[1].fx(), 1.0, "fraction is clamped");
    assert_eq!(ui.dragging(), None);
}

// ── Text input ───────────────────────────────────────────────────────────────────────────────────

#[test]
fn text_edit_keys() {
    let e = TextEdit::new("中文ab");
    assert_eq!(e.caret, 4);
    let e = e.apply(Key::Left, 99).unwrap().apply(Key::Left, 99).unwrap();
    assert_eq!(e.caret, 2);
    let e = e.apply(Key::Char('x'), 99).unwrap();
    assert_eq!((e.text.as_str(), e.caret), ("中文xab", 3));
    let e = e.apply(Key::Backspace, 99).unwrap().apply(Key::Backspace, 99).unwrap();
    assert_eq!((e.text.as_str(), e.caret), ("中ab", 1));
    let e = e.apply(Key::Delete, 99).unwrap();
    assert_eq!(e.text, "中b");
    assert_eq!(e.apply(Key::Home, 99).unwrap().caret, 0);
    assert_eq!(e.apply(Key::End, 99).unwrap().caret, 2);
    assert_eq!(e.apply(Key::Char('z'), 2), None, "full");
    assert_eq!(e.apply(Key::Home, 99).unwrap().apply(Key::Backspace, 99), None, "nothing before the caret");
    assert_eq!(e.apply(Key::Char('\u{8}'), 99), None, "control characters are not typed");
}

#[test]
fn typing_into_the_input() {
    let mut ui = gallery_at(comp("input"));
    let carets = |ui: &GalleryUi| ui.scene().instances.len();
    let before = carets(&ui);
    click(&mut ui, "input/live");
    assert_eq!(ui.focus(), Some("input/live"));
    // The caret is drawn only while the field has focus.
    assert_eq!(carets(&ui), before + 1);
    for c in "hello wor".chars() { ui.key(Key::Char(c), false); }
    ui.key(Key::Char('l'), false);
    ui.key(Key::Char('d'), false);
    assert_eq!(ui.state.text.text, "hello world");
    // Six to the left is just after "hello": a comma there, then Delete and Backspace around it.
    for _ in 0..6 { ui.key(Key::Left, false); }
    ui.key(Key::Char(','), false);
    ui.key(Key::Delete, false);
    ui.key(Key::Char(' '), false);
    ui.key(Key::Char('x'), false);
    ui.key(Key::Backspace, false);
    ui.key(Key::End, false);
    ui.key(Key::Char('!'), false);
    assert_eq!(ui.state.text.text, "hello, world!");
    assert_eq!(ui.node("input/live/text").and_then(|n| n.text()), Some("hello, world!"));
    assert_eq!(ui.node("input/count").and_then(|n| n.text()), Some("13 个字"));
    // Space is typed, not a press.
    ui.key(Key::Char(' '), false);
    assert_eq!(ui.state.text.text, "hello, world! ");
}

// ── Modal layer ──────────────────────────────────────────────────────────────────────────────────

#[test]
fn dialog_traps_focus_and_dismisses() {
    let mut ui = gallery_at(comp("dialog"));
    let open = "dialog/open";
    let (ox, oy) = centre(&ui, open);
    ui.press(ox, oy);
    ui.release(ox, oy);
    assert_eq!(ui.modal(), Some("dialog/dialog"));
    // Focus moves to the dialog's text field.
    assert_eq!(ui.focus(), Some("dialog/name"));
    ui.key(Key::Char('A'), false);
    assert_eq!(ui.state.dialog_text.text, "A");
    // Tab goes round inside the dialog only.
    let mut seen = vec![];
    for _ in 0..8 { ui.key(Key::Tab, false); seen.push(ui.focus().unwrap().to_owned()); }
    assert!(seen.iter().all(|k| k.starts_with("dialog/dialog") || k == "dialog/name" || k == "dialog/cancel" || k == "dialog/ok"), "{seen:?}");
    assert!(seen.contains(&"dialog/dialog/close".to_owned()) && seen.contains(&"dialog/ok".to_owned()));
    ui.key(Key::Tab, true);
    // Nothing beneath can be pressed or hovered.
    let (rx, ry) = centre(&ui, "nav/button");
    ui.press(rx, ry);
    ui.release(rx, ry);
    assert_eq!(ui.state.page, comp("dialog"));
    ui.pointer_move(ox, oy);
    assert_eq!(ui.hover(), None);
    // Everything beneath is held partly loose; the layer's own instances are not.
    let layer = ui.scene().layer.expect("layer") as usize;
    assert!(ui.scene().instances[..layer].iter().all(|i| i.extra[2] >= 0.35));
    assert!(ui.scene().instances[layer..].iter().all(|i| i.extra[2] < 0.35));
    // Esc closes it and focus goes back to what opened it.
    ui.key(Key::Escape, false);
    assert_eq!(ui.modal(), None);
    assert_eq!(ui.state.dialog, 0);
    assert_eq!(ui.focus(), Some(open));
    // (Parts still flowing from the page change are drawn in a layer of their own until they land.)
    let t = ui.now();
    ui.tick(t + 1.0);
    assert_eq!(ui.scene().layer, None);
    // The close button closes it too.
    click(&mut ui, open);
    assert!(ui.modal().is_some());
    click(&mut ui, "dialog/dialog/close");
    assert_eq!(ui.modal(), None);
}

// ── Tabs, list, slider, number ───────────────────────────────────────────────────────────────────

#[test]
fn tabs_select_by_press_and_keys() {
    let mut ui = gallery_at(comp("tabs"));
    click(&mut ui, "tabs/live:2");
    assert_eq!(ui.state.tab, 2);
    assert_eq!(ui.focus(), Some("tabs/live/head"), "the row is the one focus stop");
    ui.key(Key::Left, false);
    assert_eq!(ui.state.tab, 1);
    ui.key(Key::Home, false);
    assert_eq!(ui.state.tab, 0);
    assert!(ui.node("tabs/panel0").is_some());
}

#[test]
fn list_selects_and_reveals() {
    let mut ui = gallery_at(comp("list"));
    click(&mut ui, "list/live:4");
    assert_eq!(ui.state.row, Some(4));
    assert_eq!(ui.focus(), Some("list/live"));
    for _ in 0..10 { ui.key(Key::Down, false); }
    assert_eq!(ui.state.row, Some(14));
    // The selected row was scrolled into view.
    let (l, r) = (ui.node("list/live").unwrap(), ui.node("list/live:14").unwrap());
    assert!(r.rect.y >= l.rect.y && r.rect.bottom() <= l.rect.bottom(), "row {:?} list {:?}", r.rect, l.rect);
    assert!(ui.scroll_offset("list/live") > 0.0);
    ui.key(Key::Home, false);
    assert_eq!(ui.state.row, Some(0));
    assert_eq!(ui.scroll_offset("list/live"), 0.0);
    ui.key(Key::End, false);
    assert_eq!(ui.state.row, Some(39));
}

#[test]
fn slider_follows_drag_and_keys() {
    let mut ui = gallery_at(comp("slider"));
    let r = ui.node("slider/live").unwrap().rect;
    // Knob 14 px: the run is the width less the knob, starting half a knob in.
    ui.press(r.x + 7.0, r.center().1);
    assert_eq!(ui.state.level, 0.0);
    ui.pointer_move(r.x + 7.0 + (r.w - 14.0) * 0.25, r.center().1);
    assert_eq!(ui.state.level, 25.0);
    ui.pointer_move(r.right() + 200.0, r.center().1);
    assert_eq!(ui.state.level, 100.0);
    ui.release(r.right() + 200.0, r.center().1);
    ui.key(Key::Left, true);
    assert_eq!(ui.state.level, 90.0);
    ui.key(Key::Right, false);
    assert_eq!(ui.state.level, 91.0);
}

#[test]
fn number_scrubs_sideways_but_a_click_only_focuses() {
    let mut ui = gallery_at(comp("number"));
    let (x, y) = centre(&ui, "number/live/value");
    let start = ui.state.value;
    ui.press(x, y);
    ui.pointer_move(x + 2.0, y);
    ui.release(x + 2.0, y);
    assert_eq!(ui.state.value, start, "a press that barely moves does not scrub");
    assert_eq!(ui.focus(), Some("number/live/value"));
    // Step 0.5, one step per 6 px.
    ui.press(x, y);
    ui.pointer_move(x + 30.0, y);
    assert_eq!(ui.state.value, start + 2.5);
    ui.pointer_move(x - 12.0, y);
    assert_eq!(ui.state.value, start - 1.0);
    ui.pointer_move(x + 1.0, y);
    assert_eq!(ui.state.value, start, "back to where it started");
    ui.release(x + 1.0, y);
    // Typing still works.
    for c in "7".chars() { ui.key(Key::Char(c), false); }
    ui.key(Key::Enter, false);
    assert_eq!(ui.state.value, 7.0);
}

// ── Disabled ─────────────────────────────────────────────────────────────────────────────────────

#[test]
fn disabled_controls_ignore_input() {
    let mut ui = gallery_at(comp("button"));
    click(&mut ui, "button/d");
    click(&mut ui, "button/e");
    assert_eq!(ui.state.presses, 0);
    assert!(ui.node("button/d").unwrap().disabled);
    click(&mut ui, "button/a");
    assert_eq!(ui.state.presses, 1);
    // Tab never lands on them.
    for _ in 0..80 { ui.key(Key::Tab, false); assert!(!ui.focus().is_some_and(|f| f == "button/d" || f == "button/e")); }
    // A disabled slider and number do not move.
    go(&mut ui, comp("slider"));
    let r = ui.node("slider/d").unwrap().rect;
    ui.press(r.x + 2.0, r.center().1);
    assert_eq!(ui.dragging(), None);
    go(&mut ui, comp("number"));
    let (x, y) = centre(&ui, "number/e/value");
    ui.press(x, y);
    assert_ne!(ui.focus(), Some("number/e/value"));
}

// ── Horizontal scroll, commits ───────────────────────────────────────────────────────────────────

#[test]
fn narrow_rail_scrolls_sideways() {
    let mut ui = gallery::new_ui(fonts(), 600, 900, 1.0);
    let rail = ui.node("rail").expect("rail row").rect;
    assert_eq!(rail.w, 600.0);
    assert!(ui.node("rail").unwrap().extent > 1000.0);
    let x0 = ui.node("nav/number").unwrap().rect.x;
    let (x, y) = rail.center();
    ui.wheel(x, y, 120.0);
    assert_eq!(ui.scroll_offset("rail"), 120.0);
    assert_eq!(ui.node("nav/number").unwrap().rect.x, x0 - 120.0);
    // Going to a page off to the right by keyboard brings its tab into view.
    let r = ui.node("nav/demo").unwrap().rect;
    assert!(r.x > 600.0);
    while ui.focus() != Some("nav/demo") { ui.key(Key::Tab, false); }
    let r = ui.node("nav/demo").unwrap().rect;
    assert!(r.right() <= 600.0, "{r:?}");
    // On a wide window the rail is a column.
    ui.resize(1280, 900, 1.0);
    assert_eq!(ui.node("rail").unwrap().rect.w, gallery::RAIL);
}

#[test]
fn theme_switch_keeps_the_old_frame() {
    let mut ui = gallery_at(comp("button"));
    assert!(ui.split().is_none());
    // The theme menu opens over the page; its options are commits.
    click(&mut ui, "theme");
    assert_eq!(ui.modal(), Some("theme/layer"));
    let old = ui.scene().instances.len();
    let (x, y) = centre(&ui, "theme:1");
    ui.press(x, y);
    ui.release(x, y);
    assert_eq!(ui.state.theme, 1);
    assert_eq!(ui.modal(), None, "the menu closes");
    let sp = ui.split().expect("split");
    assert_eq!(sp.at, [x, y]);
    assert_eq!(sp.scene.instances.len(), old);
    let id = sp.id;
    // Navigation is not a commit.
    click(&mut ui, "nav/number");
    assert_eq!(ui.split().unwrap().id, id);
    // Keys commit too: in the open material menu, focus is on the first option.
    let before = ui.split().unwrap().id;
    click(&mut ui, "material");
    assert_eq!(ui.focus(), Some("material:0"));
    ui.key(Key::Tab, false);
    ui.key(Key::Enter, false);
    assert_eq!(ui.state.material, lab::style::Material::Dot);
    assert_eq!(ui.split().unwrap().id, before + 1);
    // A style page changes everything, from the rail item pressed.
    click(&mut ui, "nav/style-bauhaus");
    assert_eq!(ui.split().unwrap().id, before + 2);
    assert_eq!(ui.state.material, lab::style::Material::Halftone);
    assert_eq!(ui.state.theme().name, "包豪斯");
}

#[test]
fn navigation_keeps_the_shell_and_renews_the_page() {
    let mut ui = gallery_at(comp("button"));
    ui.tick(5.0);
    click(&mut ui, "nav/switch");
    ui.tick(5.1);
    // Instances whose identity survives keep their birth time; the new page's are born now.
    let born = |ui: &GalleryUi| ui.scene().instances.iter().map(|i| i.extra[3]).collect::<Vec<_>>();
    let b = born(&ui);
    assert!(b.contains(&0.0), "the shell kept its birth time");
    assert!(b.contains(&5.0), "the new page was born when it was pressed");
    assert_eq!(ui.state.page, comp("switch"));
}

// ── Routes ───────────────────────────────────────────────────────────────────────────────────────

#[test]
fn every_page_has_a_plain_hash() {
    let forbidden = ["settings", "sidebar", "customize", "upgrade", "downloads"];
    let pages = Page::all();
    // The components, the ten thousand rows, mark sets, materials, the annotation page, styles, morph,
    // flow and 3D, type and demo, the scene list and the scenes.
    assert_eq!(pages.len(), COMPONENTS.len() + 1 + 12 + 12 + 1 + 12 + 1 + 4 + 2 + 1 + 7);
    for p in pages {
        let h = p.hash();
        assert!(!h.is_empty() && h.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'), "{h}");
        assert!(!forbidden.contains(&h.as_str()) && !h.starts_with("comment-"), "{h}");
        assert_eq!(Page::from_hash(&h), Some(p));
        assert_eq!(Page::from_hash(&format!("#{h}")), Some(p));
    }
    assert_eq!(Page::from_hash(""), Some(Page::Comp(0)));
    assert_eq!(Page::from_hash("#nope"), None);
    let mut s = gallery::Gallery::default();
    assert!(s.route("#marks-chars"));
    assert_eq!(s.page, Page::Marks(0));
    assert!(!s.route("#nope"));
    assert_eq!(s.page, Page::Marks(0));
}

#[test]
fn every_page_lays_out_in_every_theme() {
    let mut ui = gallery::new_ui(fonts(), 1280, 900, 1.0);
    for p in Page::all() {
        for t in 0..3 {
            gallery::update(&mut ui.state, Msg::Theme(t));
            go(&mut ui, p);
            assert!(ui.scene().instances.len() > 50, "{} is empty", p.hash());
        }
    }
}

#[test]
fn registry_lists_every_component_and_its_files() {
    let root = env!("CARGO_MANIFEST_DIR");
    let reg = std::fs::read_to_string(format!("{root}/registry.json")).expect("registry.json");
    for c in COMPONENTS.iter() {
        assert!(reg.contains(&format!("\"name\": \"{}\"", c.hash)), "{} is not in the registry", c.hash);
        assert!(reg.contains(&format!("\"{}\"", c.file)), "{} is not in the registry", c.file);
        assert!(std::path::Path::new(&format!("{root}/{}", c.file)).exists(), "{} does not exist", c.file);
    }
}
