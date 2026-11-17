//! Acceptance for the annotation layer (DESIGN.md §1.7), without a GPU: the records of a known tree
//! (rects, numbers, sizes, kinds), density, scope and callouts; label packing never overlaps; hover and
//! focus modes give the element and its ancestors; frames draw on and numbers count up.

#[test]
fn corner_marks_never_lie_on_ink() {
    // The Loose Thread page, its hover frames on each piece of small type, the title and the picture
    // (DESIGN.md §4.4): no pixel of a corner mark is inside any glyph's box.
    use website::gallery;
    let mut ui = gallery::new_ui(amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts"), 1280, 1000, 1.0);
    ui.pointer_leave();
    assert!(ui.state.route("style-loose-thread"));
    ui.relayout();
    let mut t = 0.0;
    for _ in 0..60 { t += 1.0 / 60.0; ui.tick(t); }
    let h = "style-loose-thread";
    let mut framed = 0;
    for k in ["kicker", "title", "caption", "pic", "body", "binary", "invert"] {
        let (x, y) = ui.node(&format!("{h}/{k}")).unwrap_or_else(|| panic!("no {k}")).rect.center();
        ui.pointer_move(x, y);
        for _ in 0..30 { t += 1.0 / 60.0; ui.tick(t); }
        let spread = amenui::text::SPREAD as f32;
        let glyphs: Vec<[f32; 4]> = ui.scene().instances.iter().take(ui.scene().instances.len()).enumerate()
            .filter(|(i, g)| g.paint[3] == amenui::scene::GLYPH && ui.owner_key(*i).is_some())
            .map(|(_, g)| [g.rect[0] + spread, g.rect[1] + spread, g.rect[0] + g.rect[2] - spread, g.rect[1] + g.rect[3] - spread]).collect();
        let marks: Vec<[f32; 4]> = ui.scene().annot.iter().filter(|a| a.paint[3] == amenui::scene::SHAPE && a.paint[1] < 0.5 && a.rect[2].min(a.rect[3]) <= 1.0 && a.rect[2].max(a.rect[3]) <= 10.0).map(|a| a.rect).collect();
        if !marks.is_empty() { framed += 1; } else { eprintln!("no corner marks over {k}"); }
        for m in &marks {
            for g in &glyphs {
                let over = m[0] < g[2] && m[0] + m[2] > g[0] && m[1] < g[3] && m[1] + m[3] > g[1];
                assert!(!over, "over {k}: a corner mark {m:?} lies on a glyph {g:?}");
            }
        }
    }
    assert!(framed >= 4, "corner marks over only {framed} of 7");
}
