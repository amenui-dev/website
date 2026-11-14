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
