//! card
//!
//! Purpose: Group related content under an optional title.
//! Props: `key`, `title: Option<&str>`, `body: El<M>`.
//! States: none of its own.
//! Messages: none; make it pressable with `.on_press(m)`.
//! Usage: `card("c", Some("Title"), text("Content", 14.0, Paint::TEXT))`
//!
//! A card: a bordered group with an optional title. Change its padding, corner radius or line with
//! `.pad`, `.radius` and `.border` on what this returns.
use super::{LABEL, LINE};
use crate::el::{El, Font, Paint, col, text};

pub fn card<M>(key: impl Into<String>, title: Option<&str>, body: El<M>) -> El<M> {
    let mut c = col().key(key).gap(10.0).pad(18.0, 16.0).radius(6.0).border(LINE, Paint::FAINT);
    if let Some(t) = title { c = c.child(text(t, LABEL + 2.0, Paint::INK).font(Font::Display)); }
    c.child(body)
}
