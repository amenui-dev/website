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
const CH: u32 = 470;

/// A cell alone: frames at rest, then the pointer moving over it (and pressing, for the drag cells).
fn cell<E: Experiment>(o: &mut Offscreen, out: &str, name: &str, drag: bool) -> f64 {
    let mut u = ui::<E>(faces(), CW, CH, 1.0);
    let look = u.look();
    let mut t = 0.0f32;
    let mut frames = vec![];
    let mut cost = (0.0f64, 0u32);
    let piece = u.nodes().iter().find(|n| n.key.ends_with("/canvas") || n.key.ends_with("/piece") || n.key.ends_with("/deck")).map(|n| n.rect).unwrap_or(lab::ui::Rect::new(24.0, 60.0, 400.0, 250.0));
    let (cx, cy, w, h) = (piece.x, piece.y, piece.w, piece.h);
    for f in 0..300u32 {
        t += 1.0 / 60.0;
        // 0..90: no pointer; then a slow figure over the piece (pressed while dragging for the drag cells).
        if f >= 90 {
            let k = (f - 90) as f32 / 210.0;
            let (px, py) = (cx + w * (0.2 + 0.6 * k), cy + h * (0.5 + 0.3 * (k * 9.0).sin()));
            if f == 90 { u.pointer_move(px, py); if drag { u.press(px, py); } }
            else if f == 299 { if drag { u.release(px, py); } }
            else { u.pointer_move(px, py); }
        }
        let t0 = std::time::Instant::now();
        u.tick(t);
        let px = o.render(&mut u, &look);
        if f >= 90 { cost.0 += t0.elapsed().as_secs_f64() * 1000.0; cost.1 += 1; }
        if [20, 60, 140, 200, 260, 298].contains(&f) { frames.push(crop(&px, CW, CH, [0, 0, CW, CH])); }
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
    for f in 0..90 { t += 1.0 / 60.0; if f == 30 { g.pointer_move(w as f32 * 0.35, 360.0); } else if f > 30 { g.pointer_move(w as f32 * 0.35 + f as f32, 360.0 + f as f32 * 0.5); } g.tick(t); }
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
        let l = g.node("decode/labels").expect("labels").rect;
        exploded.push(crop(&px, w, h, [e.x as u32, e.y.min(l.y) as u32, (e.right().max(l.right()) + 8.0).min(w as f32) as u32, (e.bottom().max(l.bottom()) + 8.0) as u32]));
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
            ("lumen", cell::<x::lumen::Lumen>(&mut o, &out, "lumen", false)),
            ("contours", cell::<x::contours::Contours>(&mut o, &out, "contours", false)),
            ("silk", cell::<x::silk::Silk>(&mut o, &out, "silk", false)),
            ("grown", cell::<x::grown::Grown>(&mut o, &out, "grown", true)),
            ("mercury", cell::<x::mercury::Mercury>(&mut o, &out, "mercury", false)),
            ("flow", cell::<x::flow::Flow>(&mut o, &out, "flow", false)),
            ("loupe", cell::<x::loupe::Loupe>(&mut o, &out, "loupe", true)),
            ("orbit", cell::<x::orbit::Orbit>(&mut o, &out, "orbit", false)),
            ("spring", cell::<x::spring::Spring>(&mut o, &out, "spring", true)),
            ("stack", cell::<x::stack::Stack>(&mut o, &out, "stack", false)),
        ];
        for (n, m) in ms { println!("{n:<10} {m:6.1} ms/frame (tick + scene + both passes + readback, {CW}x{CH}, pointer moving)"); }
    }
    if want("page") { page(&out, 1280, 2700, "lab.png"); page(&out, 390, 4200, "lab-narrow.png"); }
    if want("decode") { decode(&out); }
}
