//! Text input (DESIGN.md §1.8.1), without a GPU: every editing operation of `TextEdit`, input-method
//! composition, CJK and grapheme boundaries, undo; and the runtime: the pointer places the caret by the
//! laid-out clusters, drags select, double-click selects a word, Shift+arrows, Cmd+A / copy / cut /
//! paste / undo, composition shown inline, a long line scrolling inside its field, a multi-line field
//! moving by lines, and read-only selection across lines (source code, the article).
use amenui::el::{Key, Mods};
use website::gallery::{self, GalleryUi, Msg, Page};
use amenui::widgets::TextEdit;

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }
const CMD: Mods = Mods { shift: false, cmd: true, alt: false };

// ── The runtime ──────────────────────────────────────────────────────────────────────────────────

fn input_page() -> GalleryUi {
    let mut ui = gallery::new_ui(fonts(), 1280, 1800, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("input").unwrap()));
    ui.relayout();
    ui.tick(1.0);
    ui
}
fn field_text(ui: &GalleryUi) -> String { ui.state.text.text.clone() }

#[test]
fn pointer_places_the_caret_and_selects() {
    let mut ui = input_page();
    gallery::update(&mut ui.state, Msg::Text(TextEdit::new("hello world")));
    ui.relayout();
    let r = ui.node("input/live/text").unwrap().rect;
    // Press over the gap between "hello" and "world": the caret goes there, not to the end.
    let x = r.x + r.w * 5.5 / 11.0;
    ui.press(x, r.center().1);
    ui.release(x, r.center().1);
    assert_eq!(ui.focus(), Some("input/live"));
    let c = ui.state.text.caret;
    assert!((5..=6).contains(&c), "caret at {c}");
    // Drag from the start to the middle selects.
    ui.tick(2.0);
    ui.press(r.x + 1.0, r.center().1);
    ui.pointer_move(x, r.center().1);
    ui.release(x, r.center().1);
    assert!(ui.state.text.selected().starts_with("hello"), "dragged: {:?}", ui.state.text.selected());
    assert_eq!(ui.copy().as_deref(), Some(ui.state.text.selected()));
    // Double click selects the word.
    ui.tick(3.0);
    let wx = r.x + r.w * 8.0 / 11.0;
    ui.press(wx, r.center().1); ui.release(wx, r.center().1);
    ui.press(wx, r.center().1); ui.release(wx, r.center().1);
    assert_eq!(ui.state.text.selected(), "world");
    // The selection is drawn (halo glyphs under the selected ones) while the field has focus.
    let halos = ui.scene().instances.iter().filter(|i| i.paint[3] == 0.0 && (i.paint[1] - 0.34).abs() < 1e-4 && i.paint[2] == 1.0).count();
    assert_eq!(halos, 5, "one halo per selected glyph");
}

#[test]
fn keys_clipboard_and_composition_through_the_runtime() {
    let mut ui = input_page();
    let (x, y) = ui.node("input/live").unwrap().rect.center();
    ui.press(x, y); ui.release(x, y);
    for c in "abc def".chars() { ui.key(Key::Char(c), false); }
    assert_eq!(field_text(&ui), "abc def");
    ui.key(Key::Left, true); ui.key(Key::Left, true);
    assert_eq!(ui.copy().as_deref(), Some("ef"));
    assert_eq!(ui.cut().as_deref(), Some("ef"));
    assert_eq!(field_text(&ui), "abc d");
    ui.key_mods(Key::Home, Mods::default());
    assert!(ui.paste("ef"));
    assert_eq!(field_text(&ui), "efabc d");
    ui.key_mods(Key::Char('z'), CMD);
    assert_eq!(field_text(&ui), "abc d");
    ui.key_mods(Key::Char('a'), CMD);
    assert_eq!(ui.copy().as_deref(), Some("abc d"));
    // Typed text arrives as text (from the page's text element, or an input method's commit).
    ui.key_mods(Key::End, Mods::default());
    assert!(ui.text_input("!"));
    assert_eq!(field_text(&ui), "abc d!");
    // Composition: shown in the field, underlined, then committed; or cancelled.
    assert!(ui.ime_preedit("ni", None));
    assert_eq!(ui.node("input/live/text").unwrap().text(), Some("abc d!ni"));
    assert!(ui.scene().instances.iter().any(|i| i.paint[3] == 1.0 && i.rect[3] <= 2.0), "the underline");
    assert!(ui.ime_commit("你"));
    assert_eq!(field_text(&ui), "abc d!你");
    ui.ime_preedit("hao", None);
    ui.ime_commit("");
    assert_eq!(field_text(&ui), "abc d!你");
    // The input method's window goes where the caret is.
    let r = ui.ime_rect().expect("caret rect");
    let f = ui.node("input/live").unwrap().rect;
    assert!(r.x > f.x && r.x < f.right() && r.y >= f.y && r.bottom() <= f.bottom() + 1.0);
    // Without a text field focused there is nothing to type into.
    ui.key(Key::Tab, false);
    assert!(!ui.text_focused() || ui.focus() != Some("input/live"));
}

#[test]
fn a_long_line_scrolls_inside_its_field() {
    let mut ui = input_page();
    let f = ui.node("input/live").unwrap().rect;
    let (x, y) = f.center();
    ui.press(x, y); ui.release(x, y);
    ui.paste(&"长长的一行文字 ".repeat(12));
    let caret = ui.ime_rect().expect("caret");
    assert!(ui.scroll_offset("input/live") > 0.0, "scrolled");
    assert!(caret.x >= f.x && caret.right() <= f.right(), "caret {caret:?} inside {f:?}");
    ui.key(Key::Home, false);
    assert_eq!(ui.scroll_offset("input/live"), 0.0, "back at the start");
}

#[test]
fn a_textarea_moves_by_lines() {
    let mut ui = input_page();
    let f = ui.node("input/notes").unwrap().rect;
    ui.press(f.x + 12.0, f.y + 12.0); ui.release(f.x + 12.0, f.y + 12.0);
    assert_eq!(ui.focus(), Some("input/notes"));
    let start = ui.state.notes.caret;
    assert!(start <= 1, "at the start of the first line: {start}");
    ui.key(Key::Down, false);
    let down = ui.state.notes.caret;
    assert!(down >= 4, "on the second line: {down}");
    ui.key(Key::End, false);
    let end = ui.state.notes.caret;
    assert!(end > down, "line end {end} after {down}");
    ui.key(Key::Enter, false);
    assert!(ui.state.notes.text.contains("\n\n") || ui.state.notes.text.matches('\n').count() >= 2, "Enter broke the line");
    ui.key(Key::Up, false);
    assert!(ui.state.notes.caret < end + 1);
}

#[test]
fn read_only_selection_across_source_lines() {
    let mut ui = input_page();
    let a = ui.node("input/src0").unwrap().rect;
    let b = ui.node("input/src2").unwrap().rect;
    ui.press(a.x + 1.0, a.center().1);
    ui.pointer_move(b.x + 40.0, b.center().1);
    ui.release(b.x + 40.0, b.center().1);
    let sel = ui.selection();
    assert_eq!(sel.len(), 3, "three lines: {sel:?}");
    let copied = ui.copy().expect("copied");
    let src = gallery::docs::widget_source(gallery::COMPONENTS.iter().find(|c| c.hash == "input").unwrap().file);
    let lines: Vec<&str> = src.lines().collect();
    assert!(copied.starts_with(lines[0]), "{copied:?}");
    assert_eq!(copied.lines().count(), 3);
    assert!(copied.lines().nth(1).unwrap() == lines[1]);
    // Selected glyphs carry halos; pressing elsewhere ends the selection.
    ui.tick(3.0);
    let (x, y) = (5.0, 1790.0);
    ui.press(x, y); ui.release(x, y);
    assert!(ui.selection().is_empty());
    assert!(ui.copy().is_none());
}

#[test]
fn read_only_selection_in_the_article() {
    let mut ui = gallery::new_ui(fonts(), 1280, 900, 1.0);
    gallery::update(&mut ui.state, Msg::Go(Page::from_hash("scene-reading").unwrap()));
    ui.relayout();
    ui.tick(1.0);
    let r = ui.node("read/text").unwrap().rect;
    let (x, y) = (r.x + 4.0, r.y + 10.0);
    ui.press(x, y); ui.release(x, y);
    ui.press(x, y); ui.release(x, y);
    let word = ui.copy().expect("a word");
    assert!(!word.is_empty() && word.chars().count() <= 8, "{word:?}");
    ui.tick(2.0);
    ui.press(x, y);
    ui.pointer_move(r.x + r.w * 0.5, r.y + 80.0);
    ui.release(r.x + r.w * 0.5, r.y + 80.0);
    let copied = ui.copy().expect("lines");
    assert!(copied.chars().count() > 30, "{} chars", copied.chars().count());
}
