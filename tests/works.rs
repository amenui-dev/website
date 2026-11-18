//! Works (site-design M): the addresses, the scenes' move into the Lab, and the calm rules: a piece
//! draws no frame between its events, and after 60 s without input it rests until the next.
use website::gallery::{self, GalleryUi, Msg, Page, works};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }
fn at(p: Page, w: u32, h: u32) -> GalleryUi {
    let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
    gallery::update(&mut ui.state, Msg::Go(p));
    ui.relayout();
    ui
}

#[test]
fn works_has_an_index_and_a_page_a_piece_and_the_scenes_live_in_the_lab() {
    assert_eq!(Page::from_path("works/"), Some(Page::Works));
    for (i, p) in works::PIECES.iter().enumerate() {
        assert_eq!(Page::from_path(&format!("works/{}/", p.slug)), Some(Page::Work(i as u8)));
        assert_eq!(Page::Work(i as u8).section(), gallery::Section::Works);
    }
    assert_eq!(Page::Scenes.path(), "lab/scenes/");
    assert_eq!(Page::Scenes.section(), gallery::Section::Lab);
    // The old addresses still name the same pages.
    assert_eq!(Page::from_path("scenes/"), Some(Page::Scenes));
    let moved = gallery::routes::moved();
    assert!(moved.iter().any(|(path, p)| path == "scenes/" && *p == Page::Scenes), "{moved:?}");
    assert!(moved.iter().all(|(path, p)| Page::from_path(path) == Some(*p)));
}

#[test]
fn ground_truth_draws_nothing_between_events_and_rests_after_a_minute() {
    let mut ui = at(Page::Work(works::GROUND_TRUTH as u8), 1440, 900);
    let mut t = 0.0f32;
    let (mut frames, mut quiet) = (0, 0);
    // Ten seconds at 60 Hz: the frames drawn are the flights' (paced at 30 fps) and the chrome's fade.
    while t < 10.0 {
        t += 1.0 / 60.0;
        ui.tick(t);
        if ui.needs_frame(t) { frames += 1 } else { quiet += 1 }
        ui.frame_drawn();
    }
    assert!(frames < 400, "{frames} frames in 10 s");
    assert!(quiet > 0, "no frame was skipped");
    // A minute without input: the flight under way ends, the piece rests and asks for nothing.
    while t < 62.0 || (ui.state.works.ground.flight.is_some() && t < 90.0) { t += 1.0 / 30.0; ui.tick(t); ui.frame_drawn(); }
    assert!(ui.state.works.resting);
    assert!(ui.state.works.ground.flight.is_none(), "the flight under way ended");
    ui.tick(t + 0.1);
    ui.frame_drawn();
    assert!(!ui.needs_frame(t + 1.0), "a resting piece draws no frame");
    assert!(ui.next_wake(t + 1.0).is_none(), "a resting piece waits for input");
    // The pointer wakes it.
    ui.pointer_move(700.0, 400.0);
    ui.tick(t + 1.2);
    assert!(!ui.state.works.resting);
}

fn cjk(t: &amenui::tree::TreeNode, out: &mut Vec<String>) {
    for s in [&t.text, &t.label].into_iter().flatten() {
        if s.chars().any(|c| ('\u{3000}'..='\u{9fff}').contains(&c) || ('\u{ff00}'..='\u{ffef}').contains(&c)) { out.push(format!("{}: {s}", t.key)); }
    }
    for c in &t.children { cjk(c, out); }
}

#[test]
fn what_is_inside_a_piece_is_english_in_chinese_too() {
    let mut ui = gallery::new_ui(fonts(), 1440, 900, 1.0);
    gallery::update(&mut ui.state, Msg::Locale(amenui::i18n::Locale::ZhHans));
    for (i, p) in works::PIECES.iter().enumerate() {
        gallery::update(&mut ui.state, Msg::Go(Page::Work(i as u8)));
        ui.relayout();
        let t = ui.export_tree();
        let layer = t.find(&format!("work/{}/layer", p.slug)).expect("the piece's picture");
        let mut bad = vec![];
        cjk(layer, &mut bad);
        assert!(bad.is_empty(), "{}: {bad:?}", p.slug);
    }
}

#[test]
fn with_reduced_motion_ground_truth_holds_its_rest_pose() {
    let mut ui = at(Page::Work(works::GROUND_TRUTH as u8), 1440, 900);
    ui.state.body.reduced = true;
    let mut t = 0.0f32;
    while t < 30.0 { t += 1.0 / 30.0; ui.tick(t); ui.frame_drawn(); }
    // The birds halfway across, still.
    assert_eq!(ui.state.works.ground.flight, Some(8.0));
}

#[test]
fn a_window_of_the_index_grows_into_its_piece() {
    let mut ui = gallery::new_ui(fonts(), 1440, 824, 1.0);
    ui.pointer_leave();
    assert!(ui.state.route("works/"));
    ui.relayout();
    ui.tick(3.0);
    assert!(ui.state.route("works/ground-truth/"));
    let cmds = ui.state.cmds.clone();
    ui.relayout();
    assert!(matches!(cmds.as_slice(), [amenui::morph::Cmd::Flow(f)] if f.flow_ms == 420.0), "{cmds:?}");
    assert!(ui.flowing() > 20, "the window flies to the whole window: {}", ui.flowing());
}
