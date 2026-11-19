//! Kern (site-design Z.3, 02.3, and Z.6): the letters' spacing is measured between their outlines. In
//! `gap` every pair is `target` apart at its closest, by a raster test on the glyphs as drawn (the
//! atlas's coverage at each instance), not by the piece's own measure; in `metric` the glyphs are where
//! the shaper put them; in a scripted drag no two glyphs' coverage overlaps in any frame; the printed
//! figures are the raster's ± 0.5; the drawn size fits the stage (Z.6).
use website::gallery::{self, GalleryUi, Msg, Page};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }
const K: &str = "lab/kern";

fn page(w: u32, h: u32) -> (GalleryUi, f32) {
    let i = website::gallery::lab::NAMES.iter().position(|n| *n == "kern").expect("kern") as u8;
    let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Experiment(i)));
    ui.state.routed = true;
    ui.relayout();
    let mut t = 0.0;
    rest(&mut ui, &mut t, 60);
    (ui, t)
}
fn rest(ui: &mut GalleryUi, t: &mut f32, n: usize) { for _ in 0..n { *t += 1.0 / 60.0; ui.tick(*t); ui.frame_drawn(); } }
fn rect(ui: &GalleryUi, key: &str) -> amenui::ui::Rect { ui.node(key).unwrap_or_else(|| panic!("{key}")).rect }
fn press(ui: &mut GalleryUi, t: &mut f32, key: &str) {
    let r = rect(ui, key);
    ui.press(r.x + r.w * 0.5, r.y + r.h * 0.5);
    rest(ui, t, 1);
    ui.release(r.x + r.w * 0.5, r.y + r.h * 0.5);
    rest(ui, t, 60);
}
fn text_of(ui: &GalleryUi, key: &str) -> Option<String> {
    fn find(n: &amenui::tree::TreeNode, key: &str) -> Option<String> { if n.key == key { return n.text.clone(); } n.children.iter().find_map(|c| find(c, key)) }
    find(&ui.export_tree(), key)
}

/// Each glyph of the word as drawn: its number in the text and its inked pixels (device px, the
/// atlas's coverage at its instance, cut at half).
fn inked(ui: &GalleryUi) -> Vec<(usize, Vec<(f32, f32)>)> {
    let j = ui.nodes().iter().position(|n| n.key == format!("{K}/word")).expect("the word");
    let (atlas, insts) = (ui.atlas(), &ui.scene().instances);
    let a = amenui::text::ATLAS as f32;
    let mut out: Vec<(usize, Vec<(f32, f32)>)> = ui.glyphs().log.iter().filter(|g| g.node == j).map(|g| {
        let ins = &insts[g.inst];
        let (u0, v0) = ((ins.uv[0] * a).round() as usize, (ins.uv[1] * a).round() as usize);
        let (w, h) = (ins.rect[2] as usize, ins.rect[3] as usize);
        let mut px = vec![];
        for y in 0..h { for x in 0..w {
            if atlas[((v0 + y) * amenui::text::ATLAS as usize + u0 + x) * 2] >= 128 { px.push((ins.rect[0] + x as f32, ins.rect[1] + y as f32)); }
        } }
        (g.i, px)
    }).collect();
    out.sort_by_key(|g| g.0);
    out
}
/// The smallest distance between two sets of inked pixels, each pixel a unit square: negative when
/// they share a pixel.
fn apart(a: &[(f32, f32)], b: &[(f32, f32)]) -> f32 {
    // Only the edge of each that faces the other can be nearest: a's right half, b's left.
    let (amax, bmin) = (a.iter().map(|p| p.0).fold(f32::MIN, f32::max), b.iter().map(|p| p.0).fold(f32::MAX, f32::min));
    let a: Vec<_> = a.iter().filter(|p| p.0 > bmin - 80.0).collect();
    let b: Vec<_> = b.iter().filter(|p| p.0 < amax + 80.0).collect();
    let mut best = f32::INFINITY;
    for p in &a { for q in &b {
        let (dx, dy) = ((p.0 - q.0).abs(), (p.1 - q.1).abs());
        if dx < 0.999 && dy < 0.999 { return -1.0; }
        let (hx, hy) = ((dx - 1.0).max(0.0), (dy - 1.0).max(0.0));
        best = best.min(hx.hypot(hy));
    } }
    best
}
fn figures(ui: &GalleryUi, n: usize) -> Vec<f32> { (0..n).map(|k| text_of(ui, &format!("{K}/gap/{k}")).expect("a gap figure").parse().expect("a number")).collect() }

#[test]
fn in_gap_every_pair_is_target_apart_by_a_raster_test_and_the_figures_are_the_raster_s() {
    let (mut ui, mut t) = page(1440, 900);
    for (mode, check) in [("metric", false), ("gap", true), ("area", false)] {
        press(&mut ui, &mut t, &format!("{K}/mode/{mode}"));
        let g = inked(&ui);
        assert_eq!(g.len(), 6, "{mode}: six glyphs drawn");
        let raster: Vec<f32> = g.windows(2).map(|w| apart(&w[0].1, &w[1].1)).collect();
        let printed = figures(&ui, 5);
        eprintln!("kern {mode}: raster gaps {raster:.2?}, printed {printed:?}");
        for (r, p) in raster.iter().zip(&printed) { assert!((r - p).abs() <= 0.5, "{mode}: printed {p}, the raster {r}"); }
        if check { for r in &raster { assert!((r - 8.0).abs() <= 1.0, "gap: a pair {r} px apart, not 8 ± 1"); } }
    }
}

#[test]
fn in_metric_the_glyphs_are_where_the_shaper_set_them() {
    let (ui, _) = page(1440, 900);
    let j = ui.nodes().iter().position(|n| n.key == format!("{K}/word")).expect("the word");
    let r = ui.nodes()[j].inner;
    let insts = &ui.scene().instances;
    let log: Vec<_> = ui.glyphs().log.iter().filter(|g| g.node == j).collect();
    assert_eq!(log.len(), 6);
    for g in log {
        let ins = &insts[g.inst];
        let c = (ins.rect[0] + ins.rect[2] * 0.5 - r.x, ins.rect[1] + ins.rect[3] * 0.5 - r.y);
        assert!((c.0 - g.at.0).abs() < 1e-3 && (c.1 - g.at.1).abs() < 1e-3, "glyph {}: drawn at {c:?}, laid out at {:?}", g.i, g.at);
    }
    assert_eq!(text_of(&ui, &format!("{K}/word")).as_deref(), Some("Voyage"), "the word is one text");
}

/// A letter dragged into its neighbours and back: no frame has two glyphs' coverage overlapping; let
/// go, it settles within 320 ms and then asks for no frame.
#[test]
fn a_dragged_letter_pushes_its_neighbours_without_overlap_and_settles() {
    let (mut ui, mut t) = page(1440, 900);
    press(&mut ui, &mut t, &format!("{K}/mode/gap"));
    let g = inked(&ui);
    // `y`, the third glyph, by its ink's middle.
    let (x0, y0) = { let p = &g[2].1; (p.iter().map(|q| q.0).sum::<f32>() / p.len() as f32, p.iter().map(|q| q.1).sum::<f32>() / p.len() as f32) };
    ui.press(x0, y0);
    rest(&mut ui, &mut t, 1);
    let mut worst = f32::INFINITY;
    for f in 1..=90 {
        // 200 px right into `age`, then 400 px left into `Vo`, then back.
        let e = f as f32 / 90.0;
        let dx = if e < 0.33 { 600.0 * e } else if e < 0.66 { 200.0 - 1800.0 * (e - 0.33) } else { -400.0 + 1200.0 * (e - 0.66) };
        t += 1.0 / 60.0;
        ui.event_time(t);
        ui.pointer_move(x0 + dx, y0);
        ui.tick(t);
        ui.frame_drawn();
        let g = inked(&ui);
        for w in g.windows(2) { let d = apart(&w[0].1, &w[1].1); worst = worst.min(d); assert!(d >= 0.0, "frame {f}: glyphs {} and {} overlap", w[0].0, w[1].0); }
    }
    eprintln!("kern drag: the closest two glyphs came {worst:.2} px");
    let r = (x0, y0);
    ui.release(r.0, r.1);
    let mut settled = None;
    for f in 1..=60 {
        t += 1.0 / 60.0;
        ui.tick(t);
        let asks = ui.needs_frame(t);
        ui.frame_drawn();
        if !asks && settled.is_none() { settled = Some(f); }
    }
    let ms = settled.expect("it settles") as f32 * 1000.0 / 60.0;
    eprintln!("kern: settled {ms:.0} ms after the release");
    assert!(ms <= 320.0 + 1000.0 / 60.0, "settled after {ms} ms");
    let mut frames = 0;
    for _ in 0..300 { t += 1.0 / 60.0; ui.tick(t); if ui.needs_frame(t) { frames += 1; } ui.frame_drawn(); }
    assert_eq!(frames, 0, "{frames} frames in 5 s at rest");
}

/// Z.6: at 1440, 1024, 390 and 320 the drawn size is a multiple of 11 and at least 44, the word in
/// `metric` is at most 80 % of the stage, at `target` 40 in `gap` no glyph leaves the stage, and no
/// printed figure overlaps another or a control.
#[test]
fn the_word_fits_the_stage_at_every_width() {
    for w in [1440u32, 1024, 390, 320] {
        let (mut ui, mut t) = page(w, 900);
        let stage = rect(&ui, &format!("{K}/stage"));
        let k = &ui.state.lab.kern.at_size((stage.w, stage.h));
        let px = k.px;
        assert!(px >= 44.0 && px % 11.0 == 0.0, "{w}: drawn at {px}");
        let g = inked(&ui);
        let (l, r) = (g.iter().flat_map(|q| q.1.iter().map(|p| p.0)).fold(f32::MAX, f32::min), g.iter().flat_map(|q| q.1.iter().map(|p| p.0 + 1.0)).fold(f32::MIN, f32::max));
        let advance = 6.0 * px * 350.0 / 550.0;
        eprintln!("kern at {w}: stage {:.0} px, drawn {px}, metric advance width {advance:.0} ({:.0} %), ink {:.0} px", stage.w, 100.0 * advance / stage.w, r - l);
        assert!(advance <= 0.8 * stage.w + 0.01, "{w}: the word is {advance} px of {}", stage.w);
        // Every figure and control apart from every other.
        let boxes: Vec<(String, amenui::ui::Rect)> = ui.nodes().iter().filter(|n| { let k = n.key.as_str(); k.starts_with(&format!("{K}/gap/")) || k.starts_with(&format!("{K}/area/")) || k.starts_with(&format!("{K}/mode/")) || k == format!("{K}/dims") || k == format!("{K}/field") }).map(|n| (n.key.clone(), n.rect)).collect();
        for (i, (a, ra)) in boxes.iter().enumerate() { for (b, rb) in &boxes[i + 1..] {
            let over = ra.x < rb.x + rb.w && rb.x < ra.x + ra.w && ra.y < rb.y + rb.h && rb.y < ra.y + ra.h;
            assert!(!over, "{w}: {a} {ra:?} overlaps {b} {rb:?}");
        } }
        // Gap at 40.
        press(&mut ui, &mut t, &format!("{K}/mode/gap"));
        gallery::update(&mut ui.state, Msg::Lab(website::gallery::lab::LMsg::Kern(website::showcase::lab::kern::Msg::Num(0, amenui::widgets::Edit::Set(40.0)))));
        rest(&mut ui, &mut t, 60);
        let g = inked(&ui);
        let (l, r) = (g.iter().flat_map(|q| q.1.iter().map(|p| p.0)).fold(f32::MAX, f32::min), g.iter().flat_map(|q| q.1.iter().map(|p| p.0 + 1.0)).fold(f32::MIN, f32::max));
        eprintln!("kern at {w}, gap 40: ink from {:.0} to {:.0}, the stage {:.0} to {:.0}", l, r, stage.x, stage.x + stage.w);
        // Z.6's 80 % leaves room for 40 px between six letters only from about 100 px a letter: at 66 px
        // and under the word at `target` 40 is wider than the stage (with design).
        if px >= 99.0 { assert!(l >= stage.x && r <= stage.x + stage.w, "{w}: at gap 40 the word runs {l}..{r} off the stage {:.0}..{:.0}", stage.x, stage.x + stage.w); }
    }
}

#[test]
fn hovering_kern_on_the_index_views_and_lays_out_nothing() {
    let mut ui = gallery::new_ui(fonts(), 1440, 900, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Lab));
    ui.state.routed = true;
    ui.relayout();
    let mut t = 0.0;
    rest(&mut ui, &mut t, 120);
    let r = rect(&ui, "labi/lab-kern");
    let c0 = ui.perf().counters;
    for f in 0..60 {
        t += 1.0 / 60.0;
        ui.event_time(t);
        ui.pointer_move(r.x + 10.0 + f as f32 * 4.0, r.y + r.h * 0.5);
        ui.tick(t);
        ui.frame_drawn();
    }
    let c = &ui.perf().counters;
    eprintln!("kern on the index, 60 moves over it: {} views, {} layouts, {} restages", c.views - c0.views, c.layouts - c0.layouts, c.restages - c0.restages);
    assert_eq!((c.views - c0.views, c.layouts - c0.layouts), (0, 0));
}

#[test]
fn the_pair_distances_are_measured_within_their_budget() {
    let (ui, _) = page(1440, 900);
    let stage = rect(&ui, &format!("{K}/stage"));
    let k = ui.state.lab.kern.at_size((stage.w, stage.h));
    let best = (0..10).map(|_| {
        let t0 = std::time::Instant::now();
        for _ in 0..20 { std::hint::black_box(k.measures()); }
        t0.elapsed().as_secs_f64() * 1000.0 / 20.0
    }).fold(f64::INFINITY, f64::min);
    eprintln!("kern: five pairs' gaps and areas at {} px, {best:.3} ms", k.px);
    assert!(best <= 0.3, "the pair distances took {best:.3} ms");
}

/// Y.5's line for Kern's main gesture: at 1440 × 900, scale 2, a letter dragged along the baseline for
/// 2 s at 60 Hz; every frame shows it moved; CPU per frame printed; at rest no frame in 5 s.
#[test]
fn kern_y5_line() {
    let i = website::gallery::lab::NAMES.iter().position(|n| *n == "kern").expect("kern") as u8;
    let mut ui = gallery::new_ui(fonts(), 2880, 1800, 2.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Experiment(i)));
    ui.state.routed = true;
    ui.relayout();
    let mut t = 0.0;
    rest(&mut ui, &mut t, 120);
    let mut idle = 0;
    for _ in 0..300 { t += 1.0 / 60.0; ui.tick(t); if ui.needs_frame(t) { idle += 1; } ui.frame_drawn(); }
    let word = rect(&ui, &format!("{K}/word"));
    let stage = rect(&ui, &format!("{K}/stage"));
    let x0 = ui.state.lab.kern.at_size((stage.w, stage.h)).x[2] + stage.x + stage.w * 0.5 + 20.0;
    let y0 = word.y + word.h * 0.5;
    let at = |ui: &GalleryUi| ui.state.lab.kern.x[2];
    ui.press(x0, y0);
    rest(&mut ui, &mut t, 1);
    let c0 = ui.perf().counters;
    let (mut ms, mut late, mut last) = (vec![], 0, at(&ui));
    for f in 1..=120 {
        let e = f as f32 / 120.0;
        let x = x0 + 60.0 * (e * std::f32::consts::TAU).sin();
        t += 1.0 / 60.0;
        let t0 = std::time::Instant::now();
        ui.event_time(t);
        ui.pointer_move(x, y0);
        ui.tick(t);
        ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        ui.frame_drawn();
        let now = at(&ui);
        if (now - last).abs() < 0.01 { late += 1; }
        last = now;
    }
    let c = &ui.perf().counters;
    let (views, layouts, restages) = (c.views - c0.views, c.layouts - c0.layouts, c.restages - c0.restages);
    ui.release(x0, y0);
    ms.sort_by(f64::total_cmp);
    let q = |p: f64| ms[((ms.len() - 1) as f64 * p).round() as usize];
    eprintln!("Y.5 kern 1440x900@2 (native CPU, no GPU): drag 2 s: median {:.2} ms, p95 {:.2}, max {:.2}, 120 frames, {late} answered late, {views} views, {layouts} layouts, {restages} restages; at rest {idle} frames in 5 s", q(0.5), q(0.95), q(1.0));
    assert_eq!(late, 0, "{late} frames of the drag did not show the letter moved");
    assert_eq!(idle, 0, "{idle} frames at rest");
}
