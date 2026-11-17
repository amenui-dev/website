//! The Body scene (the home page's canvas) through its own runtime: the body is made of this file's
//! text, one glyph per cell with spaces between words; its glyph count is conserved between the
//! listing and the body and does not follow the window; glyphs facing the viewer do not overlap; the
//! listing fills the stage in 14 px lines cut at whole words; the body and its readouts stay in the
//! stage at every size; a click never releases it; the address's states open at rest and the short
//! path gathers; the rest states ask for no frames but the idle turn.
use amenui::experiment::Experiment;
use website::showcase::scenes::body::{self, Body, Msg};
use amenui::scene::Instance;
use amenui::ui::Ui;

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }
fn run(ui: &mut Ui<Body, Msg>, secs: f32) -> usize {
    let (mut t, end, mut drawn) = (ui.now(), ui.now() + secs, 0);
    while t < end { t += 1.0 / 60.0; ui.tick(t); if ui.needs_frame(t) { drawn += 1; } ui.frame_drawn(); }
    drawn
}
/// The glyphs drawn of the listing's text (`listing`) or of all of it (the listing and the rest).
fn drawn(ui: &Ui<Body, Msg>, keys: &[&str]) -> Vec<Instance> { keys.iter().flat_map(|k| ui.instances_of(k)).filter(|i| i.paint[3] == amenui::scene::GLYPH).collect() }
fn glyphs(ui: &Ui<Body, Msg>) -> Vec<Instance> { drawn(ui, &["body/text", "body/rest"]) }
fn listing(ui: &Ui<Body, Msg>) -> Vec<Instance> { drawn(ui, &["body/text"]) }
fn click(ui: &mut Ui<Body, Msg>, x: f32, y: f32) { ui.pointer_move(x, y); ui.press(x, y); ui.release(x, y); }

#[test]
fn the_text_is_this_file_one_glyph_a_cell() {
    let s = body::stream(body::CHARS);
    assert!(body::SOURCE.starts_with(&s.listing), "whole lines of the file, from its start");
    assert!(s.listing.chars().count() <= body::CHARS && body::SOURCE[s.listing.len()..].starts_with('\n'), "cut at a line end");
    assert!(s.listing.contains("pub const CHARS"), "{} lines", s.lines);
    // The body is made of about 2,700 cells (1,400 on a phone): the number the page reports.
    assert!((2_600..=2_800).contains(&s.cells), "{} cells", s.cells);
    assert!((1_300..=1_400).contains(&body::stream(body::CHARS_SMALL).cells));
    // Words are a cell apart: consecutive glyphs of one word are in consecutive cells, and the first
    // glyph after a space is two cells on.
    let words: Vec<&str> = s.listing.split_whitespace().collect();
    let mut i = 0;
    for w in &words {
        let n = w.chars().count();
        for k in 1..n { assert_eq!(s.cell[i + k], s.cell[i + k - 1] + 1, "inside {w}"); }
        if i > 0 { assert_eq!(s.cell[i], s.cell[i - 1] + 2, "before {w}"); }
        i += n;
    }
    assert!(*s.cell.last().unwrap() < s.cells);
    // The glyph size is the one at which the text fills the spiral, with no more than a ring to spare.
    let em = body::em_to_fill(s.cells);
    let room = body::spiral_count(0.62 * em, 1.45 * em);
    assert!(room >= s.cells && room < s.cells + 60, "{room} cells for {}", s.cells);
}

#[test]
fn glyphs_are_conserved_and_their_count_does_not_follow_the_window() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    run(&mut ui, 0.5);
    let n = ui.state.glyphs();
    let on = glyphs(&ui);
    // Facing away glyphs are left out of the drawing, and so are those squeezed, faded or foreshortened
    // past legibility at the rim (their cells are dots); every one is posed (none sits in the listing).
    assert!(n > 1800, "the body is made of {n} glyphs");
    assert!(on.len() > n / 8 && on.len() < n, "{} of {n} drawn on the body", on.len());
    assert!(on.iter().all(|i| i.xf != [0.0; 4]), "none in the listing");
    // Released: every glyph in the first lines is in the listing, flat.
    ui.state.update(Msg::Gather, ui.now());
    run(&mut ui, 4.0);
    assert!(ui.state.flat());
    let flat = listing(&ui);
    assert!(flat.len() > 400 && flat.iter().all(|i| i.xf == [0.0; 4]), "{} flat", flat.len());
    assert_eq!(flat.len(), ui.state.shown().map.len(), "every glyph of the listing is drawn");
    // Every glyph of the file is in the listing or in the rest, once.
    let s = ui.state.shown();
    let mut all: Vec<usize> = s.map.iter().chain(&s.rest_map).copied().collect();
    all.sort();
    assert!(all == (0..body::stream(body::CHARS).cell.len()).collect::<Vec<_>>(), "each glyph of the text once");
    // The count is the same on a wider window: the glyphs grow, their number does not.
    let mut wide = body::ui(fonts(), 2560, 1440, 1.0);
    run(&mut wide, 0.2);
    assert_eq!(wide.state.glyphs(), n);
}

#[test]
fn a_click_never_releases_it_and_a_drag_turns_it() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    run(&mut ui, 0.5);
    let (cx, cy, r) = ui.state.placement();
    for k in 0..20 { click(&mut ui, cx + (k as f32 - 10.0) * r * 0.05, cy); run(&mut ui, 0.1); }
    assert!(ui.state.target() >= 0.5 && ui.state.gather.value > 0.99, "still the body after 20 clicks");
    let yaw0 = ui.state.yaw;
    ui.pointer_move(cx, cy);
    ui.press(cx, cy);
    for k in 1..=10 { run(&mut ui, 1.0 / 60.0); ui.pointer_move(cx + 18.0 * k as f32, cy); }
    ui.release(cx + 180.0, cy);
    assert!(ui.state.yaw > yaw0 + 0.4, "dragged round: {} → {}", yaw0, ui.state.yaw);
    assert!(ui.state.target() >= 0.5, "a drag does not release");
    run(&mut ui, 8.0);
    assert!((ui.state.turn_rate() - ui.state.omega).abs() < 0.02, "back to the idle turn: {}", ui.state.turn_rate());
    // At any pointer place, nothing shows in the listing: no partial words.
    for (px, py) in [(cx, cy), (cx - r * 0.5, cy - r * 0.3), (cx + r * 0.7, cy + r * 0.2), (100.0, 100.0)] {
        ui.pointer_move(px, py);
        run(&mut ui, 0.4);
        assert!(glyphs(&ui).iter().all(|i| i.xf != [0.0; 4]), "a glyph back in the listing at {px},{py}");
    }
    // Space releases it and gathers it again.
    ui.focus_key("body");
    ui.key(amenui::el::Key::Char(' '), false);
    assert!(ui.state.target() < 0.5);
    ui.key(amenui::el::Key::Char(' '), false);
    assert!(ui.state.target() >= 0.5);
}

#[test]
fn the_pointer_moves_the_lamp_not_the_view() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    run(&mut ui, 0.3);
    ui.state.omega = 0.0;
    ui.state.spin = 0.0;
    let v0 = ui.state.view();
    let (cx, cy, r) = ui.state.placement();
    ui.pointer_move(cx + r * 0.5, cy - r * 0.2);
    run(&mut ui, 1.0);
    let v1 = ui.state.view();
    assert_eq!((v0.c, v0.r, v0.pitch), (v1.c, v1.r, v1.pitch), "the view stays");
    let l = ui.state.light();
    assert!(l[0] > 0.2 && (l[0] - body::REST_LIGHT[0]).abs() > 0.5, "the lamp went to the pointer: {l:?}");
}

#[test]
fn glyphs_facing_the_viewer_do_not_overlap_and_have_no_violet() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    run(&mut ui, 0.5);
    // Each bright glyph as an oriented box: its centre, its turn, and the face's cell at its scale
    // (the advance, 7 px at 11 px, by the cap height, 8 px).
    let gs: Vec<[f32; 5]> = glyphs(&ui).into_iter().filter(|i| i.paint[2] > 0.3).map(|i| {
        assert_eq!(i.look[1], 0.0, "glyphs are in the first colour only");
        // `xf` is the 2 × 2 transform less the identity (`Instance::turned`).
        let [m00, m01, m10, m11] = i.xf;
        let (m00, m11) = (m00 + 1.0, m11 + 1.0);
        [i.rect[0] + i.rect[2] * 0.5, i.rect[1] + i.rect[3] * 0.5, m10.atan2(m00), 7.0 * m00.hypot(m10), 8.0 * m01.hypot(m11)]
    }).collect();
    assert!(gs.len() > 300, "{} bright glyphs", gs.len());
    let mut worst = 0.0f32;
    for (i, a) in gs.iter().enumerate() {
        for b in &gs[i + 1..] {
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            if dx.hypot(dy) > a[3].max(a[4]) + b[3].max(b[4]) { continue; }
            // In a's frame (neighbours on a line are turned alike).
            let (s, c) = a[2].sin_cos();
            let (lx, ly) = (dx * c + dy * s, -dx * s + dy * c);
            let ox = ((a[3] + b[3]) * 0.5 - lx.abs()).max(0.0);
            let oy = ((a[4] + b[4]) * 0.5 - ly.abs()).max(0.0);
            // Of the two boxes' summed area.
            let o = ox.min(a[3].min(b[3])) * oy.min(a[4].min(b[4])) / (a[3] * a[4] + b[3] * b[4]).max(1.0);
            worst = worst.max(o);
        }
    }
    assert!(worst <= 0.05, "glyph boxes overlap by {:.0} % of their summed area", worst * 100.0);
    let th = body::look(0, 1.0).theme;
    for c in [th.accent, th.second, th.ink] { assert!(!(c[2] > c[1] + 0.2 && c[0] > c[1] + 0.1), "violet in the palette: {c:?}"); }
}

#[test]
fn the_body_and_its_readouts_stay_in_the_stage_at_every_size() {
    for (w, h) in [(320u32, 600u32), (390, 844), (720, 900), (1080, 600), (1080, 900), (1440, 600), (1440, 900), (2560, 900), (2560, 1440)] {
        let mut b = Body::default();
        b.resize(w as f32, h as f32);
        let st = b.stage();
        if w >= 720 { assert!(st[0] >= b.text_box()[2] + 48.0 - 0.5, "{w}×{h}: the stage keeps 48 px from the text block"); }
        let (cx, cy, r) = b.placement();
        // Inside the stage, except that a middle width lets the body run off the right edge.
        assert!(cx - r >= st[0] - 0.5 && cy - r >= st[1] - 0.5 && cy + r <= st[3] + 0.5, "{w}×{h}: the body leaves the stage");
        if !(720..1080).contains(&w) { assert!(cx + r <= st[2] + 0.5, "{w}×{h}: the body leaves the stage on the right"); }
        let (em, _) = body::em_for(r);
        let px = body::em_px(r, em) * 1.33;
        assert!(px >= 11.0 - 0.05, "{w}×{h}: glyphs at the centre are {px:.1} px");
        b.view_in("body");
        b.ptr = Some((cx, cy));
        let rs = b.readouts();
        if w >= 720 { assert!(rs.iter().any(|m| m.key == "yaw") || w < 1080, "{w}×{h}: the yaw readout"); }
        for m in &rs {
            let tw = body::ADVANCE * m.text.chars().count() as f32;
            assert!(m.at.0 >= st[0] - 0.5 && m.at.0 + tw <= st[2] + 0.5, "{w}×{h}: {} at {:?}", m.text, m.at);
            // Leaders run at 45° then level.
            let (dx, dy) = ((m.elbow.0 - m.from.0).abs(), (m.elbow.1 - m.from.1).abs());
            assert!((dx - dy).abs() < 0.5 && (m.to.1 - m.elbow.1).abs() < 0.01, "{w}×{h}: {} leader", m.text);
        }
        if w < 720 { assert!(rs.is_empty(), "no readouts on a phone"); }
    }
}

#[test]
fn the_listing_is_whole_lines_of_14_px_cut_at_whole_words() {
    for h in [600u32, 784, 900] {
        let mut ui = body::ui(fonts(), 1146, h, 1.0);
        ui.state.open("source", 0);
        ui.relayout();
        run(&mut ui, 0.3);
        let st = ui.state.stage();
        let (rows, cols) = ui.state.grid();
        assert_eq!(rows, (((st[3] - st[1] - 30.0) / 14.0).floor() as usize).min(body::stream(body::CHARS).lines), "{h}: lines");
        // The caption is 16 px under the last line.
        assert_eq!(ui.state.caption_y(), st[1] + rows as f32 * 14.0 + 16.0);
        let s = ui.state.shown();
        assert_eq!(s.rows, rows);
        let full: Vec<&str> = body::SOURCE.lines().collect();
        for (i, l) in s.text.split('\n').enumerate() {
            assert!(l.chars().count() <= cols, "{h}: line {i} is {} cells of {cols}", l.chars().count());
            let f = full[i].trim_end();
            assert!(f.starts_with(l), "{h}: line {i} is the file's");
            let next = f[l.len()..].chars().next();
            assert!(next.is_none_or(char::is_whitespace) || l.is_empty(), "{h}: line {i} cut inside a word: {l:?}");
        }
        // Laid out on whole lines 14 px apart from the stage's top, and every glyph drawn on one of
        // them (a glyph's quad is its outline's box with a margin: its centre is on its line).
        let n = ui.node("body/text").expect("the listing").rect;
        assert_eq!(n.y, st[1], "{h}: the listing's top");
        let gs = listing(&ui);
        let mut lines: Vec<i32> = gs.iter().map(|g| ((g.rect[1] + g.rect[3] * 0.5 - st[1]) / 14.0).floor() as i32).collect();
        lines.sort();
        lines.dedup();
        assert!(lines.first() == Some(&0) && lines.iter().all(|&l| (l as usize) < rows), "{h}: lines {lines:?}");
    }
}

#[test]
fn the_address_opens_a_state_at_rest_and_the_short_path_gathers() {
    let mut b = Body::default();
    b.open("body/cloud/08", 0);
    assert_eq!((b.form, b.seed, b.target(), b.gather.value), (1, 8, 1.0, 1.0));
    assert_eq!(b.state(), "body/cloud/08");
    b.open("source", 0);
    assert!(b.flat() && b.state() == "source");
    b.open("", 0);
    assert_eq!((b.form, b.seed, b.gather.value, b.state()), (0, body::DEFAULT_SEED, 1.0, String::new()));
    // No state: the listing first, then the short path to the body.
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.open("", 1);
    run(&mut ui, 0.4);
    assert!(ui.state.gather.value == 0.0, "the listing for the first half second");
    run(&mut ui, 1.2);
    assert!(ui.state.gather.value > 0.95, "on the body by 1.6 s: {}", ui.state.gather.value);
    // Keys: 1 the listing, 4 the body, F the form, ] the seed.
    ui.focus_key("body");
    ui.key(amenui::el::Key::Char('1'), false);
    assert!(ui.state.target() < 0.5);
    ui.key(amenui::el::Key::Char('4'), false);
    ui.key(amenui::el::Key::Char('f'), false);
    ui.key(amenui::el::Key::Char(']'), false);
    assert_eq!((ui.state.target(), ui.state.form, ui.state.seed), (1.0, 1, body::DEFAULT_SEED + 1));
    // Reduced motion: an opacity change, out then in, and no flight.
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.reduced = true;
    ui.state.open("", 1);
    run(&mut ui, 0.05);
    assert!(ui.state.gather.value == 0.0 && ui.state.dim() < 1.0, "fading out");
    run(&mut ui, 0.3);
    assert!(ui.state.gather.value == 1.0 && ui.state.dim() == 1.0 && ui.state.fade.is_none());
}

#[test]
fn the_rest_states_ask_for_no_frames_but_the_idle_turn() {
    let mut ui = body::ui(fonts(), 1280, 800, 1.0);
    run(&mut ui, 1.0);
    let yaw = ui.state.yaw;
    assert!(run(&mut ui, 1.0) > 50, "the idle turn draws");
    assert!(ui.state.yaw > yaw, "it turns");
    ui.state.update(Msg::Num(3, amenui::widgets::Edit::Set(0.0)), ui.now());
    run(&mut ui, 4.0);
    assert_eq!(run(&mut ui, 2.0), 0, "the body without its idle turn draws nothing");
    ui.state.update(Msg::Num(3, amenui::widgets::Edit::Set(0.16)), ui.now());
    ui.state.reduced = true;
    run(&mut ui, 4.0);
    assert_eq!(run(&mut ui, 2.0), 0, "reduced motion: no turn");
    ui.state.reduced = false;
    ui.state.update(Msg::Gather, ui.now());
    run(&mut ui, 4.0);
    assert_eq!(run(&mut ui, 2.0), 0, "the listing at rest draws nothing");
}

#[test]
fn the_glyph_readout_names_the_glyph_drawn_under_the_pointer() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    run(&mut ui, 0.5);
    let (cx, cy, r) = ui.state.placement();
    for (px, py) in [(cx, cy), (cx - r * 0.3, cy + r * 0.2)] {
        // A point on a glyph near there.
        let (px, py) = (0..400).map(|k| (px + (k % 20) as f32 - 10.0, py + (k / 20) as f32 - 10.0)).find(|&(x, y)| ui.glyph_at(x, y).is_some()).expect("glyphs near the centre");
        ui.pointer_move(px, py);
        run(&mut ui, 0.1);
        ui.pointer_move(px + 0.5, py);
        run(&mut ui, 0.05);
        let (gi, ch, at) = ui.state.glyph_under_lamp().expect("a glyph under the pointer");
        assert!(!ch.is_whitespace());
        assert!((at.0 - px).hypot(at.1 - py) < 14.0, "glyph {gi} `{ch}` is drawn at {at:?}, the pointer at ({px}, {py})");
    }
}

#[test]
fn the_short_path_sends_one_front_from_the_caret() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.open("", 1);
    run(&mut ui, 0.1);
    assert!(ui.state.fronted.is_none(), "not before 150 ms");
    run(&mut ui, 0.1);
    let at = ui.state.take_front().expect("the front at 150 ms");
    assert_eq!(at, ui.state.caret());
    let st = ui.state.stage();
    assert!(at.0 > st[0] && at.0 < st[2] && at.1 > st[1] && at.1 < ui.state.caption_y(), "from the listing's end: {at:?}");
    run(&mut ui, 2.0);
    assert!(ui.state.take_front().is_none(), "one front");
    // With reduced motion there is none.
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.reduced = true;
    ui.state.open("", 1);
    run(&mut ui, 1.0);
    assert!(ui.state.fronted.is_none());
}

#[test]
fn a_drag_keeps_the_point_under_the_pointer_and_a_flick_slows_into_the_idle_turn() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    run(&mut ui, 0.5);
    let (cx, cy, r) = ui.state.placement();
    let (x0, y0) = (cx - r * 0.2, cy - r * 0.1);
    // A point on a glyph there, so the glyph readout names the glyph pressed.
    let (x0, y0) = (0..400).map(|k| (x0 + (k % 20) as f32 - 10.0, y0 + (k / 20) as f32 - 10.0)).find(|&(x, y)| ui.glyph_at(x, y).is_some()).expect("a glyph");
    ui.pointer_move(x0, y0);
    run(&mut ui, 0.05);
    ui.press(x0, y0);
    let (_, _, at) = ui.state.glyph_under_lamp().expect("a glyph under the pointer");
    let off = (at.0 - x0, at.1 - y0);
    for k in 1..=12 {
        let (x, y) = (x0 + 14.0 * k as f32, y0 + 5.0 * k as f32);
        ui.pointer_move(x, y);
        run(&mut ui, 1.0 / 60.0);
        let (_, _, at) = ui.state.glyph_under_lamp().unwrap();
        let miss = (at.0 - x - off.0).hypot(at.1 - y - off.1);
        assert!(miss < 4.0, "step {k}: the pressed point is {miss:.1} px from the pointer");
    }
    // A flick: it keeps turning, slows, and the idle turn is back 2 s after the release, eased in.
    ui.release(x0 + 168.0, y0 + 60.0);
    let fast = ui.state.turn_rate();
    assert!(fast > ui.state.omega * 3.0, "it keeps the drag's speed: {fast}");
    run(&mut ui, 1.2);
    let slower = ui.state.turn_rate();
    assert!((slower / fast - (-1.0f32).exp()).abs() < 0.05, "slows with a 1.2 s time constant: {fast} → {slower}");
    run(&mut ui, 0.7);
    assert_eq!(ui.state.idle, 0.0, "no idle turn within 2 s of the release");
    run(&mut ui, 1.4);
    assert!(ui.state.idle >= 1.0, "the idle turn eased in over a second");
}

#[test]
fn a_click_sends_one_ring_and_the_arrows_turn_it() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    run(&mut ui, 0.5);
    let (cx, cy, _) = ui.state.placement();
    click(&mut ui, cx, cy);
    assert_eq!(ui.state.rings.len(), 1, "one ring from the click");
    run(&mut ui, 0.3);
    let lifted = glyphs(&ui);
    assert!(!lifted.is_empty());
    run(&mut ui, 2.0);
    assert!(ui.state.rings.is_empty(), "it has crossed the surface");
    // Off the body a click sends none; with reduced motion none.
    click(&mut ui, 40.0, 40.0);
    assert!(ui.state.rings.is_empty());
    ui.state.reduced = true;
    click(&mut ui, cx, cy);
    assert!(ui.state.rings.is_empty(), "reduced motion: no ring");
    // → turns it by 0.6 rad: at once with reduced motion, else slowing as a flick does.
    ui.state.omega = 0.0;
    let yaw = ui.state.yaw;
    ui.state.update(Msg::Key(amenui::el::Key::Right), ui.now());
    assert!((ui.state.yaw - yaw - body::KEY_TURN).abs() < 1e-4);
    ui.state.reduced = false;
    let yaw = ui.state.yaw;
    ui.state.update(Msg::Key(amenui::el::Key::Left), ui.now());
    run(&mut ui, 10.0);
    assert!((yaw - ui.state.yaw - body::KEY_TURN).abs() < 0.02, "turned {}", yaw - ui.state.yaw);
}

#[test]
fn the_bulge_lifts_and_lights_the_surface_near_the_pointer() {
    let t = body::Touch { at: Some([0.0, 0.0, 1.0]), k: 1.0, rings: vec![] };
    let (out, boost) = t.push([0.0, 0.0, 1.0]);
    assert!((out - 0.07).abs() < 1e-6 && (boost - 0.25).abs() < 1e-6);
    assert_eq!(t.push([0.0, body::BULGE, 1.0]), (0.0, 0.0), "nothing at 0.35");
    let ring = body::Touch { at: None, k: 0.0, rings: vec![([0.0, 0.0, 1.0], 0.5)] };
    assert!((ring.push([0.0, 0.5, 1.0]).0 - 0.05).abs() < 1e-6, "the ring lifts what it passes by 0.05");
    assert_eq!(ring.push([0.0, 0.0, 1.0]).0, 0.0);
    // Over the body the cursor is a hand; off it, the arrow.
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    run(&mut ui, 0.5);
    let (cx, cy, _) = ui.state.placement();
    ui.pointer_move(cx, cy);
    run(&mut ui, 0.05);
    assert_eq!(ui.cursor(), amenui::el::Cursor::Grab);
    ui.pointer_move(30.0, 30.0);
    run(&mut ui, 0.05);
    assert_eq!(ui.cursor(), amenui::el::Cursor::Default);
}

#[test]
fn the_walk_goes_through_the_four_states_and_a_person_takes_it_over() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.open("", 2);
    let t0 = ui.now();
    let at = |ui: &mut Ui<Body, Msg>, s: f32| { let d = t0 + s - ui.now(); run(ui, d); (ui.state.at, ui.state.spec.loosen, ui.state.spec.material) };
    assert_eq!(at(&mut ui, 0.5), (1, false, 0), "the source first");
    assert_eq!(at(&mut ui, 1.0), (2, false, 0), "the specimen from 0.7 s");
    assert!(ui.node("body/specimen/material").is_some());
    assert_eq!(at(&mut ui, 2.7), (3, true, 0), "Loosen pressed at 2.5 s: the card held loose, in plain");
    assert_eq!(at(&mut ui, 3.1), (3, true, 1), "near at 2.9 s");
    assert_eq!(at(&mut ui, 3.6), (3, true, 2), "dottr at 3.4 s");
    assert_eq!(at(&mut ui, 4.1), (3, true, 3), "carvr at 3.9 s");
    assert_eq!(at(&mut ui, 4.6), (3, false, 3), "Loosen off at 4.4 s: the card closes in carvr");
    assert_eq!(at(&mut ui, 5.0).0, 4, "gathered at 4.7 s");
    assert_eq!(at(&mut ui, 6.2), (4, false, 0), "the card, gone, is set back");
    at(&mut ui, 6.6);
    assert!(ui.state.walk.is_none() && ui.state.seen, "the landing state");
    at(&mut ui, 8.0);
    assert!(ui.state.turn_rate() > 0.1, "the idle turn eased in");
    // A person takes over: the walk stops where it is, and plays on when asked.
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.open("", 2);
    run(&mut ui, 1.5);
    assert_eq!(ui.state.at, 2);
    let (x, y) = ui.node("body/specimen/loosen").unwrap().rect.center();
    ui.pointer_move(x, y);
    ui.press(x, y);
    ui.release(x, y);
    assert_eq!(ui.state.walking(), 2, "taken over");
    assert!(ui.state.spec.loosen && ui.state.at == 3, "the press reached the switch");
    run(&mut ui, 4.0);
    assert_eq!((ui.state.at, ui.state.spec.material), (3, 0), "nothing more is pressed");
    ui.state.update(Msg::Play, ui.now());
    run(&mut ui, 4.5);
    assert_eq!(ui.state.at, 4, "played on to the body");
    // States 01, 02 and 03 at rest ask for no frames.
    for st in ["source", "interface", "material"] {
        let mut ui = body::ui(fonts(), 1440, 900, 1.0);
        ui.state.open(st, 0);
        run(&mut ui, 1.0);
        assert_eq!(run(&mut ui, 1.0), 0, "{st} at rest draws nothing");
    }
}

#[test]
fn skip_reaches_the_body_within_a_second() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.open("", 2);
    run(&mut ui, 1.6);
    assert_eq!(ui.state.at, 2);
    ui.state.update(Msg::Skip, ui.now());
    run(&mut ui, 1.0);
    assert!(ui.state.at == 4 && ui.state.walk.is_none() && ui.state.seen && ui.now() >= ui.state.lands, "at the body, landed");
}

#[test]
fn state_03_is_the_card_held_loose_and_the_rail_goes_between_02_and_03() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.open("material", 0);
    ui.relayout();
    run(&mut ui, 0.5);
    assert_eq!((ui.state.at, ui.state.spec.loosen, ui.state.spec.material), (3, true, body::REST_MATERIAL), "loose, in its material");
    assert!(ui.export_tree().find("body/specimen/row-Loosen/0").is_some_and(|n| n.legible) && ui.export_tree().find("body/specimen").is_some_and(|n| !n.legible), "the labels are text to read, the card's shapes are not");
    // 03 → 02 on the rail: the card closes at once, and plain is committed a moment later.
    ui.state.update(Msg::Show(2), ui.now());
    ui.relayout();
    run(&mut ui, 0.1);
    assert_eq!((ui.state.at, ui.state.spec.loosen), (2, false), "closed at once");
    run(&mut ui, 0.4);
    assert_eq!(ui.state.spec.material, 0, "committed to plain");
    // A press on Loosen reaches the switch (nothing drawn over the card takes it) and is state 03.
    let (x, y) = ui.node("body/specimen/loosen").unwrap().rect.center();
    click(&mut ui, x, y);
    assert!(ui.state.spec.loosen && ui.state.at == 3, "pressed, held loose");
}

#[test]
fn the_walk_gives_way_when_the_page_is_not_ready() {
    // Not ready when the card would come: one morph to the body, the same landing.
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.open("", 2);
    ui.state.set_ready(0.5);
    run(&mut ui, 1.0);
    assert!(ui.state.walk.is_none() && ui.state.seen && ui.state.at == 4, "straight to the body");
    run(&mut ui, 3.0);
    assert!(ui.state.turn_rate() > 0.1, "the idle turn eased in");
    // Ready for the card, not for the materials: their presses are left out, the walk still lands on time.
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.open("", 2);
    let t0 = ui.now();
    run(&mut ui, 2.0);
    ui.state.set_ready(0.5);
    let d = t0 + 4.6 - ui.now();
    run(&mut ui, d);
    assert_eq!((ui.state.at, ui.state.spec.material), (3, 0), "loose, no material pressed");
    let d = t0 + 6.6 - ui.now();
    run(&mut ui, d);
    assert!(ui.state.at == 4 && ui.state.walk.is_none(), "landed");
}
