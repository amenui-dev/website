//! toggle
//!
//! Purpose: A button that stays pressed (bold, italic), or a group of them where one or several are on.
//! Props: `toggle`: `key`, `label`, `on`, `msg`; `toggle_group`: `key`, `labels`, `on` (one per
//!   label), `msg(i)`.
//! States: off, on (`aria-pressed`), focused; in a group one focus stop, Left / Right move focus by
//!   choosing, Space toggles.
//! Messages: `msg` / `msg(i)` on press, Enter or Space.
//! Usage: `toggle("bold", "B", s.bold, Msg::Bold)`, `toggle_group("align", &["Left", "Center", "Right"], &s.align, Msg::Align)`
//!
//! On, the button is an ink plate with its label in the ground's colour.
use super::{CONTROL_H, LABEL, LINE, RADIUS};
use crate::el::{El, Paint, Role, row, text};
use taffy::AlignItems;

pub fn toggle<M: Clone>(key: impl Into<String>, label: &str, on: bool, msg: M) -> El<M> {
    row().key(key).height(CONTROL_H).min_width(CONTROL_H).pad(10.0, 0.0).radius(RADIUS).center()
        .fill(if on { Paint::INK } else { Paint::GROUND }).border(LINE, if on { Paint::INK } else { Paint::DIM }).border_hover(Paint::INK)
        .on_press(msg).role(Role::Button).checked(on).label(label)
        .child(text(label, LABEL, if on { Paint::GROUND } else { Paint::TEXT }))
}
pub fn toggle_group<M: Clone>(key: &str, labels: &[&str], on: &[bool], msg: impl Fn(usize) -> M) -> El<M> {
    let mut r = row().key(key).gap(4.0).align(AlignItems::CENTER).role(Role::Toolbar).label(key);
    for (i, l) in labels.iter().enumerate() { r = r.child(toggle(format!("{key}:{i}"), l, on.get(i).copied().unwrap_or(false), msg(i))); }
    r.border(LINE, Paint::GROUND)
}
