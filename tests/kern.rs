//! Kern (site-design Z.3, 02.3, and Z.6): the letters' spacing is measured between their outlines. In
//! `gap` every pair is `target` apart at its closest, by a raster test on the glyphs as drawn (the
//! atlas's coverage at each instance), not by the piece's own measure; in `metric` the glyphs are where
//! the shaper put them; in a scripted drag no pixel has coverage ≥ 0.5 from two glyphs in any frame; the printed
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
/// atlas's coverage at its instance, cut at half: the pixels where its coverage is ≥ 0.5).
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
/// One device row of a glyph's ink: its y, and where the ink starts and ends.
type Row = (f32, f32, f32);
/// Each glyph of the word as drawn, row by row: its number in the text, and for each device row with
/// ink the row's y and where its ink starts and ends (device px), the edges placed within their pixels
/// by the atlas's coverage there (the anti-aliased edge, as `pair` measures an outline).
fn edges(ui: &GalleryUi) -> Vec<(usize, Vec<Row>)> {
    let j = ui.nodes().iter().position(|n| n.key == format!("{K}/word")).expect("the word");
    let (atlas, insts) = (ui.atlas(), &ui.scene().instances);
    let a = amenui::text::ATLAS as f32;
    let mut out: Vec<(usize, Vec<Row>)> = ui.glyphs().log.iter().filter(|g| g.node == j).map(|g| {
        let ins = &insts[g.inst];
        let (u0, v0) = ((ins.uv[0] * a).round() as usize, (ins.uv[1] * a).round() as usize);
        let (w, h) = (ins.rect[2] as usize, ins.rect[3] as usize);
        let cov = |x: usize, y: usize| atlas[((v0 + y) * amenui::text::ATLAS as usize + u0 + x) * 2] as f32 / 255.0;
        let rows = (0..h).filter_map(|y| {
            let first = (0..w).find(|&x| cov(x, y) > 0.0)?;
            let last = (0..w).rev().find(|&x| cov(x, y) > 0.0)?;
            Some((ins.rect[1] + y as f32, ins.rect[0] + first as f32 + 1.0 - cov(first, y), ins.rect[0] + last as f32 + cov(last, y)))
        }).collect();
        (g.i, rows)
    }).collect();
    out.sort_by_key(|g| g.0);
    out
}
/// The smallest distance between two glyphs' rows of ink (`edges`), each row a pixel tall: negative
/// when a row of one runs into the same row of the other.
fn gap_of(a: &[Row], b: &[Row]) -> f32 {
    let mut best = f32::INFINITY;
    for p in a { for q in b {
        let dy = ((p.0 - q.0).abs() - 1.0).max(0.0);
        if dy == 0.0 && (p.0 - q.0).abs() < 0.5 && q.1 < p.2 { return -(p.2 - q.1); }
        best = best.min((q.1 - p.2).max(0.0).hypot(dy));
    } }
    best
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
/// One glyph as drawn: its number, its drawn box, and its covered rows (y, first, one past the last).
type Drawn = (usize, amenui::ui::Rect, Vec<(i32, f32, f32)>);
/// Each glyph of the word as drawn, any coverage (Z.6's form owed once amenui gives each glyph as
/// drawn, `Ui::glyphs_drawn`): its number, the box its raster covers (logical px, window coordinates;
/// at scale 1 a device px), and for each row it covers, the first covered pixel and one past the last.
fn drawn(ui: &GalleryUi) -> Vec<Drawn> {
    let key = format!("{K}/word");
    let j = ui.nodes().iter().position(|n| n.key == key).expect("the word");
    let (atlas, insts) = (ui.atlas(), &ui.scene().instances);
    let a = amenui::text::ATLAS as f32;
    let boxes = ui.glyphs_drawn(&key);
    let log: Vec<_> = ui.glyphs().log.iter().filter(|g| g.node == j).collect();
    assert_eq!(boxes.len(), log.len(), "glyphs_drawn and the glyph log differ");
    let mut out: Vec<_> = boxes.iter().zip(log).map(|(b, g)| {
        assert_eq!(b.i, g.i);
        let ins = &insts[g.inst];
        let (u0, v0) = ((ins.uv[0] * a).round() as usize, (ins.uv[1] * a).round() as usize);
        let (w, h) = (ins.rect[2] as usize, ins.rect[3] as usize);
        let cov = |x: usize, y: usize| atlas[((v0 + y) * amenui::text::ATLAS as usize + u0 + x) * 2];
        let rows: Vec<(i32, f32, f32)> = (0..h).filter_map(|y| {
            let first = (0..w).find(|&x| cov(x, y) > 0)?;
            let last = (0..w).rev().find(|&x| cov(x, y) > 0)?;
            Some(((ins.rect[1] + y as f32).round() as i32, ins.rect[0] + first as f32, ins.rect[0] + last as f32 + 1.0))
        }).collect();
        // Every covered pixel is in the box the framework says the glyph covers.
        let r = b.rect;
        for &(y, l, e) in &rows { assert!(l >= r.x - 0.01 && e <= r.x + r.w + 0.01 && y as f32 >= r.y - 0.01 && (y + 1) as f32 <= r.y + r.h + 0.01, "glyph {}: coverage at {l}..{e}, row {y}, outside its drawn box {r:?}", b.i); }
        (b.i, r, rows)
    }).collect();
    out.sort_by_key(|g| g.0);
    out
}
/// How far two neighbouring glyphs' coverage runs into each other, px, at any coverage: in each row
/// both cover, the left one's end past the right one's start. Touching (Z.6) is at most 1: faint edges
/// sharing a pixel.
fn into(a: &[(i32, f32, f32)], b: &[(i32, f32, f32)]) -> f32 {
    a.iter().filter_map(|p| b.iter().find(|q| q.0 == p.0).map(|q| p.2 - q.1)).fold(f32::NEG_INFINITY, f32::max)
}
/// The word's drawn boxes, left and right, logical px.
fn drawn_span(g: &[Drawn]) -> (f32, f32) {
    (g.iter().map(|q| q.1.x).fold(f32::MAX, f32::min), g.iter().map(|q| q.1.x + q.1.w).fold(f32::MIN, f32::max))
}
fn figures(ui: &GalleryUi, n: usize) -> Vec<f32> { (0..n).map(|k| text_of(ui, &format!("{K}/gap/{k}")).expect("a gap figure").parse().expect("a number")).collect() }

#[test]
fn in_gap_every_pair_is_target_apart_by_a_raster_test_and_the_figures_are_the_raster_s() {
    let (mut ui, mut t) = page(1440, 900);
    for (mode, check) in [("metric", false), ("gap", true), ("area", false)] {
        press(&mut ui, &mut t, &format!("{K}/mode/{mode}"));
        let g = edges(&ui);
        assert_eq!(g.len(), 6, "{mode}: six glyphs drawn");
        let raster: Vec<f32> = g.windows(2).map(|w| gap_of(&w[0].1, &w[1].1)).collect();
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

/// A letter dragged into its neighbours and back: in no frame has a pixel coverage ≥ 0.5 from two glyphs; let
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
    let (mut worst, mut deepest) = (f32::INFINITY, f32::NEG_INFINITY);
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
        for w in g.windows(2) { let d = apart(&w[0].1, &w[1].1); worst = worst.min(d); assert!(d >= 0.0, "frame {f}: a pixel has coverage ≥ 0.5 from glyphs {} and {}", w[0].0, w[1].0); }
        for w in drawn(&ui).windows(2) { let d = into(&w[0].2, &w[1].2); deepest = deepest.max(d); assert!(d <= 1.0 + 1e-3, "frame {f}: glyphs {} and {} run {d:.2} px into each other at any coverage", w[0].0, w[1].0); }
    }
    eprintln!("kern drag: the closest two glyphs came {worst:.2} px at half coverage; at any coverage they ran {deepest:.2} px into each other");
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

/// The ink's left and right edges over all glyphs, device px.
fn ink_span(g: &[(usize, Vec<(f32, f32)>)]) -> (f32, f32) {
    (g.iter().flat_map(|q| q.1.iter().map(|p| p.0)).fold(f32::MAX, f32::min), g.iter().flat_map(|q| q.1.iter().map(|p| p.0 + 1.0)).fold(f32::MIN, f32::max))
}
/// Each glyph's left ink edge, device px.
fn lefts(g: &[(usize, Vec<(f32, f32)>)]) -> Vec<f32> { g.iter().map(|q| q.1.iter().map(|p| p.0).fold(f32::MAX, f32::min)).collect() }
fn set_target(ui: &mut GalleryUi, t: &mut f32, v: f32) {
    gallery::update(&mut ui.state, Msg::Lab(website::gallery::lab::LMsg::Kern(website::showcase::lab::kern::Msg::Num(0, amenui::widgets::Edit::Set(v)))));
    rest(ui, t, 60);
}

/// Z.6 as amended: at 1440, 1024, 390 and 320, at the defaults the five gaps in `metric` are not all
/// equal and `gap` moves at least one letter by 2 px or more against its neighbour; the drawn size is
/// at least 44 and the `metric` word at most 80 % of the stage; at `target`'s ceiling in `gap`, and in
/// `area`, every glyph is 16 px inside the stage; in a drag to either edge no pixel has coverage ≥ 0.5 from
/// two glyphs, and none ≥ 0.5 outside the stage, and at any coverage (`Ui::glyphs_drawn`) no glyph's box
/// leaves the stage and no two run more than 1 px into each other; no printed figure overlaps another or
/// a control.
#[test]
fn the_word_fits_the_stage_at_every_width() {
    for w in [1440u32, 1024, 390, 320] {
        let (mut ui, mut t) = page(w, 900);
        let stage = rect(&ui, &format!("{K}/stage"));
        let k = ui.state.lab.kern.at_size((stage.w, stage.h));
        let (px, width) = (k.px, k.metric_width(&k.chars(), k.px));
        eprintln!("kern at {w}: stage {:.0} px, drawn {px}, metric width {width:.0} ({:.0} %), target's ceiling {}", stage.w, 100.0 * width / stage.w, k.ceiling);
        assert!(px >= 44.0 && px.fract() == 0.0, "{w}: drawn at {px}");
        assert!(width <= 0.8 * stage.w + 0.01, "{w}: the word is {width} px of {}", stage.w);
        let gaps = figures(&ui, 5);
        assert!(gaps.iter().any(|g| (g - gaps[0]).abs() > 0.05), "{w}: the five gaps in metric are all {}", gaps[0]);
        let metric = lefts(&inked(&ui));
        let boxes: Vec<(String, amenui::ui::Rect)> = ui.nodes().iter().filter(|n| { let k = n.key.as_str(); k.starts_with(&format!("{K}/gap/")) || k.starts_with(&format!("{K}/area/")) || k.starts_with(&format!("{K}/mode/")) || k == format!("{K}/dims") || k == format!("{K}/field") }).map(|n| (n.key.clone(), n.rect)).collect();
        for (i, (a, ra)) in boxes.iter().enumerate() { for (b, rb) in &boxes[i + 1..] {
            let over = ra.x < rb.x + rb.w && rb.x < ra.x + ra.w && ra.y < rb.y + rb.h && rb.y < ra.y + ra.h;
            assert!(!over, "{w}: {a} {ra:?} overlaps {b} {rb:?}");
        } }
        press(&mut ui, &mut t, &format!("{K}/mode/gap"));
        let gap = lefts(&inked(&ui));
        let moved = (1..6).map(|i| ((gap[i] - gap[i - 1]) - (metric[i] - metric[i - 1])).abs()).fold(0.0f32, f32::max);
        assert!(moved >= 2.0, "{w}: gap moves no letter against its neighbour by 2 px ({moved})");
        let inside = |ui: &GalleryUi, what: &str| {
            let (l, r) = ink_span(&inked(ui));
            eprintln!("kern at {w}, {what}: ink {:.0}..{:.0}, the stage {:.0}..{:.0}", l, r, stage.x, stage.x + stage.w);
            assert!(l >= stage.x + 16.0 - 0.5 && r <= stage.x + stage.w - 16.0 + 0.5, "{w}, {what}: the word {l}..{r} is not 16 px inside {:.0}..{:.0}", stage.x, stage.x + stage.w);
        };
        set_target(&mut ui, &mut t, 40.0);
        inside(&ui, "gap at the ceiling");
        press(&mut ui, &mut t, &format!("{K}/mode/area"));
        inside(&ui, "area");
        // Drag the middle letter to each edge and past it.
        press(&mut ui, &mut t, &format!("{K}/mode/metric"));
        // How near the drawn boxes came to the stage's edges: left (≥ 0 inside) and right (≤ 0 inside).
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for to in [stage.x - 40.0, stage.x + stage.w + 40.0] {
            let g = inked(&ui);
            let (x0, y0) = { let p = &g[2].1; (p.iter().map(|q| q.0).sum::<f32>() / p.len() as f32, p.iter().map(|q| q.1).sum::<f32>() / p.len() as f32) };
            ui.press(x0, y0);
            rest(&mut ui, &mut t, 1);
            for f in 1..=40 {
                t += 1.0 / 60.0;
                ui.event_time(t);
                ui.pointer_move(x0 + (to - x0) * f as f32 / 40.0, y0);
                ui.tick(t);
                ui.frame_drawn();
                let g = inked(&ui);
                let (l, r) = ink_span(&g);
                assert!(l >= stage.x && r <= stage.x + stage.w, "{w}: frame {f} of a drag to {to}: coverage ≥ 0.5 at {l}..{r}, outside the stage");
                for p in g.windows(2) { assert!(apart(&p[0].1, &p[1].1) >= 0.0, "{w}: frame {f}: a pixel has coverage ≥ 0.5 from glyphs {} and {}", p[0].0, p[1].0); }
                // Any coverage, by the glyphs as drawn: no glyph's box leaves the stage, and neighbours touch at most.
                let d = drawn(&ui);
                let (dl, dr) = drawn_span(&d);
                (lo, hi) = (lo.min(dl - stage.x), hi.max(dr - (stage.x + stage.w)));
                assert!(dl >= stage.x - 1e-3 && dr <= stage.x + stage.w + 1e-3, "{w}: frame {f} of a drag to {to}: glyphs drawn at {dl:.2}..{dr:.2}, the stage {:.2}..{:.2}", stage.x, stage.x + stage.w);
                for p in d.windows(2) { let k = into(&p[0].2, &p[1].2); assert!(k <= 1.0 + 1e-3, "{w}: frame {f}: glyphs {} and {} run {k:.2} px into each other at any coverage", p[0].0, p[1].0); }
            }
            ui.release(to, y0);
            rest(&mut ui, &mut t, 60);
        }
        eprintln!("kern at {w}, dragged to the edges: glyphs drawn {lo:.2} px inside the left edge, {:.2} px inside the right", -hi);
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
    if !cfg!(debug_assertions) { assert!(best <= 0.3, "the pair distances took {best:.3} ms"); }
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
