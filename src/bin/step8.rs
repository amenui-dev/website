//! Step 8's contact sheets (DESIGN.md §7):
//!   mesh         the same low-poly object under every material, at rest and loosened, and drawn as a
//!                picture under four mark sets → `mesh-materials.png`
//!   particles    the 粒子 page going mesh → text → picture → mesh at 200,000 tiles, six frames of each
//!                change, with the pointer in the swarm → `particles.png`
//!   transitions  every preset's transition on the 风格 page, six frames each (from Normcore; Normcore's
//!                own from Glitch), one row per preset → `transitions.png`
//! usage: step8 <out_dir> [mesh | particles | transitions]
use lab::el::{El, Paint, col, scene3d, text};
use lab::mesh::{Mat4, MeshId, Scene3d};
use lab::offscreen::{Offscreen, crop, save_png, sheet};
use lab::style::{DARK, Look, MATERIALS, Material, PicFx, set};
use lab::ui::Ui;

const S: u32 = 300;

/// One object in a box, held at a looseness, through the material or as a picture.
pub struct One { pub loose: f32, pub picture: bool, pub yaw: f32 }
pub fn object(yaw: f32) -> Scene3d {
    let mut sc = Scene3d::single(MeshId::ICO[1]);
    sc.items[0].transforms = vec![Mat4::rotate_y(yaw).mul(&Mat4::uniform(1.55))];
    sc
}
fn one(s: &One) -> El<()> {
    let mut sc = object(s.yaw);
    if s.picture { sc.picture = Some(0); }
    col().key("root").fill_width().fill_height().pad(10.0, 10.0).gap(4.0)
        .child(scene3d(sc).key("mesh").width(280.0).height(256.0).loose(s.loose))
        .child(text(if s.picture { "picture" } else if s.loose > 0.0 { "loose" } else { "rest" }, 12.0, Paint::DIM))
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| "step8".into());
    std::fs::create_dir_all(&out).expect("out dir");
    let what = a.get(2).map(String::as_str).unwrap_or("mesh");
    if what == "mesh" { mesh(&out); }
    if what == "particles" { particles(&out); }
    if what == "transitions" { transitions(&out); }
}

/// Play preset `to`'s transition from preset `from` on the 风格 page; `at` is called with the ui at each
/// of `fractions` of the transition (and once more after it).
pub fn play_transition(o: &mut Offscreen, from: usize, to: usize, fractions: &[f32], mut at: impl FnMut(&mut Offscreen, &mut lab::gallery::GalleryUi, usize)) {
    use lab::gallery::{self, Msg, Page, presets::PreMsg};
    let (w, h) = (o.w, o.h);
    let mut ui = gallery::new_ui(lab::text::load_bundled("assets").expect("fonts"), w, h, 1.0);
    gallery::update(&mut ui.state, Msg::Preset(PreMsg::From(from)));
    gallery::update(&mut ui.state, Msg::Preset(PreMsg::To(to)));
    gallery::update(&mut ui.state, Msg::Go(Page::Presets));
    ui.relayout();
    ui.pointer_leave();
    let mut t = 0.0f32;
    let step = |ui: &mut gallery::GalleryUi, t: &mut f32, to: f32| { while *t < to - 1e-4 { *t += 1.0 / 60.0; ui.tick(*t); } };
    step(&mut ui, &mut t, 2.0);
    let look = ui.state.look();
    let _ = o.render(&mut ui, &look);
    let (x, y) = ui.node("presets/play").expect("play").rect.center();
    ui.press(x, y);
    ui.release(x, y);
    ui.pointer_leave();
    let dur = ui.state.presets[to].transition.duration_ms / 1000.0;
    let start = t;
    for (k, f) in fractions.iter().enumerate() {
        step(&mut ui, &mut t, start + dur * f);
        let look = ui.state.look();
        let _ = o.render(&mut ui, &look);
        at(o, &mut ui, k);
    }
}

fn transitions(out: &str) {
    let (w, h) = (1100u32, 560u32);
    let mut o = Offscreen::new(w, h);
    let mut frames = vec![];
    let n = lab::preset::presets().len();
    for to in 0..n {
        let from = if to == 7 { 3 } else { 7 };
        play_transition(&mut o, from, to, &[0.08, 0.25, 0.42, 0.6, 0.8, 1.05], |o, ui, _| {
            let look = ui.state.look();
            let px = o.render(ui, &look);
            frames.push(half(&crop(&px, w, h, [230, 60, 1100, 560])));
        });
    }
    let (sw, sh, px) = sheet(&frames, 6);
    save_png(&format!("{out}/transitions.png"), sw, sh, &px);
    eprintln!("wrote {out}/transitions.png ({} rows of six)", n);
}

/// A frame at half size (2 × 2 average).
fn half(f: &(u32, u32, Vec<u8>)) -> (u32, u32, Vec<u8>) {
    let (w, h, px) = (f.0, f.1, &f.2);
    let (hw, hh) = (w / 2, h / 2);
    let mut o = vec![0u8; (hw * hh * 4) as usize];
    for y in 0..hh { for x in 0..hw { for c in 0..4 {
        let s = |dx: u32, dy: u32| px[(((y * 2 + dy) * w + x * 2 + dx) * 4 + c) as usize] as u32;
        o[((y * hw + x) * 4 + c) as usize] = ((s(0, 0) + s(1, 0) + s(0, 1) + s(1, 1)) / 4) as u8;
    } } }
    (hw, hh, o)
}

fn particles(out: &str) {
    use lab::gallery::{self, Msg, Page, particles::PMsg};
    let (w, h) = (1280u32, 800u32);
    let mut o = Offscreen::new(w, h);
    let mut ui = gallery::new_ui(lab::text::load_bundled("assets").expect("fonts"), w, h, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Particles));
    ui.relayout();
    let mut t = 0.0f32;
    let step = |ui: &mut gallery::GalleryUi, t: &mut f32, to: f32| { while *t < to - 1e-4 { *t += 1.0 / 60.0; ui.tick(*t); } };
    step(&mut ui, &mut t, 1.0);
    let look = ui.state.look();
    let _ = o.render(&mut ui, &look);
    let r = ui.node("parts/stage").expect("stage").rect;
    let crop_r = [(r.x - 20.0) as u32, (r.y - 20.0) as u32, (r.right() + 20.0) as u32, (r.bottom() + 20.0) as u32];
    let mut frames = vec![];
    let mut info = vec![];
    for k in 0..3 {
        gallery::update(&mut ui.state, Msg::Parts(PMsg::Next));
        ui.relayout();
        // The pointer in the middle of the stage pushes the swarm.
        ui.pointer_move(r.x + r.w * 0.55, r.y + r.h * 0.5);
        let start = t;
        for f in 0..6 {
            step(&mut ui, &mut t, start + [0.05, 0.3, 0.6, 0.9, 1.25, 1.9][f]);
            let look = ui.state.look();
            let px = o.render(&mut ui, &look);
            frames.push(crop(&px, w, h, crop_r));
            if f == 2 { info.push(format!("change {k}: {} particles, {}", ui.perf().particles, ui.perf().particle_path)); }
        }
    }
    let (sw, sh, px) = sheet(&frames, 6);
    // Half size: the sheet is for looking at.
    let (hw, hh) = (sw / 2, sh / 2);
    let mut half = vec![0u8; (hw * hh * 4) as usize];
    for y in 0..hh { for x in 0..hw { for c in 0..4 {
        let s = |dx: u32, dy: u32| px[(((y * 2 + dy) * sw + x * 2 + dx) * 4 + c) as usize] as u32;
        half[((y * hw + x) * 4 + c) as usize] = ((s(0, 0) + s(1, 0) + s(0, 1) + s(1, 1)) / 4) as u8;
    } } }
    save_png(&format!("{out}/particles.png"), hw, hh, &half);
    for i in info { eprintln!("{i}"); }
    eprintln!("wrote {out}/particles.png");
}

fn mesh(out: &str) {
    let mut o = Offscreen::new(S, S);
    let faces = || lab::text::load_bundled("assets").expect("fonts");
    let mut frames = vec![];
    let mut shot = |state: One, look: Look, label: &str| {
        let mut ui = Ui::new(state, one, |_, _| {}, faces(), S, S, 1.0);
        ui.tick(2.0);
        let px = o.render(&mut ui, &look);
        let _ = label;
        frames.push(crop(&px, S, S, [0, 0, S, S]));
    };
    for (m, name) in MATERIALS {
        for loose in [0.0, 0.7] {
            let look = Look { theme: DARK, material: m, ..Look::default() };
            shot(One { loose, picture: false, yaw: 0.0 }, look, name);
        }
    }
    for s in [set::DOTS, set::CHARS, set::STROKE, set::PIXEL] {
        let mut look = Look { theme: DARK, material: Material::Plain, ..Look::default() };
        look.pics[0] = PicFx { pitch: if s == set::CHARS { 9.0 } else { 6.0 }, levels: 10.0, ground: 0.1, follow: if s == set::STROKE { 1.0 } else { 0.0 }, ..PicFx::with_set(s) };
        shot(One { loose: 1.0, picture: true, yaw: 0.0 }, look, "marks");
    }
    let (w, h, px) = sheet(&frames, 8);
    save_png(&format!("{out}/mesh-materials.png"), w, h, &px);
    eprintln!("wrote {out}/mesh-materials.png ({} frames: every material at rest and loose 0.7, then four mark sets)", frames.len());
}
