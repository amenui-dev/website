//! switch
//!
//! Purpose: Turn one setting on or off.
//! Props: `key`, `label`, `on`, `states` (names of off / on), `msg`; `switch_sized` adds the track
//!   height.
//! States: off, on, focused, disabled.
//! Messages: `msg` on press, Enter or Space.
//! Usage: `switch("notify", "Notifications", s.notify, None, Msg::Notify)`
//!
//! A two-state switch with a label and, optionally, the name of the current state. The whole row is
//! pressable.
use super::{CONTROL_H, LABEL, LINE};
use crate::el::{El, Paint, Role, row, space, text};
use taffy::prelude::JustifyContent;

pub fn switch<M: Clone>(key: impl Into<String>, label: &str, on: bool, states: Option<(&str, &str)>, msg: M) -> El<M> {
    switch_sized(key, label, on, states, 18.0, msg)
}

/// `track` is the track's height, logical px; it is a little under twice as wide.
pub fn switch_sized<M: Clone>(key: impl Into<String>, label: &str, on: bool, states: Option<(&str, &str)>, track: f32, msg: M) -> El<M> {
    let knob = (track - 6.0).max(4.0);
    // On: an ink track and a ground knob; off: a hairline track and a dim knob.
    let track_p = if on { Paint::INK } else { Paint::HAIRLINE };
    let t = row().width((track * 1.9).round()).height(track).pad(3.0, 0.0).radius(track * 0.5).border(LINE, track_p).center()
        .fill(track_p)
        .justify(if on { JustifyContent::END } else { JustifyContent::START })
        .child(space(knob, knob).radius(knob * 0.5).fill(if on { Paint::GROUND } else { Paint::DIM }).fill_held(Paint::ACCENT));
    let mut r = row().key(key).height(CONTROL_H.max(track + 8.0)).gap(10.0).align(taffy::AlignItems::CENTER).on_press(msg).role(Role::Switch).checked(on)
        .child(text(label, LABEL, Paint::DIM).grow(1.0));
    if let Some(s) = states { r = r.child(text(if on { s.1 } else { s.0 }, LABEL, Paint::TEXT)); }
    r.child(t)
}
