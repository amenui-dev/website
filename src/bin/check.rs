//! GPU acceptance for DESIGN.md §1.6.1, §1.6.2, §1.6.3, §1.7 and §1.10, by reading pixels back. Exits non-zero on failure.
//!  1. Light theme, a fully loosened picture: no pixel inside the picture equals the page ground.
//!  2. Pitch 1 at any looseness: the picture is the original, within 1/255.
//!  3. Pixel faces at 1× and 2×: inside their text, only the ink colour and the ground colour.
//!  4. Morphs (§1.10), plain and dot: the hand-over frame equals the source at looseness 1 and the
//!     landing frame the target at looseness 1, within 2/255 over each element; mid-flight, under the
//!     light theme, tiles are drawn outside and between the two elements.
//!
//!  5. Rest (step 5): for every material but chrome, liquid and near, body text at looseness 0 is drawn
//!     exactly as `plain` draws it (print: within its paper grain; dot: the glyphs' own pixels, since
//!     its ground has units at rest); with a control that loosened text
//!     does differ.
//!  6. Every material, light theme: a fully loosened picture shows no page-ground pixel.
//!  7. Annotation frames lie on the layout rects to the pixel.
//!  8. Text around an outline (`wrap`), light theme: no glyph ink pixel within `gap - 1` px of the
//!     obstacle's ink (a disc and the cut-out bust, several gaps), with a control that ignores the gap.
//!  9. Every showcase scene at rest is calm: no instance outside its declared live area is loose,
//!     still condensing, or under a front.
//! 10. Every component page under every material but `near`, at rest (dark theme; light too for print
//!     and lino): the glyph pixels of the preview and the variants equal plain's, except by as much as
//!     the backing under them differs (an xray fill, a collage edge, the dot ground; print within its
//!     grain). The glyph pixels are found by drawing the same frame with the glyphs hidden.
//! 11. Selected and unselected states of the switch, the tabs and the choice (both kinds) differ by a
//!     visible amount under every material.
//!
//! 12. The material pass at half resolution (`Look::quality` 0.5, the reduced pass forced on): every
//!     material at rest equals its quality-1 frame exactly, and body text equals plain's as in 5; with
//!     a control that loosened text does differ between the two resolutions.
//!
//! 13. Recomposition (§1.16), plain and dot: when a keyed number changes, the first frame equals a plain
//!     render of the old text and the last frame of the flight a plain render of the new one, exactly
//!     (the hand-over to the real glyphs is invisible); mid-flight the frame differs from both.
//!
//! 14. 3D (§1.18): a mesh drawn again after turning it by 0, and by a new renderer, is the same frame
//!     byte for byte; at rest, at resolution 0.5 and 1, its silhouette is one unbroken run on every row
//!     it spans (the object is convex) and, under the `near` material, every crease the CPU projects
//!     has the outline drawn along it (with a control: a crease threshold above every angle draws none).
//!
//! 15. Drawing only what changed (§1.19): on the button page, through a hover, a press with its front
//!     and the counter recomposing, under every material, each frame drawn with dirty boxes equals the
//!     same frame drawn whole, byte for byte (with a control: the boxes were smaller than the window).
//!
//! 16. Presets (§4.1): for all twelve, the frame after the target's transition equals a fresh render of
//!     the page in the target preset, byte for byte; mid-transition it differs from both ends (but for
//!     Normcore, which has none).
//!
//! usage: check [components | recompose | mesh | dirty | presets]     (`components` runs only 10 and 11, `recompose` only 13, `mesh` only 14, `dirty` only 15)
use lab::demo::{BUST, PICTURE_SIZES, SKY, SPHERE, TERRAIN};
use lab::el::{El, Font, Paint, col, picture, row, space, text_lh};
use lab::morph::MorphSpec;
use lab::marks::PALETTES;
use lab::offscreen::Offscreen;
use lab::style::{DARK, Gaze, GazeMode, LIGHT, Look, MATERIAL_COUNT, Material, PicFx, Theme, gaze, set};
use lab::text::{MONO, PIXEL, SANS, load_bundled};
use lab::ui::Ui;

const W: u32 = 700;
const H: u32 = 470;

struct One { id: u32, loose: f32 }
fn one(s: &One) -> El<()> {
    let (w, h) = PICTURE_SIZES[s.id as usize];
    col().pad(20.0, 20.0).child(picture(s.id, h as f32 / w as f32).key("pic").loose(s.loose))
}

fn device_rect(r: lab::ui::Rect, scale: f32) -> (u32, u32, u32, u32) {
    ((r.x * scale).round() as u32, (r.y * scale).round() as u32, (r.right() * scale).round() as u32, (r.bottom() * scale).round() as u32)
}
fn px(img: &[u8], w: u32, x: u32, y: u32) -> [u8; 3] { let o = ((y * w + x) * 4) as usize; [img[o], img[o + 1], img[o + 2]] }
fn close(a: [u8; 3], b: [u8; 3]) -> bool { (0..3).all(|i| (a[i] as i32 - b[i] as i32).abs() <= 1) }
fn rgb8(c: [f32; 3]) -> [u8; 3] { [(c[0] * 255.0).round() as u8, (c[1] * 255.0).round() as u8, (c[2] * 255.0).round() as u8] }

fn main() {
    let mut failed = 0;
    let mut o = Offscreen::new(W, H);
    let fonts = || load_bundled("assets").expect("fonts");
    if std::env::args().nth(1).is_some_and(|a| a == "recompose") {
        let f = recompose(&mut o, &fonts);
        if f > 0 { eprintln!("{f} checks failed"); std::process::exit(1); }
        return eprintln!("all checks passed");
    }
    if std::env::args().nth(1).is_some_and(|a| a == "presets") {
        let f = presets(&fonts);
        if f > 0 { eprintln!("{f} checks failed"); std::process::exit(1); }
        return eprintln!("all checks passed");
    }
    if std::env::args().nth(1).is_some_and(|a| a == "dirty") {
        let f = dirty(&fonts);
        if f > 0 { eprintln!("{f} checks failed"); std::process::exit(1); }
        return eprintln!("all checks passed");
    }
    if std::env::args().nth(1).is_some_and(|a| a == "mesh") {
        let f = meshes(&mut o, &fonts);
        if f > 0 { eprintln!("{f} checks failed"); std::process::exit(1); }
        return eprintln!("all checks passed");
    }
    if std::env::args().nth(1).is_some_and(|a| a == "components") {
        let f = components(&fonts);
        if f > 0 { eprintln!("{f} checks failed"); std::process::exit(1); }
        eprintln!("all checks passed");
        return;
    }

    // ── 1. The page's ground never shows inside a loosened picture ──────────────────────────────────
    let ground = rgb8(LIGHT.ground);
    let mut configs: Vec<(String, PicFx)> = Vec::new();
    for s in 0..set::COUNT as u8 {
        for (pitch, ground_k) in [(8.0, 0.0), (13.0, 0.3), (40.0, 1.0)] {
            for c in 0..4 {
                configs.push((format!("set {} pitch {pitch} ground {ground_k} colour {c}", set::NAMES[s as usize]),
                    PicFx { pitch, ground: ground_k, colour: c as f32, levels: 12.0, spread: 0.5, ..PicFx::with_set(s) }));
            }
        }
        configs.push((format!("set {} relief glow follow", set::NAMES[s as usize]), PicFx { pitch: 12.0, relief: 1.5, glow: 2.0, follow: 1.0, ..PicFx::with_set(s) }));
    }
    for (i, p) in PALETTES.iter().enumerate() {
        configs.push((format!("palette {}", p.0), PicFx { pitch: 6.0, set: set::PIXEL as f32, palette: i as f32, colours: p.1 as f32, spread: 0.6, ..PicFx::default() }));
    }
    configs.push(("regions by hash".into(), PicFx { pitch: 7.0, regions: 5.0, region_by: 0.0, ..PicFx::default() }));
    configs.push(("regions by brightness".into(), PicFx { pitch: 7.0, regions: 5.0, region_by: 1.0, ..PicFx::default() }));
    let mut ui = Ui::new(One { id: SKY, loose: 1.0 }, one, |_, _| {}, fonts(), W, H, 1.0);
    let mut worst = 0usize;
    let mut checked = 0usize;
    for id in [SPHERE, SKY, TERRAIN, BUST] {
        ui.state.id = id;
        ui.relayout();
        ui.tick(100.0);
        let (x0, y0, x1, y1) = device_rect(ui.node("pic").expect("pic").rect, 1.0);
        for (name, f) in &configs {
            let mut look = Look { theme: LIGHT, material: Material::Dot, ..Look::default() };
            look.pics[0] = *f;
            let img = o.render(&mut ui, &look);
            let n = (y0..y1).flat_map(|y| (x0..x1).map(move |x| (x, y))).filter(|&(x, y)| close(px(&img, W, x, y), ground)).count();
            // Control: the same comparison finds the ground in the margin around the picture.
            let margin = (0..H).flat_map(|y| (0..x0).map(move |x| (x, y))).filter(|&(x, y)| close(px(&img, W, x, y), ground)).count();
            if margin == 0 { eprintln!("FAIL control: no ground found outside the picture"); failed += 1; }
            checked += 1;
            if n > 0 { eprintln!("FAIL ground shows: picture {id}, {name}: {n} pixels"); failed += 1; worst = worst.max(n); }
        }
    }
    eprintln!("1. ground rule: {checked} renders, {} failing (worst {worst} px)", failed);

    // ── 2. Pitch 1 is the picture itself, however loose ─────────────────────────────────────────────
    let before = failed;
    let mut max_diff = 0;
    for id in [SKY, BUST] {
        for f in [PicFx::with_set(set::CHARS), PicFx { glow: 2.0, relief: 2.0, follow: 1.0, depth: 10.0, ..PicFx::with_set(set::GLOW) },
                  PicFx { palette: 1.0, colours: 3.0, spread: 1.0, ..PicFx::with_set(set::DITHER) }, PicFx { regions: 4.0, ..PicFx::default() }] {
            let f = PicFx { pitch: 1.0, ..f };
            let mut look = Look { theme: LIGHT, material: Material::Dot, ..Look::default() };
            look.pics[0] = f;
            let mut ui = Ui::new(One { id, loose: 0.0 }, one, |_, _| {}, fonts(), W, H, 1.0);
            ui.tick(100.0);
            ui.pointer_move(300.0, 200.0);
            let (x0, y0, x1, y1) = device_rect(ui.node("pic").expect("pic").rect, 1.0);
            // The lens is part of looseness too; take the original with the pointer away.
            ui.pointer_leave();
            let base = o.render(&mut ui, &look);
            for k in [0.25, 0.5, 0.75, 1.0] {
                ui.state.loose = k;
                ui.relayout();
                ui.pointer_move(300.0, 200.0);
                let img = o.render(&mut ui, &look);
                let mut d = 0;
                for y in y0..y1 { for x in x0..x1 {
                    let (a, b) = (px(&img, W, x, y), px(&base, W, x, y));
                    for c in 0..3 { d = d.max((a[c] as i32 - b[c] as i32).abs()); }
                } }
                max_diff = max_diff.max(d);
                if d > 1 { eprintln!("FAIL pitch 1 differs: picture {id}, set {}, looseness {k}: {d}/255", f.set); failed += 1; }
            }
        }
    }
    // Control: at pitch 8 the same comparison does see the marks.
    {
        let mut look = Look { theme: LIGHT, material: Material::Dot, ..Look::default() };
        look.pics[0] = PicFx::with_set(set::CHARS);
        let mut ui = Ui::new(One { id: BUST, loose: 0.0 }, one, |_, _| {}, fonts(), W, H, 1.0);
        ui.tick(100.0);
        let base = o.render(&mut ui, &look);
        ui.state.loose = 1.0;
        ui.relayout();
        let img = o.render(&mut ui, &look);
        let d = base.iter().zip(&img).map(|(a, b)| (*a as i32 - *b as i32).abs()).max().unwrap_or(0);
        if d <= 1 { eprintln!("FAIL control: pitch 8 looks like the original"); failed += 1; }
    }
    eprintln!("2. pitch 1: largest difference {max_diff}/255, {} failing", failed - before);

    // ── 3. Pixel type edges are ink or ground ───────────────────────────────────────────────────────
    let before = failed;
    for scale in [1.0f32, 2.0] {
        for theme in [DARK, LIGHT] {
            let (w, h) = ((W as f32 * scale) as u32, (H as f32 * scale) as u32);
            let mut o2 = Offscreen::new(w, h);
            let mut ui = Ui::new(theme, type_page, |_, _| {}, fonts(), w, h, scale);
            ui.tick(100.0);
            let look = Look { theme, material: Material::Plain, ..Look::default() };
            let img = o2.render(&mut ui, &look);
            for (key, pixel) in [("fusion12", true), ("fusion24", true), ("mono11", true), ("mono22", true), ("sans", false)] {
                let (x0, y0, x1, y1) = device_rect(ui.node(key).expect("text").rect, scale);
                let mut colours: Vec<[u8; 3]> = Vec::new();
                for y in y0..y1.min(h) { for x in x0..x1.min(w) {
                    let c = px(&img, w, x, y);
                    if !colours.iter().any(|&k| close(k, c)) { colours.push(c); }
                } }
                let ok = colours.len() <= 2 && colours.iter().any(|&c| close(c, rgb8(theme.ground)));
                if pixel && !ok { eprintln!("FAIL pixel type {key} at {scale}×, theme {}: {} colours", theme.name, colours.len()); failed += 1; }
                if pixel { eprintln!("   {key} {scale}× {}: {} colours", theme.name, colours.len()); }
                else { eprintln!("   ({key}, an outline face, has {} colours: anti-aliased, as it should be)", colours.len()); }
            }
        }
    }
    eprintln!("3. pixel type: {} failing", failed - before);

    // ── 4. Morphs hand over without a visible change ────────────────────────────────────────────────
    let before = failed;
    let spec = MorphSpec { loosen_ms: 250.0, duration_ms: 1000.0, ..MorphSpec::default() };
    // Times that are exact in binary, so the clocks of the compared frames agree to the bit.
    let (t0, t1, t2) = (50.0f32, 50.25f32, 51.25f32);
    let landed = t2 - 1.0 / 64.0;
    let mut worst = 0;
    let gpu_ok = o.r.particle_tier() != lab::render::ParticleTier::Cpu;
    if !gpu_ok { eprintln!("   no GPU particle tier here: the GPU path is not checked"); }
    for (pair, name) in PAIR_NAMES.iter().enumerate() {
        // Both paths: the CPU's tiles, and the GPU's simulated grid (every morph above 0 tiles).
        for (material, gpu) in [(Material::Plain, false), (Material::Dot, false), (Material::Plain, true), (Material::Dot, true)] {
            if gpu && !gpu_ok { continue; }
            let look = Look { theme: DARK, material, ..Look::default() };
            let new = |s: Stage| { let mut ui = Ui::new(s, stage, |_, _| {}, fonts(), W, H, 1.0); ui.pointer_leave(); ui };
            let mut moving = Ui::new(Stage { pair, loose: [0.0, 0.0], shown: [true, false] }, stage, |_, _| {}, fonts(), W, H, 1.0)
                .on_look(if gpu { |_: &Stage| Look { particles: lab::morph::Particles { gpu_above: 0.0, ..Default::default() }, ..Look::default() } } else { |_: &Stage| Look::default() });
            moving.pointer_leave();
            moving.set_particle_gpu(gpu);
            moving.tick(t0);
            moving.morph("a", "b", spec);
            if moving.morph_run().is_some_and(|r| r.grid.is_some()) != gpu { eprintln!("FAIL morph {pair}: the {} path was not taken", if gpu { "GPU" } else { "CPU" }); failed += 1; }
            let (ra, rb) = (moving.node("a").expect("a").rect, moving.node("b").expect("b").rect);
            // (a) The hand-over.
            let mut src = new(Stage { pair, loose: [1.0, 0.0], shown: [true, false] });
            src.tick(t1);
            moving.tick(t1);
            if moving.morph_run().and_then(|r| r.clock(t1)) != Some(0.0) { eprintln!("FAIL morph {pair}: not in flight at the hand-over"); failed += 1; }
            let d = diff(&o.render(&mut src, &look), &o.render(&mut moving, &look), ra);
            // (b) The landing.
            let mut dst = new(Stage { pair, loose: [0.0, 1.0], shown: [false, true] });
            dst.tick(landed);
            moving.tick(landed);
            let e = diff(&o.render(&mut dst, &look), &o.render(&mut moving, &look), rb);
            eprintln!("   morph {} {:?} {}: hand-over {d}/255, landing {e}/255", name, material, if gpu { "gpu" } else { "cpu" });
            worst = worst.max(d).max(e);
            if d > 2 || e > 2 { eprintln!("FAIL morph {}: {:?} differs", name, material); failed += 1; }
        }
        // (c) Mid-flight, light theme: tiles outside both elements' rects.
        let look = Look { theme: LIGHT, material: Material::Dot, ..Look::default() };
        let mut moving = Ui::new(Stage { pair, loose: [0.0, 0.0], shown: [true, false] }, stage, |_, _| {}, fonts(), W, H, 1.0);
        moving.pointer_leave();
        moving.tick(t0);
        moving.morph("a", "b", spec);
        let (ra, rb) = (moving.node("a").expect("a").rect, moving.node("b").expect("b").rect);
        moving.tick(t1 + 0.5);
        let img = o.render(&mut moving, &look);
        let ground = rgb8(LIGHT.ground);
        let out = |r: lab::ui::Rect, x: u32, y: u32| (x as f32) < r.x - 16.0 || (x as f32) >= r.right() + 16.0 || (y as f32) < r.y - 16.0 || (y as f32) >= r.bottom() + 16.0;
        // The ground units of the dot material are within about 70/255 of the ground; tiles are much further.
        let far = |c: [u8; 3]| (0..3).any(|i| (c[i] as i32 - ground[i] as i32).abs() > 100);
        let n = (0..H).flat_map(|y| (0..W).map(move |x| (x, y))).filter(|&(x, y)| out(ra, x, y) && out(rb, x, y) && far(px(&img, W, x, y))).count();
        eprintln!("   morph {} mid-flight: {n} tile pixels outside both elements", name);
        if n < 300 { eprintln!("FAIL morph {}: nothing in the air", name); failed += 1; }
        // Control: the same count at rest finds nothing.
        let mut rest = Ui::new(Stage { pair, loose: [0.0, 0.0], shown: [true, false] }, stage, |_, _| {}, fonts(), W, H, 1.0);
        rest.pointer_leave();
        rest.tick(t0);
        let img = o.render(&mut rest, &look);
        let n = (0..H).flat_map(|y| (0..W).map(move |x| (x, y))).filter(|&(x, y)| out(ra, x, y) && out(rb, x, y) && far(px(&img, W, x, y))).count();
        if n > 0 { eprintln!("FAIL control: {n} far pixels outside the elements at rest"); failed += 1; }
    }
    // The GPU path is deterministic for a fixed clock: two runs give the same mid-flight frame.
    if gpu_ok {
        let mut frames = vec![];
        for _ in 0..2 {
            let mut m = Ui::new(Stage { pair: 0, loose: [0.0, 0.0], shown: [true, false] }, stage, |_, _| {}, fonts(), W, H, 1.0)
                .on_look(|_: &Stage| Look { particles: lab::morph::Particles { gpu_above: 0.0, ..Default::default() }, ..Look::default() });
            m.set_particle_gpu(true);
            let mut t = t0;
            m.tick(t);
            m.morph("a", "b", spec);
            m.pointer_move(300.0, 200.0);
            let look = Look { theme: DARK, material: Material::Dot, ..Look::default() };
            while t < t1 + 0.4 { t += 1.0 / 60.0; m.tick(t); let _ = o.render(&mut m, &look); }
            frames.push(o.render(&mut m, &look));
        }
        if frames[0] != frames[1] { eprintln!("FAIL morph: the GPU path gave two different frames for the same clock"); failed += 1; }
    }
    eprintln!("4. morphs: largest difference {worst}/255, {} failing", failed - before);

    // ── 5. Every material is restrained at rest ─────────────────────────────────────────────────────
    let before = failed;
    for theme in [DARK, LIGHT] {
        let mut ui = Ui::new(Rest { loose: 0.0 }, rest, |_, _| {}, fonts(), W, H, 1.0);
        ui.pointer_leave();
        ui.tick(100.0);
        let r = ui.node("body").expect("body").rect;
        let plain = o.render(&mut ui, &Look { theme, material: Material::Plain, ..Look::default() });
        for m in 0..MATERIAL_COUNT as u8 {
            let mat = Material::from_id(m);
            if matches!(mat, Material::Chrome | Material::Liquid | Material::Near | Material::Plain) { continue; }
            let look = Look { theme, material: mat, ..Look::default() };
            let img = o.render(&mut ui, &look);
            // The dot material's ground has its units at rest (§1.6): there only the glyphs' own
            // pixels are compared.
            let d = if mat == Material::Dot { diff_ink(&plain, &img, r, rgb8(theme.ground)) } else { diff(&plain, &img, r) };
            // Print's paper grain moves the ground by at most grain × 2.75 % (7/255 at the default).
            let allowed = if mat == Material::Print { 8 } else { 0 };
            eprintln!("   rest {} {:?}: body differs from plain by {d}/255", theme.name, mat);
            if d > allowed { eprintln!("FAIL rest: {:?} body text is not exact at rest ({d}/255)", mat); failed += 1; }
            if theme.name == DARK.name && !matches!(mat, Material::Collage | Material::Xray) {
                // Control: loosened, the same comparison sees the material.
                ui.state.loose = 1.0; ui.relayout(); ui.tick(100.0);
                let d1 = diff(&plain, &o.render(&mut ui, &look), r);
                ui.state.loose = 0.0; ui.relayout(); ui.tick(100.0);
                if d1 <= allowed { eprintln!("FAIL control: {:?} loosened looks like plain", mat); failed += 1; }
            }
        }
    }
    eprintln!("5. rest: {} failing", failed - before);

    // ── 6. No material shows the ground inside a loosened picture ───────────────────────────────────
    let before = failed;
    let ground = rgb8(LIGHT.ground);
    let mut ui = Ui::new(One { id: BUST, loose: 1.0 }, one, |_, _| {}, fonts(), W, H, 1.0);
    ui.pointer_leave();
    ui.tick(100.0);
    let (x0, y0, x1, y1) = device_rect(ui.node("pic").expect("pic").rect, 1.0);
    for m in 0..MATERIAL_COUNT as u8 {
        for f in [PicFx::default(), PicFx { pitch: 12.0, ground: 0.0, ..PicFx::with_set(set::CHARS) }, PicFx { pitch: 6.0, ..PicFx::with_set(set::STROKE) }] {
            let mut look = Look { theme: LIGHT, material: Material::from_id(m), ..Look::default() };
            look.pics[0] = f;
            // A front mid-way, too, for the materials that move things.
            ui.press(30.0, 30.0); ui.release(30.0, 30.0); ui.tick(100.3);
            let img = o.render(&mut ui, &look);
            let n = (y0..y1).flat_map(|y| (x0..x1).map(move |x| (x, y))).filter(|&(x, y)| close(px(&img, W, x, y), ground)).count();
            if n > 0 { eprintln!("FAIL ground shows in a loosened picture under {:?}: {n} px", Material::from_id(m)); failed += 1; }
            ui.tick(110.0);
        }
    }
    eprintln!("6. pictures under every material: {} failing", failed - before);

    // ── 7. Annotation frames on the layout rects ────────────────────────────────────────────────────
    let before = failed;
    let magenta = [255u8, 0, 255];
    let g = Gaze { mode: GazeMode::All, kinds: gaze::FRAME, density: 20.0, draw_ms: 100.0, colour: [1.0, 0.0, 1.0], only_marked: false };
    let mut ui = Ui::new(g, framed, |_, _| {}, fonts(), W, H, 1.0).on_gaze(|g| *g);
    ui.pointer_leave();
    ui.tick(5.0);
    let look = Look { theme: DARK, material: Material::Plain, ..Look::default() };
    let img = o.render(&mut ui, &look);
    let mut frames = 0;
    for a in ui.annots() {
        let (x0, y0, x1, y1) = device_rect(a.rect, 1.0);
        let corners = [(x0, y0), (x1 - 1, y0), (x0, y1 - 1), (x1 - 1, y1 - 1)];
        let outside = [(x0 - 1, y0), (x0, y0 - 1), (x1, y1 - 1), (x1 - 1, y1)];
        let on = corners.iter().all(|&(x, y)| px(&img, W, x, y) == magenta);
        let off = outside.iter().all(|&(x, y)| px(&img, W, x, y) != magenta);
        if !on || !off { eprintln!("FAIL frame of {} is not on its rect ({on}, {off})", a.key); failed += 1; }
        frames += 1;
    }
    if frames < 4 { eprintln!("FAIL only {frames} frames"); failed += 1; }
    eprintln!("7. annotation frames: {frames} checked, {} failing", failed - before);

    // ── 8. Text keeps its gap from the obstacle's ink ───────────────────────────────────────────────
    let before = failed;
    let ground = rgb8(LIGHT.ground);
    let inked = |img: &[u8]| -> Vec<bool> { (0..W * H).map(|i| { let o = (i * 4) as usize; (0..3).map(|c| (img[o + c] as i32 - ground[c] as i32).abs()).sum::<i32>() > 30 }).collect() };
    let mut cases = 0;
    for (kind, x, y, gap) in [(0u8, 220.0, 60.0, 12.0), (0, 40.0, 140.0, 6.0), (1, 300.0, 40.0, 16.0), (1, 120.0, 180.0, 24.0), (0, 380.0, 90.0, 0.0)] {
        for control in [false, true] {
            let st = Wr { kind, x, y, gap: if control { -1000.0 } else { gap }, show: 0 };
            let mut ui = Ui::new(st, wrapped, |_, _| {}, fonts(), W, H, 1.0);
            ui.pointer_leave();
            ui.tick(50.0);
            let look = Look { theme: LIGHT, material: Material::Plain, ..Look::default() };
            ui.state.show = 1; ui.relayout(); ui.tick(50.0);
            let obstacle = inked(&o.render(&mut ui, &look));
            ui.state.show = 2; ui.relayout(); ui.tick(50.0);
            let glyphs = inked(&o.render(&mut ui, &look));
            let r = (gap.max(1.0) as i32) + 1;
            let mut nearest = f32::INFINITY;
            for py in 0..H as i32 { for px_ in 0..W as i32 {
                if !glyphs[(py * W as i32 + px_) as usize] { continue; }
                for dy in -r..=r { for dx in -r..=r {
                    let (qx, qy) = (px_ + dx, py + dy);
                    if qx < 0 || qy < 0 || qx >= W as i32 || qy >= H as i32 || !obstacle[(qy * W as i32 + qx) as usize] { continue; }
                    nearest = nearest.min(((dx * dx + dy * dy) as f32).sqrt());
                } }
            } }
            if control {
                if nearest > 1.5 { eprintln!("FAIL control (kind {kind} at {x},{y}): text ignoring the obstacle is {nearest} px from it"); failed += 1; }
            } else {
                cases += 1;
                // Distances between pixel centres; an edge pixel counts as ink from either side.
                if nearest < gap - 1.0 { eprintln!("FAIL kind {kind} at {x},{y}: glyph ink {nearest:.1} px from the obstacle, gap {gap}"); failed += 1; }
            }
        }
    }
    eprintln!("8. text around outlines: {cases} cases, {} failing", failed - before);

    // ── 9. Scenes at rest are calm ─────────────────────────────────────────────────────────────────
    let before = failed;
    for (i, live) in [(0usize, None), (1, None), (2, Some("pr/wall")), (3, None), (4, None), (5, None), (6, None)] {
        let mut ui = lab::gallery::new_ui(fonts(), 1280, 800, 1.0);
        ui.pointer_leave();
        lab::gallery::update(&mut ui.state, lab::gallery::Msg::Go(lab::gallery::Page::Scene(i as u8)));
        ui.relayout();
        let mut t = 0.0;
        while t < 6.0 { t += 1.0 / 30.0; ui.tick(t); }
        let fx = ui.state.look().fx;
        let allowed: Vec<_> = live.map(|k| ui.instances_of(k)).unwrap_or_default();
        let loose = ui.scene().instances.iter().filter(|i| !allowed.contains(i))
            .filter(|i| i.extra[2] > 0.0 || i.extra[3] + fx.enter > t).count();
        let fronts = ui.field().fronts.iter().filter(|f| f[3] > 0.0 && t - f[2] < fx.wave_life).count();
        if loose > 0 || fronts > 0 { eprintln!("FAIL scene {i} at rest: {loose} loose instances, {fronts} fronts"); failed += 1; }
    }
    // Control: the same count sees a held picture as loose.
    {
        let mut ui = lab::gallery::new_ui(fonts(), 1280, 800, 1.0);
        lab::gallery::update(&mut ui.state, lab::gallery::Msg::Go(lab::gallery::Page::Scene(2)));
        lab::gallery::update(&mut ui.state, lab::gallery::Msg::Num(lab::gallery::Target::Scene(2, 3), lab::widgets::Edit::Set(1.0)));
        ui.relayout();
        ui.tick(6.0);
        if !ui.scene().instances.iter().any(|i| i.extra[2] > 0.0) { eprintln!("FAIL control: a held picture is not seen as loose"); failed += 1; }
    }
    eprintln!("9. scenes at rest: {} failing", failed - before);

    failed += components(&fonts);

    // ── 12. The material at half resolution leaves rest pixels exact ────────────────────────────────
    let before = failed;
    let mut ui = Ui::new(Rest { loose: 0.0 }, rest, |_, _| {}, fonts(), W, H, 1.0);
    ui.pointer_leave();
    ui.tick(100.0);
    let r = ui.node("body").expect("body").rect;
    let plain = o.render(&mut ui, &Look { material: Material::Plain, ..Look::default() });
    let mut worst = 0;
    for m in 0..MATERIAL_COUNT as u8 {
        let mat = Material::from_id(m);
        let full = o.render(&mut ui, &Look { material: mat, ..Look::default() });
        // The reduced pass is drawn even though nothing is loose, so the upsample has to choose.
        o.r.always_lo = true;
        let half = o.render(&mut ui, &Look { material: mat, quality: 0.5, ..Look::default() });
        let whole = diff(&full, &half, lab::ui::Rect::new(0.0, 0.0, W as f32, H as f32));
        worst = worst.max(whole);
        if whole > 0 { eprintln!("FAIL quality 0.5: {:?} at rest differs from quality 1 by {whole}/255", mat); failed += 1; }
        if !matches!(mat, Material::Chrome | Material::Liquid | Material::Near) {
            let d = if mat == Material::Dot { diff_ink(&plain, &half, r, rgb8(DARK.ground)) } else { diff(&plain, &half, r) };
            let allowed = if mat == Material::Print { 8 } else { 0 };
            if d > allowed { eprintln!("FAIL quality 0.5: {:?} body text is not plain's at rest ({d}/255)", mat); failed += 1; }
        }
        if matches!(mat, Material::Dot | Material::Halftone | Material::Ascii) {
            // Control: loosened, half resolution is seen.
            ui.state.loose = 1.0; ui.relayout(); ui.tick(100.0);
            let a = o.render(&mut ui, &Look { material: mat, ..Look::default() });
            let b = o.render(&mut ui, &Look { material: mat, quality: 0.5, ..Look::default() });
            ui.state.loose = 0.0; ui.relayout(); ui.tick(100.0);
            if diff(&a, &b, r) == 0 { eprintln!("FAIL control: {:?} loosened at quality 0.5 equals quality 1", mat); failed += 1; }
        }
        o.r.always_lo = false;
    }
    eprintln!("12. quality 0.5 at rest: largest difference {worst}/255, {} failing", failed - before);

    failed += recompose(&mut o, &fonts);
    failed += meshes(&mut o, &fonts);
    failed += dirty(&fonts);
    failed += presets(&fonts);

    if failed > 0 { eprintln!("{failed} checks failed"); std::process::exit(1); }
    eprintln!("all checks passed");
}

/// 13. Recomposition: the first frame is the old text and the last the new one, exactly.
fn recompose(o: &mut Offscreen, fonts: &dyn Fn() -> Vec<lab::text::Face>) -> usize {
    // ── 13. Recomposition hands over without a visible change ──────────────────────────────────────
    let mut failed = 0;
    let mut worst = 0;
    let spec = lab::morph::Recompose::default();
    let (t0, flight) = (50.0f32, spec.duration_ms / 1000.0);
    // Every tile has landed by (1 − 0.05 × stagger) of the flight; the run ends at the flight's end.
    let landed = t0 + flight * (1.0 - 0.025 * spec.stagger);
    for (from, to) in [("19", "20"), ("计数 9", "计数 10"), ("3.5 px", "3.6 px")] {
        for material in [Material::Plain, Material::Dot] {
            let look = Look { theme: DARK, material, ..Look::default() };
            let plain = |s: &str, t: f32| { let mut ui = Ui::new(Num(s.into()), num, |_, _| {}, fonts(), W, H, 1.0); ui.pointer_leave(); ui.tick(t); ui };
            let mut moving = plain(from, t0);
            moving.state.0 = to.into();
            moving.relayout();
            let whole = lab::ui::Rect::new(0.0, 0.0, W as f32, H as f32);
            let d = diff(&o.render(&mut plain(from, t0), &look), &o.render(&mut moving, &look), whole);
            let mid = o.render(&mut { moving.tick(t0 + flight * 0.5); moving }, &look);
            let mut moving = plain(from, t0);
            moving.state.0 = to.into();
            moving.relayout();
            moving.tick(landed);
            let tiles = moving.scene().instances.len() > moving.instances_of("n").len();
            let e = diff(&o.render(&mut plain(to, landed), &look), &o.render(&mut moving, &look), whole);
            let m = diff(&o.render(&mut plain(to, landed), &look), &mid, whole).min(diff(&o.render(&mut plain(from, t0), &look), &mid, whole));
            eprintln!("   recompose {from:?} → {to:?} {material:?}: start {d}/255, landing {e}/255, mid-flight differs by {m}/255");
            worst = worst.max(d).max(e);
            if d != 0 || e != 0 { eprintln!("FAIL recompose {from:?} → {to:?} {material:?}: the hand-over is visible"); failed += 1; }
            if !tiles { eprintln!("FAIL recompose {from:?} → {to:?}: no tiles in flight just before landing"); failed += 1; }
            if m == 0 { eprintln!("FAIL control: recompose {from:?} → {to:?} {material:?} mid-flight equals an end"); failed += 1; }
        }
    }
    eprintln!("13. recomposition hand-over: largest difference {worst}/255, {} failing", failed);
    failed
}

struct Num(String);
fn num(n: &Num) -> El<()> { col().pad(60.0, 60.0).child(row().child(text_lh(n.0.clone(), 56.0, 72.0, Paint::INK).key("n"))) }

const ZH: &str = "像素字体按整数倍显示，边缘只有两种颜色。";
fn type_page(_: &Theme) -> El<()> {
    col().pad(10.0, 10.0).gap(8.0)
        .child(text_lh(ZH, 12.0, 20.0, Paint::TEXT).font(Font::Face(PIXEL)).key("fusion12"))
        .child(text_lh(ZH, 24.0, 32.0, Paint::INK).font(Font::Face(PIXEL)).key("fusion24"))
        .child(text_lh("Departure Mono 0123 ABC xyz", 11.0, 18.0, Paint::TEXT).font(Font::Face(MONO)).key("mono11"))
        .child(text_lh("Departure Mono 0123 ABC xyz", 22.0, 30.0, Paint::DIM).font(Font::Face(MONO)).key("mono22"))
        .child(text_lh(ZH, 16.0, 26.0, Paint::TEXT).font(Font::Face(SANS)).key("sans"))
}

/// Two elements side by side, either of which can be drawn loose or hidden: the source "a" and the
/// target "b" of a morph.
struct Stage { pair: usize, loose: [f32; 2], shown: [bool; 2] }
const PAIRS: usize = 3;
const PAIR_NAMES: [&str; PAIRS] = ["text→picture", "shape→paragraph", "picture→text"];
/// `loose` is not inherited, so it goes on the leaf.
fn element(kind: u8, k: f32) -> El<()> {
    match kind {
        0 => text_lh("变换", 96.0, 120.0, Paint::INK).loose(k),
        1 => { let (w, h) = PICTURE_SIZES[SKY as usize]; col().width(260.0).child(picture(SKY, h as f32 / w as f32).loose(k)) }
        2 => space(160.0, 160.0).radius(80.0).fill(Paint::ACCENT).loose(k),
        _ => col().width(240.0).child(text_lh("屏幕上的任何东西都能散成粒子，飞到别处，变成另一样东西。", 16.0, 28.0, Paint::TEXT).loose(k)),
    }
}
fn stage(s: &Stage) -> El<()> {
    let (a, b) = [(0, 1), (2, 3), (1, 0)][s.pair];
    row().pad(30.0, 60.0).gap(200.0).align(taffy::AlignItems::CENTER)
        .child(element(a, s.loose[0]).key("a").hidden_until_morphed(!s.shown[0]))
        .child(element(b, s.loose[1]).key("b").hidden_until_morphed(!s.shown[1]))
}
/// Largest channel difference over a rect (logical = device px here).
fn diff(a: &[u8], b: &[u8], r: lab::ui::Rect) -> i32 {
    let (x0, y0, x1, y1) = device_rect(r, 1.0);
    let mut d = 0;
    for y in y0..y1.min(H) { for x in x0..x1.min(W) {
        let (p, q) = (px(a, W, x, y), px(b, W, x, y));
        for c in 0..3 { d = d.max((p[c] as i32 - q[c] as i32).abs()); }
    } }
    d
}

/// Body text on the ground, with a heading, a picture and a control around it.
struct Rest { loose: f32 }
fn rest(s: &Rest) -> El<()> {
    let (w, h) = PICTURE_SIZES[SKY as usize];
    col().pad(30.0, 20.0).gap(20.0).loosen(s.loose)
        .child(text_lh("松开与合拢", 52.0, 64.0, Paint::INK).key("head"))
        .child(row().gap(30.0)
            .child(col().width(330.0).child(text_lh("材质决定松开时画什么。静止时，页面是一张干净、可读的页面；波前经过时按轮廓松开。Body text 0123.", 16.0, 27.0, Paint::TEXT).key("body")))
            .child(col().width(260.0).child(picture(SKY, h as f32 / w as f32).key("pic"))))
        .child(space(120.0, 30.0).radius(4.0).fill(Paint::ACCENT))
}
/// Separate elements to frame.
fn framed(_: &Gaze) -> El<()> {
    col().pad(40.0, 40.0).gap(40.0)
        .child(space(200.0, 60.0).fill(Paint::FAINT).key("f1"))
        .child(row().gap(50.0).child(space(90.0, 90.0).radius(45.0).fill(Paint::ACCENT).key("f2")).child(text_lh("标注", 30.0, 40.0, Paint::INK).key("f3")))
        .child(space(310.0, 37.0).border(1.0, Paint::DIM).key("f4"))
}
/// Largest channel difference over the pixels of a rect where `a` is not the ground.
fn diff_ink(a: &[u8], b: &[u8], r: lab::ui::Rect, ground: [u8; 3]) -> i32 {
    let (x0, y0, x1, y1) = device_rect(r, 1.0);
    let mut d = 0;
    for y in y0..y1.min(H) { for x in x0..x1.min(W) {
        let (p, q) = (px(a, W, x, y), px(b, W, x, y));
        if close(p, ground) { continue; }
        for c in 0..3 { d = d.max((p[c] as i32 - q[c] as i32).abs()); }
    } }
    d
}

/// Text flowing around an obstacle: a disc (kind 1) or the cut-out bust (0), at (x, y). `show` 1 draws
/// only the obstacle, 2 only the text (laid out the same either way).
struct Wr { kind: u8, x: f32, y: f32, gap: f32, show: u8 }
const WRAP_TEXT: &str = "文字沿着轮廓排开，每一行只拿到这一条带子里没有被挡住的部分；被挡成两段时，左右各排一行。Text keeps its distance from the ink of whatever is in its way, whatever its shape. ";
fn wrapped(s: &Wr) -> El<()> {
    use lab::wrap::{Around, Silhouette};
    let (ob, sil) = if s.kind == 1 {
        (space(150.0, 150.0).radius(75.0).fill(Paint::ACCENT), Silhouette::circle(s.x + 75.0, s.y + 75.0, 75.0))
    } else {
        let (iw, ih, a) = lab::demo::cutout_alpha();
        let w = 240.0;
        (col().width(w).child(picture(lab::demo::CUTOUT, ih as f32 / iw as f32)), Silhouette::from_alpha(iw, ih, |x, y| a[(y * iw + x) as usize], s.x, s.y, w))
    };
    let alpha = if s.show == 1 { 0.0 } else { 1.0 };
    col().pad(20.0, 20.0).child(col().width(640.0)
        .child(text_lh(WRAP_TEXT.repeat(4), 16.0, 26.0, Paint::INK.alpha(alpha)).fill_width().around(Around::new(vec![sil], s.gap)))
        .child(ob.hidden_until_morphed(s.show == 2).absolute(Some(s.x), Some(s.y), None, None)))
}

/// Checks 10 and 11, on the gallery's component pages.
fn components(fonts: &dyn Fn() -> Vec<lab::text::Face>) -> usize {
    use lab::gallery::{self, COMPONENTS, GalleryUi, Msg, Page, comp};
    use lab::style::MATERIALS;
    let (w, h) = (1280u32, 1100u32);
    let mut o = Offscreen::new(w, h);
    let settle = |ui: &mut GalleryUi, secs: f32| { let to = ui.now() + secs; let mut t = ui.now(); while t < to { t = (t + 1.0 / 30.0).min(to); ui.tick(t); } };
    let send = |ui: &mut GalleryUi, m: Msg| { gallery::update(&mut ui.state, m); ui.relayout(); };
    let page = |c: usize, theme: usize, mat: usize| {
        let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
        ui.pointer_leave();
        send(&mut ui, Msg::Go(Page::Comp(c)));
        send(&mut ui, Msg::Theme(theme));
        send(&mut ui, Msg::Material(mat));
        settle(&mut ui, 1.0);
        // The input focused (its caret shows), the dialog open.
        if c == comp::INPUT || c == comp::DIALOG {
            let key = if c == comp::INPUT { "input/live" } else { "dialog/open" };
            let (x, y) = ui.node(key).expect("control").rect.center();
            ui.press(x, y); ui.release(x, y); ui.pointer_leave();
        }
        settle(&mut ui, 3.0);
        ui
    };
    let mut failed = 0;

    // ── 10. Control labels at rest ──────────────────────────────────────────────────────────────────
    let before = failed;
    let mut worst = 0;
    for (c, cp) in COMPONENTS.iter().enumerate() {
        let hash = cp.hash;
        for theme in [0usize, 1] {
            let mut ui = page(c, theme, 0);
            // Where the controls are: the preview and the variants; for the dialog, its card (the page
            // beneath it is held partly loose on purpose).
            let rects: Vec<lab::ui::Rect> = if c == comp::DIALOG { vec![ui.node("dialog/dialog/card").expect("card").rect] }
                else { [format!("{hash}/preview"), format!("{hash}/variants")].iter().map(|k| ui.node(k).expect("region").rect).collect() };
            let look = ui.state.look();
            let full_p = o.render(&mut ui, &look);
            o.r.hide_glyphs = true;
            let bare_p = o.render(&mut ui, &look);
            o.r.hide_glyphs = false;
            let mut mask = vec![];
            for r in &rects {
                let (x0, y0, x1, y1) = device_rect(*r, 1.0);
                for y in y0..y1.min(h) { for x in x0..x1.min(w) {
                    let (a, b) = (px(&full_p, w, x, y), px(&bare_p, w, x, y));
                    if (0..3).map(|i| (a[i] as i32 - b[i] as i32).abs()).sum::<i32>() > 24 { mask.push((x, y)); }
                } }
            }
            if mask.len() < 200 { eprintln!("FAIL control: only {} glyph pixels on {hash}", mask.len()); failed += 1; }
            for (mi, (mat, mname)) in MATERIALS.iter().enumerate() {
                if matches!(mat, Material::Plain | Material::Near) { continue; }
                if theme == 1 && !matches!(mat, Material::Print | Material::Lino) { continue; }
                let mut ui = page(c, theme, mi);
                let look = ui.state.look();
                let full = o.render(&mut ui, &look);
                o.r.hide_glyphs = true;
                let bare = o.render(&mut ui, &look);
                o.r.hide_glyphs = false;
                let allowed = if *mat == Material::Print { 8 } else { 1 };
                let (mut bad, mut d_max) = (0, 0);
                for &(x, y) in &mask {
                    let (a, b, ba, bb) = (px(&full_p, w, x, y), px(&full, w, x, y), px(&bare_p, w, x, y), px(&bare, w, x, y));
                    let mut over = 0;
                    for i in 0..3 { over = over.max((a[i] as i32 - b[i] as i32).abs() - (ba[i] as i32 - bb[i] as i32).abs() - allowed); }
                    if over > 0 { bad += 1; d_max = d_max.max(over); }
                }
                worst = worst.max(d_max);
                if bad > 0 && let Ok(dir) = std::env::var("CHECK_DUMP") {
                    let mut m = full.clone();
                    for &(x, y) in &mask {
                        let (a, b, ba, bb) = (px(&full_p, w, x, y), px(&full, w, x, y), px(&bare_p, w, x, y), px(&bare, w, x, y));
                        if (0..3).any(|i| (a[i] as i32 - b[i] as i32).abs() - (ba[i] as i32 - bb[i] as i32).abs() - allowed > 0) { let o = ((y * w + x) * 4) as usize; m[o..o + 3].copy_from_slice(&[0, 255, 0]); }
                    }
                    lab::offscreen::save_png(&format!("{dir}/{hash}-{mname}-{theme}.png"), w, h, &m);
                    lab::offscreen::save_png(&format!("{dir}/{hash}-{mname}-{theme}-raw.png"), w, h, &full);
                    lab::offscreen::save_png(&format!("{dir}/{hash}-plain-{theme}.png"), w, h, &full_p);
                }
                if bad > 0 { eprintln!("FAIL rest: {hash} under {mname} ({}): {bad} of {} glyph pixels off by up to {d_max}/255 beyond their backing", THEME_NAMES[theme], mask.len()); failed += 1; }
            }
        }
        eprintln!("   labels at rest: {hash}");
    }
    eprintln!("10. component labels at rest: largest excess {worst}/255, {} failing", failed - before);

    // ── 11. Selected and unselected look different ──────────────────────────────────────────────────
    let before = failed;
    let mut least = usize::MAX;
    for (mi, (mat, mname)) in MATERIALS.iter().enumerate() {
        if *mat == Material::Near { continue; }
        for (c, key, set) in [
            (comp::SWITCH, "switch/live", (|s: &mut gallery::Gallery, k: usize| s.on = k == 1) as fn(&mut gallery::Gallery, usize)),
            (comp::TABS, "tabs/live/head", |s, k| s.tab = k),
            (comp::CHOICE, "choice/live", |s, k| s.pick = k),
            (comp::CHOICE, "choice/live-seg", |s, k| s.pick = k),
        ] {
            let mut shots = vec![];
            let mut rect = None;
            for k in [0usize, 1, 1] {
                let mut ui = page(c, 0, mi);
                set(&mut ui.state, k);
                ui.relayout();
                settle(&mut ui, 3.0);
                let r = ui.node(key).expect("control").rect;
                rect = Some(r);
                let look = ui.state.look();
                shots.push(o.render(&mut ui, &look));
            }
            let (x0, y0, x1, y1) = device_rect(rect.expect("rect"), 1.0);
            let count = |a: &[u8], b: &[u8]| (y0..y1).flat_map(|y| (x0..x1).map(move |x| (x, y)))
                .filter(|&(x, y)| { let (p, q) = (px(a, w, x, y), px(b, w, x, y)); (0..3).any(|i| (p[i] as i32 - q[i] as i32).abs() >= 40) }).count();
            let n = count(&shots[0], &shots[1]);
            least = least.min(n);
            if n < 60 { eprintln!("FAIL states: {key} under {mname}: selected and unselected differ in only {n} px"); failed += 1; }
            // Control: the same state twice is the same.
            let same = count(&shots[1], &shots[2]);
            if same > 0 { eprintln!("FAIL control: {key} under {mname} differs from itself in {same} px"); failed += 1; }
        }
    }
    eprintln!("11. selected states: fewest differing pixels {least}, {} failing", failed - before);
    failed
}
const THEME_NAMES: [&str; 2] = ["dark", "light"];

// ── 14. Meshes ───────────────────────────────────────────────────────────────────────────────────
struct Obj { yaw: f32, scale: f32, crease: f32 }
fn obj_scene(s: &Obj) -> lab::mesh::Scene3d {
    let mut sc = lab::mesh::Scene3d::single(lab::mesh::MeshId::ICO[1]);
    sc.items[0].transforms = vec![lab::mesh::Mat4::rotate_y(s.yaw).mul(&lab::mesh::Mat4::uniform(1.55))];
    sc.spec.resolution_scale = s.scale;
    sc.spec.edge_normal = s.crease;
    sc
}
fn obj(s: &Obj) -> El<()> { col().pad(40.0, 40.0).child(lab::el::scene3d(obj_scene(s)).key("mesh").width(360.0).height(320.0)) }
/// A point through a full 4 × 4 projection to device px of `r`.
fn project(m: &lab::mesh::Mat4, p: [f32; 3], r: lab::ui::Rect) -> (f32, f32) {
    let a = &m.0;
    let c = |row: usize| a[row] * p[0] + a[4 + row] * p[1] + a[8 + row] * p[2] + a[12 + row];
    let w = c(3);
    (r.x + (c(0) / w * 0.5 + 0.5) * r.w, r.y + (0.5 - c(1) / w * 0.5) * r.h)
}
fn meshes(o: &mut Offscreen, fonts: &dyn Fn() -> Vec<lab::text::Face>) -> usize {
    use lab::mesh::{Mesh, dot, sub};
    let mut failed = 0;
    let before = failed;
    // Turning by 0 changes nothing, and a new renderer draws the same.
    let mut ui = Ui::new(Obj { yaw: 20.0, scale: 0.5, crease: 15.0 }, obj, |_, _| {}, fonts(), W, H, 1.0);
    ui.tick(2.0);
    let look = Look { material: Material::Ascii, ..Look::default() };
    let a = o.render(&mut ui, &look);
    ui.state.yaw += 0.0; ui.relayout(); ui.tick(2.1);
    let b = o.render(&mut ui, &look);
    let mut o2 = Offscreen::new(W, H);
    let c = o2.render(&mut ui, &look);
    if a != b || a != c { eprintln!("FAIL mesh: turning by 0 or a new renderer changed the frame"); failed += 1; }
    let ground = rgb8(DARK.ground);
    for scale in [0.5, 1.0] {
        for crease in [15.0, 120.0] {
            let mut ui = Ui::new(Obj { yaw: 20.0, scale, crease }, obj, |_, _| {}, fonts(), W, H, 1.0);
            ui.tick(2.0);
            let r = ui.node("mesh").expect("mesh").rect;
            let plain = o.render(&mut ui, &Look { material: Material::Plain, ..Look::default() });
            // Silhouette: on every row it spans, one run of pixels that are not the ground.
            let (x0, y0, x1, y1) = device_rect(r, 1.0);
            let (mut rows, mut broken) = (0, 0);
            for y in y0..y1 {
                let runs = (x0..x1).fold((0, false), |(n, was), x| { let on = !close(px(&plain, W, x, y), ground); (n + (on && !was) as i32, on) }).0;
                if runs > 0 { rows += 1; }
                if runs > 1 { broken += 1; }
            }
            if rows < 100 || broken > 0 { eprintln!("FAIL mesh silhouette at {scale}: {rows} rows, {broken} with more than one run"); failed += 1; }
            // Creases: project the faces the camera sees; every shared edge between two of them whose
            // normals differ by more than the threshold must have the outline drawn along it.
            let near = o.render(&mut ui, &Look { material: Material::Near, ..Look::default() });
            let sc = obj_scene(&ui.state);
            let lib = lab::mesh::Library::default();
            let mesh = lib.get(lab::mesh::MeshId::ICO[1]).expect("ico");
            let model = sc.items[0].transforms[0];
            let mvp = sc.camera.proj(r.w / r.h).mul(&sc.camera.view());
            let eye = sc.camera.eye();
            let tris: Vec<[[f32; 3]; 3]> = mesh.tris.iter().map(|t| [model.point(t[0]), model.point(t[1]), model.point(t[2])]).collect();
            let facing: Vec<bool> = tris.iter().map(|t| dot(Mesh::normal(t), sub(eye, t[0])) > 0.05).collect();
            let key = |p: [f32; 3]| [(p[0] * 1e4).round() as i64, (p[1] * 1e4).round() as i64, (p[2] * 1e4).round() as i64];
            let mut edges: std::collections::HashMap<([i64; 3], [i64; 3]), Vec<usize>> = Default::default();
            for (i, t) in tris.iter().enumerate() { for k in 0..3 { let (a, b) = (key(t[k]), key(t[(k + 1) % 3])); edges.entry(if a < b { (a, b) } else { (b, a) }).or_default().push(i); } }
            let (mut want, mut found) = (0, 0);
            let mut drawn_any = 0;
            for (i, t) in tris.iter().enumerate() { for k in 0..3 {
                let (pa, pb) = (t[k], t[(k + 1) % 3]);
                let (ka, kb) = (key(pa), key(pb));
                let Some(fs) = edges.get(&if ka < kb { (ka, kb) } else { (kb, ka) }) else { continue };
                let Some(&j) = fs.iter().find(|&&j| j != i) else { continue };
                if j < i || !facing[i] || !facing[j] { continue; }
                let creased = dot(Mesh::normal(t), Mesh::normal(&tris[j])) < (15.0f32).to_radians().cos();
                if !creased { continue; }
                for q in 1..10 {
                    let f = q as f32 / 10.0;
                    let p = [pa[0] + (pb[0] - pa[0]) * f, pa[1] + (pb[1] - pa[1]) * f, pa[2] + (pb[2] - pa[2]) * f];
                    let (sx, sy) = project(&mvp, p, r);
                    want += 1;
                    let mut hit = false;
                    for dy in -2i32..=2 { for dx in -2i32..=2 {
                        let (x, y) = ((sx as i32 + dx).clamp(0, W as i32 - 1) as u32, (sy as i32 + dy).clamp(0, H as i32 - 1) as u32);
                        let c = px(&near, W, x, y);
                        // The outline is the ink colour over the faint face: grey and light (the contour bands
                        // around it are the accents).
                        let (r8, g8, b8) = (c[0] as i32, c[1] as i32, c[2] as i32);
                        if (r8 - g8).abs() < 24 && (g8 - b8).abs() < 24 && r8 + g8 + b8 > 3 * 84 { hit = true; }
                    } }
                    if hit { found += 1; }
                }
                drawn_any += 1;
            } }
            let share = found as f32 / want.max(1) as f32;
            if crease < 90.0 {
                if drawn_any < 10 || share < 0.9 { eprintln!("FAIL mesh creases at {scale}: {found}/{want} samples on an outline over {drawn_any} edges"); failed += 1; }
            } else if share > 0.3 { eprintln!("FAIL control: with no creases the outline is still on {found}/{want} samples at {scale}"); failed += 1; }
            eprintln!("14. mesh at {scale}, crease {crease}°: {rows} rows, {broken} broken; {found}/{want} crease samples on an outline");
        }
    }
    eprintln!("14. meshes: {} failing", failed - before);
    failed
}

// ── 15. Dirty boxes ──────────────────────────────────────────────────────────────────────────────
fn dirty(fonts: &dyn Fn() -> Vec<lab::text::Face>) -> usize {
    use lab::gallery::{self, Msg, Page};
    let (w, h) = (900u32, 640u32);
    // One renderer (it keeps the glyph atlas): each frame drawn in part from the previous one, then the
    // same frame drawn whole to compare (which the next frame then starts from).
    let mut part = Offscreen::new(w, h);
    let mut failed = 0;
    let (mut frames, mut small) = (0, 0);
    // The button page under every material; the image page and the 3D page (the lens over a picture and
    // a mesh) under three.
    let runs: Vec<(&str, usize)> = (0..lab::style::MATERIALS.len()).map(|m| ("button", m)).chain([("image", 0), ("image", 1), ("three", 0), ("three", 1), ("three", 3)]).collect();
    for (page, mi) in runs {
        let (m, name) = lab::style::MATERIALS[mi];
        let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
        gallery::update(&mut ui.state, Msg::Go(Page::from_hash(page).expect("page")));
        gallery::update(&mut ui.state, Msg::Material(mi));
        ui.relayout();
        ui.pointer_leave();
        let k = ui.nodes().iter().find(|n| n.pressable() && n.key.contains("live")).or_else(|| ui.nodes().iter().find(|n| n.key == "three/object" || n.kind() == "image")).map(|n| n.key.clone()).expect("something to press");
        let (x, y) = ui.node(&k).expect("button").rect.center();
        let mut t = 3.0;
        ui.tick(t);
        let mut bad = 0;
        for step in 0..48 {
            t += 1.0 / 60.0;
            ui.tick(t);
            match step { 6 => ui.pointer_move(x, y), 18 => ui.press(x, y), 21 => ui.release(x, y), 34 => ui.pointer_move(x + 200.0, y + 120.0), _ => {} }
            let look = ui.state.look();
            let a = part.render(&mut ui, &look);
            let boxes = part.r.last_region;
            part.r.partial = false;
            ui.invalidate();
            let b = part.render(&mut ui, &look);
            part.r.partial = true;
            frames += 1;
            if let Some((_, b2)) = boxes && (b2[2] as u64) * (b2[3] as u64) < (w as u64 * h as u64) / 2 { small += 1; }
            if a != b {
                let n = a.chunks(4).zip(b.chunks(4)).filter(|(p, q)| p != q).count();
                if bad == 0 { eprintln!("FAIL dirty {page} {name} ({m:?}) frame {step}: {n} pixels differ from the whole redraw (boxes {:?})", boxes); }
                bad += 1;
            }
        }
        if bad > 0 { failed += 1; }
    }
    // Control: most frames really were drawn in part.
    if small * 2 < frames { eprintln!("FAIL control: only {small} of {frames} frames were drawn in boxes under half the window"); failed += 1; }
    eprintln!("15. dirty boxes: {frames} frames compared, {small} drawn in part, {failed} failing");
    failed
}

// ── 16. Presets end exactly ──────────────────────────────────────────────────────────────────────
fn presets(fonts: &dyn Fn() -> Vec<lab::text::Face>) -> usize {
    use lab::gallery::{self, GalleryUi, Msg, Page, presets::PreMsg};
    let (w, h) = (900u32, 600u32);
    let mut o = Offscreen::new(w, h);
    let mut failed = 0;
    let open = |from: usize, to: usize| -> GalleryUi {
        let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
        gallery::update(&mut ui.state, Msg::Preset(PreMsg::From(from)));
        gallery::update(&mut ui.state, Msg::Preset(PreMsg::To(to)));
        gallery::update(&mut ui.state, Msg::Go(Page::Presets));
        ui.relayout();
        ui.pointer_leave();
        ui
    };
    let run = |ui: &mut GalleryUi, t: &mut f32, to: f32| { while *t < to - 1e-4 { *t += 1.0 / 60.0; ui.tick(*t); } };
    let n = lab::preset::presets().len();
    for to in 0..n {
        let from = if to == 7 { 3 } else { 7 };
        let mut ui = open(from, to);
        let mut t = 0.0;
        run(&mut ui, &mut t, 2.0);
        let look = ui.state.look();
        let start = o.render(&mut ui, &look);
        let (x, y) = ui.node("presets/play").expect("play").rect.center();
        ui.press(x, y);
        ui.release(x, y);
        ui.pointer_leave();
        let dur = ui.state.presets[to].transition.duration_ms / 1000.0;
        let t0 = t;
        run(&mut ui, &mut t, t0 + dur * 0.5);
        let look = ui.state.look();
        let mid = o.render(&mut ui, &look);
        // Past the end, and past the press's burst and condensing; focus off the button (a press on the
        // top bar's empty middle), as on the fresh page.
        run(&mut ui, &mut t, t0 + dur + 1.5);
        ui.press(160.0, 25.0);
        ui.release(160.0, 25.0);
        ui.pointer_leave();
        run(&mut ui, &mut t, t0 + dur + 3.0);
        let look = ui.state.look();
        let end = o.render(&mut ui, &look);
        // A fresh page in the target preset, with focus where the press left it.
        let mut fresh = open(to, to);
        let mut tf = 0.0;
        run(&mut fresh, &mut tf, 2.0);
        gallery::update(&mut fresh.state, Msg::Preset(PreMsg::From(from)));
        fresh.state.apply_style(to);
        fresh.relayout();
        run(&mut fresh, &mut tf, 4.0);
        // The material clock is history (it ran while the transition's page moved); the frames compared are of now.
        fresh.set_material_clock(ui.material_clock());
        fresh.set_ground_clock(ui.ground_clock());
        let look = fresh.state.look();
        let want = o.render(&mut fresh, &look);
        let name = &ui.state.presets[to].name;
        let differ = end.chunks(4).zip(want.chunks(4)).filter(|(a, b)| a != b).count();
        if differ > 0 && std::env::var_os("LAB_DEBUG").is_some() { lab::offscreen::save_png(&format!("/tmp/pre-{to}-end.png"), w, h, &end); lab::offscreen::save_png(&format!("/tmp/pre-{to}-want.png"), w, h, &want); }
        if differ > 0 { eprintln!("FAIL preset {name}: the frame after its transition differs from a fresh render in {differ} pixels"); failed += 1; }
        if dur > 0.0 && (mid == start || mid == end) { eprintln!("FAIL control: {name}'s transition shows nothing mid-way"); failed += 1; }
        eprintln!("   preset {name}: {dur:.2} s, end vs fresh {differ} px");
    }
    eprintln!("16. presets: {failed} failing");
    failed
}
