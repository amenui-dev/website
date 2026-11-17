//! Acceptance for the annotation layer (DESIGN.md §1.7), without a GPU: the records of a known tree
//! (rects, numbers, sizes, kinds), density, scope and callouts; label packing never overlaps; hover and
//! focus modes give the element and its ancestors; frames draw on and numbers count up.
use lab::el::{El, Paint, col, row, space, text};
use lab::gaze::{self, Annot};
use lab::style::{Gaze, GazeMode, gaze as kinds};
use lab::ui::{Rect, Ui};

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }

struct S { gaze: Gaze }
fn view(_: &S) -> El<()> {
    col().key("root").pad(20.0, 20.0).gap(10.0)
        .child(row().key("bar").width(400.0).height(40.0).fill(Paint::FAINT).child(space(60.0, 20.0).key("chip").fill(Paint::ACCENT)))
        .child(col().key("scope").gaze_scope().width(500.0).pad(10.0, 10.0).gap(8.0)
            .child(space(200.0, 50.0).key("a").fill(Paint::INK).annotate("电源开关"))
            .child(space(100.0, 30.0).key("b").fill(Paint::INK).on_press(()))
            .child(text("说明文字", 14.0, Paint::TEXT).key("c")))
}
fn ui(g: Gaze) -> Ui<S, ()> {
    Ui::new(S { gaze: g }, view, |_, _| {}, fonts(), 800, 600, 1.0).on_gaze(|s| s.gaze)
}
fn all() -> Gaze { Gaze { mode: GazeMode::All, kinds: kinds::FRAME | kinds::INDEX | kinds::SIZE, density: 50.0, draw_ms: 300.0, ..Gaze::default() } }

#[test]
fn records_follow_the_layout() {
    let u = ui(all());
    let a: &[Annot] = u.annots();
    // Only inside the scope: the scope itself is the outermost, then a, b, c in tree order.
    let keys: Vec<&str> = a.iter().map(|r| r.key.as_str()).collect();
    assert_eq!(keys, ["scope", "a", "b", "c"]);
    assert_eq!(a.iter().map(|r| r.index).collect::<Vec<_>>(), [1, 2, 3, 4]);
    for r in a { assert_eq!(r.rect, u.node(&r.key).unwrap().rect, "{} is framed at its real rect", r.key); }
    let ra = &a[1];
    assert_eq!((ra.rect.w, ra.rect.h), (200.0, 50.0));
    assert_eq!(ra.label.as_deref(), Some("电源开关"));
    assert_eq!(a[2].kind, "control");
    assert_eq!(a[3].kind, "text");
    assert_eq!(ra.kind, "shape");
    // Instances for stage 3 exist, and the page's own scene is untouched by them.
    assert!(!u.scene().annot.is_empty());
}

#[test]
fn density_keeps_the_largest_and_callouts_can_be_the_only_ones() {
    let u = ui(Gaze { density: 2.0, ..all() });
    let keys: Vec<&str> = u.annots().iter().map(|r| r.key.as_str()).collect();
    // A callout comes first, then the largest.
    assert_eq!(keys, ["scope", "a"]);
    let u = ui(Gaze { only_marked: true, ..all() });
    let keys: Vec<&str> = u.annots().iter().map(|r| r.key.as_str()).collect();
    assert_eq!(keys, ["a"]);
}

#[test]
fn hover_and_focus_give_the_ancestor_chain() {
    let mut u = ui(Gaze { mode: GazeMode::Hover, ..all() });
    assert!(u.annots().is_empty(), "nothing under no pointer");
    let (x, y) = u.node("b").unwrap().rect.center();
    u.pointer_move(x, y);
    u.tick(0.1);
    let keys: Vec<&str> = u.annots().iter().map(|r| r.key.as_str()).collect();
    assert_eq!(keys, ["scope", "b"]);
    // The chain itself, outside the scope rule, reaches the root.
    let i = u.nodes().iter().position(|n| n.key == "b").unwrap();
    let chain: Vec<&str> = gaze::chain(u.nodes(), i).into_iter().map(|k| u.nodes()[k].key.as_str()).collect();
    assert_eq!(chain, ["b", "scope", "root"]);
    let mut u = ui(Gaze { mode: GazeMode::Focus, ..all() });
    u.press(x, y);
    u.tick(0.2);
    let keys: Vec<&str> = u.annots().iter().map(|r| r.key.as_str()).collect();
    assert_eq!(keys, ["scope", "b"]);
}

#[test]
fn labels_never_overlap() {
    // Many labels wanting nearly the same place, in a short column.
    let items: Vec<(f32, f32)> = (0..12).map(|i| (100.0 + (i % 3) as f32 * 4.0, 14.0)).collect();
    let tops = gaze::pack(&items, 0.0, 400.0, 6.0);
    let mut spans: Vec<(f32, f32)> = tops.iter().zip(&items).map(|(t, (_, h))| (*t, t + h)).collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    for w in spans.windows(2) { assert!(w[1].0 >= w[0].1 + 6.0 - 1e-3, "{:?} overlaps {:?}", w[0], w[1]); }
    assert!(spans.iter().all(|s| s.0 >= 0.0 && s.1 <= 400.0));
    // Overflowing the bottom pushes them back up.
    let items: Vec<(f32, f32)> = (0..5).map(|_| (390.0, 14.0)).collect();
    let tops = gaze::pack(&items, 0.0, 400.0, 6.0);
    assert!(tops.iter().all(|t| t + 14.0 <= 400.0 + 1e-3));
    // Placed labels, in both columns, inside the scope.
    let annots: Vec<Annot> = (0..8).map(|i| Annot {
        node: i, key: format!("k{i}"), rect: Rect::new(if i % 2 == 0 { 150.0 } else { 600.0 }, 200.0 + i as f32 * 3.0, 40.0, 20.0), index: i as u32 + 1, kind: "shape", label: None,
    }).collect();
    let scope = Rect::new(0.0, 0.0, 800.0, 500.0);
    let boxes = gaze::place_labels(&annots, scope, |_| (80.0, 14.0));
    for (i, (side, b)) in boxes.iter().enumerate() {
        assert_eq!(*side, if i % 2 == 0 { gaze::Side::Left } else { gaze::Side::Right });
        for (j, (_, c)) in boxes.iter().enumerate() {
            if i != j { assert!(b.intersect(c).w * b.intersect(c).h == 0.0, "labels {i} and {j} overlap"); }
        }
    }
}

#[test]
fn frames_draw_on_and_numbers_count_up() {
    assert_eq!(gaze::drawn(0.0, 300.0), 0.0);
    assert_eq!(gaze::drawn(0.15, 300.0), 0.5);
    assert_eq!(gaze::drawn(1.0, 300.0), 1.0);
    assert_eq!(gaze::count(12.0, 0.0), 0.0);
    assert_eq!(gaze::count(12.0, 1.0), 12.0);
    let mut u = ui(all());
    let early = u.scene().annot.len();
    u.tick(1.0);
    // Fully drawn: the frame's corners are at full length; the records do not change.
    assert_eq!(u.annots().len(), 4);
    assert!(u.scene().annot.len() >= early);
}

// ── Edge notes and the field map's readouts (DESIGN.md §4.2, §4.3) ────────────────────────────────
struct R { gaze: Gaze, cmds: Vec<lab::morph::Cmd> }
fn readout_view(_: &R) -> El<()> {
    col().key("root").pad(20.0, 20.0)
        .child(col().key("scope").gaze_scope().width(520.0).height(420.0).pad(16.0, 16.0).gap(10.0)
            .child(space(200.0, 60.0).key("deco").fill(Paint::INK).respond(lab::el::Response::Loosen))
            .child(text("说明文字", 14.0, Paint::TEXT).key("c")))
}
fn readouts(kinds: u16) -> Ui<R, ()> {
    let g = Gaze { mode: GazeMode::All, kinds, density: 10.0, draw_ms: 0.0, ..Gaze::default() };
    let mut u = Ui::new(R { gaze: g, cmds: vec![] }, readout_view, |_, _| {}, fonts(), 800, 600, 1.0)
        .on_gaze(|s| s.gaze).on_commands(|s| std::mem::take(&mut s.cmds))
        .on_look(|_| lab::style::Look { material: lab::style::Material::Plain, ..lab::style::Look::default() });
    u.pointer_leave();
    u.tick(1.0);
    u
}
fn stir(u: &mut Ui<R, ()>, x: f32, y: f32) {
    u.state.cmds.push(lab::morph::Cmd::Stir { at: lab::morph::StirAt::Point([x, y]), v: [1.0, 0.0], energy: 1.0, radius: 40.0, scope: lab::el::Scope::Global });
    u.relayout();
}

#[test]
fn edge_notes_say_what_is_true_now() {
    let mut u = readouts(kinds::EDGE);
    let sc = u.node("scope").unwrap().rect;
    let r = u.readout().clone();
    assert_eq!(r.scope, sc);
    let texts: Vec<&str> = r.notes.iter().map(|n| n.0.as_str()).collect();
    assert_eq!(texts, [format!("{:.0} × {:.0}", sc.w, sc.h).as_str(), "x — y —", "plain · dark", "loose 0.00", "0 nodes"]);
    // The hairline is on the scope's right edge, one device pixel inside it.
    assert_eq!(r.edge_x, Some(sc.right() - 1.0));
    // Every note inside the scope, none on another.
    for (i, (_, a)) in r.notes.iter().enumerate() {
        assert!(a.x >= sc.x && a.y >= sc.y && a.right() <= sc.right() && a.bottom() <= sc.bottom(), "note {i} at {a:?} is outside {sc:?}");
        for (_, b) in &r.notes[i + 1..] { let o = a.intersect(b); assert!(o.w <= 0.0 || o.h <= 0.0, "{a:?} overlaps {b:?}"); }
    }
    // The pointer in the scope; a stir: the notes follow.
    u.pointer_move(sc.x + 100.0, sc.y + 50.0);
    stir(&mut u, sc.x + 120.0, sc.y + 40.0);
    u.tick(1.05);
    let r = u.readout().clone();
    let now = u.now();
    assert_eq!(r.notes[1].0, "x 100 y 50");
    assert_eq!(r.mark_y, Some(sc.y + 50.0));
    assert_eq!(r.notes[3].0, format!("loose {:.2}", u.looseness("root", now)));
    assert!(u.looseness("root", now) > 0.0);
    assert_eq!(r.notes[4].0, format!("{} nodes", u.fieldmap().nonzero()));
    assert!(u.fieldmap().nonzero() > 0);
}

#[test]
fn the_waveform_and_the_level_read_the_field_map() {
    let mut u = readouts(kinds::OSC | kinds::LEVEL);
    let g = Gaze::default();
    let sc = u.node("scope").unwrap().rect;
    // Nothing in the map: a straight line, no level, and no frames for two seconds.
    let r = u.readout().clone();
    assert!(!r.osc.is_empty());
    let base = sc.y + sc.h * g.osc_y;
    assert!(r.osc.iter().all(|p| p[1] == base), "flat at {base}: {:?}", &r.osc[..4]);
    assert_eq!((r.level, r.level_h), (0.0, 0.0));
    let (mut t, mut drawn) = (1.0, 0);
    u.frame_drawn();
    while t < 3.0 { t += 1.0 / 60.0; u.tick(t); if u.needs_frame(t) { drawn += 1; } u.frame_drawn(); }
    assert_eq!(drawn, 0, "a still map asks for no frames");
    // A stir at x: the highest point is within a cell of x, as high as the energy there × osc_amp.
    let (x, y) = (sc.x + 300.0, sc.y + 200.0);
    stir(&mut u, x, y);
    u.tick(t + 1.0 / 60.0);
    let r = u.readout().clone();
    let m = u.fieldmap();
    let top = r.osc.iter().copied().fold([0.0, f32::MAX], |a, p| if p[1] < a[1] { p } else { a });
    assert!((top[0] - x).abs() <= m.cell, "peak at {} for a stir at {x}", top[0]);
    let i = (top[0] / m.cell).round() as usize;
    assert!(((base - top[1]) - m.byte_e(i, r.osc_row) * g.osc_amp).abs() <= 1.0);
    assert!(r.osc_row.abs_diff((y / m.cell).round() as usize) <= 1, "the strongest row {} for a stir at {y}", r.osc_row);
    // The level is the formula, and what the tree exports.
    assert!(r.level > 0.0);
    assert_eq!(r.level_h, gaze::level_height(sc.h, m.level(), g.level_full));
    assert_eq!(u.export_tree().field.map(|f| f.level), Some(r.level));
    // Settled: flat again.
    t = u.now();
    while u.fieldmap().nonzero() > 0 && t < 30.0 { t += 1.0 / 30.0; u.tick(t); }
    u.tick(t + 1.0 / 30.0);
    let r = u.readout().clone();
    assert!(r.osc.iter().all(|p| p[1] == base) && r.level_h == 0.0, "settled by {t}");
}
