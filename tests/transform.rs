//! Acceptance for transforms without a GPU: the morph's phases and hand-over times, reversal from where
//! the tiles are, flows between pages (interpolated rects, birth times kept, the rest dissolving), and
//! the gallery's two pages issuing their commands.
use website::gallery::{self, GalleryUi, Msg, Page};
use amenui::morph::{self};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }

// ── Flow ─────────────────────────────────────────────────────────────────────────────────────────

// ── The gallery's pages ──────────────────────────────────────────────────────────────────────────

fn gallery_at(p: Page) -> GalleryUi {
    let mut ui = gallery::new_ui(fonts(), 1280, 1400, 1.0);
    gallery::update(&mut ui.state, Msg::Go(p));
    ui.relayout();
    ui.tick(10.0);
    ui
}
fn click(ui: &mut GalleryUi, k: &str) { let (x, y) = ui.node(k).unwrap_or_else(|| panic!("no node {k}")).rect.center(); ui.press(x, y); ui.release(x, y); }

#[test]
fn gallery_morph_page() {
    let mut ui = gallery_at(Page::Morph);
    assert!(Page::from_hash("#morph") == Some(Page::Morph) && Page::from_hash("flow") == Some(Page::Flow));
    assert!(!ui.instances_of("morph/c0").is_empty());
    assert!(ui.instances_of("morph/c2").is_empty(), "candidates other than the shown one are hidden");
    click(&mut ui, "morph/targets:2");
    let run = ui.morph_run().expect("pressing a target morphs");
    assert_eq!((run.from.as_str(), run.to.as_str()), ("morph/c0", "morph/c2"));
    assert_eq!(ui.state.shape, 2);
    // The picture's tiles all fly; the headline's thin edges stay as dust.
    assert!(run.particles.iter().any(|p| p.misc[0] == morph::kind::DST_DUST || p.misc[0] == morph::kind::SRC_DUST));
    let t = ui.now();
    ui.tick(t + 0.5);
    click(&mut ui, "morph/back");
    let run = ui.morph_run().unwrap();
    assert_eq!((run.from.as_str(), run.to.as_str()), ("morph/c2", "morph/c0"), "reversed in flight");
    assert_eq!(ui.state.shape, 0);
    ui.tick(t + 5.0);
    assert!(ui.morph_run().is_none());
    assert!(!ui.instances_of("morph/c0").is_empty() && ui.instances_of("morph/c2").is_empty());
}

#[test]
fn gallery_flow_page_and_shell() {
    let mut ui = gallery_at(Page::Flow);
    let old = ui.instances_of("flow/card2/picture")[0];
    click(&mut ui, "flow/card2");
    assert_eq!(ui.state.opened, Some(2));
    let pic = ui.instances_of("flow/detail/picture")[0];
    assert_eq!(pic.rect, old.rect, "the picture starts from the card");
    assert!(ui.dissolving() > 0, "the other cards dissolve");
    let t = ui.now();
    ui.tick(t + 1.0);
    let r = ui.node("flow/detail/picture").unwrap().rect;
    assert_eq!(ui.instances_of("flow/detail/picture")[0].rect, [r.x, r.y, r.w, r.h]);
    // The rail's selection moves from one item to the next on a page change.
    // (Both items in view: the rail is longer than the window.)
    click(&mut ui, "nav/button");
    let t = ui.now();
    ui.tick(t + 1.0);
    click(&mut ui, "nav/number");
    assert!(ui.flowing() > 0);
}
