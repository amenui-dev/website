//! Acceptance for the skeleton: layout, pressing, scrolling and focus, all without a GPU.
use website::demo::{self, Msg};
use amenui::el::Key;
use amenui::ui::Rect;

fn font() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }

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
