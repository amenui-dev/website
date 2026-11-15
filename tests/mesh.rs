//! 3D elements (DESIGN.md §1.18) through the runtime, without a GPU: the gallery's 三维 page.
use lab::gallery::{self, Msg, Page};

fn ui() -> gallery::GalleryUi {
    let faces = lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts");
    let mut ui = gallery::new_ui(faces, 1280, 1400, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::Three));
    ui.relayout();
    ui.tick(1.0);
    ui
}

#[test]
fn the_field_is_two_thousand_copies_in_one_group() {
    let ui = ui();
    let f = gallery::three::field(&ui.state.three);
    assert_eq!(f.count(), 2000);
    assert_eq!(f.items.len(), 1, "no hover: one group, one draw call");
    let tree = ui.export_tree();
    let n = tree.find("three/field").expect("field in the tree");
    assert_eq!(n.kind, "mesh");
    assert_eq!(n.response, "loosen");
}

#[test]
fn hover_and_press_carry_the_copy() {
    let mut ui = ui();
    let r = ui.node("three/field").expect("field").rect;
    // The copies have gaps between them: look near the middle for one.
    let (cx, cy) = r.center();
    let (x, y) = (0..40).map(|k| (cx + (k % 8) as f32 * 3.0, cy + (k / 8) as f32 * 3.0))
        .find(|&(x, y)| { ui.pointer_move(x, y); ui.state.three.hover.is_some() }).expect("a copy near the middle of the field");
    let h = ui.state.three.hover.expect("hovered");
    assert_eq!(h.object, 1);
    // The hovered copy is drawn apart, in the accent.
    assert_eq!(gallery::three::field(&ui.state.three).items.len(), 2);
    ui.press(x, y);
    ui.release(x, y);
    assert_eq!(ui.state.three.picked.map(|p| p.instance), Some(h.instance));
    // Off the copies (the corner of the box is empty ground): nothing.
    ui.pointer_move(r.x + 3.0, r.y + 3.0);
    assert_eq!(ui.state.three.hover, None);
}

#[test]
fn drag_orbits_and_coasts_wheel_dollies_keys_turn() {
    let mut ui = ui();
    // On the mechanism's ground, off its parts (a drag on a gear turns the train instead).
    let r = ui.node("three/object/view").expect("object").rect;
    let (x, y) = (r.right() - 20.0, r.y + 20.0);
    let yaw = ui.state.three.mech.orbit.yaw;
    let mut t = 2.0;
    ui.tick(t);
    ui.press(x, y);
    for i in 1..=8 { t += 1.0 / 60.0; ui.tick(t); ui.pointer_move(x - 15.0 * i as f32, y); }
    ui.release(x - 120.0, y);
    assert!(ui.state.three.mech.orbit.yaw > yaw + 40.0, "{}", ui.state.three.mech.orbit.yaw);
    assert!(ui.state.three.mech.orbit.moving(), "a fast drag coasts");
    assert!(ui.needs_frame(t + 0.02));
    let after = ui.state.three.mech.orbit.yaw;
    for _ in 0..240 { t += 1.0 / 60.0; ui.tick(t); }
    assert!(ui.state.three.mech.orbit.yaw > after && !ui.state.three.mech.orbit.moving());
    ui.frame_drawn();
    assert!(!ui.needs_frame(t + 2.0), "at rest once it has stopped");
    let d = ui.state.three.mech.orbit.distance;
    ui.wheel(x, y, 120.0);
    assert!(ui.state.three.mech.orbit.distance > d);
    // Focused (the press gave it focus), the arrows turn it.
    assert_eq!(ui.focus(), Some("three/object/view"));
    let y0 = ui.state.three.mech.orbit.yaw;
    ui.key(lab::el::Key::Left, false);
    assert_eq!(ui.state.three.mech.orbit.yaw, y0 + ui.state.three.mech.orbit.step_deg);
}

#[test]
fn a_front_loosens_a_mesh() {
    let mut ui = ui();
    let r = ui.node("three/field").expect("field").rect;
    ui.state.cmds.push(lab::morph::Cmd::Front { x: r.center().0, y: r.center().1, strength: 1.0 });
    ui.relayout();
    let t = ui.now() + 0.05;
    ui.tick(t);
    assert!(ui.looseness("three/field", t) > 0.3, "{}", ui.looseness("three/field", t));
}

