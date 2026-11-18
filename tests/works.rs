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
        // A piece that is not built has no page: its address goes on to the index (N.2 item 1).
        let to = if p.built { Page::Work(i as u8) } else { Page::Works };
        assert_eq!(Page::from_path(&format!("works/{}/", p.slug)), Some(to));
        assert_eq!(Page::Work(i as u8).section(), gallery::Section::Works);
        assert_eq!(Page::all().contains(&Page::Work(i as u8)), p.built);
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
    for (i, p) in works::PIECES.iter().enumerate().filter(|(_, p)| p.built) {
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
fn a_window_of_the_index_becomes_its_piece_by_a_change_of_opacity() {
    // Until the framework's flow grows the window (N.3 item 1): 120 ms of opacity, not a cut.
    let mut ui = gallery::new_ui(fonts(), 1440, 824, 1.0);
    ui.pointer_leave();
    assert!(ui.state.route("works/"));
    ui.relayout();
    ui.tick(3.0);
    assert!(ui.state.route("works/ground-truth/"));
    let cmds = ui.state.cmds.clone();
    ui.relayout();
    assert!(matches!(cmds.as_slice(), [amenui::morph::Cmd::Flow(f)] if f.flow_ms == 0.0 && f.out_ms == 120.0), "{cmds:?}");
}

fn walk<'a>(t: &'a amenui::tree::TreeNode, out: &mut Vec<&'a amenui::tree::TreeNode>) { out.push(t); for c in &t.children { walk(c, out); } }
fn nodes(ui: &GalleryUi) -> Vec<amenui::tree::TreeNode> {
    let t = ui.export_tree();
    let mut v = vec![];
    walk(&t, &mut v);
    v.into_iter().cloned().collect()
}

#[test]
fn only_a_built_piece_is_a_window_that_answers() {
    let ui = at(Page::Works, 1440, 824);
    // With no pointer the wide window is the first built piece; Tab reaches it and no other window.
    assert_eq!(ui.state.works.wide, works::first_built());
    let n = nodes(&ui);
    let windows: Vec<_> = n.iter().filter(|t| t.key.starts_with("works/w") && t.key.len() == "works/w0".len() && t.focusable).map(|t| t.key.clone()).collect();
    assert_eq!(windows, vec![format!("works/w{}", works::GROUND_TRUTH)]);
    // An unbuilt piece's address shows the index.
    let mut ui = at(Page::Works, 1440, 824);
    assert!(ui.state.route("works/touch-grass/"));
    assert_eq!(ui.state.page, Page::Works);
    // From the one built piece the arrows go nowhere.
    let mut ui2 = at(Page::Work(works::GROUND_TRUTH as u8), 1440, 900);
    assert!(works::key(&mut ui2.state, amenui::el::Key::Right));
    assert_eq!(ui2.state.page, Page::Work(works::GROUND_TRUTH as u8));
    ui.relayout();
}

/// A label's ink box from its layout box (the type keeps its size in a smaller window).
fn ink(r: [f32; 4]) -> [f32; 4] { if r[3] >= 18.0 { [r[0] + 1.0, r[1] + 5.0, r[0] + r[2] - 2.0, r[1] + 13.0] } else { [r[0] + 1.0, r[1] + 1.0, r[0] + r[2] - 2.0, r[1] + 10.0] } }
fn meets(a: [f32; 4], b: [f32; 4]) -> bool { a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3] }

#[test]
fn a_note_in_a_window_is_whole_or_not_there() {
    for (w, h) in [(1440, 824), (390, 768)] {
        let ui = at(Page::Works, w, h);
        let n = nodes(&ui);
        let win = n.iter().find(|t| t.key == format!("works/w{}", works::GROUND_TRUTH)).expect("window").rect;
        let labels: Vec<_> = n.iter().filter(|t| t.key.starts_with("works/window/work/ground-truth/") && t.text.is_some()).collect();
        let mut seen: Vec<[f32; 4]> = vec![];
        for l in &labels {
            let r = ink(l.rect);
            assert!(r[0] >= win[0] + 8.0 && r[2] <= win[0] + win[2] - 8.0 && r[1] >= win[1] + 8.0 && r[3] <= win[1] + win[3] - 8.0, "{w}: {} {:?} in {win:?}", l.key, l.rect);
            assert!(!seen.iter().any(|s| meets(*s, r)), "{w}: {} overlaps another label", l.key);
            seen.push(r);
        }
        // Not live: no scan window and no pointer.
        assert!(!n.iter().any(|t| t.key.contains("ground-truth/scan") || t.key.contains("ground-truth/human")), "{w}");
    }
}

#[test]
fn the_phone_ruler_marks_each_band_at_its_centre() {
    let ui = at(Page::Works, 390, 768);
    let n = nodes(&ui);
    for i in 0..works::COUNT {
        let band = n.iter().find(|t| t.key == format!("works/w{i}")).expect("band").rect;
        let sq = n.iter().find(|t| t.key == format!("works/ruler/p{i}")).expect("square").rect;
        assert!(((sq[1] + sq[3] * 0.5) - (band[1] + band[3] * 0.5)).abs() <= 1.0, "{i}: {sq:?} beside {band:?}");
        assert!((band[3] - if works::PIECES[i].built { 132.0 } else { 48.0 }).abs() < 0.5, "{i}: {band:?}");
        assert!((band[0] - 32.0).abs() < 0.5 && (band[0] + band[2] - 374.0).abs() < 0.5, "{i}: {band:?}");
    }
    assert!(!n.iter().any(|t| t.text.is_some() && t.rect[0] < 32.0 && t.key.starts_with("works/ruler")), "no hours beside the bands");
}

#[test]
fn the_scan_window_says_what_it_draws() {
    for (w, h, cols, rows) in [(1440, 900, 21, 14), (390, 844, 15, 10)] {
        let ui = at(Page::Work(works::GROUND_TRUTH as u8), w, h);
        let n = nodes(&ui);
        let label = n.iter().find(|t| t.key == "work/ground-truth/scan-label").and_then(|t| t.text.clone()).expect("label");
        assert_eq!(label, format!("SCAN {}×{}   {cols} × {rows} CELLS", cols * 14, rows * 14));
        let cells: Vec<(usize, usize)> = n.iter().filter_map(|t| t.key.strip_prefix("work/ground-truth/scan/")?.split_once('/').and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))).collect();
        assert!(cells.iter().all(|&(i, j)| i < cols && j < rows), "{w}: a cell outside {cols} × {rows}");
        assert!(cells.len() > cols * rows / 2, "{w}: {} cells drawn", cells.len());
        // Every glyph is one character of the ramp.
        assert!(n.iter().filter(|t| t.key.starts_with("work/ground-truth/scan/")).all(|t| t.text.as_deref().is_some_and(|c| c.chars().count() == 1 && ".:-=+*#".contains(c))));
        // To assistive technology the characters are one picture with the label's words.
        let a = ui.a11y();
        assert!(!a.iter().any(|t| t.key.starts_with("work/ground-truth/scan/")), "{w}: a cell in the accessibility mirror");
        let pic = a.iter().find(|t| t.key == "work/ground-truth/scan").expect("the scan window in the mirror");
        assert_eq!(pic.name, label);
    }
}

#[test]
fn on_a_phone_nothing_of_ground_truth_comes_near_its_title_block() {
    let ui = at(Page::Work(works::GROUND_TRUTH as u8), 390, 844);
    let n = nodes(&ui);
    let block = ["n", "title", "micro"].iter().map(|p| n.iter().find(|t| t.key == format!("work/ground-truth/{p}")).expect("block").rect)
        .fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |a, r| [a[0].min(r[0]), a[1].min(r[1]), a[2].max(r[0] + r[2]), a[3].max(r[1] + r[3])]);
    let near = [block[0] - 24.0, block[1] + 5.0 - 24.0, block[2] + 24.0, block[3] + 24.0];
    for t in n.iter().filter(|t| t.text.is_some() && t.key.starts_with("work/ground-truth/") && !["n", "title", "micro", "made", "prev", "next", "steps", "strip"].iter().any(|p| t.key.starts_with(&format!("work/ground-truth/{p}")))) {
        let r = ink(t.rect);
        assert!(!meets(r, near), "{} {:?} near the title block {block:?}", t.key, t.rect);
        assert!(r[0] >= 8.0 && r[2] <= 382.0, "{} {:?} crosses the edge", t.key, t.rect);
    }
}

#[test]
fn ground_truth_keeps_within_its_marks() {
    // M.2: at most 6,000 instances on a desktop and 2,500 on a phone, with the flight, the scan
    // window and the pointer all on screen.
    for (w, h, cap) in [(1440, 900, 6000), (390, 844, 2500)] {
        let mut ui = at(Page::Work(works::GROUND_TRUTH as u8), w, h);
        ui.pointer_move(w as f32 * 0.4, h as f32 * 0.55);
        let mut t = 0.0;
        while t < 1.0 { t += 1.0 / 30.0; ui.tick(t); ui.relayout(); ui.frame_drawn(); }
        let s = ui.scene();
        let n = s.instances.len() + s.annot.len();
        eprintln!("ground truth at {w}x{h}: {n} instances");
        assert!(n > 100 && n <= cap, "{w}: {n}");
    }
}
