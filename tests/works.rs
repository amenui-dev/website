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
        // A piece with no page yet: its address goes on to the index (N.2 item 1). A built piece has one.
        let to = if p.page { Page::Work(i as u8) } else { Page::Works };
        assert_eq!(Page::from_path(&format!("works/{}/", p.slug)), Some(to));
        assert_eq!(Page::Work(i as u8).section(), gallery::Section::Works);
        assert_eq!(Page::all().contains(&Page::Work(i as u8)), p.page);
        assert!(p.page || !p.built);
    }
    // The old addresses lead on: the scenes' own section and the Lab's old levels.
    assert_eq!(Page::from_path("scenes/"), Some(Page::Lab));
    assert_eq!(Page::from_path("lab/scenes/reading/"), Some(Page::Scene(1)));
    assert_eq!(Page::Scene(1).path(), "lab/read-an-article/");
    let moved = gallery::routes::moved();
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
fn a_window_of_the_index_grows_into_its_piece() {
    // O.3 item 1: a Flow of 420 ms, held tight, the leaving windows out in 180 ms.
    let mut ui = gallery::new_ui(fonts(), 1440, 824, 1.0);
    ui.pointer_leave();
    assert!(ui.state.route("works/"));
    ui.relayout();
    ui.tick(3.0);
    assert!(ui.state.route("works/ground-truth/"));
    let cmds = ui.state.cmds.clone();
    ui.relayout();
    assert!(matches!(cmds.as_slice(), [amenui::morph::Cmd::Flow(f)] if f.flow_ms == 420.0 && f.flow_loose == 0.0 && f.out_ms == 180.0 && f.fade_ms == 150.0), "{cmds:?}");
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
    let mut ui = at(Page::Works, 1440, 900);
    // With no pointer the wide window is the first built piece in the day's order: W-01 (T.2 item 2).
    assert_eq!(ui.state.works.wide, works::first_built());
    assert_eq!(works::first_built(), works::TOUCH_GRASS);
    let n = nodes(&ui);
    let windows: Vec<_> = n.iter().filter(|t| t.key.starts_with("works/w") && t.key.len() == "works/w0".len() && t.focusable).map(|t| t.key.clone()).collect();
    let built: Vec<_> = works::PIECES.iter().enumerate().filter(|(_, p)| p.built).map(|(i, _)| format!("works/w{i}")).collect();
    assert_eq!(windows, built);
    assert_eq!(built, vec!["works/w0".to_owned(), format!("works/w{}", works::GROUND_TRUTH), "works/w3".to_owned()]);
    // Tab from the heading reaches the three windows, 01, 03 and 04, and no other.
    let mut seen = vec![];
    for _ in 0..40 {
        ui.key(amenui::el::Key::Tab, false);
        if let Some(f) = ui.focus() && f.starts_with("works/w") && !seen.iter().any(|s: &String| s == f) { seen.push(f.to_owned()); }
    }
    assert_eq!(seen, built, "Tab reaches {seen:?}");
    // The address of a piece with no page shows the index; a built piece's is its page.
    let mut ui = at(Page::Works, 1440, 824);
    assert!(ui.state.route("works/line-too-long/"));
    assert_eq!(ui.state.page, Page::Works);
    assert!(ui.state.route("works/touch-grass/"));
    assert_eq!(ui.state.page, Page::Work(works::TOUCH_GRASS as u8));
    ui.relayout();
    // The arrows go between the built pieces, round the day.
    for (from, to) in [(works::GROUND_TRUTH, 3), (3, works::TOUCH_GRASS), (works::TOUCH_GRASS, works::GROUND_TRUTH)] {
        let mut ui2 = at(Page::Work(from as u8), 1440, 900);
        assert!(works::key(&mut ui2.state, amenui::el::Key::Right));
        assert_eq!(ui2.state.page, Page::Work(to as u8));
    }
}

/// X.3: W-04 is a built piece on the index: the arrows reach it, its window widens with its title at
/// full ink and its ruler square filled in the accent, and the window and the phone band carry the line
/// of type.
#[test]
fn nightly_is_a_built_piece_on_the_index() {
    assert!(works::on_index(3));
    let mut ui = at(Page::Works, 1440, 900);
    while ui.state.works.wide != 3 { assert!(works::key(&mut ui.state, amenui::el::Key::Right)); }
    ui.relayout();
    let n = nodes(&ui);
    let rect = |k: &str| n.iter().find(|x| x.key == k).unwrap_or_else(|| panic!("{k}")).rect;
    let (w, log) = (rect("works/w3"), rect("work/nightly/log"));
    assert!(log[0] < w[0] + w[2] && w[0] < log[0] + log[2] && log[1] >= w[1] && log[1] + log[3] <= w[1] + w[3], "the line {log:?} in the window {w:?}");
    let mut ui = at(Page::Works, 390, 844);
    ui.relayout();
    let n = nodes(&ui);
    let rect = |k: &str| n.iter().find(|x| x.key == k).unwrap_or_else(|| panic!("{k}")).rect;
    // The band shows the line's glyphs (posed onto the curve, inside the band; its box lies higher).
    let (b, _) = (rect("works/w3"), rect("work/nightly/log"));
    assert!(b[3] > 100.0, "the band {b:?}");
    gallery::update(&mut ui.state, Msg::Works(works::WMsg::Tap(3)));
    gallery::update(&mut ui.state, Msg::Works(works::WMsg::Tap(3)));
    assert_eq!(ui.state.page, Page::Work(3), "a second tap on W-04's band opens it");
}

/// X.2 item 1: from 720 to 1,440 wide, where the small line would come within 32 px of the strip's
/// leftmost word, the title block rises (one row of the strip's small type and 8 px) and the strip keeps
/// the bottom row. The small line is one line with every word; the clear ground goes with the block: no
/// note of W-03's and no dash of W-04's reflection in it (W-01's stalks: `no_stalk_of_touch_grass_comes_near_its_type`).
/// At 1440 × 900 nothing moves.
#[test]
fn between_720_and_1440_the_title_block_rises_clear_of_the_strip() {
    let grow = |r: [f32; 4], d: f32| [r[0] - d, r[1] - d, r[2] + d, r[3] + d];
    let bx = |r: [f32; 4]| [r[0], r[1], r[0] + r[2], r[1] + r[3]];
    for (w, h, english) in [(1024u32, 768u32, true), (800, 600, true), (1440, 900, true), (1024, 768, false), (800, 600, false), (1440, 900, false)] {
        for i in [0usize, 2, 3] {
            let ui = if english { en(Page::Work(i as u8), w, h) } else { at(Page::Work(i as u8), w, h) };
            let slug = works::PIECES[i].slug;
            let n = nodes(&ui);
            let key = |p: &str| format!("work/{slug}/{p}");
            let node = |p: &str| n.iter().find(|t| t.key == key(p)).unwrap_or_else(|| panic!("{w}: {}", key(p)));
            let small = node("micro");
            let m = bx(small.rect);
            assert!(small.rect[3] <= 18.5 && small.text.as_deref().is_some_and(|t| t.ends_with("PRESS TO RUN THE NEXT STEP") || t.contains(" · ")), "{w} {slug}: the small line is one line");
            let strip: Vec<_> = n.iter().filter(|t| t.key.starts_with(&key("strip")) || t.key.starts_with(&key("made")) || t.key.starts_with(&key("steps")) || t.key.starts_with(&key("prev")) || t.key.starts_with(&key("next"))).collect();
            assert!(strip.len() >= 5, "{w} {slug}: the strip");
            let (mut dx, mut dy) = (f32::MAX, f32::MAX);
            for b in &strip {
                let r = bx(b.rect);
                let (gx, gy) = ((r[0] - m[2]).max(m[0] - r[2]), (r[1] - m[3]).max(m[1] - r[3]));
                assert!(gx >= 32.0 || gy >= 8.0, "{w}x{h} {slug}: the small line {m:?} and {} {r:?}", b.key);
                dx = dx.min(gx); dy = dy.min(gy);
            }
            let rise = (h as f32 - 56.0) - small.rect[1];
            eprintln!("{w}x{h} {} {slug}: the block rises {rise} px; the small line {dx:.0} px from the strip in x, {dy:.0} in y", if english { "en" } else { "zh-Hans" });
            if w == 1440 { assert_eq!(rise, 0.0, "at 1440 x 900 nothing moves"); }
            // The clear ground: the block from the number's (or the title's) ink to the small line, 24 px out.
            let t = bx(node("title").rect);
            let top = n.iter().find(|t| t.key == key("n")).map_or(t[1] + 12.0, |n| n.rect[1] + 5.0);
            // (Half a pixel less: a box's width is its text's advance rounded up.)
            let clear = grow([32.0, top, m[2].max(t[2]), m[3]], 23.5);
            if i == 2 {
                for note in n.iter().filter(|t| t.text.is_some() && t.key.starts_with("work/ground-truth/") && !strip.iter().any(|s| s.key == t.key) && !["n", "title", "micro"].iter().any(|p| t.key == key(p))) {
                    assert!(!meets(bx(note.rect), clear), "{w}x{h}: {} {:?} in the block's clear ground {clear:?}", note.key, note.rect);
                }
            }
            if i == 3 {
                let dashes = ui.instances_of("work/nightly/reflection");
                assert!(!dashes.is_empty());
                for d in dashes { let r = [d.rect[0], d.rect[1], d.rect[0] + d.rect[2], d.rect[1] + d.rect[3]]; assert!(!meets(r, clear), "{w}x{h}: a dash {r:?} in the block's clear ground {clear:?}"); }
            }
        }
    }
}

/// A label's ink box from its layout box (the type keeps its size in a smaller window).
fn ink(r: [f32; 4]) -> [f32; 4] { if r[3] >= 18.0 { [r[0] + 1.0, r[1] + 5.0, r[0] + r[2] - 2.0, r[1] + 13.0] } else { [r[0] + 1.0, r[1] + 1.0, r[0] + r[2] - 2.0, r[1] + 10.0] } }
fn meets(a: [f32; 4], b: [f32; 4]) -> bool { a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3] }

#[test]
fn a_note_in_a_window_is_whole_or_not_there() {
    for (w, h) in [(1440, 824), (390, 768)] {
        let mut ui = at(Page::Works, w, h);
        ui.tick(1.0);
        let n = nodes(&ui);
        let win = n.iter().find(|t| t.key == format!("works/w{}", works::GROUND_TRUTH)).expect("window").rect;
        // A label is shown only while clear (`show_clear`): the ones at opacity 0 are not there.
        let labels: Vec<_> = n.iter().filter(|t| t.key.starts_with("work/ground-truth/") && t.text.is_some() && ui.clear_opacity(&t.key).unwrap_or(1.0) > 0.5).collect();
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
        assert!((band[3] - if works::on_index(i) { 132.0 } else { 48.0 }).abs() < 0.5, "{i}: {band:?}");
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
fn a_bird_under_the_scan_window_is_read_as_a_character() {
    // P.2 item 5: with the window pinned in the flight's lane, the boxes drawn are the birds whose box is
    // 8 px inside the window and clear of the scan window grown by 8 px; the characters are still the
    // label's product; a bird whose centre is inside is a `#`.
    let meets = |a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)| a.0 < b.2 && b.0 < a.2 && a.1 < b.3 && b.1 < a.3;
    for (w, h, cols, rows) in [(1440, 900, 21, 14), (390, 844, 15, 10)] {
        let mut ui = at(Page::Work(works::GROUND_TRUTH as u8), w, h);
        let (wf, hf) = (w as f32, h as f32);
        let lane = works::drawn(&ui.state).flock[3];
        ui.state.works.ground.pinned = true;
        ui.state.works.ground.scan = Some((wf * 0.5, (lane.1 + lane.3) * 0.5));
        let mut under = 0;
        for k in 0..32 {
            ui.state.works.ground.flight = Some(k as f32 * 0.5);
            ui.relayout();
            let d = works::drawn(&ui.state);
            let r = d.scan.expect("the scan window");
            let g = (r.0 - 8.0, r.1 - 8.0, r.2 + 8.0, r.3 + 8.0);
            let shown = d.flock.iter().filter(|b| b.0 >= 8.0 && b.1 >= 8.0 && b.2 <= wf - 8.0 && b.3 <= hf - 8.0 && !meets(**b, g)).count();
            assert_eq!(d.birds.len(), shown, "{w}×{h} at {k}");
            let inside = d.flock.iter().filter(|b| { let (x, y) = (b.0 + 11.0, b.1 + 10.0); x >= r.0 && x < r.2 && y >= r.1 && y < r.3 }).count();
            under += inside;
            let n = nodes(&ui);
            let cells = scan_cells(&n);
            assert_eq!(cells.len(), cols * rows, "{w}×{h} at {k}: every cell has its character");
            assert!(!d.echoes.iter().chain(&d.birds).any(|&e| meets(e, (r.0 - 6.0, r.1 - 6.0, r.2 + 6.0, r.3 + 6.0))), "{w}×{h} at {k}: a box inside the brackets");
            let hashes: usize = n.iter().filter(|t| t.key.starts_with("work/ground-truth/scan/row/")).map(|t| t.text.as_deref().unwrap_or("").matches('#').count()).sum();
            assert!(hashes >= inside.min(1), "{w}×{h} at {k}: {inside} birds inside, {hashes} #");
        }
        assert!(under > 0, "{w}×{h}: the flight passes through the window");
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

/// W-01 on its page: the scene's instances, and whether a frame is wanted, at `t`.
fn grass_frames(ui: &mut GalleryUi, from: f32, to: f32) -> (u32, u32) {
    let (mut t, mut frames, mut quiet) = (from, 0, 0);
    while t < to {
        t += 1.0 / 60.0;
        ui.tick(t);
        if ui.needs_frame(t) { frames += 1; ui.settle(); ui.frame_drawn(); } else { quiet += 1; }
    }
    (frames, quiet)
}

#[test]
fn touch_grass_has_its_page_and_a_built_window_on_the_index() {
    let i = 0usize;
    let p = &works::PIECES[i];
    // Signed off (T.2 item 2): its window shows its still and states the count as built at 1440.
    assert!(p.page && p.built);
    assert_eq!(p.live, "WIND OVER A FIELD OF 1,570 STALKS");
    let mut ui = gallery::new_ui(fonts(), 1440, 900, 1.0);
    gallery::update(&mut ui.state, Msg::Locale(amenui::i18n::Locale::En));
    gallery::update(&mut ui.state, Msg::Go(Page::Work(i as u8)));
    ui.relayout();
    let n = nodes(&ui);
    let small = n.iter().filter_map(|n| n.text.clone()).find(|t| t.starts_with("W-01 ·")).expect("the small line");
    let count: usize = small.split(" · ").find(|s| s.ends_with(" STALKS")).and_then(|s| s.trim_end_matches(" STALKS").replace(',', "").parse().ok()).expect("its count");
    // M.4's spacing (R.2 item 1) less what stands too near the type (R.2 item 4).
    assert!((1_300..=1_700).contains(&count), "{small}");
    assert_eq!(count, works::touch_extents(&ui.state).len(), "the small line states the count as built");
    assert_eq!(p.live, format!("WIND OVER A FIELD OF {} STALKS", small.split(" · ").find(|s| s.ends_with(" STALKS")).unwrap().trim_end_matches(" STALKS")));
    // On the index its window answers: hovered it runs with its live line; pressed, it opens.
    let mut idx = at(Page::Works, 1440, 900);
    gallery::update(&mut idx.state, Msg::Works(works::WMsg::Hover(i, Some((400.0, 400.0)))));
    idx.relayout();
    let live = nodes(&idx).into_iter().find(|t| t.key == "works/w0/live").and_then(|t| t.text).expect("its live line");
    assert_eq!(live, format!("LIVE · {} · OPEN →", p.live));
    gallery::update(&mut idx.state, Msg::Works(works::WMsg::Tap(i)));
    assert_eq!(idx.state.page, Page::Work(i as u8));
    // On a phone at most 900 stalks.
    let ui = at(Page::Work(i as u8), 390, 844);
    let small = nodes(&ui).iter().filter_map(|n| n.text.clone()).find(|t| t.starts_with("W-01 ·")).expect("the small line");
    let count: usize = small.split(" · ").find(|s| s.ends_with(" STALKS")).and_then(|s| s.trim_end_matches(" STALKS").replace(',', "").parse().ok()).expect("its count");
    assert!(count <= 900, "{small}");
    assert_eq!(count, works::touch_extents(&ui.state).len(), "the small line states the count as built");
    // S.2 item 2: about 395 of 466, about 45 of them in the five nearest rows (the roots below the
    // tenth row's).
    let roots: Vec<f32> = works::touch_extents(&ui.state).iter().map(|(e, _)| e[3]).collect();
    let mut rows: Vec<f32> = roots.clone(); rows.sort_by(f32::total_cmp); rows.dedup();
    let near = rows.len().saturating_sub(5);
    let in_near = roots.iter().filter(|y| rows[near..].contains(y)).count();
    eprintln!("touch grass on a phone: {count} stalks, {in_near} in the five nearest rows ({} rows)", rows.len());
    assert!((350..=440).contains(&count) && (35..=60).contains(&in_near), "{count} stalks, {in_near} near");
}

/// R.2 item 4: no stalk's upright extent (with its head) within 24 px of the number, the title or the
/// small line as laid out, and no head within 8 px of the strip or the callout.
#[test]
fn no_stalk_of_touch_grass_comes_near_its_type() {
    let grow = |r: [f32; 4], d: f32| [r[0] - d, r[1] - d, r[0] + r[2] + d, r[1] + r[3] + d];
    // At 1024 and 800 in both languages: in English the title block rises there (X.2 item 1).
    for (w, h, english) in [(1440, 900, false), (390, 844, false), (1024, 768, false), (800, 600, false), (1024, 768, true), (800, 600, true)] {
        let ui = if english { en(Page::Work(0), w, h) } else { at(Page::Work(0), w, h) };
        let n = nodes(&ui);
        let rect = |k: &str| n.iter().find(|n| n.key == format!("work/touch-grass/{k}")).map(|n| n.rect);
        let mut types: Vec<[f32; 4]> = ["n", "title"].iter().filter_map(|k| rect(k)).map(|r| grow(r, 24.0)).collect();
        // The small line: on a wide page one line, kept 24 px from every stalk; on a phone (S.2 item 2)
        // under the 8 px head rule with the strip and the callout, each line wrapped at a space to the
        // box's width, 18 px a line, in mono at 69 / 9 px a character.
        let m = rect("micro").expect("the small line");
        if w >= 720 { types.push(grow(m, 24.0)); }
        let mut heads: Vec<[f32; 4]> = ["strip", "wind", "countdown"].iter().filter_map(|k| rect(k)).map(|r| grow(r, 8.0)).collect();
        if w < 720 {
            let small = n.iter().find(|n| n.key == "work/touch-grass/micro").and_then(|n| n.text.clone()).expect("its text");
            let (adv, per) = (69.0 / 9.0, (m[2] / (69.0 / 9.0)) as usize);
            let (mut line, mut y) = (String::new(), m[1]);
            for word in small.split(' ') {
                if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > per { heads.push(grow([m[0], y, line.chars().count() as f32 * adv, 18.0], 8.0)); line.clear(); y += 18.0; }
                if !line.is_empty() { line.push(' '); }
                line.push_str(word);
            }
            heads.push(grow([m[0], y, line.chars().count() as f32 * adv, 18.0], 8.0));
        }
        assert!(!heads.is_empty() && types.len() >= 2);
        for (e, head) in works::touch_extents(&ui.state) {
            assert!(!types.iter().any(|b| meets(e, *b)), "{w}x{h}: a stalk at {e:?} near the type");
            if let Some(hb) = head { assert!(!heads.iter().any(|b| meets(hb, *b)), "{w}x{h}: a head at {hb:?} near the strip or the callout"); }
        }
    }
}

#[test]
fn touch_grass_moves_in_gusts_at_thirty_frames_a_second_and_is_still_between_them() {
    let mut ui = at(Page::Work(0), 1440, 900);
    ui.ambient_fps(works::ambient_fps(ui.state.page));
    // R.4 (3): every blade segment and head, at most 7,500 at 1440 and 2,500 on a phone (once the
    // way in from the page before has finished).
    let count = |w: u32, h: u32| { let mut ui = at(Page::Work(0), w, h); ui.ambient_fps(works::ambient_fps(ui.state.page)); grass_frames(&mut ui, 0.0, 1.0); ui.scene().instances.len() };
    let (wide, phone) = (count(1440, 900), count(390, 844));
    eprintln!("touch grass: {wide} instances at 1440 x 900, {phone} at 390 x 844");
    assert!(wide <= 7_500, "{wide} instances");
    assert!(phone <= 2_500, "{phone} instances on a phone");
    // S.2 item 4, T.3 (2): the first frame (200 ms in) has the band 72 % across, the mast in its
    // leading half, and the sock lifted from its 55° below the horizontal to 16° to 28°, at both sizes.
    for (w, h) in [(1440, 900), (390, 844)] {
        let mut ui = at(Page::Work(0), w, h);
        ui.ambient_fps(works::ambient_fps(ui.state.page));
        // The clock from 0, as the render tool steps it, to the frame at 200 ms.
        grass_frames(&mut ui, -1.0 / 60.0, 0.2);
        let sock = ui.state.works.grass.sock;
        eprintln!("touch grass: the sock at {sock}° in the first frame at {w}");
        assert!((16..=28).contains(&sock), "the sock at {sock}° in the first frame at {w}");
    }
    grass_frames(&mut ui, 0.0, 0.2);
    // The first gust is under way from the first frame; between it and the second only the countdown
    // draws, once a second.
    let first = grass_frames(&mut ui, 0.2, 5.5);
    let gap = (ui.state.works.grass.next - 0.3, ui.state.works.grass.next);
    eprintln!("first gust: {} frames, {} quiet; next gust at {:.2} s", first.0, first.1, gap.1);
    assert!(first.0 <= (5.5 * 31.0) as u32, "{} frames in 5.5 s", first.0);
    let minute = grass_frames(&mut ui, 5.5, 60.0);
    eprintln!("the first minute after 5.5 s: {} frames", minute.0);
    assert!(minute.0 > 0 && minute.1 > 0);
}

/// W-01's CPU time a frame, natively: during a gust, and while the pointer walks the field at 60 events
/// a second (what it stirs goes in at 30 frames a second).
#[test]
#[ignore = "timings: cargo test --profile quick --test works touch_grass_costs -- --ignored --nocapture"]
fn touch_grass_costs() {
    use std::time::Instant;
    // The index's still, the first time it is made (T.5).
    let a = Instant::now();
    let still = works::picture_pixels(works::PIC_STILL).expect("the still");
    eprintln!("touch grass's still: {:.1} ms the first time ({} x {})", a.elapsed().as_secs_f64() * 1e3, still.0, still.1);
    let mut ui = at(Page::Work(0), 1440, 900);
    ui.ambient_fps(works::ambient_fps(ui.state.page));
    let mut t = 0.0f32;
    let (mut gust, mut n) = (0.0f64, 0);
    while t < 4.0 {
        t += 1.0 / 30.0;
        let a = Instant::now();
        ui.tick(t);
        if ui.needs_frame(t) { ui.settle(); let _ = ui.scene().instances.len(); ui.frame_drawn(); gust += a.elapsed().as_secs_f64(); n += 1; }
    }
    // The pointer walks the field at 60 events a second and every 1.5 s jumps back 800 px in one
    // event, faster than a pointer that pushes a gust ahead of it.
    let (mut walk, mut m, mut worst) = (0.0f64, 0, (0.0f64, 0usize));
    let (mut jump, mut jumps) = (0.0f64, 0);
    for k in 0..240usize {
        t += 1.0 / 60.0;
        let a = Instant::now();
        ui.pointer_move(300.0 + 800.0 * ((k % 90) as f32 / 90.0), 720.0);
        ui.tick(t);
        if ui.needs_frame(t) { ui.settle(); let _ = ui.scene().instances.len(); ui.frame_drawn(); m += 1; }
        let e = a.elapsed().as_secs_f64();
        walk += e;
        if k > 0 && k % 90 == 0 { jump += e; jumps += 1; }
        if e > worst.0 { worst = (e, k); }
    }
    eprintln!("touch grass, CPU: a frame during a gust {:.2} ms ({n} frames); the pointer walking {:.2} ms an event (240 events, {m} frames), the worst {:.2} ms (event {}), a jump of 800 px {:.2} ms; {} instances",
        gust * 1e3 / n as f64, walk * 1e3 / 240.0, worst.0 * 1e3, worst.1, jump * 1e3 / jumps.max(1) as f64, ui.scene().instances.len());
}

/// W-04's CPU time a frame at 1440 x 900: at rest (the curtain's 30 frames a second) and through a
/// pulse (a press, then 1 s), and the first frame. For the recipe page.
#[test]
#[ignore = "timings: cargo test --profile quick --test works nightly_costs -- --ignored --nocapture"]
fn nightly_costs() {
    use std::time::Instant;
    let a = Instant::now();
    let mut ui = en(Page::Work(3), 1440, 900);
    let first = a.elapsed().as_secs_f64();
    ui.ambient_fps(works::ambient_fps(ui.state.page));
    let run = |ui: &mut GalleryUi, from: f32, to: f32| {
        let (mut t, mut v) = (from, vec![]);
        while t < to {
            t += 1.0 / 60.0;
            let a = Instant::now();
            ui.tick(t);
            if ui.needs_frame(t) { ui.settle(); let _ = ui.scene().instances.len(); ui.frame_drawn(); v.push(a.elapsed().as_secs_f64() * 1e3); }
        }
        v.sort_by(f64::total_cmp);
        let mean = v.iter().sum::<f64>() / v.len().max(1) as f64;
        (v.len(), mean, v[(v.len() * 95 / 100).min(v.len() - 1)], v[v.len() - 1])
    };
    let rest = run(&mut ui, 3.0, 13.0);
    press(&mut ui, 700.0, 300.0, 13.0);
    let pulse = run(&mut ui, 13.0, 14.0);
    eprintln!("nightly, CPU at 1440 x 900: the first frame {:.1} ms (with the layout); at rest {} frames in 10 s, mean {:.2} ms, p95 {:.2}, worst {:.2}; a pulse {} frames in 1 s, mean {:.2} ms, p95 {:.2}, worst {:.2}; {} instances",
        first * 1e3, rest.0, rest.1, rest.2, rest.3, pulse.0, pulse.1, pulse.2, pulse.3, ui.scene().instances.len());
}

/// R.2 item 7, T.2 item 1: the windsock hangs at 55° with no wind, is level (at most 2°) at the peak of
/// a strong gust, and with reduced motion holds at the held band's peak, 55° · (1 − 0.79) = 11.5°.
#[test]
fn the_windsock_lifts_in_a_strong_gust_and_holds_at_the_bands_peak_with_reduced_motion() {
    for (w, h) in [(1440, 900), (390, 844)] {
        let mut ui = at(Page::Work(0), w, h);
        ui.ambient_fps(works::ambient_fps(ui.state.page));
        // Through the fourth gust, the first strong one.
        let (mut t, mut least, mut most) = (-1.0 / 60.0, i32::MAX, i32::MIN);
        while ui.state.works.grass.gusts < 5 && t < 90.0 {
            t += 1.0 / 60.0;
            ui.tick(t);
            if ui.needs_frame(t) { ui.settle(); ui.frame_drawn(); }
            let g = &ui.state.works.grass;
            if g.current.is_some_and(|(n, _)| n == 3) { least = least.min(g.sock); }
            most = most.max(g.sock);
        }
        eprintln!("touch grass at {w}: the sock at {least}° at the least in the strong gust, {most}° at the most");
        assert!(least <= 2, "{w}: the sock at {least}° in a strong gust");
        assert_eq!(most, 55, "{w}: between gusts");
        let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
        amenui::app::App::set_reduced_motion(&mut ui.state, true);
        gallery::update(&mut ui.state, Msg::Go(Page::Work(0)));
        ui.relayout();
        grass_frames(&mut ui, 0.0, 3.0);
        let sock = ui.state.works.grass.sock;
        eprintln!("touch grass at {w}, reduced motion: the sock at {sock}°");
        assert!((10..=13).contains(&sock), "{w}: the sock at {sock}° with reduced motion");
    }
}

/// T.5: W-01's window crops its still about the mast: wide, centred on x 950 of the 1,440 as far as the
/// picture's right edge allows (the mast between 60 % and 75 % across); narrow, on the mast; and
/// (T.6) the callout is never drawn over it.
#[test]
fn touch_grass_s_window_crops_its_still_about_the_mast() {
    for (hover, wide) in [(None, true), (Some(works::GROUND_TRUTH), false)] {
        let mut ui = at(Page::Works, 1440, 900);
        if let Some(i) = hover { gallery::update(&mut ui.state, Msg::Works(works::WMsg::Hover(i, Some((400.0, 300.0))))); ui.relayout(); ui.tick(2.0); ui.relayout(); }
        let n = nodes(&ui);
        let rect = |k: &str| n.iter().find(|t| t.key == k).map(|t| t.rect).unwrap_or_else(|| panic!("{k}"));
        let win = rect("works/w0");
        let pic = rect("work/touch-grass/picture");
        let mast = (pic[0] + 1123.5 / 1440.0 * pic[2] - win[0]) / win[2];
        let centre = (pic[0] + 950.0 / 1440.0 * pic[2] - win[0]) / win[2];
        eprintln!("touch grass's window, {} px: the mast {:.1} % across, x 950 at {:.1} %", win[2], mast * 100.0, centre * 100.0);
        // Centred on x 950, unless that would show past the picture's right edge: then up to it.
        let right = pic[0] + pic[2] - (win[0] + win[2]);
        if wide { assert!((0.60..=0.75).contains(&mast) && ((centre - 0.5).abs() < 0.01 || right.abs() < 1.0), "{mast} {centre} {right}"); }
        // U.3 item 1: the band's lowest pale stalk (in the still, its pale side's pixels on the lowest row
        // that has any) at least 16 px inside the wide window's left edge.
        if wide {
            let (w, h, px) = works::picture_pixels(works::PIC_STILL).expect("the still");
            let pale = |o: usize| { let (r, g, b) = (px[o] as i32, px[o + 1] as i32, px[o + 2] as i32); r > 235 && g > 225 && (171..215).contains(&b) && r - b > 30 };
            let low = (0..h as usize).rev().find(|&y| (0..w as usize).any(|x| pale((y * w as usize + x) * 4))).expect("a pale stalk");
            let x = (0..w as usize).find(|&x| pale((low * w as usize + x) * 4)).unwrap() as f32 * 1440.0 / w as f32;
            let inside = pic[0] + x / 1440.0 * pic[2] - win[0];
            eprintln!("touch grass's wide window: the lowest pale stalk {inside:.1} px inside its left edge");
            assert!(inside >= 16.0, "{inside}");
        }
        else { assert!(((mast - 0.5) * win[2]).abs() <= 8.0, "{mast}"); }
        assert!(!n.iter().any(|t| t.key.starts_with("works/still/touch-grass/")), "no callout over the still");
    }
}

/// V.3 item 2, T.6 (a): on a phone the 01 band shows the phone page at rest at the band's width (no
/// zoom), its mast 78 % across.
#[test]
fn on_a_phone_touch_grass_s_band_shows_its_phone_page() {
    let ui = at(Page::Works, 390, 772);
    let n = nodes(&ui);
    let rect = |k: &str| n.iter().find(|t| t.key == k).map(|t| t.rect).unwrap_or_else(|| panic!("{k}"));
    let (band, pic) = (rect("works/w0"), rect("work/touch-grass/picture"));
    assert!((pic[0] - band[0]).abs() < 0.5 && (pic[2] - band[2]).abs() < 0.5, "{pic:?} in {band:?}");
    assert!((pic[3] - band[2] * 844.0 / 390.0).abs() < 0.5, "the phone page, 390 x 844, at the band's width: {pic:?}");
    let mast = ((0.78f32 * 390.0).round() + 0.5) / 390.0;
    eprintln!("touch grass's band on a phone: the mast {:.1} % across", mast * 100.0);
    assert!((mast - 0.78).abs() < 0.005);
}

/// T.6 (b): above the horizon the still has no pixel of the ink (#2a2b2d) but the mast's: no callout,
/// no leader.
#[test]
fn touch_grass_s_still_has_no_ink_in_its_sky_but_the_mast() {
    let (w, h, px) = works::picture_pixels(works::PIC_STILL).expect("the still");
    let hz = (0.64 * h as f32) as usize;
    let mast = (1123.5 * w as f32 / 1440.0) as usize;
    let near_mast = |x: usize| (mast.saturating_sub(1)..=mast + 1).contains(&x);
    // Ink at any coverage: a pixel well darker than the sky (whose channels sum to 610 or more) that is
    // not the sock's orange.
    let ink = |x: usize, y: usize| { let o = (y * w as usize + x) * 4; let (r, g, b) = (px[o] as i32, px[o + 1] as i32, px[o + 2] as i32); r + g + b < 580 && r - b < 60 };
    let off: Vec<_> = (0..hz).flat_map(|y| (0..w as usize).map(move |x| (x, y))).filter(|&(x, y)| !near_mast(x) && ink(x, y)).collect();
    let rows = (0..hz).filter(|&y| (0..w as usize).any(|x| near_mast(x) && ink(x, y))).count();
    eprintln!("touch grass's still: {} ink pixels in the sky off the mast, the mast {rows} px tall", off.len());
    assert!(off.is_empty(), "{:?}", &off[..off.len().min(8)]);
    assert!(rows >= 25, "the mast");
}

/// U.2 item 1: a piece is laid out once, at the window's size: W-03's window on the index shows the
/// page's own layout through one scale (its picture the page's, scaled), cropped about x 943 when wide
/// (as far as the picture's edge allows) and the flock's x 1,140 when narrow.
#[test]
fn ground_truth_s_window_shows_the_page_s_layout_through_one_scale() {
    let page = at(Page::Work(works::GROUND_TRUTH as u8), 1440, 900);
    let pp = nodes(&page).into_iter().find(|t| t.key == "work/ground-truth/picture").expect("the page's picture").rect;
    for (hover, wide) in [(Some(works::GROUND_TRUTH), true), (None, false)] {
        let mut ui = at(Page::Works, 1440, 900);
        if let Some(i) = hover { gallery::update(&mut ui.state, Msg::Works(works::WMsg::Hover(i, Some((400.0, 300.0))))); ui.relayout(); ui.tick(2.0); ui.relayout(); }
        let n = nodes(&ui);
        let rect = |k: &str| n.iter().find(|t| t.key == k).map(|t| t.rect).unwrap_or_else(|| panic!("{k}"));
        let (win, pic) = (rect(&format!("works/w{}", works::GROUND_TRUTH)), rect("work/ground-truth/picture"));
        let s = win[3] / 900.0;
        assert!((pic[2] - pp[2] * s).abs() <= 1.0 && (pic[3] - pp[3] * s).abs() <= 1.0, "{pic:?} against the page's {pp:?} at {s}");
        let at = |x: f32| pic[0] + x / 1440.0 * pic[2] - win[0];
        let (c, want) = (at(if wide { 943.0 } else { 1140.0 }), win[2] * 0.5);
        let right = pic[0] + pic[2] - (win[0] + win[2]);
        eprintln!("ground truth's window, {} px at {s:.4}: its crop's centre at {:.1} px, the picture's right edge {right:.1} px past the window's", win[2], c);
        assert!((c - want).abs() <= 1.0 || (wide && right.abs() < 1.0), "{c} {want} {right}");
    }
}

/// U.3 item 2: W-03's narrow window at 1440 × 900 frames what does not move: the sun's centre at least
/// 24 px inside it, and the cloud's box with `BIRD 0.03`, `HORIZON 0.0°` and `▲ 986` each whole and
/// 8 px inside.
#[test]
fn ground_truth_s_narrow_window_frames_the_sun_the_cloud_and_the_far_height() {
    let ui = at(Page::Works, 1440, 900);
    let n = nodes(&ui);
    let rect = |k: &str| n.iter().find(|t| t.key == k).map(|t| t.rect).unwrap_or_else(|| panic!("{k}"));
    let (win, pic) = (rect(&format!("works/w{}", works::GROUND_TRUTH)), rect("work/ground-truth/picture"));
    assert!(win[2] < 300.0, "03 is narrow: {win:?}");
    let (wx0, wy0, wx1, wy1) = (win[0], win[1], win[0] + win[2], win[1] + win[3]);
    let to = |x: f32, y: f32| (pic[0] + x / 1440.0 * pic[2], pic[1] + y / 900.0 * pic[3]);
    let sun = to(980.0, 0.575 * 900.0);
    let sun_in = (sun.0 - wx0).min(wx1 - sun.0).min(sun.1 - wy0).min(wy1 - sun.1);
    let inside = |r: [f32; 4]| (r[0] - wx0).min(wx1 - r[2]).min(r[1] - wy0).min(wy1 - r[3]);
    let (c0, c1) = (to(works::CLOUD.0 as f32 - 48.0, works::CLOUD.1 as f32 - 15.0), to(works::CLOUD.0 as f32 + 52.0, works::CLOUD.1 as f32 + 11.0));
    let cloud_box = [c0.0, c0.1, c1.0, c1.1];
    let mut figures = vec![("sun".to_owned(), sun_in)];
    figures.push(("cloud's box".to_owned(), inside(cloud_box)));
    for (k, t) in [("cloud", "BIRD 0.03"), ("horizon", "HORIZON 0.0°"), ("height2", "▲ 986")] {
        let node = n.iter().find(|x| x.key == format!("work/ground-truth/{k}")).unwrap_or_else(|| panic!("{t} is drawn"));
        assert_eq!(node.text.as_deref(), Some(t));
        figures.push((t.to_owned(), inside(ink(node.rect))));
    }
    eprintln!("ground truth's narrow window: {}", figures.iter().map(|(k, v)| format!("{k} {v:.1} px in")).collect::<Vec<_>>().join(", "));
    assert!(sun_in >= 24.0, "the sun {sun_in} px in");
    for (k, v) in &figures[1..] { assert!(*v >= 8.0, "{k} {v} px in"); }
}

fn en(p: Page, w: u32, h: u32) -> GalleryUi {
    let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
    gallery::update(&mut ui.state, Msg::Locale(amenui::i18n::Locale::En));
    gallery::update(&mut ui.state, Msg::Go(p));
    ui.relayout();
    ui.tick(3.0);
    ui.settle();
    ui
}
fn press(ui: &mut GalleryUi, x: f32, y: f32, t: f32) {
    for down in [true, false] {
        let p = amenui::el::Point { x, y, down, glyph: None, z: 0.0, t };
        gallery::update(&mut ui.state, Msg::Works(works::WMsg::Point(Some(p))));
    }
}
fn callout(ui: &GalleryUi) -> String { nodes(ui).into_iter().find(|n| n.key == "work/nightly/callout").and_then(|n| n.text).expect("the callout") }

/// M.4, W-04: 540 rays hang from one line of 150 glyphs, each ray one instance with a ramp (a tipped one two), all of them
/// one draw, and every one over the picture (no ramp across hues drawn off one, audit P1 RampHue).
#[test]
fn nightly_hangs_540_rays_from_150_glyphs_in_one_draw_over_its_picture() {
    let ui = en(Page::Work(3), 1440, 900);
    let p = &works::PIECES[3];
    assert!(p.page && p.built);
    assert!(ui.problems().is_empty(), "{:?}", ui.problems());
    let sc = ui.scene();
    let ramp = |i: &amenui::scene::Instance| i.paint[3] == amenui::scene::SHAPE && i.uv[2] <= -0.5;
    let rays = sc.instances.iter().filter(|i| ramp(i)).count();
    let draws = sc.batches.iter().filter(|b| (b.start..b.end).any(|k| ramp(&sc.instances[k as usize]))).count();
    // A tipped ray is two strokes, meeting where its hue starts to turn (W.1 item 2).
    assert!((544..=560).contains(&rays) && draws == 1, "{rays} strokes for 540 rays in {draws} draws");
    let n = nodes(&ui);
    let log = n.iter().find(|n| n.key == "work/nightly/log").and_then(|n| n.text.clone()).expect("the line of type");
    assert_eq!(log.chars().count(), 150);
    assert!(log.starts_with("nightly  ok  ok  ok  build 0412  ok  passing"), "{log}");
    assert!(n.iter().any(|n| n.text.as_deref() == Some("W-04 · 23:10 · 100 GLYPHS, 540 RAYS · PRESS TO RUN THE NEXT STEP")));
    assert_eq!(callout(&ui), "KP 5   STEP 3/7");
    eprintln!("nightly: {} instances at 1440 x 900", sc.instances.len());
    // On a phone too, and in the index's windows, the picture under every ray drawn.
    for (p, w, h) in [(Page::Work(3), 390, 844), (Page::Works, 1440, 900), (Page::Works, 390, 844)] {
        let ui = en(p, w, h);
        assert!(ui.problems().is_empty(), "{p:?} {w}: {:?}", ui.problems());
    }
}

fn log_text(ui: &GalleryUi) -> String { nodes(ui).into_iter().find(|n| n.key == "work/nightly/log").and_then(|n| n.text).expect("the line") }

/// A press runs the next step: the callout counts, the reflection shifts once; after step 7 the
/// curtain flares and the build passes: the callout says PASSING and the build number, on the line,
/// rises by one (V.2 i); the press after that is step 1 of the new build.
#[test]
fn a_press_on_nightly_runs_the_next_step_and_after_seven_the_build_passes() {
    let mut ui = en(Page::Work(3), 1440, 900);
    let mut t = 3.0f32;
    ui.tick(t);
    assert!(log_text(&ui).contains("build 0412"));
    for step in 4..=7 {
        press(&mut ui, 700.0, 300.0, t);
        ui.relayout();
        assert_eq!(callout(&ui), format!("KP 5   STEP {step}/7"));
        assert_eq!(ui.state.works.night.shift, step - 3);
        assert!(ui.state.works.night.pulse.is_some());
        t += 1.5; ui.tick(t);
    }
    let flare = ui.state.works.night.flare;
    assert!(flare > 0.0);
    while t < flare + 1.0 { t += 1.0 / 30.0; ui.tick(t); }
    ui.relayout();
    assert_eq!(callout(&ui), "KP 5   PASSING");
    let line = log_text(&ui);
    assert!(line.contains("build 0413") && !line.contains("build 0412"), "{line}");
    press(&mut ui, 700.0, 300.0, t);
    ui.relayout();
    assert_eq!(callout(&ui), "KP 5   STEP 1/7");
    assert!(log_text(&ui).contains("build 0413"));
}

/// V.2 h: with reduced motion a press counts the step, recomposes the callout and shifts the reflection
/// once, with no pulse and no flare; after step 7 the build passes at once and the new curtain comes in
/// by opacity over 120 ms.
#[test]
fn with_reduced_motion_a_press_on_nightly_counts_and_runs_nothing() {
    let mut ui = en(Page::Work(3), 1440, 900);
    ui.state.body.reduced = true;
    let mut t = 3.0f32;
    for step in 4..=6 {
        press(&mut ui, 700.0, 300.0, t);
        ui.relayout();
        assert_eq!(callout(&ui), format!("KP 5   STEP {step}/7"));
        let n = &ui.state.works.night;
        assert!(n.pulse.is_none() && n.flare < 0.0, "no pulse, no flare");
        assert_eq!(n.shift, step - 3);
        t += 1.0; ui.tick(t);
    }
    press(&mut ui, 700.0, 300.0, t);
    ui.relayout();
    assert_eq!(callout(&ui), "KP 5   PASSING");
    assert!(log_text(&ui).contains("build 0413"));
    let n = nodes(&ui);
    assert!(n.iter().any(|n| n.key == "work/nightly/rays/old"), "the old curtain goes by opacity");
    let (frames, _) = grass_frames(&mut ui, t, t + 0.5);
    ui.relayout();
    assert!(nodes(&ui).iter().all(|n| n.key != "work/nightly/rays/old"), "and is gone after 120 ms");
    assert!((3..=10).contains(&frames), "{frames} frames for the 120 ms");
}

/// V.2 e, j and W.1 items 5, 6: the small line states the glyphs drawn (the line's cells less its spaces)
/// and the rays as built; under 1440 wide the line keeps the pitch of 1440 from 6 % to 94 %, cut on a
/// word, with rays 2.35 px apart, and the light and its whole callout are 8 px inside the window. On a
/// phone the horizon is 74 % down, clear of the title block.
#[test]
fn nightly_states_its_counts_as_built_and_has_a_phone_layout() {
    for (w, h) in [(1440u32, 900u32), (1024, 768), (390, 844)] {
        let ui = en(Page::Work(3), w, h);
        let n = nodes(&ui);
        let line = log_text(&ui);
        let glyphs = line.chars().filter(|&c| c != ' ').count();
        let l = website::gallery::nightly::lay_for(w as f32, h as f32);
        let small = n.iter().filter_map(|n| n.text.clone()).find(|t| t.starts_with("W-04 ·")).expect("the small line");
        assert_eq!(small, format!("W-04 · 23:10 · {glyphs} GLYPHS, {} RAYS · PRESS TO RUN THE NEXT STEP", l.rays()));
        let call = n.iter().find(|n| n.key == "work/nightly/callout").expect("the callout").rect;
        let light = website::gallery::nightly::light_at(&l);
        let inside = |r: [f32; 4]| r[0].min(r[1]).min(w as f32 - r[2]).min(h as f32 - r[3]);
        let (ci, li) = (inside([call[0], call[1], call[0] + call[2], call[1] + call[3]]), inside(light));
        let places = website::gallery::nightly::places(&l);
        let (gx0, gx1) = places.iter().fold((f32::MAX, f32::MIN), |m, p| (m.0.min(p.0 - 3.0), m.1.max(p.0 + 3.0)));
        eprintln!("nightly at {w}: {} cells, {glyphs} glyphs, {} rays; the callout {ci:.0} px inside, the light {li:.0}; the glyphs from {:.1} % to {:.1} %", line.chars().count(), l.rays(), gx0 / w as f32 * 100.0, gx1 / w as f32 * 100.0);
        assert!(ci >= 8.0 && li >= 8.0);
        assert!(gx0 >= 0.06 * w as f32 && gx1 <= 0.94 * w as f32);
        if w == 1440 { assert_eq!((line.chars().count(), l.rays()), (150, 540)); } else {
            let rays = if w == 1024 { 379..=387 } else { 140..=150 };
            assert!(rays.contains(&l.rays()), "{}", l.rays());
            assert!(!line.ends_with(' ') && line.chars().count() < 150);
        }
        if w == 390 {
            // W.1 item 5: the horizon 74 % down; the horizon, the light, its leader and the callout 24 px
            // from the title and the number, 8 px from the small line.
            let hz = l.hz();
            assert!((623.0..=627.0).contains(&hz), "the horizon at {hz}");
            // Each by its ink: the mono's 5 px and the serif's 12 px under its box's top.
            let rect = |k: &str| { let r = n.iter().find(|n| n.key == k).expect(k).rect; let top = if k.ends_with("title") { 12.0 } else { 5.0 }; [r[0], r[1] + top, r[0] + r[2], r[1] + r[3]] };
            let gap = |a: [f32; 4], b: [f32; 4]| (b[0] - a[2]).max(a[0] - b[2]).max(b[1] - a[3]).max(a[1] - b[3]);
            let lead = [call[0].min(light[0]), call[1], light[2].max(call[0] + call[2]), light[3]];
            let marks = [[0.0, hz, w as f32, hz + 1.0], light, lead, [call[0], call[1], call[0] + call[2], call[1] + call[3]]];
            for (k, clear) in [("work/nightly/n", 24.0), ("work/nightly/title", 24.0), ("work/nightly/micro", 8.0)] {
                let near = marks.iter().map(|&m| gap(m, rect(k))).fold(f32::MAX, f32::min);
                eprintln!("nightly at 390: {k} {near:.0} px from the horizon, the light and the callout");
                assert!(near >= clear, "{k} {near}");
            }
        }
    }
}

/// It moves all the time while it is awake, at 30 frames a second; held, resting or with reduced
/// motion it is the rest pose and draws nothing.
#[test]
fn nightly_moves_at_thirty_frames_a_second_and_is_still_with_reduced_motion() {
    let mut ui = en(Page::Work(3), 1440, 900);
    ui.ambient_fps(works::ambient_fps(ui.state.page));
    // Past the chrome's fade (3 s without input), then two seconds.
    grass_frames(&mut ui, 3.0, 4.5);
    let (frames, _) = grass_frames(&mut ui, 4.5, 6.5);
    assert!((56..=64).contains(&frames), "{frames} frames in 2 s");
    assert!(ui.state.works.night.run > 1.5);
    let mut ui = en(Page::Work(3), 1440, 900);
    ui.state.body.reduced = true;
    let run = ui.state.works.night.run;
    grass_frames(&mut ui, 3.0, 4.5);
    let (frames, _) = grass_frames(&mut ui, 4.5, 6.5);
    assert_eq!(frames, 0, "frames with reduced motion");
    assert_eq!(ui.state.works.night.run, run);
}

/// A label in a window drawn smaller keeps its own size (`El::own_size`), at a device pixel ratio of 2
/// too: W-03's `BIRD 0.94` in its wide window is 69 px wide, as on the page.
#[test]
fn ground_truth_s_labels_keep_their_size_in_a_window_at_2x() {
    let mut ui = gallery::new_ui(fonts(), 1440, 900, 2.0);
    gallery::update(&mut ui.state, Msg::Locale(amenui::i18n::Locale::En));
    gallery::update(&mut ui.state, Msg::Go(Page::Works));
    gallery::update(&mut ui.state, Msg::Works(works::WMsg::Hover(works::GROUND_TRUTH, Some((900.0, 300.0)))));
    ui.relayout();
    ui.tick(3.0);
    ui.settle();
    let n = nodes(&ui);
    let bird = n.iter().find(|n| n.text.as_deref() == Some("BIRD 0.94")).expect("BIRD 0.94 in the wide window");
    assert!((bird.rect[2] - 69.0).abs() <= 0.5, "{:?}", bird.rect);
}
