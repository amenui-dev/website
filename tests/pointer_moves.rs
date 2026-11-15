//! Moving the pointer costs only what changed (DESIGN.md §1.19): on every film chapter and gallery
//! page, 200 pointer moves that do not change what is hovered run no layout and shape no text, and the
//! area to redraw stays bounded (never the whole window).
use lab::dirty::{self, Region};
use lab::gallery::{self, GalleryUi, Msg, Page};
use lab::gallery::film::{self, FMsg};

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }

const W: f32 = 1280.0;
const H: f32 = 800.0;
/// What any move may redraw: a quarter of the window, or more only inside what the lens can reach
/// (pictures, 3D elements, the dot ground under it: `dirty::LENS_REACH_DOT` radii, grown by the
/// farthest any material or picture reads), old and new place, and 3D elements under it.
const MAX_AREA: f32 = 0.25;
fn lens_bound(ui: &GalleryUi, from: (f32, f32), to: (f32, f32)) -> dirty::Bx {
    let look = ui.state.look();
    let s = ui.scale();
    let rch = (0..lab::style::MATERIAL_COUNT as u8).map(|m| dirty::reach(lab::style::Material::from_id(m), &look, s)).fold(0.0, f32::max)
        + look.pics.iter().map(|p| 2.0 * p.pitch * s + p.depth * s).fold(0.0, f32::max);
    let r = dirty::LENS_REACH_DOT * look.fx.lens * s + rch + 2.0;
    let mut b = dirty::union([from.0 * s - r, from.1 * s - r, from.0 * s + r, from.1 * s + r], [to.0 * s - r, to.1 * s - r, to.0 * s + r, to.1 * s + r]);
    for m in &ui.scene().meshes { b = dirty::union(b, dirty::grow([m.rect[0], m.rect[1], m.rect[0] + m.rect[2], m.rect[1] + m.rect[3]], rch + 40.0)); }
    b
}

fn settle(ui: &mut GalleryUi, secs: f32) {
    let to = ui.now() + secs;
    let mut t = ui.now();
    while t < to { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
}

/// 200 moves on a path over the window; returns a line per failure.
fn audit(ui: &mut GalleryUi, name: &str, bad: &mut Vec<String>) {
    ui.text_mut().log_shapes = true;
    // The first input after arriving changes things once (the film's hints go): not what is measured.
    ui.pointer_move(4.0, 4.0);
    settle(ui, 1.5);
    let mut t = ui.now();
    let look = ui.state.look();
    let mut prev = ui.dirty_inputs(&look);
    let mut worst = 0.0f32;
    let mut at = (4.0f32, 4.0f32);
    for i in 0..200 {
        // A slow zigzag: 20 columns of 10 steps each, 3 px apart, so most moves stay over one thing.
        let (c, r) = (i / 10, i % 10);
        let x = 60.0 + c as f32 * 58.0 + r as f32 * 3.0;
        let y = 90.0 + ((c * 37) % 11) as f32 * 60.0 + r as f32 * 2.0;
        // What is hovered: the control, the 3D copy, and what the annotation layer looks at.
        let looked = |ui: &GalleryUi| ui.annots().iter().map(|a| a.key.clone()).collect::<Vec<_>>();
        let hover = (ui.hover().map(str::to_owned), ui.mesh_hover().cloned(), looked(ui));
        let before = ui.counters();
        // Something still moving from an earlier change (hover marks sliding or fading, a burst) is
        // not this move's cost: the area is measured from rest.
        let rest = !ui.moving_at(t);
        t += 1.0 / 60.0;
        ui.event_time(t);
        let last = at;
        at = (x, y);
        ui.pointer_move(x, y);
        // What the move itself costs (a state clock, like the departure board's second, ticks apart).
        let after = ui.counters();
        ui.tick(t);
        // A state clock that rebuilt the scene in this tick (the departure board) is not the move's.
        let clocked = ui.counters().scenes != after.scenes;
        // The crosshair and rulers follow the pointer across the whole annotated scope.
        let g = ui.state.look().gaze;
        let follows = g.mode != lab::style::GazeMode::Off && g.kinds & (lab::style::gaze::CROSS | lab::style::gaze::RULER) != 0;
        let same = hover == (ui.hover().map(str::to_owned), ui.mesh_hover().cloned(), looked(ui));
        let cur = ui.dirty_inputs(&ui.state.look());
        let region = dirty::region(&prev, &cur, ui.scale());
        // The annotation layer's labels that show the pointer's coordinates are new text each move.
        let labels = prev.annot != cur.annot && after.layouts == before.layouts;
        if std::env::var("AUDIT_DEBUG").is_ok() && region == Region::All { eprintln!("{name} {i}: look same {} global {} {} size {:?} {:?} calm {} meshes {} {} gclock {} {}", prev.look == cur.look, prev.global, cur.global, prev.size, cur.size, prev.field.calm == cur.field.calm, prev.meshes.len(), cur.meshes.len(), prev.gclock, cur.gclock); }
        if std::env::var("AUDIT_DEBUG").is_ok() && same {
            let (a, b) = (&prev.instances, &cur.instances);
            let n = (0..a.len().min(b.len())).filter(|&k| a[k] != b[k]).count();
            if n > 0 || a.len() != b.len() { let k = (0..a.len().min(b.len())).find(|&k| a[k] != b[k]); eprintln!("{name} {i}: {n} instances differ, len {} -> {}; first {:?}", a.len(), b.len(), k.map(|k| (a[k], b[k]))); }
        }
        prev = cur;
        ui.frame_drawn();
        if !same { continue; }
        if after.layouts != before.layouts || (after.shaped != before.shaped && !labels) {
            bad.push(format!("{name}: move {i} to ({x}, {y}) with the same hover ran {} layouts and shaped {}: {:?}", after.layouts - before.layouts, after.shaped - before.shaped, ui.text_mut().shape_log.iter().rev().take(3).collect::<Vec<_>>()));
            return;
        }
        if !rest || clocked || follows { continue; }
        let area = match region {
            Region::All => 1.0,
            Region::Boxes { stage1, stage2 } => { let b = dirty::union(stage1, stage2); if dirty::is_empty(b) { 0.0 } else { (b[2] - b[0]) * (b[3] - b[1]) / (W * H) } }
        };
        worst = worst.max(area);
        let lb = lens_bound(ui, last, (x, y));
        let inside = |b: dirty::Bx| dirty::is_empty(b) || (b[0] >= lb[0] && b[1] >= lb[1] && b[2] <= lb[2] && b[3] <= lb[3]);
        let within = match region { Region::All => false, Region::Boxes { stage1, stage2 } => inside(stage1) && inside(stage2) };
        if area > MAX_AREA && !within { bad.push(format!("{name}: move {i} to ({x}, {y}) with the same hover redraws {:.0}% of the window ({region:?})", area * 100.0)); return; }
    }
    eprintln!("{name:<28} worst redraw {:.1}%", worst * 100.0);
}

#[test]
fn pointer_moves_on_every_film_chapter() {
    let mut ui = gallery::new_ui(fonts(), W as u32, H as u32, 1.0);
    assert!(ui.state.route("film"));
    if ui.state.film.playing { gallery::update(&mut ui.state, Msg::Film(FMsg::Play)); }
    let mut bad = vec![];
    for c in 0..=13 {
        film::go(&mut ui.state, c, None);
        ui.relayout();
        settle(&mut ui, 3.0);
        audit(&mut ui, &format!("film chapter {c}"), &mut bad);
        ui.pointer_leave();
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn pointer_moves_on_every_gallery_page() {
    let mut ui = gallery::new_ui(fonts(), W as u32, H as u32, 1.0);
    let mut bad = vec![];
    for p in Page::all() {
        if p == Page::Film { continue; }
        gallery::update(&mut ui.state, Msg::Go(p));
        ui.relayout();
        settle(&mut ui, 2.5);
        audit(&mut ui, &p.hash(), &mut bad);
        ui.pointer_leave();
        if matches!(p, Page::Scene(_)) { ui.key(lab::el::Key::Escape, false); ui.relayout(); }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
