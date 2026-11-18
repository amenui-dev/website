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

fn han(t: &amenui::tree::TreeNode) -> bool { t.text.as_deref().is_some_and(|s| s.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))) }

#[test]
fn the_chrome_of_works_follows_the_locale() {
    // O.2 item 5: in zh-Hans the lead, the keys line and `制作方法 ↗` hold Han characters; the
    // picture's layer, the titles and the live line hold none.
    for (w, h) in [(1440, 824), (390, 768)] {
        let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
        gallery::update(&mut ui.state, Msg::Locale(amenui::i18n::Locale::ZhHans));
        gallery::update(&mut ui.state, Msg::Go(Page::Works));
        if w > 720 { gallery::update(&mut ui.state, Msg::Works(works::WMsg::Hover(works::GROUND_TRUTH, Some((900.0, 400.0))))); }
        ui.relayout();
        let n = nodes(&ui);
        let find = |k: &str| n.iter().find(|t| t.key == k).unwrap_or_else(|| panic!("{w}: {k}"));
        assert!(han(find("works/lead")), "{w}: the lead");
        if w > 720 {
            assert!(han(find("works/keys")), "{w}: the keys line");
            let live = find(&format!("works/w{}/live", works::GROUND_TRUTH));
            assert!(!han(live), "{w}: {:?}", live.text);
        }
        assert!(!han(find("works/title")));
        for t in n.iter().filter(|t| t.key.contains("/title") && t.key.starts_with("works/w") || t.key.contains("work/ground-truth/")) { assert!(!han(t), "{w}: {} {:?}", t.key, t.text); }
    }
    for (w, h) in [(1440, 900), (390, 844)] {
        let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
        gallery::update(&mut ui.state, Msg::Locale(amenui::i18n::Locale::ZhHans));
        gallery::update(&mut ui.state, Msg::Go(Page::Work(works::GROUND_TRUTH as u8)));
        ui.relayout();
        let n = nodes(&ui);
        let made = n.iter().find(|t| t.key.starts_with("work/ground-truth/made") && t.text.is_some()).expect("made");
        assert!(han(made), "{w}: {:?}", made.text);
        assert_eq!(made.text.as_deref(), Some("制作方法 ↗"));
        for t in n.iter().filter(|t| t.key == "work/ground-truth/title" || t.key.starts_with("work/ground-truth/layer") || t.key.starts_with("work/ground-truth/bird")) { assert!(!han(t), "{w}: {}", t.key); }
    }
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

/// The characters of the scan window drawn on a piece's page (column, row, the cell's rect), from its
/// lines of type, one a row (a glyph stands 3 px in from its cell's left; a line's box starts 2 px above
/// its cell).
fn scan_cells(n: &[amenui::tree::TreeNode]) -> Vec<(usize, usize, [f32; 4])> {
    n.iter().filter_map(|t| {
        let j: usize = t.key.strip_prefix("work/ground-truth/scan/row/")?.parse().ok()?;
        let (x0, y0) = (t.rect[0] - 3.0, t.rect[1] + 2.0);
        Some(t.text.as_deref()?.chars().enumerate().filter(|(_, c)| *c != ' ').map(move |(i, _)| (i, j, [x0 + 14.0 * i as f32, y0, x0 + 14.0 * (i + 1) as f32, y0 + 14.0])).collect::<Vec<_>>())
    }).flatten().collect()
}

#[test]
fn the_scan_window_says_what_it_draws_and_is_a_full_grid() {
    // O.2 item 2: the characters inside the window are the label's product, less the cells under the
    // pointer's box and its label's plate.
    for (w, h, cols, rows) in [(1440, 900, 21, 14), (390, 844, 15, 10)] {
        let ui = at(Page::Work(works::GROUND_TRUTH as u8), w, h);
        let n = nodes(&ui);
        let label = n.iter().find(|t| t.key == "work/ground-truth/scan-label").and_then(|t| t.text.clone()).expect("label");
        assert_eq!(label, format!("SCAN {}×{}   {cols} × {rows} CELLS", cols * 14, rows * 14));
        let cells = scan_cells(&n);
        assert!(cells.iter().all(|&(i, j, _)| i < cols && j < rows), "{w}: a cell outside {cols} × {rows}");
        assert_eq!(cells.len(), cols * rows, "{w}: with no pointer every cell has its character");
        // Every glyph is one character of the ramp, a line a row.
        let lines: Vec<_> = n.iter().filter(|t| t.key.starts_with("work/ground-truth/scan/row/")).collect();
        assert_eq!(lines.len(), rows, "{w}: a line a row");
        assert!(lines.iter().all(|t| t.text.as_deref().is_some_and(|l| l.chars().count() == cols && l.chars().all(|c| ".:-=+*#".contains(c)))), "{w}");
        // To assistive technology the characters are one picture with the label's words.
        let a = ui.a11y();
        assert!(!a.iter().any(|t| t.key.starts_with("work/ground-truth/scan/")), "{w}: a cell in the accessibility mirror");
        let pic = a.iter().find(|t| t.key == "work/ground-truth/scan").expect("the scan window in the mirror");
        assert_eq!(pic.name, label);
        // With the pointer over the window: the hole is the box and the plate, and nothing else.
        let (x0, y0) = cells.iter().find(|c| c.0 == 0 && c.1 == 0).map(|c| (c.2[0], c.2[1])).expect("the first cell");
        let mut ui = at(Page::Work(works::GROUND_TRUTH as u8), w, h);
        let (qx, qy) = (x0 + 14.0 * 5.0 + 3.0, y0 + 14.0 * 6.0 + 5.0);
        // The window rests where the pointer is once its spring has caught up: hold it there.
        ui.pointer_move(qx, qy);
        ui.state.works.ground.pinned = true;
        ui.state.works.ground.scan = Some((x0 + cols as f32 * 7.0, y0 + rows as f32 * 7.0));
        ui.relayout();
        let n = nodes(&ui);
        let cells = scan_cells(&n);
        let meets = |a: [f32; 4], b: [f32; 4]| a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3];
        let bx = [qx - 16.0, qy - 16.0, qx + 16.0, qy + 16.0];
        let human = n.iter().find(|t| t.key.starts_with("work/ground-truth/human/")).expect("HUMAN?").rect;
        let plate = [human[0] + 1.0 - 6.0, human[1] + 5.0 - 3.0, human[0] + human[2] - 2.0 + 6.0, human[1] + 13.0 + 3.0];
        let hole = (0..cols).flat_map(|i| (0..rows).map(move |j| (i, j))).filter(|&(i, j)| {
            let c = [x0 + 14.0 * i as f32, y0 + 14.0 * j as f32, x0 + 14.0 * (i + 1) as f32, y0 + 14.0 * (j + 1) as f32];
            meets(c, bx) || meets(c, plate)
        }).count();
        assert!(hole > 0, "{w}: the pointer is over the window");
        assert_eq!(cells.len(), cols * rows - hole, "{w}: {} cells drawn, {hole} under the pointer", cells.len());
    }
}

#[test]
fn the_flight_flies_above_the_cloud_and_no_bird_is_left_out() {
    // O.2 item 1, over a whole flight sampled every 500 ms.
    let meets = |a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)| a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3;
    for (w, h) in [(1440, 900), (390, 844), (1280, 800), (1920, 1080), (360, 740), (768, 1024), (1440, 700)] {
        let mut ui = at(Page::Work(works::GROUND_TRUTH as u8), w, h);
        let (wf, hf) = (w as f32, h as f32);
        for k in 0..32 {
            ui.state.works.ground.flight = Some(k as f32 * 0.5);
            let d = works::drawn(&ui.state);
            let (cloud, cloud_label) = d.cloud.expect("the cloud");
            let inside = d.flock.iter().filter(|r| r.0 >= 8.0 && r.1 >= 8.0 && r.2 <= wf - 8.0 && r.3 <= hf - 8.0).count();
            assert_eq!(d.birds.len(), inside, "{w}×{h} at {k}: {} drawn, {inside} inside", d.birds.len());
            let lowest = d.tags.iter().map(|t| t.3).fold(f32::MIN, f32::max);
            assert!(cloud.1 - lowest >= 24.0, "{w}×{h}: the lowest label ends at {lowest}, the cloud's box starts at {}", cloud.1);
            assert!(!d.echoes.iter().any(|&e| meets(e, cloud) || meets(e, cloud_label)), "{w}×{h} at {k}: an echo on the cloud");
            assert!(!d.labels.iter().any(|&l| d.birds.iter().any(|&b| meets(l, b))), "{w}×{h} at {k}: a label on a bird");
        }
        // The rest pose: halfway.
        ui.state.works.ground.flight = Some(8.0);
        let d = works::drawn(&ui.state);
        assert_eq!(d.birds.len(), 7, "{w}×{h}: the rest pose");
    }
}

#[test]
fn the_cloud_label_and_the_number_read() {
    // O.2 item 3: the cloud's label at 4.5 : 1 or more at both sizes; the title block's number in the
    // accent only where the accent measures 4.5 : 1 against the median under it.
    // P.2 item 1: else in the foot's ink where that reads, else not drawn. W-03: accent at 1440, absent
    // at 390.
    for (w, h, accent, shown) in [(1440, 900, true, true), (390, 844, false, false)] {
        let ui = at(Page::Work(works::GROUND_TRUTH as u8), w, h);
        let d = works::drawn(&ui.state);
        assert!(d.cloud_contrast >= 4.5, "{w}: the cloud's label at {:.2}", d.cloud_contrast);
        let (c, ink) = works::number_contrast(&ui.state, works::GROUND_TRUTH, w as f32, h as f32);
        assert_eq!(c >= 4.5, accent, "{w}: the accent at {c:.2}");
        assert_eq!(c >= 4.5 || ink >= 4.5, shown, "{w}: the foot's ink at {ink:.2}");
        assert_eq!(nodes(&ui).iter().any(|t| t.key == "work/ground-truth/n"), shown, "{w}: the number drawn");
        eprintln!("{w}: cloud label {:.2}, number's accent {c:.2}, ink {ink:.2}", d.cloud_contrast);
    }
}

#[test]
fn on_a_phone_nothing_of_ground_truth_comes_near_its_title_block() {
    let ui = at(Page::Work(works::GROUND_TRUTH as u8), 390, 844);
    let n = nodes(&ui);
    let block = ["n", "title", "micro"].iter().filter_map(|p| n.iter().find(|t| t.key == format!("work/ground-truth/{p}")).map(|t| t.rect))
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

#[test]
fn the_scan_window_at_rest_lays_nothing_out_again() {
    // O.2 item 4: during a flight, with the window still, its characters are not laid out again (a line
    // of type a row, its glyphs coloured by their poses).
    let mut ui = at(Page::Work(works::GROUND_TRUTH as u8), 1440, 900);
    let mut t = 0.0f32;
    for _ in 0..10 { t += 1.0 / 30.0; ui.tick(t); ui.relayout(); ui.frame_drawn(); }
    let before = ui.counters().shaped;
    for _ in 0..60 { t += 1.0 / 30.0; ui.tick(t); ui.relayout(); ui.frame_drawn(); }
    assert!(ui.state.works.ground.flight.is_some(), "the flight is under way");
    assert_eq!(ui.counters().shaped, before, "paragraphs shaped during a flight");
    let rows = nodes(&ui).iter().filter(|n| n.key.starts_with("work/ground-truth/scan/")).count();
    assert_eq!(rows, 14, "a line a row");
}
