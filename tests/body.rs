//! The Body scene (DESIGN.md §6.2) through its own runtime: the glyph count is conserved between the
//! page and the body, the page at rest is exactly the plain layout, the two rest states ask for no
//! frames but the body's idle turn, and the body turns, opens under the pointer and goes back.
use lab::experiment::Experiment;
use lab::recipes::scenes::body::{self, Body, Msg};
use lab::scene::Instance;
use lab::ui::Ui;

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }
fn run(ui: &mut Ui<Body, Msg>, secs: f32) -> usize {
    let (mut t, end, mut drawn) = (ui.now(), ui.now() + secs, 0);
    while t < end { t += 1.0 / 60.0; ui.tick(t); if ui.needs_frame(t) { drawn += 1; } ui.frame_drawn(); }
    drawn
}
fn glyphs(ui: &Ui<Body, Msg>) -> Vec<Instance> { ui.instances_of("body/text").into_iter().filter(|i| i.paint[3] == lab::scene::GLYPH).collect() }
fn gather(ui: &mut Ui<Body, Msg>) { let (x, y) = ui.node("body").unwrap().rect.center(); ui.press(x, y); ui.release(x, y); }

#[test]
fn glyphs_are_conserved_and_the_page_is_the_plain_layout() {
    lab::i18n::set_current(lab::i18n::Locale::En);
    let mut ui = body::ui(fonts(), 1280, 800, 1.0);
    ui.state.auto = false;
    run(&mut ui, 0.5);
    let page = glyphs(&ui);
    assert!(page.len() > 1200, "the passage is long: {} glyphs", page.len());
    assert!(page.iter().all(|i| i.xf == [0.0; 4]), "the page is not posed");
    // At rest on the page the text is not posed at all: what is drawn is the layout itself, and every
    // glyph has the text's own paint.
    assert!(ui.state.flat());
    assert!(page.iter().all(|i| i.paint[0] == 1.0 && i.paint[1] == 0.0 && i.paint[2] == 1.0));
    gather(&mut ui);
    // Mid-way and in the body: every glyph is still there (none dropped, none added).
    run(&mut ui, 0.35);
    let mid = glyphs(&ui);
    assert_eq!(mid.len(), page.len(), "mid-way");
    assert!(mid.iter().any(|i| i.xf != [0.0; 4]), "glyphs are on their way");
    run(&mut ui, 3.0);
    let b = glyphs(&ui);
    assert_eq!(b.len(), page.len(), "in the body");
    assert!(ui.state.gather.value == 1.0 && !ui.state.gather.moving());
    // Back: exactly the page again, glyph for glyph.
    // Back by the keyboard: Space on the focused scene.
    ui.focus_key("body/stage");
    ui.key(lab::el::Key::Char(' '), false);
    run(&mut ui, 3.0);
    assert!(ui.state.flat(), "back on the page");
    let back = glyphs(&ui);
    assert_eq!(back.len(), page.len());
    for (a, b) in page.iter().zip(&back) { assert_eq!((a.rect, a.xf, a.paint), (b.rect, b.xf, b.paint)); }
}

#[test]
fn the_rest_states_ask_for_no_frames_but_the_idle_turn() {
    lab::i18n::set_current(lab::i18n::Locale::En);
    let mut ui = body::ui(fonts(), 1280, 800, 1.0);
    // Arriving, the page gathers by itself after a moment.
    run(&mut ui, body::DWELL + 0.2);
    assert!(ui.state.target() >= 0.5, "it gathered by itself");
    // Back to the page, and the page rests: nothing at all.
    ui.state.update(Msg::Gather, ui.now());
    run(&mut ui, 4.0);
    assert_eq!(run(&mut ui, 2.0), 0, "the page at rest draws nothing");
    // The body turns by itself: frames, and the turn moves.
    gather(&mut ui);
    run(&mut ui, 2.5);
    let yaw = ui.state.yaw;
    assert!(run(&mut ui, 1.0) > 50, "the idle turn draws");
    assert!(ui.state.yaw > yaw, "it turns");
    // Without the idle turn the body rests too.
    ui.state.update(Msg::Num(3, lab::widgets::Edit::Set(0.0)), ui.now());
    run(&mut ui, 4.0);
    assert_eq!(run(&mut ui, 2.0), 0, "the body without its idle turn draws nothing");
    // Reduced motion: the same.
    ui.state.update(Msg::Num(3, lab::widgets::Edit::Set(0.16)), ui.now());
    ui.state.reduced = true;
    run(&mut ui, 4.0);
    assert_eq!(run(&mut ui, 2.0), 0, "reduced motion: no turn");
}

#[test]
fn dragging_turns_it_and_the_pointer_opens_it() {
    lab::i18n::set_current(lab::i18n::Locale::En);
    let mut ui = body::ui(fonts(), 1280, 800, 1.0);
    ui.state.auto = false;
    run(&mut ui, 0.5);
    gather(&mut ui);
    run(&mut ui, 3.0);
    let r = ui.node("body/text").unwrap().rect;
    let (cx, cy) = r.center();
    // A drag turns it and it coasts on after the release.
    let yaw0 = ui.state.yaw;
    ui.pointer_move(cx, cy);
    ui.press(cx, cy);
    for k in 1..=10 { run(&mut ui, 1.0 / 60.0); ui.pointer_move(cx + 18.0 * k as f32, cy); }
    ui.release(cx + 180.0, cy);
    assert!(ui.state.yaw > yaw0 + 0.4, "dragged round: {} → {}", yaw0, ui.state.yaw);
    assert!(ui.state.spin > 1.0, "thrown: {}", ui.state.spin);
    assert!(ui.state.target() >= 0.5, "a drag does not toggle");
    run(&mut ui, 4.0);
    assert!((ui.state.spin - ui.state.omega).abs() < 0.02, "it coasts back to the idle turn: {}", ui.state.spin);
    // The opening: near the pointer the glyphs are back at their places on the page, far ones not.
    let (px, py) = (r.x + r.w * 0.3, r.y + r.h * 0.4);
    ui.pointer_move(px, py);
    run(&mut ui, 0.8);
    let open = glyphs(&ui);
    let flat_near = open.iter().filter(|i| i.xf == [0.0; 4] || (i.xf.iter().all(|v| v.abs() < 1e-3))).filter(|i| { let (x, y) = (i.rect[0] + i.rect[2] * 0.5, i.rect[1] + i.rect[3] * 0.5); (x - px).hypot(y - py) < 40.0 }).count();
    assert!(flat_near > 5, "words near the pointer are flat again: {flat_near}");
    // Leaving closes it.
    ui.pointer_leave();
    run(&mut ui, 1.0);
    assert!(ui.state.open.value < 0.01);
}
