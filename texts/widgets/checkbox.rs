//! checkbox
//!
//! Purpose: Choose whether one option applies (a tick in a box), independently of others.
//! Props: `key`, `label`, `checked`, `msg`.
//! States: unchecked, checked, focused, disabled.
//! Messages: `msg` on press, Space or Enter (WAI-ARIA checkbox: Space toggles).
//! Usage: `checkbox("terms", "Accept the terms", s.terms, Msg::Terms)`
//!
//! A square box and its label; the whole row is pressable and is one focus stop. Checked, the box is
//! outlined in ink and carries an ink tick made of two bars.
use super::{CONTROL_H, LABEL, LINE, RADIUS};
use crate::el::{El, Paint, Role, row, space, stack, text};
use taffy::AlignItems;

pub fn checkbox<M: Clone>(key: impl Into<String>, label: &str, checked: bool, msg: M) -> El<M> {
    let s = 16.0;
    let mut bx = stack([space(s, s).radius(RADIUS - 1.0).fill(Paint::GROUND).border(LINE, if checked { Paint::INK } else { Paint::DIM }).border_hover(Paint::INK)]);
    // The tick: a short bar and a long one meeting low on the left (no rotation needed: an L in the box).
    if checked { bx = stack([bx, row().width(8.0).height(5.0).align(AlignItems::END).child(space(2.0, 5.0).fill(Paint::INK)).child(space(6.0, 2.0).fill(Paint::INK))]); }
    row().key(key).height(CONTROL_H).gap(10.0).align(AlignItems::CENTER).on_press(msg).role(Role::Checkbox).checked(checked).label(label)
        .child(bx).child(text(label, LABEL, Paint::TEXT))
}
