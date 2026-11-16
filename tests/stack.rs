//! The stacked-card fan through the runtime (DESIGN.md §1.24): the pointer on the top card (a 3D pick)
//! fans every card beneath to angle0 × falloff^n, each starting delay_ms × n later; leaving closes it;
//! at rest nothing asks for frames. The page's look follows the material.
use lab::recipes::stack::{App, Msg, ui};
use lab::ui::Ui;
use lab::widgets::StackLook;

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }

#[test]
fn the_fan_follows_its_formula() {
    let mut ui: Ui<App, Msg> = ui(fonts(), 640, 480, 1.0);
    let mut t = 0.0;
    ui.tick(t);
    let b = ui.node("deck").unwrap().rect;
    ui.pointer_move(b.x + b.w * 0.55, b.y + b.h * 0.6);
    assert_eq!(ui.state.stack.under(), Some(0), "the pick found the top card");
    // After 40 ms: card 1 (n 0) and card 2 (n 1, 28 ms) have started, card 3 (56 ms) has not.
    for _ in 0..2 { t += 0.02; ui.tick(t); }
    let s = &ui.state.stack;
    assert!(s.angle_of(1) > 0.0 && s.angle_of(2) > 0.0 && s.angle_of(3) == 0.0, "{} {} {}", s.angle_of(1), s.angle_of(2), s.angle_of(3));
    for _ in 0..120 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
    let s = &ui.state.stack;
    for i in 1..s.count { assert!((s.angle_of(i) - s.angle0 * s.falloff.powi(i as i32 - 1)).abs() < 1e-3, "card {i}: {}", s.angle_of(i)); }
    assert_eq!(s.angle_of(0), 0.0);
    assert!(!ui.needs_frame(t + 0.2), "fanned and still: no frames");
    ui.pointer_move(2.0, 2.0);
    for _ in 0..120 { t += 1.0 / 60.0; ui.tick(t); }
    assert!((1..5).all(|i| ui.state.stack.angle_of(i) == 0.0), "leaving closes it");
}

#[test]
fn the_look_follows_the_material() {
    let mut g = lab::gallery::Gallery { material: lab::style::Material::Plain, ..Default::default() };
    assert_eq!(lab::gallery::craft::stack_now(&g).look, StackLook::Plain);
    g.material = lab::style::Material::Chrome;
    assert_eq!(lab::gallery::craft::stack_now(&g).look, StackLook::Thick);
    g.material = lab::style::Material::Ascii;
    assert_eq!(lab::gallery::craft::stack_now(&g).look, StackLook::Marks(lab::style::Material::Ascii));
}
