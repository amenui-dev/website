//! Pointers, touch and the wheel (DESIGN.md §1.8.2), without a GPU: fling decay and the rubber band
//! as functions; a finger scrolling the 10,000 rows, flinging on and coming to rest; a tap pressing; a
//! scroll that started on a row not pressing it; pulling past the top loosening the edge, not moving
//! the content, and springing back; a second finger cancelling; pointer capture for a finger on a
//! slider; small controls hit within 28 px; no hover from a finger; the pointer's shapes.
use amenui::el::Cursor;
use website::gallery::{self, GalleryUi, Msg, Page};
use amenui::ui::PointerKind;

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }
const T: PointerKind = PointerKind::Touch;
fn list() -> GalleryUi {
    let mut ui = gallery::new_ui(fonts(), 1280, 900, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("list-10k").unwrap()));
    ui.relayout();
    ui.tick(1.0);
    ui
}
/// Tick at 60 Hz for `secs`.
fn wait(ui: &mut GalleryUi, secs: f32) { let to = ui.now() + secs; let mut t = ui.now(); while t < to { t = (t + 1.0 / 60.0).min(to); ui.tick(t); ui.frame_drawn(); } }
/// A finger from (x, y0) to (x, y1) in `n` steps of 1/60 s.
fn swipe(ui: &mut GalleryUi, x: f32, y0: f32, y1: f32, n: usize) {
    ui.pointer_down(1, T, x, y0, false);
    for k in 1..=n {
        let t = ui.now() + 1.0 / 60.0;
        ui.tick(t);
        ui.pointer_moved(1, T, x, y0 + (y1 - y0) * k as f32 / n as f32);
    }
    ui.pointer_up(1, T, x, y1);
}

#[test]
fn a_finger_scrolls_the_long_list_and_flings() {
    let mut ui = list();
    let r = ui.node("long").unwrap().rect;
    let x = r.x + 200.0;
    swipe(&mut ui, x, r.y + 500.0, r.y + 200.0, 10);
    let after = ui.scroll_offset("long");
    assert!((after - (300.0 - 0.0)).abs() <= 1.0, "follows the finger: {after}");
    assert_eq!(ui.state.long_pick, None, "a scroll is not a press");
    assert!(ui.flinging(), "flings on");
    assert!(ui.needs_frame(ui.now()));
    wait(&mut ui, 0.3);
    let mid = ui.scroll_offset("long");
    assert!(mid > after + 100.0, "kept going: {mid}");
    wait(&mut ui, 4.0);
    let end = ui.scroll_offset("long");
    assert!(!ui.flinging() && !ui.needs_frame(ui.now()), "comes to rest");
    // At 30 px a frame (1800 px/s) and friction 3.5, a fling goes about 1800 / 3.5 ≈ 514 px.
    assert!((end - after - 514.0).abs() < 80.0, "flung {}", end - after);
    // Only rows that show are in the tree, wherever it went.
    let rows: Vec<usize> = ui.nodes().iter().filter_map(|n| n.key.strip_prefix("long:").and_then(|i| i.parse().ok())).collect();
    assert!(rows.len() <= 40, "{} rows {:?}..{:?} at {}", rows.len(), rows.iter().min(), rows.iter().max(), ui.scroll_offset("long"));
}

#[test]
fn a_tap_presses_and_shows_at_once() {
    let mut ui = list();
    let r = ui.node("long:3").unwrap().rect;
    let (x, y) = (r.x + 200.0, r.center().1);
    let fronts = |ui: &GalleryUi| ui.field().fronts.iter().filter(|f| f[3] > 0.0).count();
    let before = fronts(&ui);
    ui.pointer_down(7, T, x, y, false);
    assert_eq!(fronts(&ui), before + 1, "the front starts with the finger down");
    // Held as loose as a legible control may be (DESIGN.md §1.15), at once.
    let cap = ui.state.look().legible_cap;
    assert!(ui.instances_of("long:3").iter().all(|i| (i.extra[2] - cap).abs() < 1e-4), "the row is held loose at once");
    assert_eq!(ui.state.long_pick, None, "not sent yet");
    ui.pointer_up(7, T, x, y);
    assert_eq!(ui.state.long_pick, Some(3), "sent on lift");
    assert_eq!(ui.hover(), None, "a finger does not hover");
    // A mouse does hover.
    ui.pointer_moved(0, PointerKind::Mouse, x, y);
    assert!(ui.hover().is_some());
}

#[test]
fn pulling_past_the_top_loosens_the_edge_and_springs_back() {
    let mut ui = list();
    let r = ui.node("long").unwrap().rect;
    let x = r.x + 200.0;
    ui.pointer_down(1, T, x, r.y + 100.0, false);
    for k in 1..=10 { let t = ui.now() + 1.0 / 60.0; ui.tick(t); ui.pointer_moved(1, T, x, r.y + 100.0 + k as f32 * 20.0); }
    assert_eq!(ui.scroll_offset("long"), 0.0, "the content does not move past its start");
    let o = ui.overscroll("long");
    assert!(o < -40.0, "pulled {o}");
    // Rows near the top edge loosen; rows far from it do not.
    let top = ui.instances_of("long:0").iter().map(|i| i.extra[2]).fold(0.0f32, f32::max);
    let far = ui.instances_of("long:8").iter().map(|i| i.extra[2]).fold(0.0f32, f32::max);
    // The rows are text to read: the pull loosens them only up to the legibility cap (DESIGN.md §1.15).
    let cap = ui.state.look().legible_cap;
    assert!(top > cap * 0.9 && top <= cap + 1e-4 && far == 0.0, "top {top}, far {far}");
    ui.pointer_up(1, T, x, r.y + 300.0);
    assert!(!ui.flinging(), "a pull does not fling");
    wait(&mut ui, 1.5);
    assert_eq!(ui.overscroll("long"), 0.0);
    assert!(ui.instances_of("long:0").iter().all(|i| i.extra[2] == 0.0), "tight again");
    assert!(!ui.needs_frame(ui.now()));
}

#[test]
fn a_second_finger_cancels() {
    let mut ui = list();
    let r = ui.node("long:2").unwrap().rect;
    let (x, y) = (r.x + 200.0, r.center().1);
    ui.pointer_down(1, T, x, y, false);
    ui.pointer_down(2, T, x + 100.0, y + 50.0, false);
    ui.pointer_moved(1, T, x, y - 150.0);
    ui.pointer_moved(2, T, x + 100.0, y + 200.0);
    ui.pointer_up(2, T, x + 100.0, y + 200.0);
    ui.pointer_up(1, T, x, y - 150.0);
    assert_eq!((ui.state.long_pick, ui.scroll_offset("long")), (None, 0.0), "a pinch neither presses nor scrolls");
}

#[test]
fn a_finger_on_a_slider_is_captured() {
    let mut ui = gallery::new_ui(fonts(), 1280, 1800, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("slider").unwrap()));
    ui.relayout();
    ui.tick(1.0);
    let k = ui.nodes().iter().find(|n| n.key.contains("live") && ui.dragging().is_none() && n.focusable).map(|n| n.key.clone()).expect("the slider");
    let r = ui.node(&k).unwrap().rect;
    let before = ui.state.level;
    ui.pointer_down(3, T, r.x + 4.0, r.center().1, false);
    assert!(ui.dragging().is_some(), "captured at once");
    // Far outside, past the end: still the slider's.
    ui.pointer_moved(3, T, r.right() + 300.0, r.center().1 + 200.0);
    ui.pointer_up(3, T, r.right() + 300.0, r.center().1 + 200.0);
    assert!(ui.state.level > before, "{before} → {}", ui.state.level);
    assert_eq!(ui.dragging(), None);
}

#[test]
fn pointer_shapes() {
    let mut ui = gallery::new_ui(fonts(), 1280, 1800, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("input").unwrap()));
    ui.relayout();
    let at = |ui: &mut GalleryUi, k: &str| { let (x, y) = ui.node(k).unwrap().rect.center(); ui.pointer_move(x, y); ui.cursor() };
    assert_eq!(at(&mut ui, "input/live"), Cursor::Text);
    assert_eq!(at(&mut ui, "input/src3"), Cursor::Text, "selectable text");
    assert_eq!(at(&mut ui, "nav/button"), Cursor::Pointer);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("scene-printing").unwrap()));
    ui.relayout();
    assert_eq!(at(&mut ui, "pr/seam1"), Cursor::ColResize, "a seam");
    let (x, y) = ui.node("pr/seam1").unwrap().rect.center();
    ui.press(x, y);
    assert_eq!(ui.cursor(), Cursor::ColResize, "while dragged, still");
    ui.release(x, y);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("scene-reading").unwrap()));
    ui.relayout();
    assert_eq!(at(&mut ui, "read/bust"), Cursor::Grab);
    let (x, y) = ui.node("read/bust").unwrap().rect.center();
    ui.press(x, y);
    assert_eq!(ui.cursor(), Cursor::Grabbing);
    ui.release(x, y);
}
