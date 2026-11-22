//! list
//!
//! Purpose: Pick one row of a list (`list`), or of a very long one (`list_virtual`).
//! Props: `key`, `rows` (label, note) or `count` + `row_of(i)`, `selected`, `row_h`, `height`,
//!   `msg`.
//! States: selected row highlighted; one focus stop; Up / Down / Home / End / Page keys move the
//!   selection.
//! Messages: `msg(i)` with the row index; rows are keyed `key:i`.
//! Usage: `list("files", &ROWS, s.picked, 36.0, 220.0, Msg::Pick)`
//!
//! A list of rows, one of which can be selected; it scrolls when it holds more than it shows. The list
//! is one focus stop: Up / Down / Home / End move the selection, and the selected row is scrolled into
//! view. Rows are pressed with the pointer.
use super::{LABEL, LINE, SELECTED};
use crate::el::{El, Font, Key, Paint, Role, col, row, text};
use std::rc::Rc;
use taffy::AlignItems;

/// Each row is a label and a dim note. Rows are keyed `key:i`.
pub fn list<M: Clone + 'static>(key: &str, rows: &[(&str, &str)], selected: Option<usize>, row_h: f32, height: f32, msg: impl Fn(usize) -> M + 'static) -> El<M> {
    let msg: Rc<dyn Fn(usize) -> M> = Rc::new(msg);
    let n = rows.len();
    let keys = {
        let msg = msg.clone();
        move |k: Key, _: bool| -> Option<M> {
            if n == 0 { return None; }
            let to = match (k, selected) {
                (Key::Down, None) | (Key::Home, _) => 0,
                (Key::Up, None) | (Key::End, _) => n - 1,
                (Key::Down, Some(i)) if i + 1 < n => i + 1,
                (Key::Up, Some(i)) if i > 0 => i - 1,
                _ => return None,
            };
            (Some(to) != selected).then(|| msg(to))
        }
    };
    let mut l = col().key(key).scroll().bar().height(height).radius(6.0).border(LINE, Paint::FAINT).pad(0.0, 4.0).on_key(keys).role(Role::List);
    for (i, (label, note)) in rows.iter().enumerate() {
        let on = selected == Some(i);
        let mut r = row().key(format!("{key}:{i}")).height(row_h).pad(14.0, 0.0).gap(12.0).align(AlignItems::CENTER)
            .on_press(msg(i)).focusable(false).reveal(on).role(Role::ListItem).selected(on)
            .child(text(*label, LABEL + 1.0, if on { Paint::INK } else { Paint::TEXT }).grow(1.0))
            .child(text(*note, LABEL - 1.0, if on { Paint::TEXT } else { Paint::DIM }).font(Font::Mono));
        if on { r = r.fill(SELECTED); }
        l = l.child(r);
    }
    l
}

/// A list of `count` rows, `row_h` tall, that builds only the rows it shows and a few around them
/// (`El::virtual_rows`), so it can hold any number of rows. Like `list` it is one focus stop: Up / Down,
/// Page Up / Page Down, Home / End move the selection, and the runtime scrolls the selected row into view
/// (built or not). `row(i)` is what row `i` shows; the row itself (keyed `key:i`) is pressed to select it.
pub fn list_virtual<M: Clone + 'static>(key: &str, count: usize, row_h: f32, height: f32, selected: Option<usize>, select: impl Fn(usize) -> M + 'static, row_of: impl Fn(usize) -> El<M> + 'static) -> El<M> {
    let select: Rc<dyn Fn(usize) -> M> = Rc::new(select);
    let page = ((height / row_h).floor() as usize).saturating_sub(1).max(1);
    let keys = {
        let select = select.clone();
        move |k: Key, _: bool| -> Option<M> {
            if count == 0 { return None; }
            let to = match (k, selected) {
                (Key::Down, None) | (Key::Home, _) => 0,
                (Key::Up, None) | (Key::End, _) => count - 1,
                (Key::Down, Some(i)) => (i + 1).min(count - 1),
                (Key::Up, Some(i)) => i.saturating_sub(1),
                (Key::PageDown, s) => s.map_or(0, |i| (i + page).min(count - 1)),
                (Key::PageUp, s) => s.map_or(0, |i| i.saturating_sub(page)),
                _ => return None,
            };
            (Some(to) != selected).then(|| select(to))
        }
    };
    let k = key.to_owned();
    col().key(key).scroll().bar().height(height).radius(6.0).border(LINE, Paint::FAINT).on_key(keys).role(Role::List)
        .label(crate::i18n::tr_args("widget.list.rows", &[("n", crate::i18n::Arg::from(count))]))
        .virtual_rows(count, row_h, 6, selected, move |i| {
            let on = selected == Some(i);
            let r = row().key(format!("{k}:{i}")).pad(14.0, 0.0).gap(12.0).align(AlignItems::CENTER).on_press(select(i)).focusable(false).role(Role::ListItem).selected(on).child(row_of(i));
            if on { r.fill(SELECTED) } else { r }
        })
}
