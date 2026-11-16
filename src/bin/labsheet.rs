//! Contact sheets of the Lab (DESIGN.md §5.3) and the decode transition (§1.28), drawn off-screen with a
//! fixed 60 Hz clock and scripted pointer input, and the milliseconds a frame costs per cell.
//!
//! usage: labsheet <out_dir> [cells | page | decode | perf] …   (all when none is named)
use lab::experiment::{Experiment, IDLE_S, ui};
use lab::gallery::{self, Msg, Page};
use lab::offscreen::{Offscreen, crop, save_png, sheet};
use lab::recipes::lab as x;

fn faces() -> Vec<lab::text::Face> { lab::text::load_bundled("assets").expect("fonts") }
const CW: u32 = 460;
const CH: u32 = 520;

/// A cell alone: frames at rest, then the pointer moving over it (and pressing, for the drag cells).
fn cell<E: Experiment>(o: &mut Offscreen, out: &str, name: &str, drag: bool, long: bool) -> f64 {
    let mut u = ui::<E>(faces(), CW, CH, 1.0);
    let look = u.look();
    let mut t = 0.0f32;
    let mut frames = vec![];
    let mut cost = (0.0f64, 0u32);
    u.tick(0.0);
    let piece = u.nodes().iter().find(|n| n.key.ends_with("/stage")).map(|n| n.rect).unwrap_or(lab::ui::Rect::new(24.0, 60.0, 400.0, 250.0));
    let (cx, cy, w, h) = (piece.x, piece.y, piece.w, piece.h);
    // Long cells (one that changes shape by itself) run 25 s and show the turns; the others 5 s.
    let (n, at): (u32, Vec<u32>) = if long { (1500, vec![60, 250, 420, 470, 700, 860, 900, 1200, 1490]) } else { (300, vec![20, 60, 140, 200, 260, 298]) };
    for f in 0..n {
        t += 1.0 / 60.0;
        // 0..90: no pointer; then a slow figure over the piece (pressed while dragging for the drag cells).
        if (90..300).contains(&f) {
            let k = (f - 90) as f32 / 210.0;
            let (px, py) = (cx + w * (0.2 + 0.6 * k), cy + h * (0.5 + 0.3 * (k * 9.0).sin()));
            if f == 90 { u.pointer_move(px, py); if drag { u.press(px, py); } }
            else if f == 299 { if drag { u.release(px, py); } }
            else { u.pointer_move(px, py); }
        }
        let t0 = std::time::Instant::now();
        u.tick(t);
        if f >= 300 && !at.contains(&f) { continue; }
        let px = o.render(&mut u, &look);
        if (90..300).contains(&f) { cost.0 += t0.elapsed().as_secs_f64() * 1000.0; cost.1 += 1; }
        if at.contains(&f) { frames.push(crop(&px, CW, CH, [0, 0, CW, CH])); }
    }
    let (sw, sh, spx) = sheet(&frames, 3);
    save_png(&format!("{out}/{name}.png"), sw, sh, &spx);
    cost.0 / cost.1.max(1) as f64
}

fn page(out: &str, w: u32, h: u32, file: &str) {
    let mut o = Offscreen::new(w, h);
    let mut g = gallery::new_ui(faces(), w, h, 1.0);
    gallery::update(&mut g.state, Msg::Go(Page::Lab));
    g.relayout();
    let mut t = 0.0;
    // The pointer comes into the first cell's stage and moves slowly across it, so its accent shows.
    let st = g.nodes().iter().find(|n| n.key == "lab/lumen/stage").map(|n| n.rect).unwrap_or(lab::ui::Rect::new(300.0, 200.0, 300.0, 200.0));
    for f in 0..150 { t += 1.0 / 60.0; if f >= 30 { let k = (f - 30) as f32 / 120.0; g.pointer_move(st.x + st.w * (0.25 + 0.4 * k), st.y + st.h * (0.45 + 0.1 * (k * 6.0).sin())); } g.tick(t); }
    let look = g.state.look();
    let px = o.render(&mut g, &look);
    save_png(&format!("{out}/{file}"), w, h, &px);
}

fn decode(out: &str) {
    let (w, h) = (1280u32, 900u32);
    let mut o = Offscreen::new(w, h);
    let mut g = gallery::new_ui(faces(), w, h, 1.0);
    gallery::update(&mut g.state, Msg::Go(Page::Decode));
    g.relayout();
    let (mut flat, mut exploded) = (vec![], vec![]);
    let mut t = 0.0;
    for u in [0.08, 0.2, 0.35, 0.5, 0.62, 0.72, 0.85, 1.0] {
        gallery::update(&mut g.state, Msg::Decode(gallery::decode::DMsg::Num(0, lab::widgets::Edit::Set(u))));
        g.relayout();
        for _ in 0..3 { t += 1.0 / 60.0; g.tick(t); }
        let look = g.state.look();
        let px = o.render(&mut g, &look);
        let c = g.node("decode/card").expect("card").rect;
        flat.push(crop(&px, w, h, [(c.x - 12.0) as u32, (c.y - 12.0) as u32, (c.right() + 12.0) as u32, (c.bottom() + 12.0) as u32]));
        let e = g.node("decode/exploded").expect("exploded").rect;
        let e = g.node("decode/exploded-view").map(|n| n.rect).unwrap_or(e);
        exploded.push(crop(&px, w, h, [e.x as u32, e.y as u32, (e.right() + 8.0).min(w as f32) as u32, (e.bottom() + 8.0).min(h as f32) as u32]));
    }
    let (sw, sh, spx) = sheet(&flat, 4);
    save_png(&format!("{out}/decode.png"), sw, sh, &spx);
    let (sw, sh, spx) = sheet(&exploded, 2);
    save_png(&format!("{out}/decode-exploded.png"), sw, sh, &spx);
    // The dialog: opening (frames through the transition), open, closing.
    gallery::update(&mut g.state, Msg::Decode(gallery::decode::DMsg::Open(true)));
    g.relayout();
    let mut frames = vec![];
    for f in 0..110 {
        t += 1.0 / 60.0;
        if f == 60 { gallery::update(&mut g.state, Msg::Decode(gallery::decode::DMsg::Open(false))); g.relayout(); }
        g.tick(t);
        if [3, 12, 22, 32, 48, 66, 76, 90].contains(&f) {
            let look = g.state.look();
            let px = o.render(&mut g, &look);
            frames.push(crop(&px, w, h, [340, 250, 940, 650]));
        }
    }
    let (sw, sh, spx) = sheet(&frames, 4);
    save_png(&format!("{out}/decode-dialog.png"), sw, sh, &spx);
}

/// The Body scene: eight frames (the page, gathering, the body turning, dragged, the opening, the
/// release, the page again), one large still at 1600 × 1000 (`body-large.png`, and `body-cloud.png`),
/// and what a frame costs with one and with three copies of the passage.
fn body(out: &str) {
    use lab::recipes::scenes::body;
    lab::i18n::set_current(lab::i18n::Locale::En);
    let step = |u: &mut lab::ui::Ui<body::Body, body::Msg>, t: &mut f32, n: u32| { for _ in 0..n { *t += 1.0 / 60.0; u.tick(*t); } };
    let (w, h) = (1280u32, 800u32);
    let mut o = Offscreen::new(w, h);
    let mut u = body::ui(faces(), w, h, 1.0);
    let look = body::look(0);
    let mut t = 0.0f32;
    let mut frames = vec![];
    step(&mut u, &mut t, 60); frames.push(o.render(&mut u, &look));
    step(&mut u, &mut t, 100); frames.push(o.render(&mut u, &look));
    step(&mut u, &mut t, 25); frames.push(o.render(&mut u, &look));
    step(&mut u, &mut t, 150); frames.push(o.render(&mut u, &look));
    let (cx, cy) = (w as f32 * 0.5, h as f32 * 0.47);
    u.pointer_move(cx - 120.0, cy); u.press(cx - 120.0, cy);
    for k in 1..=12 { step(&mut u, &mut t, 1); u.pointer_move(cx - 120.0 + 16.0 * k as f32, cy + 3.0 * k as f32); }
    u.release(cx + 72.0, cy + 36.0);
    step(&mut u, &mut t, 60); frames.push(o.render(&mut u, &look));
    for k in 0..30 { u.pointer_move(cx - 90.0 + k as f32, cy - 40.0); step(&mut u, &mut t, 1); }
    frames.push(o.render(&mut u, &look));
    u.pointer_leave();
    step(&mut u, &mut t, 30);
    u.state.update(body::Msg::Gather, t);
    step(&mut u, &mut t, 40); frames.push(o.render(&mut u, &look));
    step(&mut u, &mut t, 160); frames.push(o.render(&mut u, &look));
    let small: Vec<(u32, u32, Vec<u8>)> = frames.iter().map(|px| crop(px, w, h, [0, 0, w, h])).collect();
    let (sw, sh, spx) = sheet(&small, 4);
    save_png(&format!("{out}/body.png"), sw, sh, &spx);
    // One large still of each form at 1600 × 1000, the pointer resting to one side.
    let (lw, lh) = (1600u32, 1000u32);
    let mut ol = Offscreen::new(lw, lh);
    for (form, file) in [(0u32, "body-large.png"), (1, "body-cloud.png")] {
        let mut ul = body::ui(faces(), lw, lh, 1.0);
        ul.state.form = form;
        let mut t = 0.0;
        ul.state.look_at(0.18, -0.08);
        step(&mut ul, &mut t, 420);
        let px = ol.render(&mut ul, &body::look(form));
        save_png(&format!("{out}/{file}"), lw, lh, &px);
    }
    // The cost of a frame while it turns: one copy of the passage, and three (a taller window, so
    // every glyph is drawn: several thousand).
    for copies in [1usize, 3] {
        let hh = if copies > 1 { 2700 } else { h };
        let mut o = Offscreen::new(w, hh);
        let mut u = body::ui(faces(), w, hh, 1.0);
        u.state.copies = copies;
        u.relayout();
        let mut t = 0.0;
        step(&mut u, &mut t, 360);
        let n = u.instances_of("body/text").len();
        let (mut tick, mut all) = (0.0f64, 0.0f64);
        for _ in 0..60 {
            t += 1.0 / 60.0;
            let t0 = std::time::Instant::now();
            u.tick(t);
            tick += t0.elapsed().as_secs_f64() * 1000.0;
            let _ = o.render(&mut u, &look);
            all += t0.elapsed().as_secs_f64() * 1000.0;
        }
        let b = lab::budget::usage();
        println!("body ×{copies} ({w}×{hh}): {n} glyphs, {} marks in all, {:.1} ms/frame in the runtime (tick, poses, drawings, scene), {:.1} ms with both passes and readback on this adapter; budget scale {:.2}", u.scene().instances.len(), tick / 60.0, all / 60.0, b.scale);
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| "out".into());
    std::fs::create_dir_all(&out).expect("out dir");
    let all = a.len() <= 2;
    let want = |s: &str| all || a[2..].iter().any(|x| x == s);
    let _ = IDLE_S;
    if want("cells") || want("perf") {
        let mut o = Offscreen::new(CW, CH);
        let ms = [
            ("lumen", cell::<x::lumen::Lumen>(&mut o, &out, "lumen", false, false)),
            ("contours", cell::<x::contours::Contours>(&mut o, &out, "contours", false, false)),
            ("silk", cell::<x::silk::Silk>(&mut o, &out, "silk", false, false)),
            ("grown", cell::<x::grown::Grown>(&mut o, &out, "grown", true, true)),
            ("mercury", cell::<x::mercury::Mercury>(&mut o, &out, "mercury", false, false)),
            ("flow", cell::<x::flow::Flow>(&mut o, &out, "flow", false, false)),
            ("loupe", cell::<x::loupe::Loupe>(&mut o, &out, "loupe", true, false)),
            ("orbit", cell::<x::orbit::Orbit>(&mut o, &out, "orbit", false, false)),
            ("spring", cell::<x::spring::Spring>(&mut o, &out, "spring", true, false)),
            ("stack", cell::<x::stack::Stack>(&mut o, &out, "stack", false, false)),
        ];
        for (n, m) in ms { println!("{n:<10} {m:6.1} ms/frame (tick + scene + both passes + readback, {CW}x{CH}, pointer moving)"); }
    }
    if want("page") { page(&out, 1280, 2700, "lab.png"); page(&out, 390, 4200, "lab-narrow.png"); }
    if want("decode") { decode(&out); }
    if want("body") { body(&out); }
}
