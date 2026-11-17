//! The stacked-card fan through the runtime (DESIGN.md §1.24): the pointer on the top card (a 3D pick)
//! fans every card beneath to angle0 × falloff^n, each starting delay_ms × n later; leaving closes it;
//! at rest nothing asks for frames. The page's look follows the material.
use amenui::widgets::StackLook;

#[test]
fn the_look_follows_the_material() {
    let mut g = website::gallery::Gallery { material: amenui::style::Material::Plain, ..Default::default() };
    assert_eq!(website::gallery::craft::stack_now(&g).look, StackLook::Plain);
    g.material = amenui::style::Material::Print;
    assert_eq!(website::gallery::craft::stack_now(&g).look, StackLook::Thick);
    g.material = amenui::style::Material::Ascii;
    assert_eq!(website::gallery::craft::stack_now(&g).look, StackLook::Marks(amenui::style::Material::Ascii));
}
