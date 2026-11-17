//! Acceptance for step 2: layout, pressing, scrolling and focus, all without a GPU.
use amenui::demo::{self, Msg};
use amenui::el::{El, Key, Paint, col, row, space, text_lh};
use amenui::ui::{Rect, Ui};

fn font() -> Vec<amenui::text::Face> { amenui::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }

const PARA: &str = "一段用来测量的文字，中文和 Latin words 混排，宽度变窄时会多出几行。";

fn card(_: &()) -> El<()> {
    col().key("root")
        .child(row().key("card").wrap().pad(12.0, 10.0).gap(8.0)
            .child(space(50.0, 30.0).key("a"))
            .child(space(70.0, 40.0).key("b"))
            .child(space(30.0, 20.0).key("c")))
        .child(text_lh(PARA, 16.0, 24.0, Paint::TEXT).key("para"))
}

fn rect(ui: &Ui<(), ()>, k: &str) -> Rect { ui.node(k).unwrap_or_else(|| panic!("no node {k}")).rect }

#[test]
fn card_at_two_widths() {
    let mut ui = Ui::new((), card, |_, _| {}, font(), 400, 600, 1.0);
    assert_eq!(rect(&ui, "card"), Rect::new(0.0, 0.0, 400.0, 60.0));
    assert_eq!(rect(&ui, "a"), Rect::new(12.0, 10.0, 50.0, 30.0));
    assert_eq!(rect(&ui, "b"), Rect::new(70.0, 10.0, 70.0, 40.0));
    assert_eq!(rect(&ui, "c"), Rect::new(148.0, 10.0, 30.0, 20.0));
    let wide = rect(&ui, "para");
    assert_eq!((wide.x, wide.y, wide.w), (0.0, 60.0, 400.0));

    // 126 px inside the padding: a alone on the first line, b and c on the second.
    ui.resize(150, 600, 1.0);
    assert_eq!(rect(&ui, "card"), Rect::new(0.0, 0.0, 150.0, 98.0));
    assert_eq!(rect(&ui, "a"), Rect::new(12.0, 10.0, 50.0, 30.0));
    assert_eq!(rect(&ui, "b"), Rect::new(12.0, 48.0, 70.0, 40.0));
    assert_eq!(rect(&ui, "c"), Rect::new(90.0, 48.0, 30.0, 20.0));
    let narrow = rect(&ui, "para");
    assert_eq!((narrow.x, narrow.y, narrow.w), (0.0, 98.0, 150.0));

    let lines = |h: f32| { assert_eq!(h % 24.0, 0.0, "height {h} is not whole lines"); (h / 24.0) as u32 };
    let (lw, ln) = (lines(wide.h), lines(narrow.h));
    assert!(lw >= 2 && ln > lw, "wide {lw} lines, narrow {ln} lines");
}

fn demo_ui(h: u32) -> demo::DemoUi { demo::new_ui(font(), 1200, h, 1.0) }

#[test]
fn press_counts() {
    let mut ui = demo_ui(900);
    assert_eq!(ui.node("count").and_then(|n| n.text()), Some("计数 0"));
    let (x, y) = ui.node("inc").expect("button").rect.center();
    ui.press(x, y);
    ui.release(x, y);
    assert_eq!(ui.state.count, 1);
    assert_eq!(ui.node("count").and_then(|n| n.text()), Some("计数 1"));
    // Pressing beside the button does nothing to the count.
    let r = ui.node("inc").unwrap().rect;
    ui.press(r.x - 30.0, r.y - 30.0);
    assert_eq!(ui.state.count, 1);
}

#[test]
fn wheel_scrolls_list_and_clamps() {
    let mut ui = demo_ui(2400);
    let list = ui.node("list").expect("list").rect;
    let first = ui.node("item/0").unwrap().rect;
    let (x, y) = list.center();
    ui.wheel(x, y, 100.0);
    assert_eq!(ui.scroll_offset("list"), 100.0);
    assert_eq!(ui.node("item/0").unwrap().rect.y, first.y - 100.0);
    // Children outside the list are clipped to its inside.
    assert_eq!(ui.node("item/0").unwrap().clip, Rect::new(list.x + 1.0, list.y + 1.0, list.w - 2.0, list.h - 2.0));
    ui.wheel(x, y, 1e5);
    let n = ui.node("list").unwrap();
    let max = n.extent - n.rect.h;
    assert_eq!(max, 40.0 * 36.0 + 10.0 - 232.0);
    assert_eq!(ui.scroll_offset("list"), max);
    ui.wheel(x, y, -1e5);
    assert_eq!(ui.scroll_offset("list"), 0.0);
}

#[test]
fn tab_moves_focus_and_enter_activates() {
    let mut ui = demo_ui(900);
    assert_eq!(ui.focus(), None);
    ui.key(Key::Tab, false);
    assert_eq!(ui.focus(), Some("inc"));
    ui.key(Key::Enter, false);
    ui.key(Key::Char(' '), false);
    assert_eq!(ui.state.count, 2);
    ui.key(Key::Tab, false);
    assert_eq!(ui.focus(), Some("item/0"));
    ui.key(Key::Tab, true);
    ui.key(Key::Tab, true);
    assert_eq!(ui.focus(), Some("reset"), "Shift-Tab wraps to the last control");
    ui.key(Key::Tab, false);
    assert_eq!(ui.focus(), Some("inc"));
}

#[test]
fn number_takes_typed_digits() {
    let mut ui = demo_ui(900);
    let (x, y) = ui.node("param/0/value").expect("field").rect.center();
    ui.press(x, y);
    assert_eq!(ui.focus(), Some("param/0/value"));
    for c in "7.5".chars() { ui.key(Key::Char(c), false); }
    assert_eq!(ui.node("param/0/shown").and_then(|n| n.text()), Some("7.5"));
    ui.key(Key::Enter, false);
    assert_eq!(ui.state.fx.pitch, 7.5);
    ui.key(Key::Up, true);
    assert_eq!(ui.state.fx.pitch, 12.5);
    ui.key(Key::Down, false);
    assert_eq!(ui.state.fx.pitch, 12.0);
    ui.key(Key::Char('9'), false);
    ui.key(Key::Escape, false);
    assert_eq!(ui.state.fx.pitch, 12.0);
    assert_eq!(ui.state.edit, None);
    // Out of range is clamped; the step buttons step.
    for c in "99".chars() { ui.key(Key::Char(c), false); }
    ui.key(Key::Enter, false);
    assert_eq!(ui.state.fx.pitch, 16.0);
    let (x, y) = ui.node("param/0/less").unwrap().rect.center();
    ui.press(x, y);
    assert_eq!(ui.state.fx.pitch, 15.5);
    let _ = Msg::Inc;
}
