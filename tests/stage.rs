//! The control never waits for the piece (site-design Y.5 row 3, as amended by the framework's Y1 ruling):
//! with 40 ms added to the stage's draw (`Ui::slow_stage`), a 2 s scrub of a Lab cell's parameter at 60 Hz
//! shows the new value in every frame that does not redraw the stage, each of those frames lays out the
//! page at most twice (the move, the piece's step) and makes no stage on its own, and the stage redraws at most once per
//! X = max(100 ms, 6 × its draw). The time a held frame takes is a bound only in a release build: a
//! debug build on a shared software rasterizer measures the runner, not the page.
//! On the index, a running cell's frames make and draw its stage alone: no view, no layout.
use website::gallery::{self, GalleryUi, Msg, Page};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }

/// The text a node shows, and the glyphs drawn for it (rect and atlas cell of each instance).
fn shown(ui: &GalleryUi, key: &str) -> (String, Vec<[f32; 8]>) {
    let nodes = ui.nodes();
    let j = nodes.iter().position(|n| n.key == key).unwrap_or_else(|| panic!("{key}"));
    let insts = &ui.scene().instances;
    let glyphs = ui.owners().iter().enumerate().filter(|(_, o)| o.0 == j).map(|(k, _)| { let i = &insts[k]; [i.rect[0], i.rect[1], i.rect[2], i.rect[3], i.uv[0], i.uv[1], i.uv[2], i.uv[3]] }).collect();
    let mut text = String::new();
    fn find(n: &amenui::tree::TreeNode, key: &str, out: &mut String) -> bool {
        if n.key == key { *out = n.text.clone().unwrap_or_default(); return true; }
        n.children.iter().any(|c| find(c, key, out))
    }
    find(&ui.export_tree(), key, &mut text);
    (text, glyphs)
}

#[test]
fn a_slow_lab_stage_holds_while_a_parameter_is_scrubbed_and_the_value_answers_every_frame() {
    let name = "flow";
    let i = website::gallery::lab::NAMES.iter().position(|n| *n == name).expect("the piece") as u8;
    let mut ui = gallery::new_ui(fonts(), 1440, 900, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Experiment(i)));
    ui.state.routed = true;
    ui.relayout();
    let mut t = 0.0f32;
    let frame = |ui: &mut GalleryUi, t: &mut f32| { *t += 1.0 / 60.0; ui.tick(*t); ui.frame_drawn(); };
    for _ in 0..90 { frame(&mut ui, &mut t); }
    // Open the parameters.
    let toggle = ui.node(&format!("lab/{name}/params")).expect("the toggle").rect;
    ui.press(toggle.x + toggle.w * 0.5, toggle.y + toggle.h * 0.5);
    frame(&mut ui, &mut t);
    ui.release(toggle.x + toggle.w * 0.5, toggle.y + toggle.h * 0.5);
    for _ in 0..30 { frame(&mut ui, &mut t); }
    let (stage, value, field) = (format!("lab/{name}/stage"), format!("lab/{name}/p1/shown"), format!("lab/{name}/p1/value"));
    let r = ui.node(&field).expect("the value field").rect;
    // The stage becomes slow with 40 ms added to its draw.
    ui.slow_stage(40.0);
    for _ in 0..12 { frame(&mut ui, &mut t); }
    assert_eq!(ui.stage_state(&stage).map(|s| s.0), Some(true), "40 ms draws are slow");
    let (x0, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    ui.press(x0, y);
    frame(&mut ui, &mut t);
    let mut before = shown(&ui, &value);
    let (mut redraws, mut held, mut late, mut last) = (0, 0, 0, t);
    let (mut gaps, mut held_ms): (Vec<f32>, Vec<f32>) = (vec![], vec![]);
    for f in 1..=120 {
        // Back and forth over 20 steps, 7 px a frame (a step is 6 px): every frame asks for a new value.
        let k = f % 40;
        let x = x0 + 7.0 * if k <= 20 { k } else { 40 - k } as f32;
        let drawn = ui.stage_state(&stage).map_or(0, |s| s.1);
        let c0 = ui.perf().counters;
        let w0 = std::time::Instant::now();
        ui.pointer_move(x, y);
        t += 1.0 / 60.0;
        ui.tick(t);
        let ms = w0.elapsed().as_secs_f32() * 1000.0;
        ui.frame_drawn();
        let c = &ui.perf().counters;
        let redrew = ui.stage_state(&stage).map_or(0, |s| s.1) > drawn;
        let now = shown(&ui, &value);
        if redrew {
            redraws += 1;
            gaps.push(t - last);
            last = t;
        } else {
            held += 1;
            held_ms.push(ms);
            if ms > 16.7 { late += 1; }
            // The value's frame lays out for the move and for the piece's step, no more, and makes no stage on its own.
            assert!(c.layouts - c0.layouts <= 2, "frame {f}: {} views and {} layouts", c.views - c0.views, c.layouts - c0.layouts);
            assert_eq!(c.restages, c0.restages, "frame {f}: a stage was made on its own while the page was");
            // Row 2: the value shown is the new one in this frame.
            assert_ne!(now.0, before.0, "frame {f}: the value did not change ({})", now.0);
            assert_ne!(now.1, before.1, "frame {f}: the value's glyphs did not change");
        }
        before = now;
    }
    held_ms.sort_by(f32::total_cmp);
    let q = |p: f32| held_ms[((held_ms.len() - 1) as f32 * p).round() as usize];
    eprintln!("{name}: {redraws} stage redraws in a 2 s scrub (gaps {gaps:.3?} s); {held} frames held, median {:.2} ms, p95 {:.2}, max {:.2}, over 16.7 ms: {late}", q(0.5), q(0.95), q(1.0));
    let x = (6.0 * 0.040f32).clamp(0.1, 0.5);
    assert!(gaps.iter().skip(1).all(|g| *g >= x - 1.0 / 60.0 - 1e-3), "two stage redraws closer than X = {x} s: {gaps:?}");
    assert!(redraws <= (2.0 / x).ceil() as usize + 1, "{redraws} redraws in 2 s");
    assert!(held >= 100, "{held} frames of 120 held the stage");
    if !cfg!(debug_assertions) { assert_eq!(late, 0, "a frame that did not redraw the stage took over 16.7 ms"); }
    // The release draws the stage on the next frame.
    let drawn = ui.stage_state(&stage).map_or(0, |s| s.1);
    ui.release(x0, y);
    frame(&mut ui, &mut t);
    assert!(ui.stage_state(&stage).map_or(0, |s| s.1) > drawn, "the frame after the release did not draw the stage");
}

#[test]
fn a_running_cell_on_the_index_makes_its_stage_alone() {
    let mut ui = gallery::new_ui(fonts(), 1440, 900, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Lab));
    ui.state.routed = true;
    ui.relayout();
    let mut t = 0.0f32;
    for _ in 0..120 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
    // The pointer on Flow's frame: it runs, the other cells are stills.
    let r = ui.node("labi/lab-flow").expect("Flow's cell").rect;
    ui.pointer_move(r.x + 4.0, r.y + 4.0);
    for _ in 0..10 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
    let c0 = ui.perf().counters;
    let drawn = ui.stage_state("labi/lab-flow/stage").map_or(0, |s| s.1);
    for _ in 0..60 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
    let c = &ui.perf().counters;
    assert_eq!((c.views - c0.views, c.layouts - c0.layouts), (0, 0), "a second of Flow running viewed or laid out the page");
    assert!(c.restages - c0.restages >= 55, "{} frames of 60 made Flow's stage alone", c.restages - c0.restages);
    assert!(ui.stage_state("labi/lab-flow/stage").map_or(0, |s| s.1) >= drawn + 55, "Flow's stage drew in its frames");
}

#[test]
fn every_lab_piece_and_every_works_piece_is_a_stage() {
    let mut ui = gallery::new_ui(fonts(), 1440, 900, 1.0);
    let mut t = 0.0f32;
    let mut on = |ui: &mut GalleryUi, p: Page, key: String| {
        gallery::update(&mut ui.state, Msg::Go(p));
        ui.state.routed = true;
        ui.relayout();
        for _ in 0..3 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
        assert!(ui.stage_state(&key).is_some_and(|s| s.1 > 0), "{key} on {} is not a drawn stage", p.hash());
    };
    for (i, n) in website::gallery::lab::NAMES.iter().enumerate() { on(&mut ui, Page::Experiment(i as u8), format!("lab/{n}/stage")); }
    on(&mut ui, Page::Lab, "labi/lab-flow/stage".to_owned());
    for (i, p) in website::gallery::works::PIECES.iter().enumerate().filter(|(_, p)| p.page) { on(&mut ui, Page::Work(i as u8), format!("work/{}/layer", p.slug)); }
}
