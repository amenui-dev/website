//! Loupe (site-design Z.3, 07): this file set at 2.75 px and kept as one cached picture; the glass sets
//! the lines under it again at reading size. The rim reads the line under the glass's centre; pinned,
//! the glass's text selects and copies as shown; nothing is drawn at rest; the glass costs at most 800
//! instances and 1.0 ms; a fast sweep keeps every frame under 16.7 ms; on the index the cell is the
//! glass alone at 11 px.
use website::gallery::{self, GalleryUi, Msg, Page};
use website::showcase::lab::loupe::{self, Loupe};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }
const K: &str = "lab/loupe";

fn page(w: u32, h: u32, scale: f32) -> (GalleryUi, f32) {
    let i = website::gallery::lab::NAMES.iter().position(|n| *n == "loupe").expect("loupe") as u8;
    let mut ui = gallery::new_ui(fonts(), w, h, scale);
    gallery::update(&mut ui.state, Msg::Go(Page::Experiment(i)));
    ui.state.routed = true;
    ui.relayout();
    let mut t = 0.0;
    rest(&mut ui, &mut t, 60);
    (ui, t)
}
fn rest(ui: &mut GalleryUi, t: &mut f32, n: usize) { for _ in 0..n { *t += 1.0 / 60.0; ui.tick(*t); ui.frame_drawn(); } }
fn rect(ui: &GalleryUi, key: &str) -> amenui::ui::Rect { ui.node(key).unwrap_or_else(|| panic!("{key}")).rect }
fn text_of(ui: &GalleryUi, key: &str) -> Option<String> {
    fn find(n: &amenui::tree::TreeNode, key: &str) -> Option<String> { if n.key == key { return n.text.clone(); } n.children.iter().find_map(|c| find(c, key)) }
    find(&ui.export_tree(), key)
}
/// The piece as the page shows it.
fn shown(ui: &GalleryUi) -> Loupe {
    // The stage the page gives the piece (`lab::on_page`): its box less the cell's padding, 4 : 3.
    let w = rect(ui, &format!("{K}/stage")).w;
    ui.state.lab.loupe.at_size((w, (w * amenui::experiment::STAGE).floor()))
}
/// Move the pointer to (x, y) on the stage over `n` frames.
fn glide(ui: &mut GalleryUi, t: &mut f32, from: (f32, f32), to: (f32, f32), n: usize) {
    for f in 1..=n {
        let e = f as f32 / n as f32;
        *t += 1.0 / 60.0;
        ui.event_time(*t);
        ui.pointer_move(from.0 + (to.0 - from.0) * e, from.1 + (to.1 - from.1) * e);
        ui.tick(*t);
        ui.frame_drawn();
    }
}

#[test]
fn the_page_holds_this_file_and_the_rim_reads_the_line_under_the_centre() {
    let (mut ui, mut t) = page(1440, 900, 1.0);
    let s = rect(&ui, &format!("{K}/stage"));
    let l = shown(&ui);
    let sheet = l.sheet();
    let glyphs: usize = (0..sheet.cols).filter_map(|c| text_of(&ui, &format!("{K}/sheet/{c}"))).map(|t| t.chars().filter(|c| !c.is_whitespace()).count()).sum();
    eprintln!("loupe: {} columns of {} rows, {} characters a row, {glyphs} glyphs on the page", sheet.cols, sheet.rows, sheet.chars);
    assert!(glyphs >= 18_000, "{glyphs} glyphs on the page");
    // Over three places, at rest: the rim's line is the source line of the row under the centre.
    for (fx, fy) in [(0.3, 0.4), (0.62, 0.55), (0.85, 0.3)] {
        let to = (s.x + s.w * fx, s.y + s.h * fy);
        glide(&mut ui, &mut t, (s.x + s.w * 0.5, s.y + s.h * 0.5), to, 20);
        rest(&mut ui, &mut t, 60);
        let l = shown(&ui);
        let (gx, gy) = l.at;
        let sheet = l.sheet();
        let c = (((gx - loupe::PAD) / (sheet.col_w + loupe::GAP)).floor() as usize).min(sheet.cols - 1);
        let row = c * sheet.rows + ((gy - loupe::TOP) / loupe::LINE).floor() as usize;
        let rows = l.rows(sheet.chars);
        let line = rows[row].0;
        let rim = text_of(&ui, &format!("{K}/rim")).expect("the rim's reading");
        eprintln!("loupe at ({gx:.0}, {gy:.0}): {rim}; row {row} is line {line}: {:?}", rows[row].2);
        let num = |after: &str| rim.split(after).nth(1).and_then(|r| r.split_whitespace().next()).and_then(|n| n.parse::<usize>().ok());
        assert_eq!(num("LINE"), Some(line), "the rim reads {rim:?}, the centre is over line {line}");
        let i = (((gx - loupe::PAD - c as f32 * (sheet.col_w + loupe::GAP)) / (loupe::SMALL * loupe::ADV)).floor() as usize).min(sheet.chars - 1).min(rows[row].2.chars().count().max(1) - 1);
        let col = rows[row].1 + i;
        assert_eq!(num("COL"), Some(col), "the rim reads {rim:?}, the centre is over column {col}");
    }
}

#[test]
fn pinned_the_glass_s_text_copies_as_shown() {
    let (mut ui, mut t) = page(1440, 900, 1.0);
    let s = rect(&ui, &format!("{K}/stage"));
    let c = (s.x + s.w * 0.4, s.y + s.h * 0.45);
    glide(&mut ui, &mut t, (s.x + s.w * 0.5, s.y + s.h * 0.5), c, 20);
    rest(&mut ui, &mut t, 60);
    // A press without a drag pins it.
    ui.press(c.0, c.1);
    rest(&mut ui, &mut t, 1);
    ui.release(c.0, c.1);
    rest(&mut ui, &mut t, 2);
    assert!(ui.state.lab.loupe.pinned, "a press did not pin the glass");
    let lines = shown(&ui).in_glass();
    let n = lines.len();
    assert!(n > 2, "{n} lines in the glass");
    let (a, b) = (rect(&ui, &format!("{K}/glass/line/0")), rect(&ui, &format!("{K}/glass/line/{}", n - 1)));
    // On the first line's first character, at its foot: the part of it inside the circle.
    ui.press(a.x + 2.0, a.y + a.h * 0.9);
    rest(&mut ui, &mut t, 1);
    ui.pointer_move(b.x + b.w + 40.0, b.y + b.h * 0.5);
    rest(&mut ui, &mut t, 1);
    ui.release(b.x + b.w + 40.0, b.y + b.h * 0.5);
    rest(&mut ui, &mut t, 2);
    let want: Vec<&str> = lines.iter().map(|l| l.3.as_str()).collect();
    let copied = ui.copy().expect("a selection");
    assert_eq!(copied, want.join("\n"), "the copy is not the text in the glass");
    assert!(ui.state.lab.loupe.pinned, "a drag freed the glass");
    // A second press, on the page beside the glass, frees it.
    let r = ui.state.lab.loupe.p.get(1);
    let c = (c.0 + r + 30.0, c.1);
    ui.press(c.0, c.1);
    rest(&mut ui, &mut t, 1);
    ui.release(c.0, c.1);
    rest(&mut ui, &mut t, 2);
    assert!(!ui.state.lab.loupe.pinned, "a second press did not free the glass");
}

#[test]
fn the_glass_costs_at_most_800_instances_and_nothing_is_drawn_at_rest() {
    let (mut ui, mut t) = page(1440, 900, 1.0);
    let s = rect(&ui, &format!("{K}/stage"));
    let mut most = 0;
    // At the default radius (the lesser of 132 and 30 % of the stage), and at the range's top, set by
    // hand: there the glass holds what the page has within 240 / zoom px, all of it drawn.
    for (z, top) in [(4.0f32, false), (8.0, false), (12.0, false), (4.0, true), (8.0, true), (12.0, true)] {
        ui.state.lab.loupe.p.v[0] = z;
        if top { (ui.state.lab.loupe.p.v[1], ui.state.lab.loupe.sized) = (240.0, true); }
        ui.relayout();
        glide(&mut ui, &mut t, (s.x + s.w * 0.5, s.y + s.h * 0.5), (s.x + s.w * 0.45, s.y + s.h * 0.5), 10);
        rest(&mut ui, &mut t, 60);
        let j = ui.nodes().iter().position(|n| n.key == format!("{K}/glass")).expect("the glass");
        let inside = |k: usize| { let mut p = Some(k); while let Some(q) = p { if q == j { return true; } p = ui.nodes()[q].parent; } false };
        let n = ui.owners().iter().filter(|o| inside(o.0)).count();
        let l = shown(&ui);
        let t0 = std::time::Instant::now();
        for _ in 0..20 { std::hint::black_box(l.in_glass()); }
        let ms = t0.elapsed().as_secs_f64() * 1000.0 / 20.0;
        let r = ui.state.lab.loupe.p.get(1);
        let glyphs: usize = l.in_glass().iter().map(|t| t.3.chars().filter(|c| !c.is_whitespace()).count()).sum();
        eprintln!("loupe ×{z} radius {r}: {n} instances in the glass ({glyphs} glyphs), its text found in {ms:.3} ms");
        most = most.max(n);
    }
    assert!(most <= 800, "{most} instances in the glass");
    let mut frames = 0;
    for _ in 0..300 { t += 1.0 / 60.0; ui.tick(t); if ui.needs_frame(t) { frames += 1; } ui.frame_drawn(); }
    assert_eq!(frames, 0, "{frames} frames in 5 s at rest");
}

#[test]
fn the_distance_bands_are_traced_within_the_budget() {
    let (mut ui, _) = page(1440, 900, 1.0);
    ui.state.lab.loupe.mode = 2;
    let l = shown(&ui);
    let text = l.in_glass();
    let best = (0..10).map(|_| {
        let t0 = std::time::Instant::now();
        for _ in 0..5 { std::hint::black_box(l.bands(&text)); }
        t0.elapsed().as_secs_f64() * 1000.0 / 5.0
    }).fold(f64::INFINITY, f64::min);
    let n = l.bands(&text).len();
    eprintln!("loupe distance at ×8, radius 132: {n} band lines in {best:.3} ms");
    assert!(n > 0, "no bands");
    if !cfg!(debug_assertions) { assert!(best <= 1.0, "the bands took {best:.3} ms"); }
}

#[test]
fn on_the_index_the_cell_is_the_glass_alone_and_hovering_it_views_nothing() {
    let mut ui = gallery::new_ui(fonts(), 1440, 900, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Lab));
    ui.state.routed = true;
    ui.relayout();
    let mut t = 0.0;
    rest(&mut ui, &mut t, 120);
    assert!(!ui.nodes().iter().any(|n| n.key.starts_with("labi/") && n.key.contains("loupe") && n.key.contains("/sheet")), "the small page is drawn on the index");
    let small = ui.nodes().iter().filter(|n| n.key.contains("loupe") && n.key.contains("/glass/line/")).filter_map(|n| match &n.content { amenui::el::Content::Text { size, .. } => Some(*size), _ => None }).fold(f32::INFINITY, f32::min);
    assert!(small >= 11.0, "text in the index cell at {small} px");
    let r = rect(&ui, "labi/lab-loupe");
    let c0 = ui.perf().counters;
    for f in 0..60 {
        t += 1.0 / 60.0;
        ui.event_time(t);
        ui.pointer_move(r.x + 10.0 + f as f32 * 4.0, r.y + r.h * 0.5);
        ui.tick(t);
        ui.frame_drawn();
    }
    let c = &ui.perf().counters;
    eprintln!("loupe on the index, 60 moves over it: {} views, {} layouts", c.views - c0.views, c.layouts - c0.layouts);
    assert_eq!((c.views - c0.views, c.layouts - c0.layouts), (0, 0));
}

/// Y.5's line for Loupe's gesture, and Z.3's sweep: at 1440 × 900, scale 2, the glass swept 1,000 px
/// in 1 s and back; every frame shows it moved, no frame's CPU over 16.7 ms; at rest no frame in 5 s.
#[test]
fn loupe_y5_line() { y5(None); }

/// Z.8: the budget is frame time, so the sweep holds at the most the glass draws, × 4 and radius 240.
#[test]
fn loupe_y5_line_at_x4_radius_240() { y5(Some((4.0, 240.0))); }

fn y5(at: Option<(f32, f32)>) {
    let (mut ui, mut t) = page(2880, 1800, 2.0);
    if let Some((z, r)) = at {
        let l = &mut ui.state.lab.loupe;
        (l.p.v[0], l.p.v[1], l.sized) = (z, r, true);
        ui.relayout();
        // What the change set going settles before the rest is counted.
        for _ in 0..120 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
    }
    let (z, r) = (ui.state.lab.loupe.p.get(0), ui.state.lab.loupe.p.get(1));
    let mut idle = 0;
    for _ in 0..300 { t += 1.0 / 60.0; ui.tick(t); if ui.needs_frame(t) { idle += 1; } ui.frame_drawn(); }
    let s = rect(&ui, &format!("{K}/stage"));
    let (y, x0) = (s.y + s.h * 0.5, s.x + 20.0);
    // 1,000 px in 1 s and back (Z.7): across the stage and back again, as often as that takes.
    let span = s.w - 40.0;
    let along = |d: f32| { let m = d.rem_euclid(2.0 * span); if m <= span { m } else { 2.0 * span - m } };
    let c0 = ui.perf().counters;
    let (mut ms, mut late, mut last) = (vec![], 0, ui.state.lab.loupe.at);
    for f in 1..=120 {
        let e = f as f32 / 60.0;
        let x = x0 + along(1000.0 * if e <= 1.0 { e } else { 2.0 - e });
        t += 1.0 / 60.0;
        let t0 = std::time::Instant::now();
        ui.event_time(t);
        ui.pointer_move(x, y);
        ui.tick(t);
        ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        ui.frame_drawn();
        let now = ui.state.lab.loupe.at;
        if (now.0 - last.0).hypot(now.1 - last.1) < 0.01 { late += 1; }
        last = now;
    }
    let c = &ui.perf().counters;
    let (views, layouts) = (c.views - c0.views, c.layouts - c0.layouts);
    ms.sort_by(f64::total_cmp);
    let q = |p: f64| ms[((ms.len() - 1) as f64 * p).round() as usize];
    eprintln!("Y.5 loupe 1440x900@2 ×{z} radius {r} (native CPU, no GPU): sweep 1,000 px in 1 s and back over {span:.0} px: median {:.2} ms, p95 {:.2}, max {:.2}, 120 frames, {late} answered late, {views} views, {layouts} layouts; at rest {idle} frames in 5 s", q(0.5), q(0.95), q(1.0));
    assert_eq!(late, 0, "{late} frames did not show the glass moved");
    assert_eq!(idle, 0, "{idle} frames at rest");
    assert_eq!(layouts, 0, "{layouts} layout passes during the sweep");
    if !cfg!(debug_assertions) {
        assert!(q(0.95) < 8.3, "p95 {:.2} ms", q(0.95));
        assert!(q(1.0) <= 16.7, "a frame took {:.2} ms", q(1.0));
    }
}

/// Review 2: over the plates' limit the page has no kept image and the glass is its rim alone: no
/// fill, no lines, the page seen through it at its own size and the rim reading ×1, with no warning.
/// Back under the limit the glass fills again at once.
#[test]
fn over_the_kept_image_limit_the_glass_is_its_rim_alone_and_reads_x1() {
    use amenui::experiment::Experiment;
    fn find<'a, M>(e: &'a amenui::el::El<M>, key: &str) -> Option<&'a amenui::el::El<M>> {
        if e.props.key == key { return Some(e); }
        e.children.iter().find_map(|c| find(c, key))
    }
    let lines = |ui: &GalleryUi| ui.nodes().iter().filter(|n| n.key.starts_with(&format!("{K}/glass/line/"))).count();
    // The piece's texts other than the page and the glass's lines: no warning comes among them.
    let text_keys = |ui: &GalleryUi| -> Vec<String> {
        ui.nodes().iter().filter(|n| n.key.starts_with(&format!("{K}/")) && !n.key.starts_with(&format!("{K}/sheet")) && !n.key.starts_with(&format!("{K}/glass/line/")))
            .filter(|n| matches!(n.content, amenui::el::Content::Text { .. })).map(|n| n.key.clone()).collect()
    };
    let (mut ui, mut t) = page(1440, 900, 1.0);
    let sheet = format!("{K}/sheet");
    assert_eq!(ui.kept_image(&sheet), Some(true), "the page is a cached element with its kept image");
    let filled = (lines(&ui), text_of(&ui, &format!("{K}/rim")).expect("rim"), text_keys(&ui));
    assert!(filled.0 > 0 && filled.1.starts_with("×8 "), "{filled:?}");

    // Over the limit: what `Ui::on_kept` tells the state.
    gallery::kept(&mut ui.state, &sheet, false);
    ui.relayout();
    rest(&mut ui, &mut t, 2);
    assert_eq!(lines(&ui), 0, "the glass still sets lines it cannot show over");
    let rim = text_of(&ui, &format!("{K}/rim")).expect("rim");
    assert!(rim.starts_with("×1 · LINE"), "the rim reads {rim:?}");
    let l = shown(&ui);
    let v = l.view(K);
    let glass = find(&v, &format!("{K}/glass")).expect("glass");
    assert!(glass.props.fill.is_none(), "the glass is filled over the limit");
    assert!(glass.props.border.is_some(), "the glass has no rim");
    assert_eq!(text_keys(&ui), filled.2, "the glass says more than its rim over the limit");

    // Back under: filled again by a cut, the zoom on the rim.
    gallery::kept(&mut ui.state, &sheet, true);
    ui.relayout();
    rest(&mut ui, &mut t, 1);
    assert_eq!((lines(&ui), text_of(&ui, &format!("{K}/rim")).expect("rim"), text_keys(&ui)), filled, "the glass did not come back as it was");
    assert!(find(&shown(&ui).view(K), &format!("{K}/glass")).expect("glass").props.fill.is_some());
}
