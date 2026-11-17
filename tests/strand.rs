//! scribblr's thread (DESIGN.md §1.30) through the runtime at 60 Hz: nothing at rest and no frames; a
//! floating window dragged past a decorative title writes it out as a thread whose tail runs with the
//! map's flow; text to read stays within its cap with only its outline; it all goes back to nothing
//! when the map settles; the budget keeps it one line; recomposing and morphing elements have no
//! thread; `hold_ms` holds the pose for a beat; 回 is four outlines (two rings, each an outer and an inner edge) and three bridges.
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
    let (mut tail_ok, mut tails, mut most_legible, mut quiet_ok) = (0, 0, 0.0f32, true);
    for i in 1..=40 {
        t += 1.0 / 60.0;
        ui.pointer_move(x + i as f32 * 6.0, y + 20.0 + i as f32 * 0.5);
        ui.tick(t);
        ui.frame_drawn();
        most_legible = most_legible.max(ui.looseness("legible", t));
        if let Some((_, _, segs)) = thread_of(&ui, "legible") { quiet_ok &= segs.iter().all(|s| s.kind == Kind::Line); }
        if let Some((_, k, segs)) = thread_of(&ui, "title") {
            assert!(*k > 0.0);
            // One line.
            for w in segs.windows(2) { assert_eq!(w[0].b, w[1].a, "the title's thread is broken"); }
            let tail: Vec<_> = segs.iter().filter(|s| s.kind == Kind::Tail).collect();
            if let (Some(a), Some(b)) = (tail.first(), tail.last()) {
                tails += 1;
                let d = [b.b[0] - a.a[0], b.b[1] - a.a[1]];
                let f = ui.fieldmap().flow_at(a.a[0], a.a[1]);
                if f[0] * f[0] + f[1] * f[1] > 0.0025 && d[0] * f[0] + d[1] * f[1] > 0.0 { tail_ok += 1; }
            }
        }
    }
    ui.release(x + 240.0, y + 40.0);
    assert!(tails > 0, "the title never trailed off");
    assert!(tail_ok * 2 >= tails, "the tail ran with the flow in only {tail_ok} of {tails} frames");
    assert!(most_legible <= Look::default().legible_cap + 1e-4, "text to read loosened {most_legible}");
    assert!(quiet_ok, "text to read had a bridge, a knot or a tail");
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
    fn v(_: &()) -> El<()> { col().pad(40.0, 40.0).child(text("回", 96.0, Paint::INK).key("hui").legible(false).loose(1.0)) }
    let mut ui = Ui::new((), v, |_, _| {}, fonts(), 400, 300, 1.0).on_look(|_| Look { material: Material::Scribblr, ..Look::default() });
    ui.tick(5.0);
    let (_, _, segs) = ui.strands().iter().find(|s| s.0 == "hui").expect("a thread");
    // Runs of bridge pieces: 回 is two rings, each with an outer and an inner edge, so four outlines
    // joined by three bridges, in one line.
    let runs = segs.windows(2).filter(|w| w[0].kind != Kind::Bridge && w[1].kind == Kind::Bridge).count() + (segs[0].kind == Kind::Bridge) as usize;
    assert_eq!(runs, 3, "{runs} bridges");
    for w in segs.windows(2) { assert_eq!(w[0].b, w[1].a); }
}
