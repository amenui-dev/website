//! Weld (site-design Z.3, 02.4): the outline is the smooth union of the chips' distance fields; two
//! chips 2k or more apart keep a lone chip's outline, closer than k / 2 they share one; joining makes a
//! group in the tree and leaves the labels' glyphs as they were; nothing is drawn at rest.
use website::gallery::{self, GalleryUi, Msg, Page};
use website::showcase::lab::weld::{self, rounded};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }

/// The union's outline traced as the piece traces it: on a 2 px grid over `w` × `h`.
fn outline(boxes: &[weld::Box2], r: f32, k: f32, _w: f32, _h: f32) -> Vec<Vec<(f32, f32)>> {
    weld::trace(boxes, r, k, 2.0 * k + 4.0 + 12.0, &[0.0]).into_iter().map(|(_, l)| l).collect()
}

#[test]
fn chips_two_k_apart_keep_their_own_outline_and_half_k_apart_share_one() {
    let (k, r) = (18.0, 6.0);
    let half = (weld::chip_w(1) * 0.5, weld::CHIP_H * 0.5);
    for gap in [2.0 * k, 2.0 * k + 7.0, 60.0] {
        let a = ((100.0, 100.0), half);
        let b = ((100.0 + 2.0 * half.0 + gap, 100.0), half);
        let lines = outline(&[a, b], r, k, 400.0, 200.0);
        assert_eq!(lines.len(), 2, "gap {gap}: {} outlines", lines.len());
        // Every point of the traced outline is on one lone chip's outline, within 1 px.
        let worst = lines.iter().flatten().map(|&(x, y)| rounded(x, y, a.0, a.1, r).abs().min(rounded(x, y, b.0, b.1, r).abs())).fold(0.0f32, f32::max);
        eprintln!("gap {gap} px: farthest outline point from a lone chip's outline {worst:.3} px");
        assert!(worst <= 1.0, "gap {gap}: {worst} px off a lone chip's outline");
    }
    for gap in [0.5 * k, 0.25 * k, 0.0] {
        let a = ((100.0, 100.0), half);
        let b = ((100.0 + 2.0 * half.0 + gap, 100.0), half);
        let lines = outline(&[a, b], r, k, 400.0, 200.0);
        assert_eq!(lines.len(), 1, "gap {gap}: {} outlines, not one round both", lines.len());
        let l = &lines[0];
        assert_eq!(l.first(), l.last(), "gap {gap}: the outline is not closed");
    }
}

fn page() -> (GalleryUi, f32) {
    let i = website::gallery::lab::NAMES.iter().position(|n| *n == "weld").expect("weld") as u8;
    let mut ui = gallery::new_ui(fonts(), 1440, 900, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Experiment(i)));
    ui.state.routed = true;
    ui.relayout();
    let mut t = 0.0;
    for _ in 0..60 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
    (ui, t)
}
fn rest(ui: &mut GalleryUi, t: &mut f32, n: usize) { for _ in 0..n { *t += 1.0 / 60.0; ui.tick(*t); ui.frame_drawn(); } }
fn centre(ui: &GalleryUi, key: &str) -> (f32, f32) { let r = ui.node(key).unwrap_or_else(|| panic!("{key}")).rect; (r.x + r.w * 0.5, r.y + r.h * 0.5) }
fn glyphs(ui: &GalleryUi, key: &str) -> Vec<(f32, f32)> {
    let j = ui.nodes().iter().position(|n| n.key == key).unwrap_or_else(|| panic!("{key}"));
    let insts = &ui.scene().instances;
    ui.owners().iter().enumerate().filter(|(_, o)| o.0 == j).map(|(k, _)| (insts[k].rect[2], insts[k].rect[3])).collect()
}
fn text_of(ui: &GalleryUi, key: &str) -> String {
    fn find(n: &amenui::tree::TreeNode, key: &str) -> Option<String> { if n.key == key { return n.text.clone(); } n.children.iter().find_map(|c| find(c, key)) }
    find(&ui.export_tree(), key).unwrap_or_default()
}
/// A drag from `a` to `b` in `n` frames, then the release.
fn drag(ui: &mut GalleryUi, t: &mut f32, a: (f32, f32), b: (f32, f32), n: usize) {
    ui.press(a.0, a.1);
    rest(ui, t, 1);
    for f in 1..=n {
        let e = f as f32 / n as f32;
        *t += 1.0 / 60.0;
        ui.event_time(*t);
        ui.pointer_move(a.0 + (b.0 - a.0) * e, a.1 + (b.1 - a.1) * e);
        ui.tick(*t);
        ui.frame_drawn();
    }
    ui.release(b.0, b.1);
    rest(ui, t, 1);
}

#[test]
fn a_chip_let_go_inside_join_is_one_group_with_its_glyphs_unchanged() {
    let (mut ui, mut t) = page();
    let k = "lab/weld";
    assert_eq!(text_of(&ui, &format!("{k}/count")), "6 CHIPS · 6 GROUPS");
    let before = glyphs(&ui, &format!("{k}/chip/sdf/label"));
    assert!(!before.is_empty());
    // `sdf` to just left of `rust`, 6 px from it (join is 12).
    let (a, b) = (centre(&ui, &format!("{k}/chip/sdf")), ui.node(&format!("{k}/chip/rust")).expect("rust").rect);
    let sdf_w = ui.node(&format!("{k}/chip/sdf")).expect("sdf").rect.w;
    let to = (b.x - 6.0 - sdf_w * 0.5, b.y + b.h * 0.5);
    drag(&mut ui, &mut t, a, to, 30);
    rest(&mut ui, &mut t, 40);
    assert_eq!(text_of(&ui, &format!("{k}/count")), "6 CHIPS · 5 GROUPS");
    // In the tree: one group holding both.
    fn group_of<'a>(n: &'a amenui::tree::TreeNode, want: &[&str]) -> Option<&'a amenui::tree::TreeNode> {
        fn has(n: &amenui::tree::TreeNode, k: &str) -> bool { n.key == k || n.children.iter().any(|c| has(c, k)) }
        if n.key.contains("/group/") && want.iter().all(|w| has(n, w)) { return Some(n); }
        n.children.iter().find_map(|c| group_of(c, want))
    }
    let tree = ui.export_tree();
    let g = group_of(&tree, &[&format!("{k}/chip/sdf"), &format!("{k}/chip/rust")]).expect("a group holding sdf and rust");
    assert!(g.children.len() == 2, "the group holds {} chips", g.children.len());
    // The labels' glyphs are the same size joined as apart.
    let after = glyphs(&ui, &format!("{k}/chip/sdf/label"));
    assert_eq!(before.len(), after.len());
    for (p, q) in before.iter().zip(&after) { assert!((p.0 - q.0).abs() < 0.01 && (p.1 - q.1).abs() < 0.01, "{p:?} → {q:?}"); }
    // Side by side, abutting: sdf's right edge on rust's left.
    let (s, r) = (ui.node(&format!("{k}/chip/sdf")).expect("sdf").rect, ui.node(&format!("{k}/chip/rust")).expect("rust").rect);
    assert!((s.x + s.w - r.x).abs() < 0.6 && (s.y - r.y).abs() < 0.6, "sdf {s:?} rust {r:?}");
    // Pulled away fast (40 px a frame), a member tears off: it is alone again and the count goes back up.
    let a = centre(&ui, &format!("{k}/chip/sdf"));
    ui.press(a.0, a.1);
    rest(&mut ui, &mut t, 1);
    for f in 1..=4 {
        t += 1.0 / 60.0;
        ui.event_time(t);
        ui.pointer_move(a.0 - 40.0 * f as f32, a.1);
        ui.tick(t);
        ui.frame_drawn();
    }
    ui.release(a.0 - 160.0, a.1);
    rest(&mut ui, &mut t, 1);
    rest(&mut ui, &mut t, 40);
    assert_eq!(text_of(&ui, &format!("{k}/count")), "6 CHIPS · 6 GROUPS");
}

#[test]
fn hovering_weld_on_the_index_views_and_lays_out_nothing() {
    let mut ui = gallery::new_ui(fonts(), 1440, 900, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Lab));
    ui.state.routed = true;
    ui.relayout();
    let mut t = 0.0;
    rest(&mut ui, &mut t, 120);
    let r = ui.node("labi/lab-weld").expect("Weld's cell").rect;
    let c0 = ui.perf().counters;
    let mut frames = 0;
    for f in 0..60 {
        t += 1.0 / 60.0;
        ui.event_time(t);
        ui.pointer_move(r.x + 10.0 + f as f32 * 4.0, r.y + r.h * 0.5);
        ui.tick(t);
        if ui.needs_frame(t) { frames += 1; }
        ui.frame_drawn();
    }
    let c = &ui.perf().counters;
    eprintln!("weld on the index, 60 moves over it: {} views, {} layouts, {} restages, {frames} frames asked", c.views - c0.views, c.layouts - c0.layouts, c.restages - c0.restages);
    assert_eq!((c.views - c0.views, c.layouts - c0.layouts), (0, 0));
}

#[test]
fn weld_draws_nothing_at_rest_and_a_press_turns_a_chip_on() {
    let (mut ui, mut t) = page();
    rest(&mut ui, &mut t, 60);
    let mut frames = 0;
    for _ in 0..300 { t += 1.0 / 60.0; ui.tick(t); if ui.needs_frame(t) { frames += 1; } ui.frame_drawn(); }
    assert_eq!(frames, 0, "{frames} frames in 5 s at rest");
    let c = centre(&ui, "lab/weld/chip/field");
    ui.press(c.0, c.1);
    rest(&mut ui, &mut t, 1);
    ui.release(c.0, c.1);
    rest(&mut ui, &mut t, 2);
    let n = ui.nodes().iter().find(|n| n.key == "lab/weld/chip/field").expect("field");
    assert_eq!(n.access.as_ref().and_then(|a| a.checked), Some(true), "a press without a drag did not turn the chip on");
}

#[test]
fn the_union_is_traced_within_its_budget() {
    // Six chips where they start on the page's stage at 1440 (694 × 520), at the defaults: the best of
    // ten runs of 20, so a busy machine does not decide it.
    let w = weld::Weld::default().at_size((694.0, 520.0));
    let boxes: Vec<_> = (0..6).map(|i| w.bx(i)).collect();
    let (k, bands) = (18.0, 2);
    let reach = weld::BANDS[bands - 1] + 2.0 * k + 2.0 * weld::STEP;
    let best = (0..10).map(|_| {
        let t0 = std::time::Instant::now();
        for _ in 0..20 { std::hint::black_box(weld::trace(&boxes, 6.0, k, reach, &[6.0, 12.0, 0.0])); }
        t0.elapsed().as_secs_f64() * 1000.0 / 20.0
    }).fold(f64::INFINITY, f64::min);
    use amenui::experiment::Experiment;
    let t0 = std::time::Instant::now();
    for _ in 0..20 { std::hint::black_box(w.view("cell")); }
    eprintln!("weld: the union sampled and traced {best:.3} ms; the whole view {:.3} ms", t0.elapsed().as_secs_f64() * 1000.0 / 20.0);
    assert!(best <= 1.0, "the union took {best:.3} ms");
}

/// Y.5's line for Weld's main gesture: at 1440 × 900, scale 2, a chip dragged across the stage for 2 s
/// at 60 Hz. Every frame after a move shows the chip in a new place (row 2); CPU per frame (view,
/// layout, scene) is printed; at rest no frame is asked for in 5 s.
#[test]
fn weld_y5_line() {
    let i = website::gallery::lab::NAMES.iter().position(|n| *n == "weld").expect("weld") as u8;
    let mut ui = gallery::new_ui(fonts(), 2880, 1800, 2.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Experiment(i)));
    ui.state.routed = true;
    ui.relayout();
    let mut t = 0.0;
    rest(&mut ui, &mut t, 120);
    let mut idle = 0;
    for _ in 0..300 { t += 1.0 / 60.0; ui.tick(t); if ui.needs_frame(t) { idle += 1; } ui.frame_drawn(); }
    let a = centre(&ui, "lab/weld/chip/glyph");
    ui.press(a.0, a.1);
    rest(&mut ui, &mut t, 1);
    let c0 = ui.perf().counters;
    let (mut ms, mut late, mut last) = (vec![], 0, a);
    for f in 1..=120 {
        let e = f as f32 / 120.0;
        let p = (a.0 + 420.0 * e, a.1 + 60.0 * (e * std::f32::consts::TAU).sin());
        t += 1.0 / 60.0;
        let t0 = std::time::Instant::now();
        ui.event_time(t);
        ui.pointer_move(p.0, p.1);
        ui.tick(t);
        ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        ui.frame_drawn();
        let now = centre(&ui, "lab/weld/chip/glyph");
        if (now.0 - last.0).hypot(now.1 - last.1) < 0.5 { late += 1; }
        last = now;
    }
    let c = &ui.perf().counters;
    let (views, layouts, restages) = (c.views - c0.views, c.layouts - c0.layouts, c.restages - c0.restages);
    ui.release(last.0, last.1);
    // A chip let go next to nothing rests where it is; scattered, the chips spring home with no input.
    rest(&mut ui, &mut t, 30);
    let s = centre(&ui, "lab/weld/scatter");
    ui.press(s.0, s.1);
    ui.release(s.0, s.1);
    rest(&mut ui, &mut t, 1);
    let c1 = ui.perf().counters;
    rest(&mut ui, &mut t, 40);
    let c = &ui.perf().counters;
    eprintln!("Y.5 weld, the springs after a scatter, 40 frames: {} views, {} layouts, {} restages", c.views - c1.views, c.layouts - c1.layouts, c.restages - c1.restages);
    ms.sort_by(f64::total_cmp);
    let q = |p: f64| ms[((ms.len() - 1) as f64 * p).round() as usize];
    eprintln!("Y.5 weld 1440x900@2 (native CPU, no GPU): drag 2 s: median {:.2} ms, p95 {:.2}, max {:.2}, 120 frames, {late} answered late, {views} views, {layouts} layouts, {restages} restages; at rest {idle} frames in 5 s", q(0.5), q(0.95), q(1.0));
    assert_eq!(late, 0, "{late} frames of the drag did not show the chip moved");
    assert_eq!(idle, 0, "{idle} frames at rest");
}
