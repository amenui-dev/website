//! Ripple rules and recomposition (DESIGN.md §1.15, §1.16), without a GPU: what a press reaches, how each
//! element answers, that fronts from one element never stack, and that a changed string recomposes only
//! its changed glyphs and hands over to the plain new text.
use amenui::el::{El, Paint, Response, col, row, text};
use amenui::gallery::{self, GalleryUi, Msg, Page};
use amenui::morph::Recompose;
use amenui::style::Look;
use amenui::ui::Ui;

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }

fn page() -> GalleryUi {
    let mut ui = gallery::new_ui(fonts(), 1280, 1600, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("button").unwrap()));
    ui.relayout();
    run(&mut ui, 2.0);
    ui
}
/// Tick at 60 Hz up to `to`, "drawing" each frame; returns the highest `looseness` of `key` seen.
fn run_peak(ui: &mut GalleryUi, to: f32, key: &str) -> f32 {
    let mut t = ui.now();
    let mut peak = ui.looseness(key, t);
    while t < to { t = (t + 1.0 / 60.0).min(to); ui.tick(t); ui.frame_drawn(); peak = peak.max(ui.looseness(key, t)); }
    peak
}
fn run(ui: &mut GalleryUi, to: f32) { run_peak(ui, to, ""); }
fn centre(ui: &GalleryUi, key: &str) -> (f32, f32) { ui.node(key).unwrap_or_else(|| panic!("no {key}")).rect.center() }
fn press(ui: &mut GalleryUi, key: &str) { let (x, y) = centre(ui, key); ui.press(x, y); ui.release(x, y); }

#[test]
fn defaults_answer_by_what_an_element_is() {
    let ui = page();
    let n = |k: &str| ui.node(k).unwrap_or_else(|| panic!("no {k}"));
    // The counter's label is text in a control: nudged and legible; the decorative shape loosens.
    let label = ui.nodes().iter().find(|m| m.parent.is_some_and(|p| ui.nodes()[p].key == "ripple/counter") && m.text().is_some()).expect("label");
    assert_eq!(label.respond, Response::Nudge);
    assert!(label.legible);
    assert_eq!(n("ripple/n2/shape").respond, Response::Loosen);
    assert!(!n("ripple/n2/shape").legible);
    assert_eq!(n("ripple/n0/shape").respond, Response::Ignore);
    assert_eq!(n("ripple/n1/label").respond, Response::Nudge);
    // Keyed text recomposes; text in a field does not.
    assert!(n("ripple/count").recompose);
}

#[test]
fn a_press_reaches_only_its_scope() {
    let mut ui = page();
    // Own (the default for a control): the front's scope is the counter's box.
    gallery::update(&mut ui.state, Msg::RipScope(0));
    ui.relayout();
    let t0 = ui.now();
    press(&mut ui, "ripple/counter");
    let r = ui.node("ripple/counter").unwrap().rect;
    let f = ui.field();
    let j = f.fronts.iter().position(|f| f[3] > 0.0 && f[2] >= t0).expect("a front");
    assert_eq!(f.scopes[j], [r.x, r.y, r.right(), r.bottom()]);
    let peak = run_peak(&mut ui, t0 + 1.6, "ripple/n2/shape");
    assert_eq!(peak, 0.0, "a neighbour outside the counter's own scope loosened");
    // The row (Container): the loosening neighbour loosens fully, the nudged one not at all, the ignored one not.
    gallery::update(&mut ui.state, Msg::RipScope(1));
    ui.relayout();
    let t1 = ui.now();
    press(&mut ui, "ripple/counter");
    let area = ui.node("ripple/area").unwrap().rect;
    let j = ui.field().fronts.iter().position(|f| f[3] > 0.0 && f[2] >= t1).expect("a front");
    assert_eq!(ui.field().scopes[j], [area.x, area.y, area.right(), area.bottom()]);
    let mut peaks = [0.0f32; 3];
    let mut t = t1;
    while t < t1 + 1.6 {
        t += 1.0 / 60.0;
        ui.tick(t);
        for (i, p) in peaks.iter_mut().enumerate() { *p = p.max(ui.looseness(&format!("ripple/n{i}/shape"), t)); }
    }
    assert_eq!(peaks[0], 0.0, "ignore");
    assert_eq!(peaks[1], 0.0, "nudge");
    assert!(peaks[2] > 0.5, "loosen: {}", peaks[2]);
    // Text to read under a front never passes the legibility cap.
    let cap = ui.state.look().legible_cap;
    let t2 = ui.now();
    press(&mut ui, "ripple/counter");
    assert!(run_peak(&mut ui, t2 + 1.6, "ripple/n2/label") <= cap + 1e-5);
    // Plain ground sends nothing.
    let n = ui.field().fronts.iter().filter(|f| f[3] > 0.0).count();
    ui.press(1270.0, 1590.0);
    ui.release(1270.0, 1590.0);
    assert_eq!(ui.field().fronts.iter().filter(|f| f[3] > 0.0).count(), n);
}

#[test]
fn fast_presses_do_not_stack() {
    let label_of = |ui: &GalleryUi| ui.nodes().iter().find(|m| m.parent.is_some_and(|p| ui.nodes()[p].key == "ripple/counter") && m.text().is_some()).map(|m| m.key.clone()).expect("label");
    for scope in [0, 1, 2] {
        let mut one = page();
        let mut ten = page();
        for ui in [&mut one, &mut ten] { ui.state.rip_scope = scope; ui.relayout(); }
        let key = label_of(&one);
        let t0 = one.now();
        press(&mut one, "ripple/counter");
        let p1 = run_peak(&mut one, t0 + 2.5, &key);
        // Ten presses in 300 ms, by the page's own button.
        let t0 = ten.now();
        press(&mut ten, "ripple/burst");
        let p10 = run_peak(&mut ten, t0 + 0.32, &key);
        assert_eq!(ten.state.rip_count, 11, "ten presses landed");
        let fronts = ten.field().sources.iter().zip(ten.field().fronts.iter()).filter(|(s, f)| **s != 0 && f[3] > 0.0 && ten.now() - f[2] < 1.4).count();
        assert!(fronts <= 2, "fronts alive after the burst: {fronts}");
        let after = ten.now();
        let p10 = p10.max(run_peak(&mut ten, after + 2.5, &key));
        assert!(p10 <= p1 + 1e-5, "ten presses {p10} vs one {p1}");
        assert!(p10 <= ten.state.look().legible_cap + 1e-5);
        // At rest within the normal settle time after the last press.
        let l = ten.state.look();
        let settle = l.fx.wave_life.max(l.fx.enter).max(l.recompose.duration_ms / 1000.0) + 0.05;
        let mut t = after;
        while t < after + settle { t += 1.0 / 60.0; ten.tick(t); ten.frame_drawn(); }
        ten.tick(t);
        ten.frame_drawn();
        assert!(!ten.needs_frame(t), "still moving {:.2} s after the last press", t - after);
    }
}

#[test]
fn the_loosening_neighbour_is_clamped_by_loose_max() {
    let mut ui = page();
    ui.state.rip_scope = 1;
    ui.state.loose_max = 0.4;
    ui.relayout();
    let t0 = ui.now();
    press(&mut ui, "ripple/counter");
    let p = run_peak(&mut ui, t0 + 1.6, "ripple/n2/shape");
    assert!(p > 0.35 && p <= 0.4 + 1e-5, "{p}");
}

// ── Recomposition ────────────────────────────────────────────────────────────────────────────────

struct Num { s: String, keyed: bool }
fn num_view(n: &Num) -> El<()> {
    let t = text(n.s.clone(), 40.0, Paint::INK);
    let t = if n.keyed { t.key("n") } else { t };
    col().pad(40.0, 40.0).child(row().child(t))
}
fn num_ui(s: &str, keyed: bool) -> Ui<Num, ()> {
    let look = Look::default();
    let mut ui = Ui::new(Num { s: s.into(), keyed }, num_view, |_, _| {}, fonts(), 400, 200, 1.0).on_look(|_| Look::default());
    let _ = look;
    ui.tick(5.0);
    ui
}
fn set(ui: &mut Ui<Num, ()>, s: &str) { ui.state.s = s.into(); ui.relayout(); }
fn alpha_of(ui: &Ui<Num, ()>) -> Vec<f32> { ui.instances_of("n").iter().map(|i| i.paint[2]).collect() }

#[test]
fn a_changed_digit_recomposes_and_hands_over() {
    let mut ui = num_ui("Count 19", true);
    let before = ui.instances_of("n");
    set(&mut ui, "Count 20");
    let spec = Recompose::default();
    // The two digits wait under their tiles; "Count " stays and keeps its birth.
    let a = alpha_of(&ui);
    let hidden = a.iter().filter(|&&x| x == 0.0).count();
    assert_eq!(hidden, 2, "the changed glyphs are hidden while they recompose: {a:?}");
    let now = ui.instances_of("n");
    for (b, n) in before.iter().zip(&now).take(5) { assert_eq!(b.extra[3], n.extra[3], "unchanged glyphs keep their birth"); assert_eq!(b.rect, n.rect, "and their place"); }
    // Tiles are drawn beyond the page instances.
    assert!(ui.scene().instances.len() > ui.instances_of("n").len() + 20, "tiles in the scene");
    assert!(ui.needs_frame(ui.now()));
    // Mid-flight, a third value retargets from where the tiles are (no queue): still one run, still moving.
    let t = ui.now() + spec.duration_ms / 2000.0;
    ui.tick(t);
    set(&mut ui, "Count 21");
    assert_eq!(alpha_of(&ui).iter().filter(|&&x| x == 0.0).count(), 2, "both digits are still recomposing");
    // After the flight: nothing hidden, no tiles, at rest, and no condensing of the new glyphs.
    let end = ui.now() + spec.duration_ms / 1000.0 + 0.02;
    ui.tick(end);
    ui.tick(end + 0.01);
    ui.frame_drawn();
    assert!(alpha_of(&ui).iter().all(|&x| x > 0.99));
    assert_eq!(ui.scene().instances.len(), ui.instances_of("n").len());
    assert!(ui.instances_of("n").iter().all(|i| i.extra[3] <= ui.now() - 0.45), "nothing condenses after recomposing");
    let t = ui.now();
    assert!(!ui.needs_frame(t), "moving {} births {:?}", ui.moving_at(t), ui.scene().instances.iter().map(|i| i.extra[3]).collect::<Vec<_>>());
}

#[test]
fn unkeyed_text_is_replaced_and_slides_are_made_for_moved_glyphs() {
    let mut ui = num_ui("7", false);
    set(&mut ui, "8");
    assert!(alpha_of(&ui).is_empty() || ui.scene().instances.iter().all(|i| i.paint[2] > 0.0), "no recomposition without a key");
    // Keyed: a longer number pushes nothing here (one glyph, left aligned), but "9" → "10" changes both.
    let mut ui = num_ui("9", true);
    set(&mut ui, "10");
    assert_eq!(alpha_of(&ui).iter().filter(|&&x| x == 0.0).count(), 2);
}

#[test]
fn a_number_that_changes_every_frame_stays_bounded() {
    // 2,000 frames in a row, a new value each: faster than the minimum interval, so it is replaced at once.
    let mut ui = num_ui("0", true);
    let mut peak = 0usize;
    let mut t = ui.now();
    for i in 1..=2000 {
        t += 1.0 / 60.0;
        ui.tick(t);
        set(&mut ui, &format!("{}", i * 7919 % 100_000));
        ui.frame_drawn();
        peak = peak.max(ui.recompose_tiles());
        assert!(ui.scene().instances.len() < 4000, "frame {i}: {} instances", ui.scene().instances.len());
    }
    assert!(peak <= 6000, "peak {peak} tiles");
    // Every 100 ms (slower than the interval, faster than a flight): each change retargets the tiles in
    // flight, and their number stays bounded instead of multiplying.
    let mut peak = 0usize;
    for i in 1..=300 {
        for _ in 0..6 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
        set(&mut ui, &format!("{}", i * 7919 % 100_000));
        peak = peak.max(ui.recompose_tiles());
    }
    let spec = Recompose::default();
    assert!(peak as f32 <= spec.max_tiles, "peak {peak} tiles");
    assert!(peak <= 6 * spec.count_max as usize * 3, "peak {peak} tiles for five digits");
    // And it comes to rest.
    for _ in 0..60 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
    assert_eq!(ui.recompose_tiles(), 0);
    assert!(!ui.needs_frame(t));
}

struct Pic;
fn pic_view(_: &Pic) -> El<()> {
    col().pad(40.0, 40.0)
        .child(row().key("thumb").on_press(()).child(amenui::el::picture(amenui::demo::BUST, 1.0).key("in-control").width(120.0)))
        .child(amenui::el::picture(amenui::demo::BUST, 1.0).key("loose-pic").width(120.0))
}

#[test]
fn a_picture_inside_a_control_is_held_to_the_legibility_cap() {
    let mut ui = Ui::new(Pic, pic_view, |_, _| {}, fonts(), 600, 600, 1.0).on_look(|_| Look::default());
    ui.tick(2.0);
    let n = |k: &str| ui.node(k).unwrap_or_else(|| panic!("no {k}")).legible;
    assert!(n("in-control"), "a picture in a control is to be read");
    assert!(!n("loose-pic"), "a picture on its own is decoration");
    // Its instances carry the legible push code, which the material reads to hold the lens to the cap.
    assert!(ui.instances_of("in-control").iter().all(|i| i.resp[1] > 1.5), "{:?}", ui.instances_of("in-control").iter().map(|i| i.resp).collect::<Vec<_>>());
    assert!(ui.instances_of("loose-pic").iter().all(|i| i.resp[1] < 1.5));
    // A front through the control loosens the picture at most to the cap.
    let (x, y) = ui.node("thumb").unwrap().rect.center();
    ui.press(x, y);
    let cap = Look::default().legible_cap;
    let mut t = ui.now();
    for _ in 0..30 { t += 1.0 / 60.0; ui.tick(t); assert!(ui.looseness("in-control", t) <= cap + 1e-4); }
    ui.release(x, y);
}
