//! scribblr's thread (DESIGN.md §1.30) through the runtime at 60 Hz: nothing at rest and no frames; a
//! floating window dragged past a decorative title writes it out as a thread whose tail runs with the
//! map's flow; text to read has no thread and holds (its fill, its place); it all goes back to nothing
//! when the map settles; the budget keeps it one line; recomposing and morphing elements have no
//! thread; `hold_ms` holds the pose for a beat; 回 is four outlines (two rings, each an outer and an inner
//! edge) and three bridges; a 64 px title loosened is a few long chords well off its outline; the Loose
//! Thread page loosens its title all the way, holds it, and lets it go, with its tangle at rest.
use lab::el::{El, Paint, Response, col, layers, row, space, text};
use lab::strand::Kind;
use lab::style::{Look, Material};
use lab::ui::Ui;
use lab::widgets::{FloatMsg, Floating, floating};

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }

#[derive(Clone)] enum M { F(FloatMsg) }
struct S { f: Floating, t: f32, hold: f32, word: String, cmds: Vec<lab::morph::Cmd> }
fn view(s: &S) -> El<M> {
    let page = col().key("page").pad(40.0, 40.0).gap(24.0)
        .child(text("Thread", 72.0, Paint::INK).key("title").legible(false).respond(Response::Loosen))
        .child(row().gap(20.0).child(text("To read beside it", 16.0, Paint::INK).key("legible").respond(Response::Loosen)).child(space(40.0, 40.0).fill(Paint::hue(0)).key("deco")))
        .child(text(s.word.clone(), 28.0, Paint::INK).key("word").legible(false).respond(Response::Loosen));
    let box_ = floating("float", &s.f, col().pad(10.0, 10.0).child(text("box", 12.0, Paint::INK)), M::F);
    layers([page, box_]).width(1000.0).height(600.0)
}
fn update(s: &mut S, m: M) { let M::F(f) = m; let t = s.t; s.f.update(f, t); }
fn look(s: &S) -> Look { Look { material: Material::Scribblr, hold_ms: s.hold, ..Look::default() } }
fn open(hold: f32) -> Ui<S, M> {
    let mut f = Floating::default();
    (f.size, f.area) = ((140.0, 80.0), (1000.0, 600.0));
    f.place(0);
    let mut ui = Ui::new(S { f, t: 0.0, hold, word: "one".into(), cmds: vec![] }, view, update, fonts(), 1000, 600, 1.0)
        .on_look(look)
        .on_commands(|s| std::mem::take(&mut s.cmds))
        .on_tick(|s, t| { s.t = t; s.f.step(t) });
    for i in 1..=60 { ui.tick(i as f32 / 60.0); ui.frame_drawn(); }
    ui
}
fn run(ui: &mut Ui<S, M>, secs: f32) -> usize {
    let (mut t, to, mut drawn) = (ui.now(), ui.now() + secs, 0);
    while t < to { t += 1.0 / 60.0; ui.tick(t); if ui.needs_frame(t) { drawn += 1; } ui.frame_drawn(); }
    drawn
}
fn thread_of<'a>(ui: &'a Ui<S, M>, key: &str) -> Option<&'a (String, f32, Vec<lab::strand::Seg>)> { ui.strands().iter().find(|s| s.0 == key) }

#[test]
fn nothing_at_rest_a_thread_behind_the_window_and_nothing_after() {
    let mut ui = open(0.0);
    assert!(ui.strands().is_empty(), "threads at rest");
    assert_eq!(run(&mut ui, 1.0), 0, "frames at rest");
    // Drag the window across the title, slowly enough to stay over it.
    let (x, y) = ui.node("float").expect("box").rect.center();
    let mut t = ui.now();
    ui.press(x, y);
    let (mut tail_ok, mut tails, mut held, mut trailed) = (0, 0, true, false);
    // Text to read as it is at rest: its instances' fill and place.
    let legible = |ui: &Ui<S, M>| -> Vec<([f32; 4], f32, f32)> { ui.scene().instances.iter().enumerate().filter(|(i, _)| ui.owner_key(*i) == Some("legible")).map(|(_, x)| (x.rect, x.paint[2], x.look[3])).collect() };
    let at_rest = legible(&ui);
    assert!(!at_rest.is_empty());
    for i in 1..=40 {
        t += 1.0 / 60.0;
        ui.pointer_move(x + i as f32 * 6.0, y + 20.0 + i as f32 * 0.5);
        ui.tick(t);
        ui.frame_drawn();
        held &= thread_of(&ui, "legible").is_none() && legible(&ui) == at_rest && at_rest.iter().all(|x| x.2 > 0.5);
        if let Some((_, k, segs)) = thread_of(&ui, "title") {
            assert!(*k > 0.0);
            // One line.
            for w in segs.windows(2) { assert_eq!(w[0].b, w[1].a, "the title's thread is broken"); }
            // The tail out of the end (the last run of tail pieces).
            let tail: Vec<_> = segs.iter().rev().take_while(|s| s.kind == Kind::Tail).collect::<Vec<_>>().into_iter().rev().collect();
            // Counted where the map has a flow at the tail's root.
            if let (Some(a), Some(b)) = (tail.first(), tail.last()) {
                trailed = true;
                let d = [b.b[0] - a.a[0], b.b[1] - a.a[1]];
                let f = ui.fieldmap().flow_at(a.a[0], a.a[1]);
                if f[0] * f[0] + f[1] * f[1] > 0.0025 { tails += 1; if d[0] * f[0] + d[1] * f[1] > 0.0 { tail_ok += 1; } }
            }
        }
    }
    ui.release(x + 240.0, y + 40.0);
    assert!(trailed && tails > 0, "the title never trailed off along a flow");
    assert!(tail_ok * 2 >= tails, "the tail ran with the flow in only {tail_ok} of {tails} frames");
    assert!(held, "text to read had a thread, faded, or was carried by the map");
    // Let go: it glides to a corner, then the map settles and the threads are gone.
    let mut n = 0;
    while ui.state.f.moving() && n < 300 { run(&mut ui, 1.0 / 60.0); n += 1; }
    run(&mut ui, 1.6);
    assert!(ui.strands().is_empty(), "{} threads left", ui.strands().len());
    let t = ui.now();
    assert!(!ui.needs_frame(t), "frames after the threads were gone");
}

#[test]
fn a_small_budget_keeps_one_line_and_recomposing_text_has_none() {
    let mut ui = open(0.0);
    lab::budget::set(lab::budget::MAX_MARKS, 0.25);
    stir(&mut ui);
    let t = ui.now() + 1.0 / 60.0;
    ui.tick(t);
    let (_, _, segs) = thread_of(&ui, "title").expect("a thread");
    assert!(!segs.is_empty());
    for w in segs.windows(2) { assert_eq!(w[0].b, w[1].a, "broken under a small budget"); }
    assert!(lab::budget::usage().max as f32 * 0.25 >= segs.len() as f32);
    lab::budget::set(lab::budget::MAX_MARKS, 1.0);
    // Change the keyed word: while it recomposes, it has no thread (before, it had one).
    assert!(thread_of(&ui, "word").is_some(), "the word was not loose");
    ui.state.word = "two".into();
    ui.relayout();
    let t = ui.now() + 1.0 / 60.0;
    ui.tick(t);
    assert!(thread_of(&ui, "word").is_none(), "recomposing text drew a thread");
}

#[test]
fn morphing_elements_have_no_thread() {
    let mut ui = open(0.0);
    stir(&mut ui);
    let t = ui.now() + 1.0 / 60.0;
    ui.tick(t);
    assert!(thread_of(&ui, "title").is_some() && thread_of(&ui, "deco").is_some(), "the title and the square were not loose");
    // A morph from the title to the square: while it runs, neither end has a thread (they loosen as plain).
    ui.state.cmds.push(lab::morph::Cmd::Morph { from: "title".into(), to: "deco".into(), spec: lab::morph::MorphSpec::default() });
    ui.relayout();
    for _ in 0..10 {
        let t = ui.now() + 1.0 / 60.0;
        ui.tick(t);
        assert!(thread_of(&ui, "title").is_none() && thread_of(&ui, "deco").is_none(), "a morphing element drew a thread");
    }
}

#[test]
fn hold_ms_holds_the_pose_for_a_beat() {
    // Frames 1/120 s apart while the map moves; the pose follows the material clock.
    for hold in [83.0f32, 0.0] {
        let mut ui = open(hold);
        stir(&mut ui);
        let mut t = ui.now();
        let mut seen: Vec<(i64, Vec<lab::strand::Seg>)> = vec![];
        for _ in 0..30 {
            t += 1.0 / 120.0;
            ui.tick(t);
            let beat = if hold > 0.0 { (ui.material_clock() * 1000.0 / hold).floor() as i64 } else { (ui.material_clock() * 1e6) as i64 };
            seen.push((beat, thread_of(&ui, "title").expect("a thread").2.clone()));
        }
        let (mut same, mut across) = (0, 0);
        for w in seen.windows(2) {
            if w[0].0 == w[1].0 { assert_eq!(w[0].1, w[1].1, "hold {hold}: two frames of one beat differ"); same += 1; }
            else { assert_ne!(w[0].1, w[1].1, "hold {hold}: the next beat kept the pose"); across += 1; }
        }
        if hold > 0.0 { assert!(same > 0 && across > 0, "{same} within, {across} across"); } else { assert_eq!(same, 0); }
    }
}

/// Stir the title hard through the map, so it is loose without a window.
fn stir(ui: &mut Ui<S, M>) {
    ui.state.cmds.push(lab::morph::Cmd::Stir { at: lab::morph::StirAt::Key("title".into()), v: [1.0, 0.2], energy: 1.0, radius: 240.0, scope: lab::el::Scope::Global });
    ui.relayout();
}

#[test]
fn hui_is_four_outlines_and_three_bridges() {
    // Loose enough for the bridges, not yet for chords across them.
    fn v(_: &()) -> El<()> { col().pad(40.0, 40.0).child(text("回", 96.0, Paint::INK).key("hui").legible(false).loose(0.25)) }
    let mut ui = Ui::new((), v, |_, _| {}, fonts(), 400, 300, 1.0).on_look(|_| Look { material: Material::Scribblr, ..Look::default() });
    ui.tick(5.0);
    let (_, _, segs) = ui.strands().iter().find(|s| s.0 == "hui").expect("a thread");
    // Runs of bridge pieces: 回 is two rings, each with an outer and an inner edge, so four outlines
    // joined by three bridges, in one line.
    let runs = segs.windows(2).filter(|w| w[0].kind != Kind::Bridge && w[1].kind == Kind::Bridge).count() + (segs[0].kind == Kind::Bridge) as usize;
    assert_eq!(runs, 3, "{runs} bridges");
    for w in segs.windows(2) { assert_eq!(w[0].b, w[1].a); }
}

/// The thread of a 64 px "Loose thread" with no knots and no tails, as plain polylines (`angular` 1).
fn title_thread(k: f32) -> Vec<lab::strand::Seg> {
    fn v(k: &f32) -> El<()> { col().pad(60.0, 60.0).child(text("Loose thread", 64.0, Paint::INK).font(lab::el::Font::Display).key("title").legible(false).loose(*k)) }
    fn look(_: &f32) -> Look {
        let mut l = Look { material: Material::Scribblr, ..Look::default() };
        let p = &mut l.params[Material::Scribblr as usize].0;
        (p[5], p[7]) = (0.0, 1.0);
        l
    }
    let mut ui = Ui::new(k, v, |_, _| {}, fonts(), 900, 300, 1.0).on_look(look);
    ui.pointer_leave();
    ui.tick(5.0);
    ui.strands().iter().find(|s| s.0 == "title").map(|s| s.2.clone()).unwrap_or_default()
}

#[test]
fn a_loosened_title_is_a_few_long_chords_off_its_outline() {
    let rest = title_thread(0.001);
    let all = title_thread(1.0);
    assert!(!rest.is_empty() && !all.is_empty());
    let reach = lab::strand::StrandFx::default().reach;
    assert!(all.len() * 100 <= (rest.len() + 1) * 15, "{} pieces against {} rest vertices", all.len(), rest.len() + 1);
    let mut lens: Vec<f32> = all.iter().map(|s| ((s.b[0] - s.a[0]).powi(2) + (s.b[1] - s.a[1]).powi(2)).sqrt()).collect();
    lens.sort_by(f32::total_cmp);
    assert!(lens[lens.len() / 2] >= 24.0, "median piece {}", lens[lens.len() / 2]);
    // Its ink, a pixel apart along every piece: at most a quarter within 2 px of the outline at rest.
    let off = |p: [f32; 2]| rest.iter().map(|g| {
        let (ex, ey) = (g.b[0] - g.a[0], g.b[1] - g.a[1]);
        let t = (((p[0] - g.a[0]) * ex + (p[1] - g.a[1]) * ey) / (ex * ex + ey * ey).max(1e-9)).clamp(0.0, 1.0);
        ((g.a[0] + ex * t - p[0]).powi(2) + (g.a[1] + ey * t - p[1]).powi(2)).sqrt()
    }).fold(f32::INFINITY, f32::min);
    let (mut near, mut total) = (0, 0);
    for s in &all {
        let l = ((s.b[0] - s.a[0]).powi(2) + (s.b[1] - s.a[1]).powi(2)).sqrt();
        for j in 0..l.ceil() as usize { let f = j as f32 / l.max(1.0); total += 1; if off([s.a[0] + (s.b[0] - s.a[0]) * f, s.a[1] + (s.b[1] - s.a[1]) * f]) <= 2.0 { near += 1; } }
    }
    assert!(near * 4 <= total, "{near} of {total} px within 2 px of the outline");
    // It reaches past the letters on every side by at least 0.6 × reach.
    let bbox = |s: &[lab::strand::Seg]| s.iter().flat_map(|g| [g.a, g.b]).fold([f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY], |b, p| [b[0].min(p[0]), b[1].min(p[1]), b[2].max(p[0]), b[3].max(p[1])]);
    let (a, b) = (bbox(&rest), bbox(&all));
    let grow = [a[0] - b[0], a[1] - b[1], b[2] - a[2], b[3] - a[3]];
    assert!(grow.iter().all(|g| *g >= 0.6 * reach), "it reaches past the letters by {grow:?}");
}

#[test]
fn the_loose_thread_page_loosens_its_title_all_the_way_and_text_to_read_holds() {
    use lab::gallery;
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
    let legible_keys: Vec<String> = { let mut v = vec![]; fn keys(n: &lab::tree::TreeNode, v: &mut Vec<String>) { if n.legible { v.push(n.key.clone()); } for c in &n.children { keys(c, v); } } keys(&tree, &mut v); v };
    let mut by_key: std::collections::BTreeMap<String, [f32; 4]> = Default::default();
    let sp = lab::text::SPREAD as f32;
    for (i, x) in ui.scene().instances.iter().enumerate() {
        let Some(k) = ui.owner_key(i) else { continue };
        if x.paint[3] != lab::scene::GLYPH || !k.starts_with(h) || !legible_keys.iter().any(|l| l == k) { continue; }
        let b = by_key.entry(k.to_owned()).or_insert([f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY]);
        *b = [b[0].min(x.rect[0] + sp), b[1].min(x.rect[1] + sp), b[2].max(x.rect[0] + x.rect[2] - sp), b[3].max(x.rect[1] + x.rect[3] - sp)];
    }
    let boxes: Vec<[f32; 4]> = by_key.into_values().collect();
    assert!(boxes.len() >= 4, "{} texts to read", boxes.len());
    let glyphs = |ui: &gallery::GalleryUi| -> Vec<([f32; 4], f32)> { ui.scene().instances.iter().enumerate().filter(|(i, x)| x.paint[3] == lab::scene::GLYPH && ui.owner_key(*i).is_some_and(|k| legible_keys.iter().any(|l| l == k))).map(|(_, x)| (x.rect, x.paint[2])).collect() };
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
    let head: Vec<&lab::strand::Seg> = segs.iter().take_while(|s| s.kind == Kind::Tail).collect();
    let tail: Vec<&lab::strand::Seg> = segs.iter().rev().take_while(|s| s.kind == Kind::Tail).collect();
    assert!(!head.is_empty() && !tail.is_empty(), "two tails");
    let span = |v: &[&lab::strand::Seg]| { let (a, b) = (v.first().expect("a piece").a, v.last().expect("a piece").b); ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt() };
    let (lh, lt) = (span(&head), span(&tail));
    assert!(lt >= 300.0 * kk && lh >= 0.5 * lt, "tails {lh} and {lt} at {kk}");
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
