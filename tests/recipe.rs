//! Look recipes (DESIGN.md §1.11): a preset as the difference from the built-in one, `look:v1:` and
//! base64url; every preset and an edited one go there and back unchanged; a wrong string says why;
//! the gallery takes one from its address; the render tool's PNG carries one.
use amenui::preset::{Preset, RECIPE_PREFIX, b64url, presets, un_b64url};

#[test]
fn base64url_round_trips() {
    for n in 0..40usize {
        let data: Vec<u8> = (0..n).map(|i| (i * 37 + n) as u8).collect();
        let s = b64url(&data);
        assert!(!s.contains('=') && !s.contains('+') && !s.contains('/'));
        assert_eq!(un_b64url(&s), Some(data));
    }
    assert_eq!(b64url(b"hello"), "aGVsbG8");
    assert!(un_b64url("a*b").is_none());
}

#[test]
fn every_preset_round_trips() {
    for p in presets() {
        let r = p.recipe();
        assert!(r.starts_with(RECIPE_PREFIX));
        assert!(r.len() < 80, "an unedited preset is only its name: {r}");
        assert_eq!(Preset::from_recipe(&r).expect("reads back"), p);
    }
    // Edited: a number, a step, the material, the theme.
    let mut p = presets()[3].clone();
    p.set_number("interaction.press_ms", 512.0).unwrap();
    p.set_number("transition.steps.0.a", 0.25).unwrap();
    p.material = amenui::style::Material::Halftone;
    p.theme = amenui::style::THEMES[1].name.to_owned();
    let r = p.recipe();
    let back = Preset::from_recipe(&format!("#{r}")).expect("reads back with a leading #");
    assert_eq!(back, p);
    assert_eq!(Preset::from_recipe(&back.recipe()).unwrap(), p, "stable");
    // Wrong strings.
    assert!(Preset::from_recipe("look:v2:abc").is_err());
    assert!(Preset::from_recipe(&format!("{RECIPE_PREFIX}!!")).is_err());
    let e = Preset::from_recipe(&format!("{RECIPE_PREFIX}{}", b64url(br#"{"base":"nope","set":{}}"#))).unwrap_err().to_string();
    assert!(e.contains("nope"), "{e}");
}

#[test]
fn the_gallery_takes_a_recipe_from_its_address() {
    let faces = amenui::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts");
    let mut ui = amenui::gallery::new_ui(faces, 900, 600, 1.0);
    let mut p = presets()[5].clone();
    p.set_number("loose_scale", 0.42).unwrap();
    assert!(ui.state.route(&format!("#{}", p.recipe())));
    assert_eq!(ui.state.style, Some(5));
    assert_eq!(ui.state.presets[5], p);
    assert_eq!(ui.state.recipe(), p.recipe());
}

#[test]
fn a_png_carries_its_recipe() {
    let rgba = vec![200u8; 4 * 4 * 4];
    let r = presets()[2].recipe();
    let png = amenui::offscreen::png_bytes_with(4, 4, &rgba, &[("amenui-recipe", &r)]);
    assert_eq!(amenui::offscreen::png_text(&png, "amenui-recipe"), Some(r));
}
