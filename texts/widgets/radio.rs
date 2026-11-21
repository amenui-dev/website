//! radio
//!
//! Purpose: Choose exactly one of a few options, all visible.
//! Props: `key`, `options`, `current`, `msg`.
//! States: one option selected; one focus stop for the group.
//! Messages: `msg(i)`; options are keyed `key:i`. Arrow keys move the selection (WAI-ARIA radio
//!   group: Up/Left the previous, Down/Right the next, wrapping; Home / End the ends).
//! Usage: `radio_group("size", &["Small", "Medium", "Large"], s.size, Msg::Size)`
//!
//! A column of round marks and labels. The group is one focus stop; the arrows select as they move.
use super::{CONTROL_H, LABEL, LINE};
use crate::el::{El, Key, Paint, Role, col, row, space, stack, text};
use std::rc::Rc;
use taffy::AlignItems;

pub fn radio_group<M: Clone + 'static>(key: &str, options: &[&str], current: usize, msg: impl Fn(usize) -> M + 'static) -> El<M> {
    let msg: Rc<dyn Fn(usize) -> M> = Rc::new(msg);
    let n = options.len();
    let keys = { let msg = msg.clone(); move |k: Key, _: bool| -> Option<M> {
        if n == 0 { return None; }
        let to = match k { Key::Up | Key::Left => (current + n - 1) % n, Key::Down | Key::Right => (current + 1) % n, Key::Home => 0, Key::End => n - 1, _ => return None };
        (to != current).then(|| msg(to))
    } };
    let mut g = col().key(key).gap(2.0).on_key(keys).focusable(true).role(Role::RadioGroup);
    for (i, o) in options.iter().enumerate() {
        let on = i == current;
        let mark = stack([space(16.0, 16.0).radius(8.0).fill(Paint::GROUND).border(LINE, if on { Paint::INK } else { Paint::DIM }).border_hover(Paint::INK), space(8.0, 8.0).radius(4.0).fill(if on { Paint::INK } else { Paint::GROUND })]);
        g = g.child(row().key(format!("{key}:{i}")).height(CONTROL_H).gap(10.0).align(AlignItems::CENTER).on_press(msg(i)).focusable(false).role(Role::Radio).checked(on).label(*o)
            .child(mark).child(text(*o, LABEL, Paint::TEXT)));
    }
    g
}
