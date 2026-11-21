//! tabs
//!
//! Purpose: Switch between panels.
//! Props: `key`, `labels`, `current`, `gap`, `mark` (underline thickness), `msg`, `panel` (the
//!   current one).
//! States: current tab underlined; one focus stop, Left / Right move.
//! Messages: `msg(i)`; tabs are keyed `key:i`.
//! Usage: `tabs("t", &["Overview", "Settings"], s.tab, 24.0, 2.0, Msg::Tab, panel)`
//!
//! Tabs: a row of labels over the panel of the current one. The row is one focus stop; Left / Right
//! move between tabs. Tabs are keyed `key:i`.
use super::{LABEL, LINE};
use crate::el::{El, Key, Paint, Role, col, row, space, text};
use std::rc::Rc;

/// `gap` is the space between labels and `mark` the thickness of the line under the current one.
pub fn tabs<M: Clone + 'static>(key: &str, labels: &[&str], current: usize, gap: f32, mark: f32, msg: impl Fn(usize) -> M + 'static, panel: El<M>) -> El<M> {
    let msg: Rc<dyn Fn(usize) -> M> = Rc::new(msg);
    let n = labels.len();
    let keys = {
        let msg = msg.clone();
        move |k: Key, _: bool| -> Option<M> {
            let to = match k {
                Key::Left if current > 0 => current - 1,
                Key::Right if current + 1 < n => current + 1,
                Key::Home => 0,
                Key::End => n.saturating_sub(1),
                _ => return None,
            };
            (to != current).then(|| msg(to))
        }
    };
    let mut head = row().key(format!("{key}/head")).gap(gap).on_key(keys).role(Role::TabList);
    for (i, l) in labels.iter().enumerate() {
        let on = i == current;
        let line = space(0.0, mark).fill_width();
        head = head.child(col().key(format!("{key}:{i}")).gap(6.0).pad(2.0, 6.0).on_press(msg(i)).focusable(false).role(Role::Tab).selected(on)
            .child(text(*l, LABEL + 1.0, if on { Paint::INK } else { Paint::DIM }))
            .child(if on { line.fill(Paint::INK) } else { line }));
    }
    // The list and its rule are the control: held at the legibility cap with the tabs' labels, so an
    // effect on the page leaves them as they rest (the panel is the page's own).
    col().key(key).gap(0.0)
        .child(head.legible(true))
        .child(space(0.0, LINE).fill_width().fill(Paint::FAINT).legible(true))
        .child(col().key(format!("{key}/panel")).pad(0.0, 14.0).child(panel))
}
