//! The frame-time readout and automatic quality (DESIGN.md §1.12), without a GPU.
use amenui::el::Key;
use website::gallery::{self, Msg, Page};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }

#[test]
fn f2_shows_the_readout_in_the_annotation_layer() {
    let mut ui = gallery::new_ui(fonts(), 1280, 800, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("button").unwrap()));
    ui.relayout();
    ui.tick(1.0);
    let before = ui.scene().annot.len();
    assert!(ui.key(Key::F(2), false), "F2 is used");
    assert_eq!(ui.state.look().overlay, 1, "one line first");
    ui.tick(1.1);
    let one = ui.scene().annot.len();
    assert!(one > before + 20, "readout glyphs in the annotation layer: {before} → {one}");
    assert!(ui.key(Key::F(2), false));
    assert_eq!(ui.state.look().overlay, 2, "then every number");
    ui.tick(1.2);
    let all = ui.scene().annot.len();
    assert!(all > one + 200, "every number: {one} → {all}");
    // It never keeps frames coming: at most one more, a second after the last frame, to say idle.
    ui.frame_drawn();
    let t = ui.now();
    let w = ui.next_wake(t);
    assert!(w.is_none_or(|w| w <= t + 1.1), "{w:?}");
    ui.tick(t + 1.2);
    ui.frame_drawn();
    let t = ui.now();
    assert!(ui.next_wake(t).is_none(), "idle: no more frames for the readout");
    assert!(!ui.needs_frame(t + 5.0));
    // A third F2: off again.
    assert!(ui.key(Key::F(2), false));
    ui.relayout();
    ui.tick(t + 0.1);
    assert_eq!(ui.state.look().overlay, 0);
    assert_eq!(ui.scene().annot.len(), before);
}
