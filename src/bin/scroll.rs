//! Filmstrips of scrolling (DESIGN.md §1.8.3): three pages (the article, the 10k list, a component
//! page) scrolled at a steady 120, 800 and 3000 px/s under six materials, four consecutive frames each
//! (60 Hz, fixed clock), cropped to the middle of the scroller; and a stillness measure: how far two
//! consecutive frames of the slow scroll are from being the same picture moved by the scroll.
//!
//! usage: scroll <out_dir> [name]   (writes <name>-<page>.png and <name>.png, the article's sheet; name
//! defaults to "scroll")
use lab::gallery::{self, GalleryUi, Msg, Page};
use lab::offscreen::{Offscreen, crop, save_png, sheet};
use lab::style::Material;

/// The window for a page: the article is set narrow so it is much longer than its window.
fn size(page: &str) -> (u32, u32) { if page == "scene-reading" { (900, 640) } else { (1280, 800) } }
pub const SPEEDS: [f32; 3] = [120.0, 800.0, 3000.0];
const MATS: [Material; 6] = [Material::Plain, Material::Dot, Material::Halftone, Material::Ascii, Material::Print, Material::Glitch];

fn faces() -> Vec<lab::text::Face> { lab::text::load_bundled("assets").expect("fonts") }

/// The page, where the wheel goes, and the crop (logical px).
fn open(page: &str, m: Material) -> (GalleryUi, (f32, f32)) {
    let (ww, wh) = size(page);
    let mut ui = gallery::new_ui(faces(), ww, wh, 1.0);
    ui.pointer_leave();
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash(page).expect("page")));
    gallery::update(&mut ui.state, Msg::Material(m.id() as usize));
    if page == "scene-reading" {
        // Larger type and no parameter strip: the article is several windows long.
        ui.state.sc.p[1][2] = 24.0;
        ui.state.sc.p[1][3] = 44.0;
        gallery::update(&mut ui.state, Msg::Scene(gallery::SMsg::Strip));
    }
    ui.relayout();
    let at = match page {
        "list-10k" => ui.node("long").map(|n| (n.rect.x + 200.0, n.rect.center().1)).unwrap_or((460.0, 450.0)),
        p if p.starts_with("scene-") => (ww as f32 * 0.5, wh as f32 * 0.55),
        _ => (220.0 + (ww as f32 - 220.0) * 0.5, wh as f32 * 0.55),
    };
    (ui, at)
}

/// How far frame `b` is from frame `a` moved up by some whole number of px (the best shift, 0..=64),
/// over the crop: the mean absolute difference per channel (0..255) at the best shift, and the shift.
pub fn moved_diff(a: &[u8], b: &[u8], w: u32, h: u32) -> (f32, u32) {
    let mut best = (f32::MAX, 0);
    for s in 0..=64u32.min(h / 2) {
        let mut sum = 0u64;
        let mut n = 0u64;
        for y in 0..h - s {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let j = (((y + s) * w + x) * 4) as usize;
                for c in 0..3 { sum += (b[i + c] as i32 - a[j + c] as i32).unsigned_abs() as u64; n += 1; }
            }
        }
        let d = sum as f32 / n.max(1) as f32;
        if d < best.0 { best = (d, s); }
    }
    best
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| "scroll".into());
    let name = a.get(2).cloned().unwrap_or_else(|| "scroll".into());
    std::fs::create_dir_all(&out).expect("out dir");
    let (cw, ch) = (360.0f32, 200.0f32);
    let mut report = String::new();
    for page in ["scene-reading", "list-10k", "button"] {
        let (ww, wh) = size(page);
        let mut o = Offscreen::new(ww, wh);
        let mut frames = vec![];
        for &m in &MATS {
            for &v in &SPEEDS {
                let (mut ui, (px, py)) = open(page, m);
                let mut t = 0.0f32;
                // Settle: everything born, nothing moving.
                while t < 1.5 { t += 1.0 / 60.0; ui.tick(t); }
                let look = ui.state.look();
                o.draw(&mut ui, &look);
                let step = v / 60.0;
                let mut shots = vec![];
                for f in 0..9 {
                    t += 1.0 / 60.0;
                    ui.tick(t);
                    ui.wheel(px, py, step);
                    ui.tick(t);
                    let look = ui.state.look();
                    if std::env::var_os("SCROLL_DEBUG").is_some() { eprintln!("f{f} motion {:?}", ui.scroll_motion("scene/1/body")); }
                    if f >= 5 {
                        let full = o.render(&mut ui, &look);
                        shots.push(crop(&full, ww, wh, [(px - cw * 0.5) as u32, (py - ch * 0.5) as u32, (px + cw * 0.5) as u32, (py + ch * 0.5) as u32]));
                    } else {
                        o.draw(&mut ui, &look);
                    }
                }
                if v == SPEEDS[0] {
                    let mut worst = (0.0f32, 0);
                    for p in shots.windows(2) { let d = moved_diff(&p[0].2, &p[1].2, p[0].0, p[0].1); if d.0 >= worst.0 { worst = d; } }
                    report.push_str(&format!("{page:14} {:9} slow: worst moved diff {:.3}/255 (shift {} px)\n", m.name(), worst.0, worst.1));
                }
                frames.extend(shots);
                eprintln!("{page} {} {v}", m.name());
            }
        }
        let (w, h, px) = sheet(&frames, 4);
        save_png(&format!("{out}/{name}-{page}.png"), w, h, &px);
        if page == "scene-reading" { save_png(&format!("{out}/{name}.png"), w, h, &px); }
    }
    print!("{report}");
    std::fs::write(format!("{out}/{name}.txt"), report).expect("report");
}
