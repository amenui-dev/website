//! The Body scene (the home page's canvas) through its own runtime (AA.12, AA.14): the file's first
//! four lines are the console's constants and the scene's defaults; the console sits on the listing's
//! grid inside the stage, its controls apart; the glyphs of the console's lines and of the stone are the
//! listing's; every state rests without drawing, TURN on turns on every frame and stops; a hover or a
//! scrub lays nothing out; the thumb is under the hand; the stone turns 1:1, tilts at most 20° and
//! settles upright; the walk lands on time; the way back is the first frame.
use amenui::experiment::Experiment;
use website::showcase::scenes::body::{self, Body, Msg};
use amenui::scene::Instance;
use amenui::ui::Ui;

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }
/// Ticks at 60 Hz for `secs`; the frames the scene asked for.
fn run(ui: &mut Ui<Body, Msg>, secs: f32) -> usize {
    let (mut t, end, mut drawn) = (ui.now(), ui.now() + secs, 0);
    while t < end - 1e-4 { t += 1.0 / 60.0; ui.tick(t); if ui.needs_frame(t) { drawn += 1; } ui.frame_drawn(); }
    drawn
}
fn at(state: &str, w: u32, h: u32) -> Ui<Body, Msg> {
    let mut ui = body::ui(fonts(), w, h, 1.0);
    ui.state.open(state, 0);
    ui.relayout();
    run(&mut ui, 0.5);
    ui
}
fn glyphs(ui: &Ui<Body, Msg>, key: &str) -> Vec<Instance> { ui.instances_of(key).into_iter().filter(|i| i.paint[3] == amenui::scene::GLYPH).collect() }
fn layouts(ui: &Ui<Body, Msg>) -> u64 { ui.counters().layouts }
fn shaped(ui: &Ui<Body, Msg>) -> u64 { ui.counters().shaped }
/// Everything the scene draws, as numbers (identical input gives identical instances).
fn picture(ui: &Ui<Body, Msg>) -> Vec<u32> { ui.scene().instances.iter().flat_map(|i| i.rect.iter().chain(&i.paint).chain(&i.xf).chain(&i.extra).map(|v| v.to_bits()).collect::<Vec<_>>()).collect() }
fn inside(a: [f32; 4], b: [f32; 4]) -> bool { a[0] >= b[0] - 0.01 && a[1] >= b[1] - 0.01 && a[2] <= b[2] + 0.01 && a[3] <= b[3] + 0.01 }
fn apart(a: [f32; 4], b: [f32; 4], gap: f32) -> bool { a[2] + gap <= b[0] || b[2] + gap <= a[0] || a[3] + gap <= b[1] || b[3] + gap <= a[1] }

#[test]
fn the_first_four_lines_are_the_consoles_constants_and_the_scenes_defaults() {
    let lines: Vec<&str> = body::SOURCE.lines().take(4).collect();
    assert!(lines[0].starts_with("pub const GATHER: f32 = 1.0;"), "{}", lines[0]);
    assert_eq!(lines[1], "pub const MATERIAL: Material = Material::Plain;");
    assert_eq!(lines[2], "pub const TURN: bool = false;");
    assert_eq!(lines[3], "pub const SEED: u32 = 26;");
    let b = Body::default();
    assert_eq!((b.gather.value, b.at), (body::GATHER, 4));
    assert_eq!((body::MATERIALS[b.material].1, b.turn, b.seed), (body::MATERIAL, body::TURN, body::SEED));
    assert!(!b.turn, "TURN is off by default (AA.14)");
}

#[test]
fn the_text_is_this_file_and_its_glyphs_are_the_consoles_and_the_stones() {
    let s = body::stream(body::CHARS, 4);
    assert!(body::SOURCE.starts_with(&s.listing), "whole lines of the file, from its start");
    assert!(s.listing.contains("pub const CHARS"));
    assert!((2_600..=2_800).contains(&s.all), "{} cells", s.all);
    // Words are a cell apart on the stone.
    let rest: Vec<&str> = s.listing.split('\n').skip(4).flat_map(|l| l.split_whitespace()).collect();
    let mut i = 0;
    for w in &rest {
        let n = w.chars().count();
        for k in 1..n { assert_eq!(s.cell[i + k], s.cell[i + k - 1] + 1, "inside {w}"); }
        if i > 0 { assert_eq!(s.cell[i], s.cell[i - 1] + 2, "before {w}"); }
        i += n;
    }
    // AA.12.8: the console's lines' glyphs and the stone's are the listing's.
    let all = s.listing.chars().filter(|c| !c.is_whitespace()).count();
    let head = s.listing.split('\n').take(4).flat_map(|l| l.chars()).filter(|c| !c.is_whitespace()).count();
    assert_eq!(head + s.cell.len(), all);
    let ui = at("body", 1440, 900);
    let sh = ui.state.shown();
    assert_eq!(sh.map.len() + sh.rest_map.len(), ui.state.stone_glyphs(), "each of the stone's glyphs once");
}

#[test]
fn the_console_is_on_the_listings_grid_inside_the_stage_and_its_controls_are_apart() {
    for (w, h) in [(1440u32, 900u32), (1024, 768), (390, 844)] {
        for st in ["interface", "material", "body"] {
            let ui = at(st, w, h);
            let b = &ui.state;
            let s = b.stage();
            let c = b.console();
            assert!(inside(c.rect(), s), "{w}×{h} {st}: console {:?} in stage {s:?}", c.rect());
            assert_eq!(c.x, s[0] + body::GUTTER, "on the listing's text column");
            assert!((c.w / body::ADVANCE).fract() == 0.0 && c.w <= body::CONSOLE_W, "whole cells");
            let cap = b.caption_box();
            let boxes: Vec<[f32; 4]> = vec![c.segment(0), c.segment(1), c.segment(2), c.segment(3), c.turn(), c.seed()];
            for (i, a) in boxes.iter().enumerate() {
                assert!(inside(*a, c.rect()), "{w}×{h}: control {i} {a:?} in the console");
                for bb in &boxes[i + 1..] { assert!(apart(*a, *bb, 0.0), "{w}×{h}: {a:?} and {bb:?} overlap"); }
                assert!(apart(*a, cap, 0.0));
            }
            // AA.12.2: no outline within 3 px of another: the TURN frame and the SEED frame.
            assert!(apart(c.turn_frame(), c.seed(), 3.0), "{w}×{h}");
            // The console's tree: what the layout gave is what the console says, on whole pixels.
            let n = ui.node("body/console").expect("the console").rect;
            assert_eq!([n.x, n.y, n.x + n.w, n.y + n.h], c.rect(), "{w}×{h} {st}");
            for k in ["gather", "material", "turn", "seed"] {
                let r = ui.node(&format!("body/console/{k}")).unwrap_or_else(|| panic!("{k}")).rect;
                for v in [r.x, r.y, r.w, r.h] { assert_eq!(v.fract(), 0.0, "{w}×{h}: {k} at {r:?}"); }
            }
            // No material, looseness or ripple on any part of the console.
            let t = ui.export_tree();
            let con = t.find("body/console").expect("console");
            fn each(n: &amenui::tree::TreeNode, f: &mut dyn FnMut(&amenui::tree::TreeNode)) { f(n); for c in &n.children { each(c, f); } }
            each(con, &mut |n| assert!(n.loose == 0.0 && n.ripple.is_none(), "{}: loose {} ripple {:?}", n.key, n.loose, n.ripple));
        }
    }
}

#[test]
fn every_state_rests_without_drawing() {
    // AA.12.5, AA.14 (22): 60 s at rest in each state draws nothing, TURN off.
    for st in ["source", "interface", "material", "body"] {
        let mut ui = at(st, 1440, 900);
        run(&mut ui, 2.0);
        assert!(!ui.needs_frame(ui.now() + 0.016), "{st}: a frame asked for at rest");
        assert_eq!(run(&mut ui, 60.0), 0, "{st}: frames drawn in a minute at rest");
    }
    // The walk lands, arrives, and rests.
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.open("", 2);
    run(&mut ui, 8.5);
    assert_eq!(ui.state.at, 4);
    assert_eq!(run(&mut ui, 60.0), 0, "after the walk's arrival");
    // AA.14 (23): the stone still, two frames a second apart are the same: the dust has no clock.
    let a = picture(&ui);
    ui.invalidate();
    run(&mut ui, 1.0);
    assert!(a == picture(&ui), "the picture at rest changed");
}

#[test]
fn identical_input_gives_identical_frames() {
    // AA.12.3: the same chapter at 3,000, 3,016 and 4,000 ms, and two runs.
    for st in ["interface", "material"] {
        let mut a = body::ui(fonts(), 1440, 900, 1.0);
        a.state.open(st, 0);
        a.relayout();
        run(&mut a, 3.0);
        let p0 = picture(&a);
        run(&mut a, 0.016);
        assert!(p0 == picture(&a), "{st}: 3,000 → 3,016 ms");
        run(&mut a, 0.984);
        assert!(p0 == picture(&a), "{st}: 3,000 → 4,000 ms");
        let mut b = body::ui(fonts(), 1440, 900, 1.0);
        b.state.open(st, 0);
        b.relayout();
        run(&mut b, 3.0);
        assert!(p0 == picture(&b), "{st}: two runs");
    }
}

#[test]
fn turn_on_turns_on_every_frame_and_stops_after_twenty_seconds() {
    // AA.14 (24): with TURN on and no input, a frame on every refresh; none after 21 s.
    let mut ui = at("body", 1440, 900);
    ui.state.update(Msg::Turn, ui.now());
    run(&mut ui, 1.2);
    let yaw = ui.state.yaw;
    assert_eq!(run(&mut ui, 5.0), 300, "every refresh while it turns");
    assert!(ui.state.yaw > yaw + 0.5, "it turns: {yaw} → {}", ui.state.yaw);
    run(&mut ui, 15.0);
    assert_eq!(run(&mut ui, 5.0), 0, "it eased out after 20 s without input");
    // Out of view nothing turns; reduced motion shows it off.
    let mut ui = at("body", 1440, 900);
    ui.state.update(Msg::Turn, ui.now());
    ui.state.update(Msg::Away(true), ui.now());
    run(&mut ui, 0.3);
    assert_eq!(run(&mut ui, 3.0), 0, "away (after the switch's 140 ms slide)");
    let mut ui = at("body", 1440, 900);
    ui.state.reduced = true;
    ui.state.update(Msg::Turn, ui.now());
    assert!(!ui.state.turn, "reduced motion: TURN stays off");
}

#[test]
fn a_hover_across_the_console_lays_nothing_out() {
    // AA.12.6: a pointer move across the console: 0 layouts, 0 shapings.
    for st in ["interface", "body"] {
        let mut ui = at(st, 1440, 900);
        let c = ui.state.console();
        let (l0, s0) = (layouts(&ui), shaped(&ui));
        let (mut x, y) = (c.x + 2.0, c.y + c.row * 1.5);
        while x < c.x + c.w { ui.pointer_move(x, y); run(&mut ui, 1.0 / 60.0); ui.pointer_move(x, c.y + 10.0); run(&mut ui, 1.0 / 60.0); x += 9.0; }
        assert_eq!(layouts(&ui) - l0, 0, "{st}: layouts in a hover");
        assert_eq!(shaped(&ui) - s0, 0, "{st}: shapings in a hover");
    }
}

#[test]
fn the_thumb_is_under_the_hand_and_lets_go_to_the_nearer_end() {
    // AA.12.7.
    for (end, want) in [(0.49f32, (0.0f32, 2u8)), (0.51, (1.0, 4))] {
        let mut ui = at("interface", 1440, 900);
        let c = ui.state.console();
        let (t0, t1) = c.track();
        let travel = t1 - t0 - 8.0;
        let y = c.y + c.row * 0.5;
        let x0 = t0 + 4.0;
        ui.pointer_move(x0, y);
        ui.press(x0, y);
        run(&mut ui, 1.0 / 60.0);
        let l0 = layouts(&ui);
        for k in 1..=30 {
            let v = end * k as f32 / 30.0;
            let x = x0 + travel * v;
            ui.pointer_move(x, y);
            run(&mut ui, 1.0 / 60.0);
            assert!((ui.state.gather.value - v).abs() < 1e-3, "the gather is the thumb's value: {} for {v}", ui.state.gather.value);
            let thumb = ui.instances_of("body/console/gather/3");
            let cx = thumb.first().map(|i| i.rect[0] + i.rect[2] * 0.5).expect("the thumb");
            assert!((cx - x).abs() <= 1.0, "frame {k}: thumb at {cx}, pointer at {x}");
        }
        assert_eq!(layouts(&ui) - l0, 0, "layouts while scrubbing");
        ui.release(x0 + travel * end, y);
        run(&mut ui, 3.0);
        assert_eq!((ui.state.gather.value, ui.state.at), want, "released at {end}");
    }
}

#[test]
fn the_stone_turns_one_to_one_tilts_softly_and_settles_upright() {
    let mut ui = at("body", 1440, 900);
    let (cx, cy, r) = ui.state.placement();
    let (yaw0, pitch0) = (ui.state.yaw, ui.state.pitch);
    // AA.12.14: a horizontal drag of R px turns it 1 rad.
    ui.pointer_move(cx, cy);
    ui.press(cx, cy);
    ui.pointer_move(cx + 4.0, cy);
    run(&mut ui, 1.0 / 60.0);
    for k in 1..=20 { ui.pointer_move(cx + 4.0 + r * k as f32 / 20.0, cy); run(&mut ui, 1.0 / 60.0); }
    let turned = ui.state.yaw - yaw0;
    assert!((turned - 1.0).abs() < 0.01, "a drag of R turned {turned} rad");
    // Held still 60 ms and let go: no inertia.
    run(&mut ui, 0.07);
    ui.release(cx + 4.0 + r, cy);
    let y1 = ui.state.yaw;
    run(&mut ui, 1.0);
    assert!((ui.state.yaw - y1).abs() < 1e-4, "no throw after a rest: {}", ui.state.yaw - y1);
    // A vertical drag of any length stays inside ±20°; 500 ms after release the tilt is upright.
    ui.pointer_move(cx, cy);
    ui.press(cx, cy);
    for k in 1..=40 { ui.pointer_move(cx, cy + 4.0 + 40.0 * k as f32); run(&mut ui, 1.0 / 60.0); assert!((ui.state.pitch - body::REST_PITCH).abs() <= 0.35 + 1e-4, "tilt {}", ui.state.pitch); }
    assert!((ui.state.pitch - body::REST_PITCH).abs() > 0.3, "it gave under the hand");
    ui.release(cx, cy + 1604.0);
    run(&mut ui, 0.5);
    assert!((ui.state.pitch - pitch0).abs() < 0.5f32.to_radians(), "upright again: {}", ui.state.pitch);
    // A throw slows under 0.02 rad/s and stops. AA.12.15 says within 2 s, but its own decay (±6 rad/s,
    // τ 0.45 s) takes τ·ln(6 / 0.02) ≈ 2.57 s from the limit; the decay is kept as specified.
    ui.pointer_move(cx, cy);
    ui.press(cx, cy);
    for k in 1..=10 { ui.pointer_move(cx + 30.0 * k as f32, cy); run(&mut ui, 1.0 / 60.0); }
    ui.release(cx + 300.0, cy);
    assert!(ui.state.turn_rate().abs() > 1.0 && ui.state.turn_rate().abs() <= body::THROW_MAX, "a throw: {}", ui.state.turn_rate());
    run(&mut ui, body::THROW_TAU * (body::THROW_MAX / 0.02).ln() + 0.02);
    assert!(ui.state.turn_rate().abs() < 0.02, "slowed: {}", ui.state.turn_rate());
    assert_eq!(run(&mut ui, 3.0), 0, "and at rest");
    // AA.12.16: the frame after a pointer move shows the stone at that move's position.
    ui.pointer_move(cx, cy);
    ui.press(cx, cy);
    ui.pointer_move(cx + 10.0, cy);
    let y = ui.state.yaw;
    ui.pointer_move(cx + 50.0, cy);
    assert!((ui.state.yaw - y - 40.0 / r).abs() < 1e-4, "the view follows the move it is given");
    ui.release(cx + 50.0, cy);
}

#[test]
fn only_the_stone_is_reachable_and_the_address_keeps_its_seed() {
    // AA.12.17.
    let mut b = Body::default();
    b.open("body/cloud/08", 0);
    assert_eq!((b.seed, b.at, b.state()), (8, 4, "body/08".to_owned()));
    b.open("body/31", 0);
    assert_eq!(b.seed, 31);
    b.open("source", 0);
    assert!(b.flat() && b.state() == "source");
    b.open("", 0);
    assert_eq!((b.seed, b.state()), (body::SEED, String::new()));
}

#[test]
fn the_walk_lands_on_time_and_the_way_back_is_the_first_frame() {
    // AA.12.11: the walk lands at 6,850 ms ± 2 frames.
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.open("", 2);
    let t0 = ui.now();
    let (mut t, mut seen) = (t0, vec![]);
    let mut landed = None;
    while t < t0 + 9.0 {
        t += 1.0 / 60.0;
        ui.tick(t);
        ui.frame_drawn();
        if seen.last() != Some(&ui.state.at) { seen.push(ui.state.at); }
        if landed.is_none() && ui.state.walk.is_none() && ui.state.seen { landed = Some(t - t0); }
    }
    assert_eq!(seen, vec![1, 2, 3, 2, 4], "the states of the walk");
    // The first frame is the clock's start (`intro_pending` waits one frame).
    let l = landed.expect("it lands");
    assert!((l - 6.85).abs() <= 3.0 / 60.0, "landed at {l}");
    // AA.12.12: 04 → 01 from the rail, then the frame is body#source's at rest.
    ui.state.update(Msg::Show(1), ui.now());
    run(&mut ui, 4.0);
    assert_eq!((ui.state.at, ui.state.gather.value), (1, 0.0));
    let src = at("source", 1440, 900);
    let (a, b) = (glyphs(&ui, "body/head"), glyphs(&src, "body/head"));
    assert!(!a.is_empty() && a.len() == b.len() && a.iter().zip(&b).all(|(x, y)| x.rect == y.rect), "the head lines are back where body#source draws them");
    let (a, b) = (glyphs(&ui, "body/text"), glyphs(&src, "body/text"));
    assert!(a.len() == b.len() && a.iter().zip(&b).all(|(x, y)| x.rect == y.rect && x.xf == y.xf), "the listing is body#source's");
    assert!(ui.node("body/console").is_some_and(|n| n.rect.w > 0.0));
}

#[test]
fn the_console_drives_the_text() {
    let mut ui = at("interface", 1440, 900);
    // A material: 03, the word in it.
    ui.state.update(Msg::Material(3), ui.now());
    assert_eq!((ui.state.at, ui.state.material), (3, 3));
    run(&mut ui, 1.2);
    let w = ui.node("body/word").expect("the word").rect;
    let (k, b) = ui.state.word_box();
    assert_eq!([w.x, w.y, w.x + w.w, w.y + w.h], b);
    // AA.12.9: the word inside the stage, 16 px clear of the console and the caption.
    let s = ui.state.stage();
    assert!(inside(b, s) && apart(b, ui.state.console().rect(), 16.0) && apart(b, ui.state.caption_box(), 16.0), "word {b:?}");
    assert_eq!(k, (0.9 * ui.state.cols() as f32 * 7.0 / 35.0).floor());
    // Another material is a commit inside the stage; plain is 02.
    ui.state.update(Msg::Material(1), ui.now());
    assert!(ui.state.cmds.iter().any(|c| matches!(c, amenui::morph::Cmd::Commit { scope: Some(s), .. } if s == "body/word-box")));
    ui.state.update(Msg::Material(0), ui.now());
    assert_eq!((ui.state.at, ui.state.material), (2, 0));
    // SEED steps and wraps.
    ui.state.seed = 99;
    ui.state.update(Msg::Seed(1), ui.now());
    assert_eq!(ui.state.seed, 0);
    // From 03, the thumb opens the word back into the listing first.
    let mut ui = at("material", 1440, 900);
    ui.state.update(Msg::Scrub(0, 0.3), ui.now());
    assert_eq!((ui.state.at, ui.state.material), (2, 0));
}

#[test]
fn a_click_sends_one_ring_and_the_keys_turn_it() {
    let mut ui = at("body", 1440, 900);
    let (cx, cy, _) = ui.state.placement();
    ui.pointer_move(cx, cy); ui.press(cx, cy); ui.release(cx, cy);
    assert_eq!(ui.state.rings.len(), 1);
    run(&mut ui, 3.0);
    assert!(ui.state.rings.is_empty());
    // → turns 15° in 200 ms; 0 goes back to the opening view.
    let yaw = ui.state.yaw;
    ui.state.update(Msg::Key(amenui::el::Key::Right), ui.now());
    run(&mut ui, 0.25);
    assert!((ui.state.yaw - yaw - body::KEY_TURN).abs() < 1e-4);
    ui.state.update(Msg::Key(amenui::el::Key::Char('0')), ui.now());
    run(&mut ui, 0.5);
    assert!((ui.state.yaw - body::REST_YAW).abs() < 1e-4 && (ui.state.pitch - body::REST_PITCH).abs() < 1e-4);
}

#[test]
fn reduced_motion_changes_state_by_opacity_and_does_not_walk() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.reduced = true;
    ui.state.open("", 2);
    run(&mut ui, 0.05);
    assert!(ui.state.walk.is_none());
    run(&mut ui, 0.5);
    assert_eq!((ui.state.at, ui.state.gather.value, ui.state.dim()), (4, 1.0, 1.0));
    ui.state.update(Msg::Show(2), ui.now());
    run(&mut ui, 0.05);
    assert!(ui.state.dim() < 1.0, "fading");
    run(&mut ui, 0.3);
    assert_eq!((ui.state.at, ui.state.gather.value, ui.state.dim()), (2, 0.0, 1.0));
}

#[test]
fn the_stone_stays_in_the_stage_under_the_console_at_every_size() {
    for (w, h) in [(320u32, 600u32), (390, 844), (720, 900), (1024, 768), (1080, 900), (1440, 900), (2560, 1440)] {
        let mut b = Body::default();
        b.resize(w as f32, h as f32);
        let st = b.stage();
        let (cx, cy, r) = b.placement();
        assert!(cy - r >= b.room_top() - 0.5 && cy + r <= st[3] + 0.5 && cx - r >= st[0] - 0.5, "{w}×{h}: the stone leaves its room");
        if !(720..1080).contains(&w) { assert!(cx + r <= st[2] + 0.5, "{w}×{h}: off the right"); }
        b.view_in("body");
        for m in b.readouts() {
            let tw = body::ADVANCE * m.text.chars().count() as f32;
            assert!(m.at.0 >= st[0] - 0.5 && m.at.0 + tw <= st[2] + 0.5, "{w}×{h}: {}", m.text);
        }
    }
}
