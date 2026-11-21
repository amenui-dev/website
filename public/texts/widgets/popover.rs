//! popover
//!
//! Purpose: Rich content in a layer anchored to a button (a small form, details), opened on demand.
//! Props: `key`, `trigger` (its label), `open`, `toggle`, `close`, `body`.
//! States: closed, open (a panel below the trigger, focus moves into it).
//! Messages: `toggle` on the trigger (Enter / Space too); `close` on Esc or a press outside.
//! Usage: `popover("share", "Share", s.open, Msg::Toggle, Msg::Close, body)`
//!
//! Like a menu but with any content: the panel is a layer just below the trigger.
use super::{LINE, RADIUS, Variant, button_as};
use crate::el::{El, Role, col};

pub fn popover<M: Clone>(key: &str, trigger: &str, open: bool, toggle: M, close: M, body: El<M>) -> El<M> {
    let mut r = col().key(format!("{key}/popover")).child(button_as(key, trigger, Variant::Default, toggle).expanded(open));
    if open {
        let panel = col().key(format!("{key}/panel")).anchor(key).min_width(220.0).pad(14.0, 12.0).gap(10.0).radius(RADIUS + 2.0)
            .fill(crate::el::Paint::GROUND).border(LINE, crate::el::Paint::HAIRLINE).role(Role::Dialog).label(trigger).child(body);
        r = r.child(col().key(format!("{key}/layer")).modal(close.clone(), 0.0).on_press(close).focusable(false).child(panel));
    }
    r
}
