//! Incremental update and long lists (DESIGN.md §1.13), without a GPU: the taffy tree is kept and only
//! re-run when a size changed; unchanged text is never shaped again; the wheel moves things without
//! layout or shaping; a virtual list of 10,000 rows builds only what it shows, with the whole list's
//! scroll extent, keyboard navigation and scrolling a row into view; lines a clip hides are not made
//! into instances.
use lab::el::{Content, El, Key, Paint, col, space, text};
use lab::ui::Ui;
use lab::gallery::{self, GalleryUi, Msg, Page};

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }
fn at(p: &str, h: u32) -> GalleryUi {
    let mut ui = gallery::new_ui(fonts(), 1280, h, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash(p).unwrap()));
    ui.relayout();
    ui.tick(1.0);
    ui
}
fn rows(ui: &GalleryUi) -> Vec<usize> { ui.nodes().iter().filter_map(|n| n.key.strip_prefix("long:").and_then(|i| i.parse().ok())).collect() }
fn texts(ui: &GalleryUi) -> std::collections::HashMap<String, String> {
    ui.nodes().iter().filter_map(|n| if let Content::Text { s, .. } = &n.content { Some((n.key.clone(), s.clone())) } else { None }).collect()
}

#[test]
fn ten_thousand_rows_build_only_what_shows() {
    let mut ui = at("list-10k", 900);
    let list = ui.node("long").expect("the list");
    let (h, row) = (list.rect.h, 40.0);
    let visible = (h / row).ceil() as usize + 1;
    let built = rows(&ui);
    assert!(!built.is_empty());
    assert!(built.len() <= visible + 2 * 6, "{} rows built for {visible} visible", built.len());
    // The scroll extent is the whole list's.
    assert!((list.extent - 10_000.0 * row).abs() <= 4.0, "extent {} (the rows and the border)", list.extent);
    // The wheel inside what is built: no layout, no shaping.
    let (x, y) = list.rect.center();
    let c0 = ui.counters();
    ui.wheel(x, y, 100.0);
    let c1 = ui.counters();
    assert_eq!((c1.layouts, c1.shaped, c1.views), (c0.layouts, c0.shaped, c0.views), "within the built rows");
    assert_eq!(ui.scroll_offset("long"), 100.0);
    // Far down: the rows there are built (a layout of the new rows); still only what shows.
    for _ in 0..50 { ui.wheel(x, y, 400.0); }
    let built = rows(&ui);
    let first = (ui.scroll_offset("long") / row) as usize;
    assert!(built.contains(&first) && built.contains(&(first + visible - 2)), "rows {first}.. are built: {:?}", (built.iter().min(), built.iter().max()));
    assert!(built.len() <= visible + 2 * 6);
    // A row's switch is pressable where it is drawn.
    let k = format!("long/sw{}", first + 1);
    let (sx, sy) = ui.node(&k).expect("switch").rect.center();
    ui.press(sx, sy); ui.release(sx, sy);
    assert_eq!(ui.state.long_on.get(first + 1).copied(), Some(true), "picked {:?}, switch at {:?}, list {:?}", ui.state.long_pick, (sx, sy), ui.node("long").map(|n| n.clip));
}

#[test]
fn keys_move_the_selection_and_scroll_it_into_view() {
    let mut ui = at("list-10k", 900);
    let r = ui.node("long:2").unwrap().rect;
    ui.press(r.x + 100.0, r.center().1); ui.release(r.x + 100.0, r.center().1);
    assert_eq!(ui.state.long_pick, Some(2));
    assert_eq!(ui.focus(), Some("long"));
    ui.key(Key::End, false);
    assert_eq!(ui.state.long_pick, Some(9_999));
    let list = ui.node("long").unwrap().clip.intersect(&ui.node("long").unwrap().rect);
    let last = ui.node("long:9999").expect("the last row is built").rect;
    assert!(last.y >= list.y - 0.5 && last.bottom() <= list.bottom() + 0.5, "{last:?} in {list:?}");
    ui.key(Key::PageUp, false);
    let p = ui.state.long_pick.unwrap();
    assert!(p < 9_999 && p > 9_960, "a page up: {p}");
    ui.key(Key::Home, false);
    assert_eq!((ui.state.long_pick, ui.scroll_offset("long")), (Some(0), 0.0));
}

#[test]
fn the_wheel_does_not_lay_out_or_shape() {
    // (Not a page with the annotation layer on: its labels show coordinates and sizes, which are new
    // strings to shape when things move.)
    for page in ["button", "input", "marks-chars", "type", "dialog", "morph"] {
        let mut ui = at(page, 480);
        let main = format!("main/{page}");
        // At the content's left edge (in its padding), where no inner scroll area is.
        let r = ui.node(&main).unwrap().rect;
        let (x, y) = (r.x + 12.0, r.y + 60.0);
        let c0 = ui.counters();
        for _ in 0..6 { ui.wheel(x, y, 90.0); }
        let c1 = ui.counters();
        assert!(ui.scroll_offset(&main) > 0.0, "{page} scrolled");
        assert_eq!((c1.layouts, c1.views), (c0.layouts, c0.views), "{page}: the wheel only moves things");
        // A text first drawn when it scrolls into view can be shaped once at its drawn width (a
        // documentation page's text whose box gives its width): never more than a couple.
        assert!(c1.shaped <= c0.shaped + 4, "{page}: shaped {} → {}", c0.shaped, c1.shaped);
        assert!(c1.scenes > c0.scenes);
    }
}

#[test]
fn a_press_shapes_only_the_text_that_changed() {
    let mut ui = at("button", 800);
    ui.text_mut().log_shapes = true;
    for t in [2.0, 3.0] {
        let before = texts(&ui);
        ui.text_mut().shape_log.clear();
        let (x, y) = ui.node("button/live").unwrap().rect.center();
        ui.tick(t);
        ui.press(x, y);
        ui.release(x, y);
        let after = texts(&ui);
        let changed: std::collections::HashSet<&String> = after.iter().filter(|(k, s)| before.get(*k) != Some(s)).map(|(_, s)| s).collect();
        assert_eq!(changed.len(), 1, "the count changed");
        let log = ui.text_mut().shape_log.clone();
        // Taffy asks for the changed text at a few widths (min-content, max-content, the one it gets).
        assert!(!log.is_empty() && log.iter().all(|s| changed.contains(s)), "shaped {log:?}; changed {changed:?}");
    }
}

struct Paints { on: bool }
fn paints(s: &Paints) -> El<()> {
    col().key("root").pad(20.0, 20.0).gap(8.0)
        .child(text("a label that stays", 14.0, Paint::TEXT).key("label"))
        .child(space(120.0, 30.0).key("swatch").fill(if s.on { Paint::ACCENT } else { Paint::DIM }).on_press(()))
}

#[test]
fn a_change_of_paint_only_skips_layout() {
    let mut ui = Ui::new(Paints { on: false }, paints, |s: &mut Paints, _| s.on = !s.on, fonts(), 400, 300, 1.0);
    let (x, y) = ui.node("swatch").unwrap().rect.center();
    let c0 = ui.counters();
    ui.press(x, y);
    ui.release(x, y);
    let c1 = ui.counters();
    assert!(ui.state.on);
    assert_eq!(c1.views, c0.views + 1, "a new view");
    assert_eq!((c1.layouts, c1.shaped), (c0.layouts, c0.shaped), "nothing changed size: no layout, no shaping");
    assert!(c1.layouts_skipped > c0.layouts_skipped);
}

#[test]
fn lines_a_clip_hides_are_not_instances() {
    // The source code of the button page is a scroll area 380 px tall with many more lines in it.
    let ui = at("input", 1800);
    let code = ui.node("input/code").unwrap();
    let clip = code.clip.intersect(&code.rect);
    let s = ui.scale();
    let glyphs_below = ui.scene().instances.iter().filter(|i| i.paint[3] == 0.0 && i.clip[3] <= (clip.bottom() + 1.0) * s && i.rect[1] > (clip.bottom() + 40.0) * s).count();
    assert_eq!(glyphs_below, 0, "no glyph instances far below the code's clip");
}
