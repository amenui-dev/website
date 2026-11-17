//! The mark atlas, palettes, pixel faces and the compare wipe, without a GPU.
use website::demo;
use amenui::text::{PIXEL, SANS, load_bundled};

#[test]
fn dragging_the_picture_moves_the_compare_line() {
    let mut ui = demo::new_ui(load_bundled(amenui::text::ASSETS).expect("fonts"), 1200, 6000, 1.0);
    let r = ui.node("marks-picture").expect("marks picture").rect;
    ui.press(r.x + r.w * 0.25, r.y + 10.0);
    assert_eq!(ui.state.wipe, 0.25);
    ui.pointer_move(r.x + r.w * 0.8, r.y + 30.0);
    assert_eq!(ui.state.wipe, 0.8);
    ui.release(r.x + r.w * 0.8, r.y + 30.0);
    ui.pointer_move(r.x + r.w * 0.1, r.y + 30.0);
    assert_eq!(ui.state.wipe, 0.8, "moving after release does not drag");
    // The line is a half-plane source in the field, over the picture.
    let f = ui.field();
    assert_eq!(f.wipe[0], r.x + r.w * 0.8);
    assert_eq!(f.wipe[1], 1.0);
    assert_eq!(f.wipe_rect, [r.x, r.y, r.right(), r.bottom()]);
}

#[test]
fn theme_picks_faces_for_the_page() {
    let mut ui = demo::new_ui(load_bundled(amenui::text::ASSETS).expect("fonts"), 1200, 900, 1.0);
    let face = |ui: &demo::DemoUi, k: &str| match &ui.node(k).expect("node").content {
        amenui::el::Content::Text { font, .. } => *font,
        _ => panic!("not text"),
    };
    assert_eq!(face(&ui, "count"), amenui::el::Font::Face(SANS));
    let (x, y) = ui.node("theme:2").expect("theme choice").rect.center();
    ui.press(x, y);
    assert_eq!(face(&ui, "count"), amenui::el::Font::Face(PIXEL));
    // Every theme can be chosen, including the first one again.
    let (x, y) = ui.node("theme:0").expect("theme choice").rect.center();
    ui.press(x, y);
    assert_eq!(ui.state.theme, 0);
}
