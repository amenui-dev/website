//! Acceptance for step 4 without a GPU: the morph's phases and hand-over times, reversal from where
//! the tiles are, flows between pages (interpolated rects, birth times kept, the rest dissolving), and
//! the gallery's two pages issuing their commands.
use amenui::el::{El, Paint, col, space, text};
use amenui::gallery::{self, GalleryUi, Msg, Page};
use amenui::morph::{self, Cmd, Ease, FlowSpec, MorphSpec};
use amenui::ui::{Ui, ramp};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }

// ── Morph ────────────────────────────────────────────────────────────────────────────────────────

#[derive(Default)]
struct Two { cmds: Vec<Cmd> }
fn two(_: &Two) -> El<()> {
    col().key("root").pad(20.0, 20.0).gap(40.0)
        .child(text("变形", 64.0, Paint::INK).key("a"))
        .child(space(160.0, 80.0).key("b").radius(12.0).fill(Paint::ACCENT).hidden_until_morphed(true))
}
fn two_ui() -> Ui<Two, ()> {
    let mut ui = Ui::new(Two::default(), two, |_, _| {}, fonts(), 600, 400, 1.0).on_commands(|s| std::mem::take(&mut s.cmds));
    ui.tick(10.0);
    ui
}
fn loose_of(ui: &Ui<Two, ()>, key: &str) -> Option<f32> { ui.instances_of(key).first().map(|i| i.extra[2]) }

#[test]
fn morph_hands_over_at_the_right_times() {
    let mut ui = two_ui();
    let spec = MorphSpec::default();
    assert!(loose_of(&ui, "b").is_none(), "the target is laid out but hidden");
    assert!(ui.node("b").is_some());
    ui.morph("a", "b", spec);
    let run = ui.morph_run().expect("a run");
    assert!(!run.particles.is_empty());
    assert_eq!(run.tile, 5.0, "tiles are the lattice cells");
    let (l, d, t) = (spec.loosen_ms / 1000.0, spec.duration_ms / 1000.0, spec.tighten_ms / 1000.0);
    // Phase 1: the source loosens, the target is hidden, no tiles yet.
    ui.tick(10.0 + l * 0.5);
    let k = loose_of(&ui, "a").expect("source drawn");
    assert!((k - 0.5).abs() < 0.02, "half-way loose: {k}");
    assert!(loose_of(&ui, "b").is_none());
    assert!(ui.morph_run().unwrap().clock(ui.now()).is_none());
    ui.tick(10.0 + l - 0.001);
    assert!(loose_of(&ui, "a").expect("source drawn") > 0.99);
    // Phase 2: from the hand-over on, only the tiles.
    ui.tick(10.0 + l + 0.001);
    assert!(loose_of(&ui, "a").is_none() && loose_of(&ui, "b").is_none());
    let clock = ui.morph_run().unwrap().clock(ui.now()).expect("in flight");
    assert!(clock < 0.01);
    // Every tile is at its source cell when the flight starts and at its target cell when it ends.
    let run = ui.morph_run().unwrap();
    for p in &run.particles {
        assert_eq!(morph::progress(p, 0.0, spec.ease), 0.0);
        assert_eq!(morph::progress(p, d - 0.001, spec.ease), 1.0, "landed before the flight is over: {p:?}");
    }
    // Phase 3: the target, fully loose, tightening; it does not condense on top of that.
    ui.tick(10.0 + l + d + 0.001);
    let b = ui.instances_of("b");
    assert!(!b.is_empty() && loose_of(&ui, "a").is_none());
    assert!(b[0].extra[2] > 0.99, "{}", b[0].extra[2]);
    assert!(b[0].extra[3] < 0.0, "fully born");
    ui.tick(10.0 + l + d + t * 0.5);
    assert!((loose_of(&ui, "b").unwrap() - 0.5).abs() < 0.02);
    // Over: the view decides again (here the source is not hidden, the target is).
    ui.tick(10.0 + l + d + t + 0.01);
    assert!(ui.morph_run().is_none());
    assert_eq!(loose_of(&ui, "a"), Some(0.0));
}

#[test]
fn reversal_turns_the_tiles_around_where_they_are() {
    let mut ui = two_ui();
    let spec = MorphSpec { ease: Ease::OutExpo, ..MorphSpec::default() };
    ui.morph("a", "b", spec);
    let t1 = 10.0 + spec.loosen_ms / 1000.0;
    ui.tick(t1 + 0.4);
    let clock = ui.morph_run().unwrap().clock(ui.now()).expect("in flight");
    let before: Vec<f32> = ui.morph_run().unwrap().particles.iter().map(|p| morph::progress(p, clock, spec.ease)).collect();
    assert!(before.iter().any(|&u| u > 0.2 && u < 1.0), "mid-flight");
    // The reverse of the morph in progress, as `update` would ask for it.
    ui.state.cmds.push(Cmd::Morph { from: "b".into(), to: "a".into(), spec });
    ui.relayout();
    let run = ui.morph_run().expect("still running");
    assert_eq!((run.from.as_str(), run.to.as_str()), ("b", "a"));
    let after: Vec<f32> = run.particles.iter().map(|p| morph::progress(p, 0.0, spec.ease)).collect();
    assert!(before == after, "the reverse starts from the current positions");
    assert!(run.particles.iter().all(|p| p.run[1] == 0.0));
    // It ends with the source back, tightening.
    let end = run.end;
    ui.tick(run.dst_from + 0.001);
    assert!(loose_of(&ui, "a").unwrap() > 0.99 && loose_of(&ui, "b").is_none());
    ui.tick(end + 0.01);
    assert!(ui.morph_run().is_none());
}

#[test]
fn reversal_while_loosening_tightens_again() {
    let mut ui = two_ui();
    let spec = MorphSpec::default();
    ui.morph("a", "b", spec);
    ui.tick(10.0 + 0.11);
    let k = loose_of(&ui, "a").unwrap();
    ui.morph("b", "a", spec);
    let run = ui.morph_run().unwrap();
    assert!(run.flight.is_none());
    assert!((ramp(run.src_ramp, ui.now()) - k).abs() < 1e-4, "from where it was");
    ui.tick(10.0 + 0.11 + spec.tighten_ms / 1000.0 * k * 0.5);
    assert!(loose_of(&ui, "a").unwrap() < k);
    ui.tick(11.5);
    assert!(ui.morph_run().is_none());
    assert_eq!(loose_of(&ui, "a"), Some(0.0));
}

// ── Flow ─────────────────────────────────────────────────────────────────────────────────────────

#[derive(Default)]
struct Pages { second: bool, cmds: Vec<Cmd> }
fn pages(s: &Pages) -> El<bool> {
    let root = col().key("root").pad(20.0, 20.0).gap(20.0);
    if !s.second {
        root.child(text("标题", 16.0, Paint::INK).key("p1/title").flow("t"))
            .child(space(80.0, 40.0).key("p1/box").fill(Paint::INK).flow("box"))
            .child(text("只在第一页", 16.0, Paint::TEXT).key("p1/only"))
    } else {
        root.child(space(1.0, 200.0))
            .child(space(240.0, 120.0).key("p2/box").fill(Paint::INK).flow("box"))
            .child(text("标题", 32.0, Paint::INK).key("p2/title").flow("t"))
    }
}
fn go(s: &mut Pages, second: bool) { s.second = second; s.cmds.push(Cmd::Flow(FlowSpec::default())); }

#[test]
fn shared_parts_flow_between_pages() {
    let mut ui = Ui::new(Pages::default(), pages, go, fonts(), 800, 600, 1.0).on_commands(|s| std::mem::take(&mut s.cmds));
    ui.tick(5.0);
    let old_box = ui.instances_of("p1/box")[0];
    let old_title = ui.instances_of("p1/title");
    assert_eq!(old_title.len(), 2);
    let born = old_box.extra[3];
    // A page change, as a press would send it.
    ui.state.second = true;
    ui.state.cmds.push(Cmd::Flow(FlowSpec::default()));
    ui.relayout();
    let spec = FlowSpec::default();
    let target = {
        // Where layout puts it, once the flow is over.
        let r = ui.node("p2/box").unwrap().rect;
        [r.x, r.y, r.w, r.h]
    };
    let at = |ui: &Ui<Pages, bool>| ui.instances_of("p2/box")[0];
    // t = 0: still where it was, and it kept its birth time (no condensing again).
    let b = at(&ui);
    assert_eq!(b.rect, old_box.rect);
    assert_eq!(b.extra[3], born);
    assert_eq!(b.extra[2], 0.0);
    // Half-way: half-way there (in-out cubic is ½ at ½), loosened by the bump's peak.
    ui.tick(5.0 + spec.flow_ms / 1000.0 * 0.5);
    let b = at(&ui);
    for k in 0..4 { assert!((b.rect[k] - (old_box.rect[k] + target[k]) * 0.5).abs() < 0.01, "{k}: {:?} {:?}", b.rect, target); }
    assert!((b.extra[2] - spec.flow_loose).abs() < 1e-4);
    // The end: exactly the new rect, crisp.
    ui.tick(5.0 + spec.flow_ms / 1000.0 + 0.001);
    let b = at(&ui);
    assert_eq!(b.rect, target);
    assert_eq!(b.extra[2], 0.0);
    assert_eq!(ui.flowing(), 0);
    // The title's glyphs (same text, twice the size) flowed too, and kept their birth times.
    let t = ui.instances_of("p2/title");
    assert_eq!(t.len(), 2);
    assert!(t.iter().zip(&old_title).all(|(n, o)| n.extra[3] == o.extra[3]));
}

#[test]
fn unshared_parts_dissolve() {
    let mut ui = Ui::new(Pages::default(), pages, go, fonts(), 800, 600, 1.0).on_commands(|s| std::mem::take(&mut s.cmds));
    ui.tick(5.0);
    let only = ui.instances_of("p1/only").len();
    assert!(only >= 4);
    ui.state.second = true;
    ui.state.cmds.push(Cmd::Flow(FlowSpec::default()));
    ui.relayout();
    assert_eq!(ui.dissolving(), only, "every glyph of the text only on the first page");
    // They loosen and fade, then they are gone.
    ui.tick(5.09);
    let fading = ui.scene().instances.iter().filter(|i| i.extra[2] > 0.4 && i.paint[2] < 0.6).count();
    assert_eq!(fading, only);
    ui.tick(5.2);
    assert_eq!(ui.dissolving(), 0);
    // Without a flow command a page change just replaces (nothing dissolves).
    ui.state.second = false;
    ui.relayout();
    assert_eq!(ui.dissolving(), 0);
}

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


// ── Sampling follows transforms and poses (round 16, DESIGN.md §1.26) ──────────────────────────────

#[derive(Default)]
struct Posed { cmds: Vec<Cmd> }
/// A word whose glyphs are moved 120 px right and down and turned a quarter: its tiles must start
/// where it is drawn, not where the layout put it.
fn posed(_: &Posed) -> El<()> {
    col().key("root").pad(20.0, 20.0).gap(40.0)
        .child(text("III", 48.0, Paint::INK).key("a").pose(|_| amenui::el::Pose { dx: 120.0, dy: 140.0, angle: std::f32::consts::FRAC_PI_2, ..Default::default() }))
        .child(space(80.0, 40.0).key("b").fill(Paint::ACCENT).hidden_until_morphed(true))
}

#[test]
fn a_posed_glyph_loosens_from_where_it_is_drawn() {
    let mut ui = Ui::new(Posed::default(), posed, |_, _| {}, fonts(), 600, 400, 1.0).on_commands(|s| std::mem::take(&mut s.cmds));
    ui.tick(10.0);
    let insts = ui.instances_of("a");
    assert_eq!(insts.len(), 3);
    // Every instance is turned (its own transform) and moved.
    assert!(insts.iter().all(|i| i.xf != [0.0; 4]));
    let laid = ui.node("a").unwrap().rect;
    let src = morph::cells(&insts, 4.0, ui.atlas());
    let inked: Vec<&morph::Cell> = src.iter().filter(|c| c.cover > 0.2).collect();
    assert!(!inked.is_empty());
    // The covered cells are near the posed place, not over the laid-out text.
    for c in &inked {
        assert!(c.x > laid.x + 60.0 && c.y > laid.y + 80.0, "a cell at ({}, {}) is where the layout put the text ({laid:?})", c.x, c.y);
    }
    // A vertical bar turned a quarter is a horizontal bar: the ink spans more across than down.
    let (x0, x1) = inked.iter().fold((f32::MAX, f32::MIN), |a, c| (a.0.min(c.x), a.1.max(c.x)));
    let (y0, y1) = inked.iter().fold((f32::MAX, f32::MIN), |a, c| (a.0.min(c.y), a.1.max(c.y)));
    assert!(x1 - x0 > 1.5 * (y1 - y0), "turned glyphs lie across: {}×{}", x1 - x0, y1 - y0);
    // The morph starts its tiles there too.
    ui.morph("a", "b", MorphSpec::default());
    let run = ui.morph_run().expect("a run");
    let n = run.particles.len() as f32;
    let cy = run.particles.iter().map(|p| p.ab[1]).sum::<f32>() / n;
    assert!(cy > laid.y + 80.0, "tiles start at the posed glyphs (mean y {cy}, laid out at {})", laid.y);
}

#[test]
fn a_turned_shape_covers_its_turned_box() {
    let i = amenui::scene::Instance { rect: [100.0, 100.0, 80.0, 10.0], paint: [1.0, 0.0, 1.0, amenui::scene::SHAPE], clip: [-1e5, -1e5, 1e5, 1e5], xf: amenui::scene::Instance::turned(std::f32::consts::FRAC_PI_2, 1.0, 1.0), ..Default::default() };
    // Centre (140, 105): turned, it runs from y 65 to 145 at x 140.
    assert!(morph::cover_at(&i, 140.0, 70.0, &[]) > 0.9);
    assert!(morph::cover_at(&i, 105.0, 105.0, &[]) < 0.01, "the untransformed ends are empty");
    // A fill in the ground's tone is an outline too; only `ink_at` leaves it out.
    let ground = amenui::scene::Instance { paint: [0.0, 0.0, 1.0, amenui::scene::SHAPE], xf: [0.0; 4], ..i };
    assert!(morph::cover_at(&ground, 140.0, 105.0, &[]) > 0.9);
    assert_eq!(morph::ink_at(&ground, 140.0, 105.0, &[]), 0.0);
}
