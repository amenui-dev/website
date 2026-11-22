//! input
//!
//! Purpose: Edit one line (`input`) or several (`textarea`) of text.
//! Props: `key`, `edit: &TextEdit` (text, caret, selection, composition, undo), `placeholder`,
//!   `width`, (`rows`,) `max` characters, `msg`.
//! States: empty with placeholder, focused with caret and selection, composing (underlined),
//!   disabled.
//! Messages: `msg(TextEdit)` with the edited value for every text event; typing never recomposes.
//! Usage: `input("name", &s.name, "Name", 280.0, 40, Msg::Name)`
//!
//! Text fields: a single-line `input` and a multi-line `textarea`. The application keeps the text, the
//! caret, the selection, text being composed and the undo history in a `TextEdit` in its state; the field
//! sends the edited value for every text event it uses (see `el::TextInput`):
//! - typing inserts at the caret over the selection; Backspace / Delete remove the selection, or one
//!   character (a grapheme cluster), or with Cmd / Alt a word;
//! - Left / Right move by a character, with Cmd / Alt by a word, Home / End to the ends; Shift extends
//!   the selection; Cmd+A selects everything; Cmd+Z undoes (Shift, or Cmd+Y, redoes), at least 50 steps;
//! - an input method's composition is shown at the caret, underlined, and replaced by what it commits;
//! - the pointer places the caret (by the laid-out clusters), drags to select, double-click selects a
//!   word, triple-click everything;
//! - paste inserts, cut removes the selection (the runtime puts it on the clipboard first).
//!
//! A single-line field scrolls sideways when its text is longer than it; the runtime keeps the caret in
//! view. In a `textarea` Enter breaks the line and the runtime moves the caret by lines.
use super::{CONTROL_H, LABEL, LINE, RADIUS};
use crate::el::{Cursor, El, Key, Mods, Paint, Phase, Role, TextInput, col, row, text, text_lh};
use crate::text::{next_grapheme, next_word, prev_grapheme, prev_word, word_range};
use taffy::AlignItems;

/// A byte range.
pub type Range = (usize, usize);

/// The most undo steps kept.
pub const UNDO: usize = 100;

/// Text, a caret and a selection (character indices: 0 is before the first character; `anchor` is
/// where the selection started, equal to `caret` when nothing is selected), text being composed by an
/// input method, and the undo and redo history.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextEdit {
    pub text: String,
    pub caret: usize,
    pub anchor: usize,
    pub preedit: Option<String>,
    undo: Vec<Snap>,
    redo: Vec<Snap>,
    group: Group,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Snap { text: String, caret: usize, anchor: usize }
/// What the last change was: typing and deleting one character at a time are one undo step each run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Group { #[default] None, Typing, Deleting }

impl TextEdit {
    pub fn new(s: &str) -> Self { let n = s.chars().count(); Self { text: s.to_owned(), caret: n, anchor: n, ..Self::default() } }
    fn byte(&self, at: usize) -> usize { self.text.char_indices().nth(at).map_or(self.text.len(), |(b, _)| b) }
    fn char_at(&self, b: usize) -> usize { self.text[..b.min(self.text.len())].chars().count() }
    /// The caret as a byte index into `text`.
    pub fn caret_byte(&self) -> usize { self.byte(self.caret) }
    /// The selection as character indices, if anything is selected.
    pub fn selection(&self) -> Option<(usize, usize)> { (self.caret != self.anchor).then(|| (self.caret.min(self.anchor), self.caret.max(self.anchor))) }
    pub fn selected(&self) -> &str { self.selection().map_or("", |(a, b)| &self.text[self.byte(a)..self.byte(b)]) }
    pub fn can_undo(&self) -> bool { !self.undo.is_empty() }
    /// The same text, caret, selection, composition and history (what the last change was aside).
    fn same(&self, o: &Self) -> bool {
        (&self.text, self.caret, self.anchor, &self.preedit, self.undo.len(), self.redo.len()) == (&o.text, o.caret, o.anchor, &o.preedit, o.undo.len(), o.redo.len())
    }

    /// What the field shows: the text with any composition inserted at the caret, the caret, the
    /// selection and the composed part, as byte ranges of that string.
    pub fn display(&self) -> (String, usize, Option<Range>, Option<Range>) {
        let c = self.caret_byte();
        match &self.preedit {
            Some(p) => (format!("{}{}{}", &self.text[..c], p, &self.text[c..]), c + p.len(), None, Some((c, c + p.len()))),
            None => (self.text.clone(), c, self.selection().map(|(a, b)| (self.byte(a), self.byte(b))), None),
        }
    }

    fn snapshot(&mut self, g: Group) {
        // A run of typing (or of deleting) is one step; anything else starts a new one.
        if g == Group::None || g != self.group {
            self.undo.push(Snap { text: self.text.clone(), caret: self.caret, anchor: self.anchor });
            if self.undo.len() > UNDO { self.undo.remove(0); }
        }
        self.redo.clear();
        self.group = g;
    }
    fn set(&mut self, caret: usize, extend: bool) {
        self.caret = caret;
        if !extend { self.anchor = caret; }
        self.group = Group::None;
    }
    /// Remove the selection (no snapshot); returns whether there was one.
    fn delete_selection(&mut self) -> bool {
        let Some((a, b)) = self.selection() else { return false };
        let (ba, bb) = (self.byte(a), self.byte(b));
        self.text.replace_range(ba..bb, "");
        self.caret = a;
        self.anchor = a;
        true
    }
    /// Insert `s` over the selection, keeping to `max` characters.
    fn insert(&mut self, s: &str, max: usize, multiline: bool, g: Group) -> bool {
        let s = s.replace("\r\n", "\n").replace('\r', "\n");
        let clean: String = s.chars().map(|c| if c == '\n' && !multiline { ' ' } else { c }).filter(|&c| !c.is_control() || c == '\n').collect();
        let room = max.saturating_sub(self.text.chars().count() - self.selection().map_or(0, |(a, b)| b - a));
        let clean: String = clean.chars().take(room).collect();
        if clean.is_empty() && self.selection().is_none() { return false; }
        self.snapshot(g);
        self.delete_selection();
        let b = self.caret_byte();
        self.text.insert_str(b, &clean);
        let n = self.caret + clean.chars().count();
        self.caret = n;
        self.anchor = n;
        true
    }
    fn restore(&mut self, from_undo: bool) -> bool {
        let (take, put) = if from_undo { (&mut self.undo, &mut self.redo) } else { (&mut self.redo, &mut self.undo) };
        let Some(s) = take.pop() else { return false };
        put.push(Snap { text: std::mem::take(&mut self.text), caret: self.caret, anchor: self.anchor });
        self.text = s.text;
        self.caret = s.caret.min(self.text.chars().count());
        self.anchor = s.anchor.min(self.text.chars().count());
        self.group = Group::None;
        true
    }

    /// What a text event does, if anything. `max` is the most characters the text may hold; `multiline`
    /// lets Enter break the line.
    pub fn handle(&self, ev: &TextInput, max: usize, multiline: bool) -> Option<TextEdit> {
        let mut e = self.clone();
        let changed = match ev {
            TextInput::Insert(s) => { e.preedit = None; e.insert(s, max, multiline, if s.chars().count() == 1 { Group::Typing } else { Group::None }) || self.preedit.is_some() }
            TextInput::Paste(s) => e.insert(s, max, multiline, Group::None),
            TextInput::Cut => { self.selection()?; e.snapshot(Group::None); e.delete_selection() }
            TextInput::Compose { text, .. } => {
                if text.is_empty() { e.preedit = None; }
                else {
                    // A composition replaces the selection.
                    if self.preedit.is_none() && self.selection().is_some() { e.snapshot(Group::None); e.delete_selection(); }
                    e.preedit = Some(text.clone());
                }
                true
            }
            // While composing, the input method owns the keys and the pointer.
            _ if self.preedit.is_some() => return None,
            TextInput::Pointer { at, phase, clicks, shift } => {
                let c = e.char_at(*at);
                match (phase, clicks) {
                    (Phase::Start, 1) => e.set(c, *shift),
                    (Phase::Start, 2) => { let (a, b) = word_range(&e.text, e.byte(c)); let (a, b) = (e.char_at(a), e.char_at(b)); e.anchor = a; e.caret = b; }
                    (Phase::Start, _) => { e.anchor = 0; e.caret = e.text.chars().count(); }
                    (Phase::Move, 1) => e.caret = c,
                    (Phase::Move, 2) => {
                        // Dragging after a double click extends by words.
                        let (a, b) = word_range(&e.text, e.byte(c));
                        let (a, b) = (e.char_at(a), e.char_at(b));
                        let (wa, wb) = word_range(&self.text, self.byte(self.anchor.min(self.caret)));
                        let (wa, wb) = (self.char_at(wa), self.char_at(wb));
                        if a < wa { e.anchor = wb.max(self.anchor.max(self.caret)); e.caret = a; } else { e.anchor = wa.min(self.anchor.min(self.caret)); e.caret = b.max(wb); }
                    }
                    _ => {}
                }
                true
            }
            TextInput::Goto { at, extend } => { let c = e.char_at(*at); e.set(c, *extend); true }
            TextInput::Key { key, mods } => e.key(*key, *mods, multiline),
        };
        (changed && !e.same(self)).then_some(e)
    }
    fn key(&mut self, k: Key, m: Mods, multiline: bool) -> bool {
        let n = self.text.chars().count();
        let word = m.cmd || m.alt;
        let cb = self.caret_byte();
        match k {
            Key::Char(c) if m.cmd => match c.to_ascii_lowercase() {
                'a' => { self.anchor = 0; self.caret = n; true }
                'z' if m.shift => self.restore(false),
                'z' => self.restore(true),
                'y' => self.restore(false),
                _ => false,
            },
            Key::Left => {
                if !m.shift && let Some((a, _)) = self.selection() { self.set(a, false); return true; }
                let b = if word { prev_word(&self.text, cb) } else { prev_grapheme(&self.text, cb) };
                let c = self.char_at(b);
                self.set(c, m.shift);
                true
            }
            Key::Right => {
                if !m.shift && let Some((_, b)) = self.selection() { self.set(b, false); return true; }
                let b = if word { next_word(&self.text, cb) } else { next_grapheme(&self.text, cb) };
                let c = self.char_at(b);
                self.set(c, m.shift);
                true
            }
            Key::Home => { self.set(0, m.shift); true }
            Key::End => { self.set(n, m.shift); true }
            Key::Backspace | Key::Delete => {
                if self.selection().is_some() { self.snapshot(Group::None); return self.delete_selection(); }
                let (a, b) = if k == Key::Backspace {
                    (if word { prev_word(&self.text, cb) } else { prev_grapheme(&self.text, cb) }, cb)
                } else {
                    (cb, if word { next_word(&self.text, cb) } else { next_grapheme(&self.text, cb) })
                };
                if a == b { return false; }
                self.snapshot(Group::Deleting);
                self.text.replace_range(a..b, "");
                let c = self.char_at(a);
                self.caret = c;
                self.anchor = c;
                true
            }
            Key::Enter if multiline => self.insert("\n", usize::MAX, true, Group::None),
            _ => false,
        }
    }

    /// What a key does to the text, if anything (the single-line rules; `max` is the most characters it
    /// may hold). Kept for code that drives a `TextEdit` with keys only.
    pub fn apply(&self, k: Key, max: usize) -> Option<TextEdit> {
        let ev = match k { Key::Char(c) => TextInput::Insert(c.to_string()), key => TextInput::Key { key, mods: Mods::default() } };
        self.handle(&ev, max, false)
    }
}

/// The text leaf of a field: the text with its caret, selection and composition, or the placeholder.
fn shown<M>(edit: &TextEdit, placeholder: &str, size: f32, line: Option<f32>) -> El<M> {
    let (s, caret, select, mark) = edit.display();
    let t = |s: String, p: Paint| match line { Some(l) => text_lh(s, size, l, p), None => text(s, size, p) };
    if s.is_empty() { return t(placeholder.to_owned(), Paint::DIM).caret(0); }
    let mut leaf = t(s, Paint::INK).caret(caret);
    if let Some((a, b)) = select { leaf = leaf.select(a, b); }
    if let Some((a, b)) = mark { leaf = leaf.mark(a, b); }
    leaf
}

/// `width` is the field's width and `max` the most characters it takes; `placeholder` is shown, dim,
/// while it is empty.
pub fn input<M: Clone + 'static>(key: &str, edit: &TextEdit, placeholder: &str, width: f32, max: usize, msg: impl Fn(TextEdit) -> M + 'static) -> El<M> {
    // The body is built once per message type, not once for every closure an application passes.
    input_in(key, edit, placeholder, width, max, std::rc::Rc::new(msg))
}
fn input_in<M: Clone + 'static>(key: &str, edit: &TextEdit, placeholder: &str, width: f32, max: usize, msg: std::rc::Rc<dyn Fn(TextEdit) -> M>) -> El<M> {
    let now = edit.clone();
    row().key(key).width(width).height(CONTROL_H).pad(10.0, 0.0).radius(RADIUS).align(AlignItems::CENTER)
        .fill(Paint::GROUND).border(LINE, Paint::DIM).border_hover(Paint::INK).scroll_x().cursor(Cursor::Text).role(Role::TextBox).label(placeholder).text_value(edit.text.clone())
        .on_text(move |ev| now.handle(&ev, max, false).map(&*msg))
        .child(shown(edit, placeholder, LABEL, None).nowrap().key(format!("{key}/text")))
}

/// A field of several lines, `rows` lines tall; it scrolls when the text is longer.
pub fn textarea<M: Clone + 'static>(key: &str, edit: &TextEdit, placeholder: &str, width: f32, rows: usize, max: usize, msg: impl Fn(TextEdit) -> M + 'static) -> El<M> {
    let now = edit.clone();
    let line = 22.0;
    // Outlined, not filled: a filled box this tall is one of the "big" shapes chrome and liquid shade,
    // and the text in it would sit on metal.
    col().key(key).width(width).height(rows as f32 * line + 16.0).pad(10.0, 8.0).radius(RADIUS)
        .border(LINE, Paint::DIM).border_hover(Paint::INK).scroll().bar().multiline().cursor(Cursor::Text).role(Role::TextBox).label(placeholder).text_value(edit.text.clone())
        .on_text(move |ev| now.handle(&ev, max, true).map(&msg))
        .child(shown(edit, placeholder, LABEL, Some(line)).fill_width().key(format!("{key}/text")))
}
