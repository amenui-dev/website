//! The Lab (DESIGN.md §5.3): every experiment runs headless, deterministically, steps only while seen,
//! answers the pointer, and its drawing and glyph poses come from the foundation (fields, drawings,
//! poses, outlines) without layout or shaping per frame.
use amenui::experiment::{Experiment, ui};
use amenui::recipes::lab as x;

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }
fn run<E: Experiment>(ms: u32, mv: Option<(f32, f32)>) -> amenui::ui::Ui<amenui::experiment::Solo<E>, E::Msg> {
    let mut u = ui::<E>(fonts(), 480, 460, 1.0);
    let mut t = 0.0;
    for f in 0..(ms / 16) {
        t += 1.0 / 60.0;
        u.tick(t);
        if let Some((px, py)) = mv && f > 5 { u.pointer_move(px + (f % 7) as f32, py); }
    }
    u
}

#[test]
fn grown_fills_its_outline() {
    let u = run::<x::grown::Grown>(2500, None);
    let g = &u.state.e.gs;
    assert!(u.state.e.mask.is_some(), "the hidden shape sent its outline");
    let inside: f32 = (0..g.v.len()).filter(|&k| g.mask[k] >= 0.5).map(|k| g.v[k]).sum();
    let outside: f32 = (0..g.v.len()).filter(|&k| g.mask[k] < 0.5).map(|k| g.v[k]).sum();
    assert!(inside > 20.0, "{inside}");
    assert_eq!(outside, 0.0);
}

#[test]
fn lumen_lights_some_glyphs_and_dims_the_rest() {
    let u = run::<x::lumen::Lumen>(600, None);
    let g = u.instances_of("cell/text");
    let tones: Vec<f32> = g.iter().map(|i| i.paint[0]).collect();
    let (lo, hi) = (tones.iter().copied().fold(1.0, f32::min), tones.iter().copied().fold(0.0, f32::max));
    assert!(lo < 0.3 && hi > 0.9, "{lo} {hi}");
}
