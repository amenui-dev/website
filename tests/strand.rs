//! scribblr's thread (DESIGN.md §1.30) through the runtime at 60 Hz: nothing at rest and no frames; a
//! floating window dragged past a decorative title writes it out as a thread whose tail runs with the
//! map's flow; text to read has no thread and holds (its fill, its place); it all goes back to nothing
//! when the map settles; the budget keeps it one line; recomposing and morphing elements have no
//! thread; a beat holds the pose for a step; 回 is four outlines (two rings, each an outer and an inner
//! edge) and three bridges; a 64 px title loosened is a few long chords well off its outline; the Loose
//! Thread page loosens its title all the way, holds it, and lets it go, with its tangle at rest.
use amenui::strand::Kind;

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }

#[test]
fn the_loose_thread_page_loosens_its_title_all_the_way_and_text_to_read_holds() {
    use website::gallery;
    let mut ui = gallery::new_ui(fonts(), 1280, 1000, 1.0);
    ui.pointer_leave();
    assert!(ui.state.route("style-loose-thread"));
    ui.relayout();
    let mut t = 0.0;
    let step = |ui: &mut gallery::GalleryUi, t: &mut f32| { *t += 1.0 / 60.0; ui.tick(*t); let d = ui.needs_frame(*t); ui.frame_drawn(); d };
    for _ in 0..90 { step(&mut ui, &mut t); }
    let h = "style-loose-thread";
    let comp = ui.node(&format!("{h}/comp")).expect("comp").rect;
    let scope = [comp.x, comp.y, comp.right(), comp.bottom()];
    assert!(ui.strands().is_empty());
    assert_eq!((0..120).filter(|_| step(&mut ui, &mut t)).count(), 0, "frames at rest");
    // Text to read and controls: the boxes of their glyphs' ink (without the spread), and their glyphs
    // as they are at rest.
    let tree = ui.export_tree();
    let inside = |p: [f32; 2], b: &[f32; 4]| p[0] >= b[0] && p[1] >= b[1] && p[0] < b[2] && p[1] < b[3];
    let legible_keys: Vec<String> = { let mut v = vec![]; fn keys(n: &amenui::tree::TreeNode, v: &mut Vec<String>) { if n.legible { v.push(n.key.clone()); } for c in &n.children { keys(c, v); } } keys(&tree, &mut v); v };
    let mut by_key: std::collections::BTreeMap<String, [f32; 4]> = Default::default();
    let sp = amenui::text::SPREAD as f32;
    for (i, x) in ui.scene().instances.iter().enumerate() {
        let Some(k) = ui.owner_key(i) else { continue };
        if x.paint[3] != amenui::scene::GLYPH || !k.starts_with(h) || !legible_keys.iter().any(|l| l == k) { continue; }
        let b = by_key.entry(k.to_owned()).or_insert([f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY]);
        *b = [b[0].min(x.rect[0] + sp), b[1].min(x.rect[1] + sp), b[2].max(x.rect[0] + x.rect[2] - sp), b[3].max(x.rect[1] + x.rect[3] - sp)];
    }
    let boxes: Vec<[f32; 4]> = by_key.into_values().collect();
    assert!(boxes.len() >= 4, "{} texts to read", boxes.len());
    let glyphs = |ui: &gallery::GalleryUi| -> Vec<([f32; 4], f32)> { ui.scene().instances.iter().enumerate().filter(|(i, x)| x.paint[3] == amenui::scene::GLYPH && ui.owner_key(*i).is_some_and(|k| legible_keys.iter().any(|l| l == k))).map(|(_, x)| (x.rect, x.paint[2])).collect() };
    let still = glyphs(&ui);
    assert!(still.len() > 100, "{} glyphs to read", still.len());
    // The tangle at rest: most of the page's height, out of every box of text to read.
    let tg = ui.node(&format!("{h}/tangle")).expect("the tangle");
    let pts: Vec<[f32; 2]> = tg.trace.as_ref().expect("a trace").points.iter().map(|p| [tg.rect.x + p[0], tg.rect.y + p[1]]).collect();
    let (top, foot) = pts.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), p| (a.min(p[1]), b.max(p[1])));
    assert!(foot - top >= 0.8 * comp.h, "the tangle is {} of {} high", foot - top, comp.h);
    for w in pts.windows(2) {
        for j in 0..=8 { let p = [w[0][0] + (w[1][0] - w[0][0]) * j as f32 / 8.0, w[0][1] + (w[1][1] - w[0][1]) * j as f32 / 8.0]; if let Some(b) = boxes.iter().find(|b| inside(p, b)) { panic!("the tangle crosses text to read at {p:?}: {b:?} (scope {scope:?})"); } }
    }
    // The pointer through the title and down across the page.
    let title = ui.node(&format!("{h}/title")).expect("title").rect;
    let mut most: f32 = 0.0;
    let mut held = true;
    for i in 0..=30 {
        let k = i as f32 / 30.0;
        ui.pointer_move(comp.x + 10.0 + (title.w + 40.0) * k, title.y + title.h * 0.5 + 220.0 * k * k);
        step(&mut ui, &mut t);
        most = most.max(ui.looseness(&format!("{h}/title"), t));
        held &= ui.strands().iter().all(|s| !legible_keys.contains(&s.0)) && glyphs(&ui) == still;
    }
    let k = ui.looseness(&format!("{h}/title"), t);
    assert!(k >= 0.8, "after the sweep the title is only {k} loose (most {most})");
    // Its tails: from the edge into the line and out to an edge, clear of text to read.
    let (_, kk, segs) = ui.strands().iter().find(|s| s.0 == format!("{h}/title")).expect("the title's thread").clone();
    let head: Vec<&amenui::strand::Seg> = segs.iter().take_while(|s| s.kind == Kind::Tail).collect();
    let tail: Vec<&amenui::strand::Seg> = segs.iter().rev().take_while(|s| s.kind == Kind::Tail).collect();
    // A tail takes a clear way out to the scope's edge, or there is none: the head may have no clear way
    // past the text to read, the end has one.
    assert!(!tail.is_empty(), "no tail at the end");
    let span = |v: &[&amenui::strand::Seg]| { let (a, b) = (v.first().expect("a piece").a, v.last().expect("a piece").b); ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() };
    let on_edge = |p: [f32; 2]| [p[0] - scope[0], p[1] - scope[1], scope[2] - p[0], scope[3] - p[1]].iter().any(|d| d.abs() <= 1.0);
    let lt = span(&tail);
    assert!(on_edge(tail.last().expect("a piece").b) || lt >= 300.0 * kk, "the end's tail {lt} at {kk} stops short of the edge");
    if let Some(first) = head.first() { assert!(on_edge(first.a), "the head's tail starts at {:?}, not at the scope's edge", first.a); }
    for s in &segs {
        for p in [s.a, s.b] {
            assert!(p[0] >= scope[0] - 0.5 && p[1] >= scope[1] - 0.5 && p[0] <= scope[2] + 0.5 && p[1] <= scope[3] + 0.5, "{p:?} out of the scope");
            if s.kind == Kind::Tail { assert!(!boxes.iter().any(|b| inside(p, b)), "a tail in text to read at {p:?}"); }
        }
    }
    // Let go: still loose a second later, gone by three.
    ui.pointer_leave();
    for _ in 0..60 { step(&mut ui, &mut t); held &= ui.strands().iter().all(|s| !legible_keys.contains(&s.0)) && glyphs(&ui) == still; }
    let k1 = ui.looseness(&format!("{h}/title"), t);
    // (The map's energy falls by its life and a fixed 0.15 a second, so it can be at 0 three seconds
    // after only if it is down to about a third after one: DESIGN.md §9.)
    assert!(k1 >= 0.3, "a second after, the title is {k1} loose");
    for _ in 0..120 { step(&mut ui, &mut t); }
    assert_eq!(ui.looseness(&format!("{h}/title"), t), 0.0);
    assert!(ui.strands().is_empty());
    assert!(!ui.needs_frame(t), "frames after it settled");
    assert!(held, "text to read had a thread, faded or moved");
}
