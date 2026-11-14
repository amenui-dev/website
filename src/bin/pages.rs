//! Render every gallery page in every theme and in the plain and dot materials to PNG files (one per
//! combination), plus a few states: a dialog open, the narrow layout, a theme switch half-way, typing
//! into an input. Step 5: every material's specimen at looseness 0, 0.5 and 1, the annotation page in
//! each mode, every style page at rest and with a front mid-flight, and two contact sheets
//! (`styles.png`, `materials.png`).
//! Components: every component page under every material (but the debug `near`), at rest and with a
//! front mid-way across the preview, dark theme (and light for print and lino), with one contact sheet
//! per material (`components-<material>[-light].png`: a row per page, at rest | front).
//! usage: pages <out_dir> [width] [height] [scale] [step5|perf|components]     (device px; `step5` only
//! does step 5; `perf` times a frame of every material page, fully loosened; `components` only does the
//! component sheets)
use lab::el::Key;
use lab::gallery::{self, GalleryUi, Msg, Page};
use lab::offscreen::{Offscreen, save_png};
use lab::style::{GAZE_MODES, MATERIAL_COUNT, MATERIALS, Material, styles};

const THEME_FILE: [&str; 3] = ["dark", "light", "pixel"];

fn settle(ui: &mut GalleryUi, secs: f32) {
    let to = ui.now() + secs;
    let mut t = ui.now();
    while t < to { t = (t + 1.0 / 30.0).min(to); ui.tick(t); }
}

fn shoot(o: &mut Offscreen, ui: &mut GalleryUi, out: &str, name: &str) {
    let look = ui.state.look();
    let px = o.render(ui, &look);
    save_png(&format!("{out}/{name}.png"), o.w, o.h, &px);
}

fn send(ui: &mut GalleryUi, m: Msg) {
    gallery::update(&mut ui.state, m);
    ui.relayout();
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| "pages".into());
    let w: u32 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1280);
    let h: u32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(1800);
    let scale: f32 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let only5 = a.get(5).is_some_and(|s| s == "step5");
    if a.get(5).is_some_and(|s| s == "perf") { perf(w, h, scale); return; }
    std::fs::create_dir_all(&out).expect("out dir");
    if a.get(5).is_some_and(|s| s == "components") { components(&out, scale); return; }
    let faces = || lab::text::load_bundled("assets").expect("fonts");
    step5(&out, scale);
    if only5 { return; }
    let mut o = Offscreen::new(w, h);
    let mut ui = gallery::new_ui(faces(), w, h, scale);
    ui.pointer_leave();

    for p in Page::all() {
        for (ti, tname) in THEME_FILE.iter().enumerate() {
            for (mi, (_, mname)) in MATERIALS.iter().enumerate().take(2) {
                send(&mut ui, Msg::Go(p));
                send(&mut ui, Msg::Theme(ti));
                send(&mut ui, Msg::Material(mi));
                settle(&mut ui, 3.0);
                shoot(&mut o, &mut ui, &out, &format!("{}-{tname}-{mname}", p.hash()));
            }
        }
        eprintln!("{}", p.hash());
    }

    // States, at window height 900.
    let mut o = Offscreen::new(w, 900);
    let mut ui = gallery::new_ui(faces(), w, 900, scale);
    ui.pointer_leave();
    send(&mut ui, Msg::Go(Page::Comp(9)));
    settle(&mut ui, 3.0);
    let (x, y) = ui.node("dialog/open").expect("open button").rect.center();
    ui.press(x, y);
    ui.release(x, y);
    for c in "Studio".chars() { ui.key(Key::Char(c), false); }
    ui.pointer_leave();
    settle(&mut ui, 0.25);
    shoot(&mut o, &mut ui, &out, "state-dialog-opening");
    settle(&mut ui, 3.0);
    shoot(&mut o, &mut ui, &out, "state-dialog-open-dark-dot");
    send(&mut ui, Msg::Material(0));
    settle(&mut ui, 1.0);
    shoot(&mut o, &mut ui, &out, "state-dialog-open-dark-plain");
    send(&mut ui, Msg::Theme(1));
    settle(&mut ui, 1.0);
    shoot(&mut o, &mut ui, &out, "state-dialog-open-light-plain");
    ui.key(Key::Escape, false);
    settle(&mut ui, 3.0);
    shoot(&mut o, &mut ui, &out, "state-dialog-closed");

    // A theme switch, pressed on the top bar: the old frame beyond the front, the new one inside.
    send(&mut ui, Msg::Theme(0));
    send(&mut ui, Msg::Material(1));
    send(&mut ui, Msg::Go(Page::Comp(0)));
    settle(&mut ui, 3.0);
    // The renderer keeps the look of the frame it last drew as the old one, as a window drawing every
    // frame would.
    shoot(&mut o, &mut ui, &out, "state-theme-before");
    let (x, y) = ui.node("theme:1").expect("theme segment").rect.center();
    ui.press(x, y);
    ui.release(x, y);
    ui.pointer_leave();
    settle(&mut ui, 0.5);
    shoot(&mut o, &mut ui, &out, "state-theme-switching");
    // And the material, from its own control.
    let (x, y) = ui.node("material:0").expect("material segment").rect.center();
    ui.press(x, y);
    ui.release(x, y);
    ui.pointer_leave();
    settle(&mut ui, 0.6);
    shoot(&mut o, &mut ui, &out, "state-material-switching");
    settle(&mut ui, 3.0);
    shoot(&mut o, &mut ui, &out, "state-theme-switched");

    // Navigation from the rail, half-way.
    let (x, y) = ui.node("nav/slider").expect("rail item").rect.center();
    ui.press(x, y);
    ui.release(x, y);
    ui.pointer_leave();
    settle(&mut ui, 0.2);
    shoot(&mut o, &mut ui, &out, "state-navigating");

    // Typing into the input.
    send(&mut ui, Msg::Go(Page::Comp(2)));
    settle(&mut ui, 3.0);
    let (x, y) = ui.node("input/live").expect("input").rect.center();
    ui.press(x, y);
    ui.release(x, y);
    for c in "hello world".chars() { ui.key(Key::Char(c), false); }
    for _ in 0..5 { ui.key(Key::Left, false); }
    ui.pointer_leave();
    settle(&mut ui, 3.0);
    shoot(&mut o, &mut ui, &out, "state-input-typed");

    // Narrow: the rail becomes a row of tabs.
    let mut o = Offscreen::new(600, 1100);
    let mut ui = gallery::new_ui(faces(), 600, 1100, scale);
    ui.pointer_leave();
    for p in [Page::Comp(0), Page::Comp(7), Page::Marks(0)] {
        send(&mut ui, Msg::Go(p));
        settle(&mut ui, 3.0);
        shoot(&mut o, &mut ui, &out, &format!("narrow-{}", p.hash()));
    }
}

/// A smaller copy (box filter, whole factor).
fn shrink(px: &[u8], w: u32, h: u32, f: u32) -> (Vec<u8>, u32, u32) {
    let (sw, sh) = (w / f, h / f);
    let mut o = vec![0u8; (sw * sh * 4) as usize];
    for y in 0..sh { for x in 0..sw { for c in 0..4 {
        let mut acc = 0u32;
        for j in 0..f { for i in 0..f { acc += px[(((y * f + j) * w + x * f + i) * 4 + c) as usize] as u32; } }
        o[((y * sw + x) * 4 + c) as usize] = (acc / (f * f)) as u8;
    } } }
    (o, sw, sh)
}
/// Tiles of the same size in a grid, `cols` wide, with a gutter.
fn sheet(tiles: &[(Vec<u8>, u32, u32)], cols: u32) -> (Vec<u8>, u32, u32) {
    let (tw, th) = (tiles[0].1, tiles[0].2);
    let g = 8;
    let rows = (tiles.len() as u32).div_ceil(cols);
    let (w, h) = (cols * (tw + g) + g, rows * (th + g) + g);
    let mut o = vec![40u8; (w * h * 4) as usize];
    for p in o.chunks_mut(4) { p[3] = 255; }
    for (k, (px, _, _)) in tiles.iter().enumerate() {
        let (cx, cy) = (g + (k as u32 % cols) * (tw + g), g + (k as u32 / cols) * (th + g));
        for y in 0..th { let d = (((cy + y) * w + cx) * 4) as usize; let s = ((y * tw) * 4) as usize; o[d..d + (tw * 4) as usize].copy_from_slice(&px[s..s + (tw * 4) as usize]); }
    }
    (o, w, h)
}

fn step5(out: &str, scale: f32) {
    let (w, h) = ((1280.0 * scale) as u32, (1000.0 * scale) as u32);
    let faces = || lab::text::load_bundled("assets").expect("fonts");
    let mut o = Offscreen::new(w, h);
    let mut ui = gallery::new_ui(faces(), w, h, scale);
    ui.pointer_leave();
    let mut mats = vec![];
    for m in 0..MATERIAL_COUNT as u8 {
        send(&mut ui, Msg::Go(Page::Material(m)));
        for (k, name) in [(0.0, "k0"), (0.5, "k05"), (1.0, "k1")] {
            send(&mut ui, Msg::Num(gallery::Target::Loose, lab::widgets::Edit::Set(k)));
            settle(&mut ui, 1.5);
            let look = ui.state.look();
            let px = o.render(&mut ui, &look);
            save_png(&format!("{out}/material-{}-{name}.png", Material::from_id(m).name()), w, h, &px);
            if k == 0.5 { mats.push(shrink(&px, w, h, 2)); }
        }
        eprintln!("material {}", Material::from_id(m).name());
    }
    send(&mut ui, Msg::Num(gallery::Target::Loose, lab::widgets::Edit::Set(0.0)));
    let (px, sw, sh) = sheet(&mats, 3);
    save_png(&format!("{out}/materials.png"), sw, sh, &px);

    // A front passing over a picture under dot: the picture shows its marks, never holes.
    send(&mut ui, Msg::Go(Page::Comp(10)));
    send(&mut ui, Msg::Material(1));
    settle(&mut ui, 1.5);
    let (x, y) = ui.node("image/live").expect("picture").rect.center();
    ui.press(x - 200.0, y);
    ui.release(x - 200.0, y);
    ui.pointer_leave();
    settle(&mut ui, 0.3);
    shoot(&mut o, &mut ui, out, "dot-ripple-picture");

    // The annotation page in each mode: the pointer over a figure, focus in the list.
    send(&mut ui, Msg::Go(Page::Gaze));
    settle(&mut ui, 1.0);
    let (fx, fy) = ui.node("gaze/fig-b").expect("figure").rect.center();
    let (rx, ry) = ui.node("gaze/row1").expect("row").rect.center();
    for (i, (_, name)) in GAZE_MODES.iter().enumerate() {
        send(&mut ui, Msg::GazeMode(i));
        ui.press(rx, ry);
        ui.release(rx, ry);
        ui.pointer_move(fx, fy);
        settle(&mut ui, 0.1);
        shoot(&mut o, &mut ui, out, &format!("gaze-{i}-mid"));
        settle(&mut ui, 2.0);
        shoot(&mut o, &mut ui, out, &format!("gaze-{i}"));
        eprintln!("gaze {name}");
    }
    ui.pointer_leave();

    // Every style page: at rest, and with a front mid-flight.
    let mut rest = vec![];
    for (i, st) in styles().iter().enumerate() {
        let mut ui = gallery::new_ui(faces(), w, h, scale);
        ui.pointer_leave();
        send(&mut ui, Msg::Go(Page::Style(i as u8)));
        settle(&mut ui, 2.5);
        let look = ui.state.look();
        let px = o.render(&mut ui, &look);
        save_png(&format!("{out}/style-{}-rest.png", st.hash), w, h, &px);
        rest.push(shrink(&px, w, h, 4));
        // A press on empty page ground, left of the composition's middle.
        let comp = ui.node(&format!("style-{}/comp", st.hash)).expect("composition").rect;
        let (x, y) = (comp.x + comp.w * 0.35, comp.y + comp.h * 0.5);
        ui.press(x, y);
        ui.release(x, y);
        ui.pointer_leave();
        settle(&mut ui, 0.38);
        shoot(&mut o, &mut ui, out, &format!("style-{}-ripple", st.hash));
        eprintln!("style {}", st.hash);
    }
    let (px, sw, sh) = sheet(&rest, 3);
    save_png(&format!("{out}/styles.png"), sw, sh, &px);
}

/// Put a component page in the state that shows the most of it: the input focused (its caret), the
/// dialog open.
fn stage_component(ui: &mut GalleryUi, c: usize) {
    use lab::gallery::comp::{DIALOG, INPUT};
    let h = lab::gallery::COMPONENTS[c].hash;
    if c == INPUT || c == DIALOG {
        let key = if c == INPUT { format!("{h}/live") } else { format!("{h}/open") };
        let (x, y) = ui.node(&key).expect("control").rect.center();
        ui.press(x, y);
        ui.release(x, y);
        ui.pointer_leave();
    }
}

/// The page without the rail and the source: header, preview, variants, numbers; at half size.
fn tile(px: &[u8], w: u32, scale: f32) -> (Vec<u8>, u32, u32) {
    let (x0, y0, cw, ch) = ((228.0 * scale) as u32, (44.0 * scale) as u32, (1040.0 * scale) as u32, (720.0 * scale) as u32);
    let mut c = Vec::with_capacity((cw * ch * 4) as usize);
    for y in y0..y0 + ch { let o = ((y * w + x0) * 4) as usize; c.extend_from_slice(&px[o..o + (cw * 4) as usize]); }
    shrink(&c, cw, ch, 2)
}

fn components(out: &str, scale: f32) {
    let (w, h) = ((1280.0 * scale) as u32, (1100.0 * scale) as u32);
    let faces = || lab::text::load_bundled("assets").expect("fonts");
    let mut o = Offscreen::new(w, h);
    for (mi, (mat, mname)) in MATERIALS.iter().enumerate() {
        if *mat == Material::Near { continue; }
        let themes: &[usize] = if matches!(mat, Material::Print | Material::Lino) { &[0, 1] } else { &[0] };
        for &ti in themes {
            let tname = THEME_FILE[ti];
            let mut tiles = vec![];
            for c in 0..lab::gallery::COMPONENTS.len() {
                let hash = lab::gallery::COMPONENTS[c].hash;
                let mut ui = gallery::new_ui(faces(), w, h, scale);
                ui.pointer_leave();
                send(&mut ui, Msg::Go(Page::Comp(c)));
                send(&mut ui, Msg::Theme(ti));
                send(&mut ui, Msg::Material(mi));
                settle(&mut ui, 1.0);
                stage_component(&mut ui, c);
                settle(&mut ui, 3.0);
                let look = ui.state.look();
                let px = o.render(&mut ui, &look);
                save_png(&format!("{out}/{hash}-{mname}-{tname}-rest.png"), w, h, &px);
                tiles.push(tile(&px, w, scale));
                // A front from the preview's left edge, half-way across it.
                let r = ui.node(&format!("{hash}/preview")).expect("preview").rect;
                let (x, y) = (r.x + 6.0, r.y + r.h * 0.5);
                ui.press(x, y);
                ui.release(x, y);
                ui.pointer_leave();
                let dt = r.w * 0.5 / ui.state.look().fx.wave_speed;
                settle(&mut ui, dt);
                let look = ui.state.look();
                let px = o.render(&mut ui, &look);
                save_png(&format!("{out}/{hash}-{mname}-{tname}-ripple.png"), w, h, &px);
                tiles.push(tile(&px, w, scale));
            }
            let (px, sw, sh) = sheet(&tiles, 4);
            let suffix = if ti == 0 { String::new() } else { format!("-{tname}") };
            save_png(&format!("{out}/components-{mname}{suffix}.png"), sw, sh, &px);
            eprintln!("components {mname} {tname}");
        }
    }
}

fn perf(w: u32, h: u32, scale: f32) {
    let mut o = Offscreen::new(w, h);
    let mut ui = gallery::new_ui(lab::text::load_bundled("assets").expect("fonts"), w, h, scale);
    ui.pointer_move(700.0, 400.0);
    for m in 0..MATERIAL_COUNT as u8 {
        send(&mut ui, Msg::Go(Page::Material(m)));
        send(&mut ui, Msg::Num(gallery::Target::Loose, lab::widgets::Edit::Set(1.0)));
        settle(&mut ui, 1.0);
        let mut look = ui.state.look();
        // QUALITY=0.5 (or 0.75, or 0 for automatic) times the material at reduced resolution.
        look.quality = std::env::var("QUALITY").ok().and_then(|v| v.parse().ok()).unwrap_or(1.0);
        o.draw(&mut ui, &look);
        let n = 10;
        let t0 = std::time::Instant::now();
        for _ in 0..n { o.draw(&mut ui, &look); }
        let cpu = ui.perf().summary(lab::perf::now_ms()).submit[1];
        eprintln!("{:<9} {:6.1} ms per frame ({w}×{h}; of which {cpu:.1} ms recording and submitting on the CPU)", Material::from_id(m).name(), t0.elapsed().as_secs_f64() * 1000.0 / n as f64);
    }
}
