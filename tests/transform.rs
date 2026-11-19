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
    // The path strip steps on to the next page (this page is in the Lab); the strip itself stays put.
    let strip = ui.node("strip").unwrap().rect;
    click(&mut ui, "strip/next");
    // Page change (05.4) is followed by Decode (05.5) on the Lab's path.
    assert_eq!(ui.state.page, Page::Decode);
    assert_eq!(ui.node("strip").unwrap().rect, strip);
}

// ── Route transitions inside the running app (F.7) ───────────────────────────────────────────────

fn routed(path: &str) -> GalleryUi {
    let mut ui = gallery::new_ui(amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts"), 1440, 900, 1.0);
    ui.pointer_leave();
    assert!(ui.state.route(path));
    ui.relayout();
    ui.tick(3.0);
    ui
}
fn nav(ui: &mut GalleryUi, path: &str, at: (f32, f32)) -> Vec<amenui::morph::Cmd> {
    ui.state.pressed_at = Some(at);
    assert!(ui.state.route(path), "{path}");
    let cmds = ui.state.cmds.clone();
    ui.relayout();
    cmds
}
fn leads(cmds: &[amenui::morph::Cmd]) -> (usize, usize) {
    use amenui::morph::Cmd;
    (cmds.iter().filter(|c| matches!(c, Cmd::Morph { .. } | Cmd::Flow(_))).count(), cmds.iter().filter(|c| matches!(c, Cmd::Commit { .. } | Cmd::Ripplr { .. })).count())
}

#[test]
fn a_change_of_section_commits_from_the_pressed_link_at_its_own_speed() {
    let mut ui = routed("components/button/");
    let cmds = nav(&mut ui, "lab/", (300.0, -24.0));
    assert!(matches!(cmds.as_slice(), [amenui::morph::Cmd::Commit { x, y, .. }] if *x == 300.0 && *y == -24.0), "{cmds:?}");
    assert!(ui.split().is_some());
    assert_eq!(ui.state.look().fx.wave_speed, gallery::SECTION_FRONT);
    // Inside a section the shared parts flow, and there is no front.
    let cmds = nav(&mut ui, "lab/lumen/", (200.0, 300.0));
    assert_eq!(leads(&cmds), (1, 0));
    assert!(ui.flowing() > 0, "the live cell flies to its page");
}

#[test]
fn a_card_name_flies_to_the_specimen_title_and_reduced_motion_fades() {
    let mut ui = routed("components/");
    let cmds = nav(&mut ui, "components/switch/", (600.0, 300.0));
    assert_eq!(leads(&cmds), (1, 0));
    assert!(ui.flowing() > 0);
    // With reduced motion a route change is an opacity change (F.10): one flow that moves nothing and
    // fades the old page out, no front.
    ui.state.body.reduced = true;
    let cmds = nav(&mut ui, "scenes/", (600.0, 300.0));
    assert!(matches!(cmds.as_slice(), [amenui::morph::Cmd::Flow(f)] if f.flow_ms == 0.0 && f.flow_loose == 0.0 && f.out_ms > 0.0), "{cmds:?}");
    // From home every door has at most one lead and one front.
    let mut ui = routed("/");
    let cmds = nav(&mut ui, "components/", (900.0, 860.0));
    let (l, f) = leads(&cmds);
    assert!(l <= 1 && f <= 1, "{cmds:?}");
}
