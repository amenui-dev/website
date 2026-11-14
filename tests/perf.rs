//! The frame-time readout and automatic quality (DESIGN.md §1.12), without a GPU.
use lab::el::Key;
use lab::gallery::{self, Msg, Page};
use lab::perf::{AutoQuality, BAD_RUN, GOOD_RUN, Perf};

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }

#[test]
fn auto_quality_steps_down_and_back_up() {
    let mut a = AutoQuality::default();
    assert_eq!(a.quality(), 1.0);
    // Slow frames: one level down after a run of them, then the next.
    for _ in 0..BAD_RUN + 5 { a.feed(20.0); }
    assert_eq!(a.quality(), 0.75);
    for _ in 0..BAD_RUN { a.feed(20.0); }
    assert_eq!(a.quality(), 0.5);
    for _ in 0..BAD_RUN * 3 { a.feed(20.0); }
    assert_eq!(a.quality(), 0.5, "never below the last level");
    // One slow frame among fast ones does not count; a long run of fast ones restores a level.
    for i in 0..GOOD_RUN + 10 { a.feed(if i == 50 { 30.0 } else { 4.0 }); }
    assert_eq!(a.quality(), 0.75);
    for _ in 0..GOOD_RUN + 10 { a.feed(4.0); }
    assert_eq!(a.quality(), 1.0);
}

#[test]
fn summary_of_the_last_second() {
    let mut p = Perf::default();
    for k in 0..5 { p.pending.update_layout = k as f32; p.pending.scene = 1.0; p.frame(2.0); }
    let s = p.summary(lab::perf::now_ms());
    assert_eq!(s.fps, 5);
    assert_eq!(s.update_layout, [4.0, 2.0, 4.0]);
    assert_eq!(s.submit, [2.0, 2.0, 2.0]);
    assert_eq!(s.total[2], 7.0);
    assert_eq!(p.drawn, 5);
    // Nothing in the last second, much later.
    assert_eq!(p.summary(lab::perf::now_ms() + 5000.0).fps, 0);
}

#[test]
fn f2_shows_the_readout_in_the_annotation_layer() {
    let mut ui = gallery::new_ui(fonts(), 1280, 800, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("button").unwrap()));
    ui.relayout();
    ui.tick(1.0);
    let before = ui.scene().annot.len();
    assert!(ui.key(Key::F(2), false), "F2 is used");
    assert!(ui.state.look().overlay);
    ui.tick(1.1);
    let shown = ui.scene().annot.len();
    assert!(shown > before + 100, "readout glyphs in the annotation layer: {before} → {shown}");
    // While shown it refreshes twice a second (a timed wake), not every frame.
    ui.frame_drawn();
    let t = ui.now() + 0.6;
    ui.tick(t);
    ui.frame_drawn();
    let w = ui.next_wake(t).expect("a refresh");
    assert!(w > t && w <= t + 0.5 + 1e-3);
    // The menu item does the same.
    gallery::update(&mut ui.state, Msg::Perf(0));
    ui.relayout();
    ui.tick(t + 0.1);
    assert!(!ui.state.look().overlay);
    assert_eq!(ui.scene().annot.len(), before);
    gallery::update(&mut ui.state, Msg::Perf(1));
    assert_eq!(ui.state.look().quality, 0.0, "automatic");
    gallery::update(&mut ui.state, Msg::Perf(4));
    assert_eq!(ui.state.look().quality, 0.5);
}
