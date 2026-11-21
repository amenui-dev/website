//! The pictures a page draws are made once each (`website::new_pictures`, which the web module's
//! `pictures` export calls after every frame): scrolling the components index down to its image, back
//! up and down again (a change of direction), the tree is asked for once, every id asked for is a
//! picture the site has, and once nothing is new no call asks for anything, so the page draws no frame
//! again for it and lays nothing out.
use website::gallery::{self, GalleryUi, Msg, Page};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }

/// One frame as the page runs it: input, the tick, then the pictures asked for.
fn frame(ui: &mut GalleryUi, t: &mut f32, dy: f32, made: &mut Vec<u32>, asked: &mut Vec<(usize, Vec<u32>)>, f: &mut usize) {
    *t += 1.0 / 60.0;
    ui.event_time(*t);
    if dy != 0.0 { ui.pointer_move(1000.0, 600.0); ui.wheel(1000.0, 600.0, dy); }
    ui.tick(*t);
    ui.frame_drawn();
    let new = website::new_pictures(ui.scene(), made);
    if !new.is_empty() { asked.push((*f, new)); }
    *f += 1;
}

#[test]
fn each_picture_is_made_once_and_nothing_is_asked_for_once_nothing_is_new() {
    // The M1's window: 1440 × 900 at scale 2.
    let mut ui = gallery::new_ui(fonts(), 2880, 1800, 2.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Components));
    ui.state.routed = true;
    ui.relayout();
    let (mut t, mut made, mut asked, mut f) = (0.0f32, vec![], vec![], 0usize);
    for _ in 0..30 { frame(&mut ui, &mut t, 0.0, &mut made, &mut asked, &mut f); }
    let c0 = ui.perf().counters;
    // Slowly down past the image, up to the top, down again, then at rest.
    for (n, dy) in [(150, 4.0), (150, -4.0), (60, 4.0), (120, 0.0)] {
        for _ in 0..n { frame(&mut ui, &mut t, dy, &mut made, &mut asked, &mut f); }
    }
    let ids: Vec<u32> = asked.iter().flat_map(|(_, v)| v.iter().copied()).collect();
    eprintln!("pictures asked for: {asked:?}");
    assert!(ids.contains(&website::demo::TREE), "the image's tree was never asked for: {asked:?}");
    let mut seen = vec![];
    for id in &ids {
        assert!(!seen.contains(id), "picture {id} was asked for twice: {asked:?}");
        seen.push(*id);
        assert!(website::picture(*id).is_some(), "picture {id} is not one the site has");
    }
    // After the frame that asked for the last one, none asks for anything.
    let last = asked.last().map_or(0, |a| a.0);
    assert!(asked.iter().all(|a| a.0 <= last));
    assert!(website::new_pictures(ui.scene(), &mut made).is_empty(), "a call with nothing new asked for something");
    let c = &ui.perf().counters;
    assert_eq!(c.layouts - c0.layouts, 0, "the scroll laid the page out again");
}
