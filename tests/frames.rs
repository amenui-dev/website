//! Demand-driven frames (DESIGN.md §1.11), without a GPU: on every gallery page and every scene, once
//! input has settled the runtime stops asking for frames within the longest animation it can start; a
//! ripple, a hover burst and the caret ask for frames (the caret as a timed wake, not every frame); a
//! state clock (the departure board) asks for timed wakes only.
use amenui::gallery::{self, GalleryUi, Msg, Page};
use amenui::ui::BLINK;

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }

/// Tick to `to` at 60 Hz, "drawing" every frame the runtime asks for. Returns how many were drawn.
fn run(ui: &mut GalleryUi, to: f32) -> usize {
    let mut t = ui.now();
    let mut drawn = 0;
    while t < to {
        t = (t + 1.0 / 60.0).min(to);
        ui.tick(t);
        if ui.needs_frame(t) { ui.frame_drawn(); drawn += 1; }
    }
    drawn
}

/// The longest thing any input can start on this page: a front's life, condensing, a commit's front
/// crossing the window, an annotation drawing on, a flow.
fn longest(ui: &GalleryUi) -> f32 {
    let l = ui.state.look();
    let (w, h) = ui.logical_size();
    let cross = (w * w + h * h).sqrt() / l.fx.wave_speed;
    [l.fx.wave_life, l.fx.enter, cross, l.gaze.draw_ms / 1000.0, 0.42].into_iter().fold(0.0, f32::max)
}

#[test]
fn every_page_comes_to_rest() {
    let mut ui = gallery::new_ui(fonts(), 1280, 800, 1.0);
    // The Lab's pieces run while they are seen (DESIGN.md §5.3, the stated exception); with reduced
    // motion they rest like every other page.
    ui.state.lab.reduced = true;
    let mut report = vec![];
    for p in Page::all() {
        gallery::update(&mut ui.state, Msg::Go(p));
        ui.relayout();
        let t0 = ui.now() + 0.1;
        run(&mut ui, t0);
        // Input: the pointer comes in and moves, a wheel turn, Tab to the first control.
        ui.pointer_move(640.0, 400.0);
        ui.pointer_move(700.0, 420.0);
        ui.wheel(700.0, 420.0, 30.0);
        ui.key(amenui::el::Key::Tab, false);
        let start = ui.now();
        let mut to = start + longest(&ui) + 0.05;
        // The departure board condenses its changed digits after each whole second: look between them.
        if p == Page::Style(1) { to = to.floor() + 0.9; }
        let drawn = run(&mut ui, to);
        let t = ui.now();
        ui.tick(t);
        ui.frame_drawn();
        let resting = !ui.needs_frame(t);
        // A timed wake (a blinking caret, the departure board's second) is allowed; it is never "now".
        let wake = ui.next_wake(t);
        report.push(format!("{:<24} drew {drawn:3} after input; at rest: {resting}; next wake {:?}", p.hash(), wake.map(|w| w - t)));
        assert!(resting, "{} still asks for frames {:.2} s after input (moving: {})", p.hash(), t - start, ui.moving_at(t));
        assert!(wake.is_none_or(|w| w > t), "{} wants a frame now", p.hash());
        // Leave scenes for the next page (Esc goes back to the list).
        ui.pointer_leave();
    }
    eprintln!("{}", report.join("\n"));
}

#[test]
fn a_ripple_asks_for_frames_until_it_is_spent() {
    let mut ui = gallery::new_ui(fonts(), 1280, 800, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("button").unwrap()));
    ui.relayout();
    run(&mut ui, 3.0);
    assert!(!ui.needs_frame(3.0), "idle on #button");
    // Three idle seconds: nothing drawn.
    assert_eq!(run(&mut ui, 6.0), 0);
    // Plain ground sends no front (DESIGN.md §1.15).
    ui.press(900.0, 700.0);
    ui.release(900.0, 700.0);
    assert!(!ui.field().fronts.iter().any(|f| f[3] > 0.0), "a press on the ground sent a front");
    // A press on a button: a front over the button.
    let k = ui.nodes().iter().find(|n| n.pressable() && n.key.ends_with("live")).map(|n| n.key.clone()).expect("the live button");
    let (x, y) = ui.node(&k).unwrap().rect.center();
    ui.press(x, y);
    ui.release(x, y);
    let life = ui.state.look().fx.wave_life;
    let n = run(&mut ui, 6.0 + life * 0.5);
    assert!(n >= 25, "frames during the ripple: {n}");
    run(&mut ui, 6.0 + life + 0.1);
    assert!(!ui.needs_frame(ui.now()), "the ripple is spent");
}

#[test]
fn the_caret_blinks_by_timed_wakes() {
    let mut ui = gallery::new_ui(fonts(), 1280, 800, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("input").unwrap()));
    ui.relayout();
    run(&mut ui, 2.0);
    let k = ui.nodes().iter().find(|n| n.focusable && n.key.ends_with("live")).map(|n| n.key.clone()).expect("an input field");
    let (x, y) = ui.node(&k).unwrap().rect.center();
    ui.press(x, y);
    ui.release(x, y);
    assert_eq!(ui.focus(), Some(k.as_str()));
    run(&mut ui, 5.0);
    // At rest with a caret: a wake at the next blink, not now; four blinks in 2 s are four frames.
    let t = ui.now();
    let wake = ui.next_wake(t).expect("the caret wakes");
    assert!(wake > t && wake - t <= BLINK + 1e-3, "next blink in {}", wake - t);
    let n = run(&mut ui, t + 4.0 * BLINK + 0.01);
    assert!((3..=5).contains(&n), "frames for four blink phases: {n}");
}

#[test]
fn a_state_clock_wakes_once_a_second() {
    let mut ui = gallery::new_ui(fonts(), 1280, 800, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Style(1)));
    ui.relayout();
    run(&mut ui, 3.2);
    let n = run(&mut ui, 6.2);
    // Three ticks of the board, each a frame (or two: the tick, and condensing of what changed).
    assert!(n >= 3, "board frames in 3 s: {n}");
    assert!(n < 3 * 40, "the board does not draw every frame: {n}");
}

#[test]
fn the_labs_pieces_run_while_seen_and_stop_when_scrolled_out() {
    let mut ui = gallery::new_ui(fonts(), 1280, 800, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Lab));
    ui.relayout();
    let t0 = ui.now();
    run(&mut ui, t0 + 4.0);
    // Long after any input, what is seen still steps: frames keep coming.
    let t = ui.now();
    assert!(ui.needs_frame(t + 1.0 / 60.0) || ui.moving_at(t), "seen cells run on");
    let drawn = run(&mut ui, t + 1.0);
    assert!(drawn > 40, "a second of living pieces draws most frames: {drawn}");
    assert!(ui.state.lab.seen.iter().any(|&v| v) && !ui.state.lab.seen.iter().all(|&v| v), "some cells are seen, not all: {:?}", ui.state.lab.seen);
    // Scrolled far down past every cell: nothing is seen, the page rests.
    for _ in 0..80 { ui.wheel(700.0, 420.0, 400.0); }
    let t = ui.now();
    run(&mut ui, t + 2.0);
    if ui.state.lab.seen.iter().all(|&v| !v) {
        let t = ui.now();
        assert_eq!(run(&mut ui, t + 1.0), 0, "nothing seen, nothing drawn");
    }
    // Reduced motion: it comes to rest.
    ui.state.lab.reduced = true;
    for _ in 0..80 { ui.wheel(700.0, 420.0, -400.0); }
    let t = ui.now();
    run(&mut ui, t + 4.0);
    let t = ui.now();
    assert_eq!(run(&mut ui, t + 1.0), 0, "reduced motion rests");
}
