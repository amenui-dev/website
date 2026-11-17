//! Look recipes (DESIGN.md §1.11): a preset as the difference from the built-in one, `look:v1:` and
//! base64url; every preset and an edited one go there and back unchanged; a wrong string says why;
//! the gallery takes one from its address; the render tool's PNG carries one.
use amenui::preset::presets;

#[test]
fn the_gallery_takes_a_recipe_from_its_address() {
    let faces = amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts");
    let mut ui = website::gallery::new_ui(faces, 900, 600, 1.0);
    let mut p = presets()[5].clone();
    p.set_number("loose_scale", 0.42).unwrap();
    assert!(ui.state.route(&format!("#{}", p.recipe())));
    assert_eq!(ui.state.style, Some(5));
    assert_eq!(ui.state.presets[5], p);
    assert_eq!(ui.state.recipe(), p.recipe());
}
