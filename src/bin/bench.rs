//! CPU timings without a GPU (DESIGN.md §1.13): update / layout and scene build for
//!  1. the gallery's button page, pressing the button (each press is a message and a new view);
//!  2. a list of 10,000 rows (text and a small switch), scrolled by the wheel;
//!  3. a 5,000-character article flowing around a disc that moves (every step reflows).
//!
//!  4. particles (DESIGN.md §1.10): a morph between two boxes sized for about 50k, 200k and 1M tiles at
//!     1280 × 800, a full frame mid-flight (simulation step, stage 1, material) on the GPU, and the CPU
//!     path at its 40k cap for comparison. Uses the GPU (lavapipe here: compare rows, not machines).
//!
//!  5. drawing only what changed (DESIGN.md §1.19): the gallery at 1280 × 800, the mean GPU frame (draw
//!     and wait) of a hover onto a button, the caret blinking in a field, and one press of the counter,
//!     drawn whole and drawn in boxes, plain and dot; with the share of the window the boxes cover.
//!
//!  6. the field map (DESIGN.md §1.29) at 1280 × 800: one step of 1/60 s with a trail being laid (the
//!     stir, the step, the bytes), and the bytes' upload to a texture and its wait (budget 0.2 ms).
//!
//! Each line is the mean over the runs, ms. Numbers are only comparable on one machine.
//!
//! usage: bench [runs] [gpu | dirty | map]     (`gpu` runs only the particle rows, `dirty` only 5, `map` only 6)
use lab::el::{El, Paint, col, row, space, text, text_lh};
use lab::gallery::{self, Msg, Page};
use lab::ui::Ui;
use lab::wrap::{Around, Silhouette};
use std::time::Instant;

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled("assets").expect("fonts") }

fn time(n: usize, mut f: impl FnMut(usize)) -> f64 {
    let t0 = Instant::now();
    for i in 0..n { f(i); }
    t0.elapsed().as_secs_f64() * 1000.0 / n as f64
}

// ── A plain list of 10,000 rows: every row is in the tree ────────────────────────────────────────
#[derive(Default)]
struct Long { on: Vec<bool> }
fn long_view(s: &Long) -> El<usize> {
    let mut list = col().key("long").fill_width().fill_height().scroll().bar();
    for i in 0..10_000 {
        let on = s.on.get(i).copied().unwrap_or(i % 3 == 0);
        list = list.child(row().key(format!("r{i}")).height(32.0).pad(12.0, 0.0).gap(12.0).align(taffy::AlignItems::CENTER)
            .child(text(format!("第 {i} 行 · row {i}"), 14.0, Paint::TEXT).grow(1.0))
            .child(space(30.0, 16.0).radius(8.0).fill(if on { Paint::ACCENT } else { Paint::FAINT }).on_press(i)));
    }
    col().key("root").child(list)
}

// ── An article around a moving disc ──────────────────────────────────────────────────────────────
struct Article { x: f32 }
fn article_text() -> String {
    let unit = "文字绕开障碍物的真实轮廓排，每一行只占用这一条带子里没有被挡住的部分。Text flows around the true outline, not its box. ";
    let mut s = String::new();
    while s.chars().count() < 5000 { s.push_str(unit); }
    s.chars().take(5000).collect()
}
thread_local! { static ARTICLE: String = article_text(); }
fn article_view(s: &Article) -> El<()> {
    let a = Around::new(vec![Silhouette::circle(s.x, 220.0, 90.0)], 8.0);
    col().key("root").pad(24.0, 24.0).child(ARTICLE.with(|t| text_lh(t.clone(), 15.0, 24.0, Paint::TEXT).key("body").fill_width().around(a)))
}

// ── Particles: two boxes in one place, morphing ─────────────────────────────────────────────────
struct Pair { w: f32, h: f32, gpu: bool }
fn pair_view(s: &Pair) -> El<()> {
    lab::el::stack([space(s.w, s.h).fill(Paint::INK).key("a"), space(s.w, s.h).radius(40.0).fill(Paint::ACCENT).key("b").hidden_until_morphed(true)])
        .key("root").fill_width().fill_height()
}
fn particles(runs: usize) {
    let (w, h) = (1280u32, 800u32);
    let Ok(mut o) = lab::offscreen::Offscreen::try_new(w, h) else { println!("particles               (no GPU adapter)"); return };
    println!("particles               tier: {}", o.r.particle_tier().name());
    for (n, bw, bh, gpu) in [(40_000usize, 176.0, 176.0, false), (50_000, 196.0, 196.0, true), (200_000, 410.0, 420.0, true), (1_000_000, 1224.0, 770.0, true)] {
        let mut ui = Ui::new(Pair { w: bw, h: bh, gpu }, pair_view, |_, _| {}, fonts(), w, h, 1.0)
            .on_look(|s| lab::style::Look { particles: lab::morph::Particles { gpu_above: if s.gpu { 0.0 } else { 1e9 }, ..Default::default() }, ..Default::default() });
        ui.tick(1.0);
        ui.set_particle_gpu(gpu);
        ui.morph("a", "b", lab::morph::MorphSpec { lattice: 1.0, count_max: n as f32, duration_ms: 60_000.0, loosen_ms: 0.0, ..Default::default() });
        let look = ui.look();
        let mut t = 1.2;
        ui.tick(t);
        o.draw(&mut ui, &look);
        let ms = time(runs.min(20), |_| { t += 1.0 / 60.0; ui.tick(t); o.draw(&mut ui, &look); });
        let count = ui.morph_run().map_or(0, |r| r.grid.map_or(r.particles.len() as u32, |g| g.count()));
        println!("particles {:>7} {:<5}{ms:8.3} ms   ({count} tiles, a frame mid-flight)", n, if gpu { "gpu" } else { "cpu" });
    }
}

fn main() {
    let runs: usize = std::env::args().nth(1).and_then(|v| v.parse().ok()).unwrap_or(40);
    if std::env::args().nth(2).is_some_and(|a| a == "gpu") { particles(runs); return; }
    if std::env::args().nth(2).is_some_and(|a| a == "dirty") { dirty(); return; }
    if std::env::args().nth(2).is_some_and(|a| a == "lab") { lab(runs); return; }
    if std::env::args().nth(2).is_some_and(|a| a == "map") { fieldmap(runs); return; }
    let (w, h) = (1280u32, 800u32);

    // 1. Gallery button page: press the live button.
    let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("button").expect("button page")));
    ui.relayout();
    ui.tick(1.0);
    let k = ui.nodes().iter().find(|n| n.pressable() && n.key.contains("live")).map(|n| n.key.clone()).expect("the live button");
    let (x, y) = ui.node(&k).expect("button").rect.center();
    let ms = time(runs, |i| { let t = 1.0 + i as f32 * 0.02; ui.tick(t); ui.press(x, y); ui.release(x, y); });
    println!("button page, press      {ms:8.3} ms   (update, view, layout and scene per press)");

    // 2. 10,000 rows, the wheel.
    let mut ui = Ui::new(Long::default(), long_view, |s: &mut Long, i| { if s.on.len() <= i { s.on.resize(i + 1, false); } s.on[i] = !s.on[i]; }, fonts(), w, h, 1.0);
    ui.tick(1.0);
    let ms = time(runs, |i| { ui.wheel(400.0, 400.0, if (i / 20) % 2 == 0 { 120.0 } else { -120.0 }); });
    println!("10k plain rows, wheel   {ms:8.3} ms   (per wheel step)");
    let first = time(3, |i| { ui.state.on.clear(); ui.state.on.resize(i + 1, true); ui.relayout(); });
    println!("10k plain rows, relayout{first:8.3} ms   (a new view of all rows)");
    gallery_long(runs);

    // 3. The article: the disc moves, everything reflows.
    let mut ui = Ui::new(Article { x: 200.0 }, article_view, |_, _| {}, fonts(), w, h, 1.0);
    ui.tick(1.0);
    let ms = time(runs.min(30), |i| { ui.state.x = 160.0 + (i % 40) as f32 * 18.0; ui.relayout(); });
    println!("5000-char article reflow{ms:8.3} ms   (the obstacle moves; layout, line breaks, scene)");
    particles(runs);
    dirty();
}

/// 5. Whole frames against frames drawn in boxes.
fn dirty() {
    let (w, h) = (1280u32, 800u32);
    let Ok(mut o) = lab::offscreen::Offscreen::try_new(w, h) else { println!("dirty boxes             (no GPU adapter)"); return };
    for material in [lab::style::Material::Plain, lab::style::Material::Dot] {
        for (case, page) in [("hover a button", "button"), ("caret blink", "input"), ("counter press", "button")] {
            let mut row = vec![];
            for partial in [false, true] {
                o.r.partial = partial;
                let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
                gallery::update(&mut ui.state, Msg::Go(Page::from_hash(page).expect("page")));
                gallery::update(&mut ui.state, Msg::Material(material as usize));
                ui.relayout();
                ui.pointer_leave();
                let mut t = 3.0;
                ui.tick(t);
                let look = ui.state.look();
                o.draw(&mut ui, &look);
                let target = if page == "input" { ui.nodes().iter().find(|n| n.editable()).map(|n| n.key.clone()) } else { ui.nodes().iter().find(|n| n.pressable() && n.key.contains("live")).map(|n| n.key.clone()) };
                let (x, y) = target.as_deref().and_then(|k| ui.node(k)).map_or((640.0, 400.0), |n| n.rect.center());
                match case {
                    "hover a button" => ui.pointer_move(x, y),
                    "caret blink" => { ui.press(x, y); ui.release(x, y); ui.pointer_leave(); for _ in 0..90 { t += 1.0 / 60.0; ui.tick(t); let l = ui.state.look(); o.draw(&mut ui, &l); } }
                    _ => { ui.press(x, y); ui.release(x, y); }
                }
                let (mut ms, mut n, mut area) = (0.0, 0, 0.0);
                for _ in 0..if case == "caret blink" { 160 } else { 90 } {
                    t += 1.0 / 60.0;
                    ui.tick(t);
                    if !ui.needs_frame(t) { continue; }
                    let look = ui.state.look();
                    let t0 = Instant::now();
                    o.draw(&mut ui, &look);
                    ms += t0.elapsed().as_secs_f64() * 1000.0;
                    n += 1;
                    area += o.r.last_region.map_or(1.0, |(_, b)| (b[2] * b[3]) as f64 / (w * h) as f64);
                }
                row.push((ms / n.max(1) as f64, n, area / n.max(1) as f64));
            }
            println!("dirty {:<6} {case:<15} whole {:8.2} ms   boxes {:8.2} ms   ({} frames, boxes cover {:.1}% of the window)",
                format!("{material:?}").to_lowercase(), row[0].0, row[1].0, row[1].1, row[1].2 * 100.0);
        }
    }
    o.r.partial = true;
}

/// The gallery's own 10,000-row page, if there is one: scrolled by the wheel.
fn gallery_long(runs: usize) {
    let Some(p) = Page::from_hash("list-10k") else { println!("10k virtual rows        (no such page)"); return };
    let mut ui = gallery::new_ui(fonts(), 1280, 800, 1.0);
    gallery::update(&mut ui.state, Msg::Go(p));
    ui.relayout();
    ui.tick(1.0);
    let ms = time(runs, |i| { ui.wheel(700.0, 450.0, if (i / 20) % 2 == 0 { 120.0 } else { -120.0 }); });
    println!("10k virtual rows, wheel {ms:8.3} ms   (gallery page 一万行, per wheel step)");
}

/// The Lab's pages with the pointer moving over a cell: the CPU cost of a frame (tick: update, view,
/// layout, scene), no GPU (DESIGN.md §5.3).
fn lab(runs: usize) {
    use lab::gallery::{self, Msg, Page};
    let faces = lab::text::load_bundled("assets").expect("fonts");
    let mut ui = gallery::new_ui(faces, 1280, 900, 1.0);
    for p in std::iter::once(Page::Lab).chain((0..lab::gallery::lab::COUNT as u8).map(Page::Experiment)) {
        gallery::update(&mut ui.state, Msg::Go(p));
        ui.relayout();
        let key = if let Page::Experiment(i) = p { format!("lab/{}", lab::gallery::lab::NAMES[i as usize]) } else { "lab/contours".into() };
        let r = ui.node(&key).map(|n| n.rect).unwrap_or(lab::ui::Rect::new(300.0, 200.0, 300.0, 300.0));
        let mut t = ui.now();
        let mut total = 0.0;
        for i in 0..runs {
            t += 1.0 / 60.0;
            ui.pointer_move(r.x + r.w * (0.2 + 0.6 * (i % 30) as f32 / 30.0), r.y + 90.0 + (i % 5) as f32 * 8.0);
            let t0 = std::time::Instant::now();
            ui.tick(t);
            total += t0.elapsed().as_secs_f64() * 1000.0;
        }
        let t0 = std::time::Instant::now();
        for _ in 0..10 { ui.relayout(); }
        let rl = t0.elapsed().as_secs_f64() * 100.0;
        let t0 = std::time::Instant::now();
        for _ in 0..10 { let _ = (lab::gallery::view)(&ui.state); }
        let vw = t0.elapsed().as_secs_f64() * 100.0;
        println!("{:<16} {:6.2} ms/frame (CPU: input, update, view, layout, scene); one relayout {rl:.2} ms, of it the view {vw:.2} ms; {} nodes", p.hash(), total / runs as f64, ui.nodes().len());
    }
}

/// 6. The field map: a step with a trail being laid, then the upload.
fn fieldmap(runs: usize) {
    use lab::fieldmap::{EVERYWHERE, FieldMap, MapFx};
    let fx = MapFx::default();
    let mut m = FieldMap::new(1280.0, 800.0, fx.cell);
    let n = runs.max(200);
    let step = time(n, |i| {
        let x = 100.0 + (i % 100) as f32 * 10.0;
        m.stir_along([x, 400.0], [x + 10.0, 404.0], fx.stirr, fx.flow, fx.stirr_radius + 48.0, EVERYWHERE);
        m.advance(1.0 / 60.0, &fx);
    });
    let (cols, rows) = (m.cols, m.rows);
    println!("field map step          {step:8.3} ms   ({cols} × {rows} nodes, {} not zero; stir, step, bytes)", m.nonzero());
    let Ok(o) = lab::offscreen::Offscreen::try_new(64, 64) else { println!("field map upload        (no GPU adapter)"); return };
    let (device, queue) = (o.r.device(), o.r.queue());
    let tex = device.create_texture(&wgpu::TextureDescriptor { label: None, size: wgpu::Extent3d { width: 512, height: 512, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format: wgpu::TextureFormat::Rgba8Unorm, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[] });
    let bytes = m.snapshot().bytes.clone();
    let up = time(n, |_| {
        queue.write_texture(wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All }, &bytes,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(cols as u32 * 4), rows_per_image: Some(rows as u32) }, wgpu::Extent3d { width: cols as u32, height: rows as u32, depth_or_array_layers: 1 });
        queue.submit([]);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    println!("field map upload        {up:8.3} ms   ({} KB a frame; step + upload {:.3} ms, budget 0.2)", bytes.len() / 1024, step + up);
}
