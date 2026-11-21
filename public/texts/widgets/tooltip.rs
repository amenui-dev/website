//! tooltip
//!
//! Purpose: A short description of a control, shown while it is hovered or focused.
//! Props: `key`, `target` (the control), `tip`, `shown`.
//! States: hidden, shown (below the target, never taking input).
//! Messages: none (the application shows it from `Ui::on_hover` / focus; Esc hides it).
//! Usage: `tooltip("save-tip", button("save", "Save", Msg::Save), "Save to this computer", s.hovered == Some("save"))`
//!
//! The tip is drawn below its target in plain type, with the tooltip role, and is ignored by fronts
//! and the pointer (WAI-ARIA tooltip: it describes, it is not focusable).
use super::{LABEL, LINE, RADIUS};
use crate::el::{El, Paint, Response, Role, col, text};

pub fn tooltip<M>(key: &str, target: El<M>, tip: &str, shown: bool) -> El<M> {
    let mut c = col().key(key).gap(6.0).child(target);
    if shown {
        c = c.child(col().key(format!("{key}/tip")).pad(8.0, 5.0).radius(RADIUS).fill(Paint::GROUND).border(LINE, Paint::HAIRLINE).role(Role::Tooltip).label(tip)
            .respond(Response::Ignore).child(text(tip, LABEL - 2.0, Paint::TEXT)).absolute(Some(0.0), None, None, Some(-30.0)));
    }
    c
}
