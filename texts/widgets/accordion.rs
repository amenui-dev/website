//! accordion
//!
//! Purpose: Sections that open and close, one heading each (show details on demand).
//! Props: `key`, `items` (title, body), `open` (which are open), `msg`.
//! States: each section open or closed; headings are buttons with `aria-expanded`.
//! Messages: `msg(i)` toggles section `i` (press, Enter, Space); Up / Down move between headings by Tab.
//! Usage: `accordion("faq", vec![("What is a material", body)], &s.open, Msg::Faq)`
//!
//! Each heading is a button that toggles its region; an open region shows its body below it.
use super::{LABEL, LINE};
use crate::el::{El, Paint, Role, col, row, space, text};
use taffy::AlignItems;

pub fn accordion<M: Clone>(key: &str, items: Vec<(&str, El<M>)>, open: &[bool], msg: impl Fn(usize) -> M) -> El<M> {
    let mut c = col().key(key).fill_width();
    for (i, (title, body)) in items.into_iter().enumerate() {
        let on = open.get(i).copied().unwrap_or(false);
        c = c.child(row().key(format!("{key}:{i}")).fill_width().height(40.0).align(AlignItems::CENTER).gap(10.0).on_press(msg(i)).focusable(true).role(Role::Button).expanded(on).label(title)
                .child(text(title, LABEL + 1.0, Paint::INK).grow(1.0)).child(text(if on { "−" } else { "+" }, LABEL + 2.0, Paint::DIM)));
        if on { c = c.child(col().key(format!("{key}:{i}/panel")).pad(0.0, 10.0).role(Role::Region).label(title).child(body)); }
        c = c.child(space(0.0, LINE).fill_width().fill(Paint::FAINT));
    }
    c
}
