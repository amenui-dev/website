//! Filmstrips of transforms (DESIGN.md §1.10), from the gallery's own pages. For each morph pair and
//! material, seven moments on one contact sheet: before, the hand-over, four in flight, the target
//! landed and tightened. For the page change, five moments opening a card and five going back.
//! `film perf` instead times frames of a morph at the particle cap. `film <out> ripple` makes the
//! ripple and recomposition strips of DESIGN.md §1.15–§1.16 from the button page, at 2× scale: the counter
//! going 1 → 2 (eight frames across the recomposition), ten fast presses (0, 100, 200, 300, 500, 800 ms),
//! and one press reaching three neighbours that each answer in their own way.
//!
//! usage: film <out_dir> [perf | ripple]
use amenui::gallery::{self, GalleryUi, Msg, Page};
use amenui::morph::MorphSpec;
use amenui::offscreen::{Offscreen, save_png, sheet};
use std::time::Instant;

const W: u32 = 1280;
const H: u32 = 900;

fn send(ui: &mut GalleryUi, m: Msg) { gallery::update(&mut ui.state, m); ui.relayout(); }
fn settle(ui: &mut GalleryUi, to: f32) {
    let mut t = ui.now();
    while t < to { t = (t + 1.0 / 60.0).min(to); ui.tick(t); }
}
/// One frame cropped to a logical rect.
fn shot(o: &mut Offscreen, ui: &mut GalleryUi, crop: amenui::ui::Rect) -> (u32, u32, Vec<u8>) {
    let look = ui.state.look();
    let px = o.render(ui, &look);
    let (x0, y0) = (crop.x.max(0.0) as u32, crop.y.max(0.0) as u32);
    let (x1, y1) = ((crop.right() as u32).min(o.w), (crop.bottom() as u32).min(o.h));
    let mut out = Vec::with_capacity(((x1 - x0) * (y1 - y0) * 4) as usize);
    for y in y0..y1 { out.extend_from_slice(&px[((y * o.w + x0) * 4) as usize..((y * o.w + x1) * 4) as usize]); }
    (x1 - x0, y1 - y0, out)
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| "film".into());
    std::fs::create_dir_all(&out).expect("out dir");
    let faces = || amenui::text::load_bundled("assets").expect("fonts");
    let mut o = Offscreen::new(W, H);
    if a.get(2).map(String::as_str) == Some("perf") { return perf(&mut o, faces()); }
    if a.get(2).map(String::as_str) == Some("ripple") { return ripple(&out, faces); }

    // ── Morphs ──────────────────────────────────────────────────────────────────────────────────
    let pairs = [(0, 2, "text-picture"), (2, 0, "picture-text"), (0, 5, "text-button"), (3, 1, "shape-paragraph"), (6, 4, "card-ring")];
    let spec = MorphSpec::default();
    let (l, d, t) = (spec.loosen_ms / 1000.0, spec.duration_ms / 1000.0, spec.tighten_ms / 1000.0);
    for (mi, mname) in ["plain", "dot"].iter().enumerate() {
        for theme in [0usize, 1] {
            for &(from, to, name) in &pairs {
                let mut ui = gallery::new_ui(faces(), W, H, 1.0);
                ui.pointer_leave();
                send(&mut ui, Msg::Go(Page::Morph));
                send(&mut ui, Msg::Theme(theme));
                send(&mut ui, Msg::Material(mi));
                ui.state.shape = from;
                ui.relayout();
                settle(&mut ui, 3.0);
                let stage = ui.node("morph/stage").expect("stage").rect;
                let mut frames = vec![shot(&mut o, &mut ui, stage)];
                send(&mut ui, Msg::MorphTo(to));
                let t0 = ui.now();
                let times = [t0 + l, t0 + l + d * 0.15, t0 + l + d * 0.35, t0 + l + d * 0.55, t0 + l + d * 0.8, t0 + l + d + t + 0.05];
                for (k, &at) in times.iter().enumerate() {
                    settle(&mut ui, at);
                    let f = shot(&mut o, &mut ui, stage);
                    if theme == 0 { save_png(&format!("{out}/frame-{name}-{mname}-{}.png", k + 1), f.0, f.1, &f.2); }
                    frames.push(f);
                }
                let (w, h, px) = sheet(&frames, 4);
                let tname = if theme == 0 { "dark" } else { "light" };
                save_png(&format!("{out}/morph-{name}-{mname}-{tname}.png"), w, h, &px);
                eprintln!("morph {name} {mname} {tname}");
            }
        }
    }

    // ── Page change ─────────────────────────────────────────────────────────────────────────────
    for (mi, mname) in ["plain", "dot"].iter().enumerate() {
        let mut ui = gallery::new_ui(faces(), W, H, 1.0);
        ui.pointer_leave();
        send(&mut ui, Msg::Go(Page::Flow));
        send(&mut ui, Msg::Material(mi));
        settle(&mut ui, 3.0);
        let flow = ui.state.flow.flow_ms / 1000.0;
        let whole = amenui::ui::Rect::new(0.0, 0.0, W as f32, H as f32);
        for (dir, key) in [("open", "flow/card4"), ("back", "flow/back")] {
            let (x, y) = ui.node(key).expect("card").rect.center();
            ui.press(x, y);
            ui.release(x, y);
            ui.pointer_leave();
            let t0 = ui.now();
            let mut frames = vec![];
            for k in 0..5 {
                settle(&mut ui, t0 + flow * k as f32 / 4.0);
                frames.push(shot(&mut o, &mut ui, whole));
            }
            let (w, h, px) = sheet(&frames, 3);
            save_png(&format!("{out}/flow-{dir}-{mname}.png"), w, h, &px);
            for (k, f) in frames.iter().enumerate() { save_png(&format!("{out}/frame-flow-{dir}-{mname}-{k}.png"), f.0, f.1, &f.2); }
            settle(&mut ui, t0 + 3.0);
            eprintln!("flow {dir} {mname}");
        }
    }
}

/// The ripple strips (see the top of this file).
fn ripple(out: &str, faces: impl Fn() -> Vec<amenui::text::Face>) {
    let scale = 2.0;
    let (w, h) = (1280u32, 1200u32);
    let mut o = Offscreen::new(w * 2, h * 2);
    let fresh = |scope: usize| {
        let mut ui = gallery::new_ui(faces(), w * 2, h * 2, scale);
        ui.pointer_leave();
        send(&mut ui, Msg::Go(Page::from_hash("button").expect("button page")));
        ui.state.rip_scope = scope;
        ui.relayout();
        // The section in view.
        let y = ui.node("ripple/area").expect("ripple area").rect.y;
        ui.wheel(640.0, 400.0, y - 120.0);
        ui.pointer_leave();
        settle(&mut ui, 3.0);
        ui
    };
    let crop = |o: &mut Offscreen, ui: &mut GalleryUi, r: amenui::ui::Rect| {
        let look = ui.state.look();
        let px = o.render(ui, &look);
        let (x0, y0) = ((r.x * scale).max(0.0) as u32, (r.y * scale).max(0.0) as u32);
        let (x1, y1) = (((r.right() * scale) as u32).min(o.w), ((r.bottom() * scale) as u32).min(o.h));
        let mut f = Vec::with_capacity(((x1 - x0) * (y1 - y0) * 4) as usize);
        for y in y0..y1 { f.extend_from_slice(&px[((y * o.w + x0) * 4) as usize..((y * o.w + x1) * 4) as usize]); }
        (x1 - x0, y1 - y0, f)
    };
    let press = |ui: &mut GalleryUi, key: &str| {
        let (x, y) = ui.node(key).expect("node").rect.center();
        ui.press(x, y);
        ui.release(x, y);
        ui.pointer_leave();
    };
    // (a) The counter 1 → 2: the digit recomposes, nothing else moves.
    let mut ui = fresh(0);
    let digit = ui.node("ripple/count").expect("count").rect;
    let r = amenui::ui::Rect::new(digit.x - 120.0, digit.y - 14.0, 220.0, digit.h + 28.0);
    let d = ui.state.recompose.duration_ms / 1000.0;
    press(&mut ui, "ripple/counter");
    let t0 = ui.now();
    let mut frames = vec![];
    for k in 0..8 { settle(&mut ui, t0 + d * k as f32 / 7.0); frames.push(crop(&mut o, &mut ui, r)); }
    let (sw, sh, px) = sheet(&frames, 4);
    save_png(&format!("{out}/a-counter-1-2.png"), sw, sh, &px);
    eprintln!("a: counter {} → {}", 1, ui.state.rip_count);
    // (b) Ten presses in 300 ms (the scope is the row, so the neighbours show the fronts too).
    let mut ui = fresh(1);
    let area = ui.node("ripple/area").expect("area").rect;
    press(&mut ui, "ripple/burst");
    let t0 = ui.now();
    let mut frames = vec![];
    for ms in [0.0, 100.0, 200.0, 300.0, 500.0, 800.0] { settle(&mut ui, t0 + ms / 1000.0); frames.push(crop(&mut o, &mut ui, area)); }
    let (sw, sh, px) = sheet(&frames, 2);
    save_png(&format!("{out}/b-ten-presses.png"), sw, sh, &px);
    eprintln!("b: ten presses, count {}", ui.state.rip_count);
    // (c) One press, the three responses side by side as the front passes them.
    let mut ui = fresh(1);
    let n0 = ui.node("ripple/n0").expect("n0").rect;
    let n2 = ui.node("ripple/n2").expect("n2").rect;
    let r = amenui::ui::Rect::new(n0.x - 24.0, n0.y - 24.0, n2.right() - n0.x + 48.0, n0.h + 48.0);
    press(&mut ui, "ripple/counter");
    let t0 = ui.now();
    let mut frames = vec![];
    for ms in [0.0, 200.0, 300.0, 400.0, 550.0, 900.0] { settle(&mut ui, t0 + ms / 1000.0); frames.push(crop(&mut o, &mut ui, r)); }
    let (sw, sh, px) = sheet(&frames, 3);
    save_png(&format!("{out}/c-three-responses.png"), sw, sh, &px);
    eprintln!("c: three responses");
}

/// Frame time with a morph at the particle cap, against the same page without one.
fn perf(o: &mut Offscreen, faces: Vec<amenui::text::Face>) {
    let mut ui = gallery::new_ui(faces, W, H, 1.0);
    ui.pointer_leave();
    send(&mut ui, Msg::Go(Page::Morph));
    // A fine lattice, so the picture alone would be more than the cap.
    ui.state.demo.fx.pitch = 2.0;
    ui.state.shape = 2;
    ui.relayout();
    settle(&mut ui, 3.0);
    let look = ui.state.look();
    let time = |o: &mut Offscreen, ui: &mut GalleryUi, n: usize| {
        let start = Instant::now();
        for _ in 0..n { o.draw(ui, &look); }
        start.elapsed().as_secs_f64() * 1000.0 / n as f64
    };
    time(o, &mut ui, 2);
    let base = time(o, &mut ui, 10);
    send(&mut ui, Msg::MorphTo(0));
    let run = ui.morph_run().expect("morph");
    let (count, tile, cap, at) = (run.particles.len(), run.tile, run.spec.count_max, run.flight.expect("flight")[0] + 0.3);
    settle(&mut ui, at);
    time(o, &mut ui, 2);
    let flying = time(o, &mut ui, 10);
    eprintln!("perf: {count} tiles (cap {cap}, tile {tile} px at lattice 2 px), frame {flying:.1} ms in flight vs {base:.1} ms without");
}
