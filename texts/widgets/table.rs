//! table
//!
//! Purpose: Rows of records with named columns.
//! Props: `key`, `columns` (name, width), `rows` (cells as text), `selected`, `msg`.
//! States: a selected row (highlighted); one focus stop, Up / Down move the selection.
//! Messages: `msg(i)` on a row press or a key.
//! Usage: `table("files", &[("Name", 200.0), ("Size", 80.0)], &rows, s.sel, Msg::Sel)`
//!
//! A header row of column headers, then data rows of cells; the table, row, columnheader and cell
//! roles go to the accessibility mirror.
use super::{LABEL, LINE, SELECTED};
use crate::el::{El, Key, Paint, Role, col, row, space, text};
use std::rc::Rc;
use taffy::AlignItems;

pub fn table<M: Clone + 'static>(key: &str, columns: &[(&str, f32)], rows: &[Vec<String>], selected: Option<usize>, msg: impl Fn(usize) -> M + 'static) -> El<M> {
    let msg: Rc<dyn Fn(usize) -> M> = Rc::new(msg);
    let n = rows.len();
    let keys = { let msg = msg.clone(); move |k: Key, _: bool| -> Option<M> {
        if n == 0 { return None; }
        let to = match (k, selected) { (Key::Down, None) | (Key::Home, _) => 0, (Key::Up, None) | (Key::End, _) => n - 1, (Key::Down, Some(i)) => (i + 1).min(n - 1), (Key::Up, Some(i)) => i.saturating_sub(1), _ => return None };
        (Some(to) != selected).then(|| msg(to))
    } };
    let mut head = row().key(format!("{key}/head")).height(34.0).align(AlignItems::CENTER).pad(10.0, 0.0).role(Role::Row);
    for (c, w) in columns { head = head.child(text(*c, LABEL - 1.0, Paint::DIM).width(*w).role(Role::ColumnHeader)); }
    let mut t = col().key(key).focusable(true).on_key(keys).role(Role::Table).child(head).child(space(0.0, LINE).fill_width().fill(Paint::DIM));
    for (i, cells) in rows.iter().enumerate() {
        let mut r = row().key(format!("{key}:{i}")).height(36.0).align(AlignItems::CENTER).pad(10.0, 0.0).on_press(msg(i)).focusable(false).role(Role::Row).selected(selected == Some(i));
        if selected == Some(i) { r = r.fill(SELECTED); }
        for (j, cell) in cells.iter().enumerate() { r = r.child(text(cell.clone(), LABEL, Paint::TEXT).width(columns.get(j).map_or(80.0, |c| c.1)).role(Role::Cell)); }
        t = t.child(r).child(space(0.0, LINE).fill_width().fill(Paint::FAINT));
    }
    t
}
