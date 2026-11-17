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
    let s = body::stream(2740);
    assert!(body::SOURCE.starts_with(&s.listing), "whole lines of the file, from its start");
    assert!(s.listing.contains("pub const LINES"), "{} lines", s.lines);
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
    assert!(*s.cell.last().unwrap() < 2740);
}

#[test]
fn glyphs_are_conserved_and_their_count_does_not_follow_the_window() {
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    run(&mut ui, 0.5);
    let n = ui.state.glyphs();
    let on = glyphs(&ui);
    // Facing away glyphs are left out of the drawing; every one is posed (none sits in the listing).
    assert!(n > 1800, "the body is made of {n} glyphs");
    assert!(on.len() > n / 3 && on.len() < n, "{} of {n} drawn on the body", on.len());
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
    assert_eq!(all, (0..n).collect::<Vec<_>>());
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
    run(&mut ui, 4.0);
    assert!((ui.state.spin - ui.state.omega).abs() < 0.02, "back to the idle turn: {}", ui.state.spin);
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
            let area = (a[3] * a[4]).min(b[3] * b[4]).max(1.0);
            let o = ox.min(a[3].min(b[3])) * oy.min(a[4].min(b[4])) / area;
            worst = worst.max(o);
        }
    }
    assert!(worst <= 0.10, "glyph boxes overlap by {:.0} %", worst * 100.0);
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
        ui.state.open("source", false);
        ui.relayout();
        run(&mut ui, 0.3);
        let st = ui.state.stage();
        let (rows, cols) = ui.state.grid();
        assert_eq!(rows, (((st[3] - st[1]) / 14.0).floor() as usize - 1).min(44), "{h}: lines");
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
        assert_eq!((n.y, n.h), (st[1], rows as f32 * 14.0), "{h}: the listing's box");
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
    b.open("body/cloud/08", false);
    assert_eq!((b.form, b.seed, b.target(), b.gather.value), (1, 8, 1.0, 1.0));
    assert_eq!(b.state(), "body/cloud/08");
    b.open("source", false);
    assert!(b.flat() && b.state() == "source");
    b.open("", false);
    assert_eq!((b.form, b.seed, b.gather.value, b.state()), (0, body::DEFAULT_SEED, 1.0, String::new()));
    // No state: the listing first, then the short path to the body.
    let mut ui = body::ui(fonts(), 1440, 900, 1.0);
    ui.state.open("", true);
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
    ui.state.open("", true);
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
