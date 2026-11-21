//! choice
//!
//! Purpose: Choose one of several named options.
//! Props: `choice(key, label, options, current, msg)`; `segmented(key, options, current, height,
//!   msg)`.
//! States: the current option is an ink plate with its label in the ground's colour; segmented is one
//!   focus stop, Left / Right move.
//! Messages: `msg(i)` with the option's index.
//! Usage: `segmented("view", &["Day", "Week"], s.view, 30.0, Msg::View)`
//!
//! Choosing one of several named options.
//!
//! - `choice`: a label and a row of small buttons that wraps; each option is its own focus stop.
//! - `segmented`: the options joined into one control with a single focus stop; Left and Right move
//!   the choice.
use super::{CONTROL_H, LABEL, LINE, RADIUS};
use crate::el::{El, Key, Paint, row, text};
use std::rc::Rc;
use taffy::AlignItems;

/// `msg` gets the index pressed. Options are keyed `key:i` (a `/` would collide with the path keys
/// of unkeyed children).
pub fn choice<M: Clone>(key: &str, label: &str, options: &[&str], current: usize, msg: impl Fn(usize) -> M) -> El<M> {
    // The body is built once per message type, not once for every closure an application passes.
    choice_in(key, label, options, current, &msg)
}
fn choice_in<M: Clone>(key: &str, label: &str, options: &[&str], current: usize, msg: &dyn Fn(usize) -> M) -> El<M> {
    let mut opts = row().key(format!("{key}/options")).wrap().gap(6.0).grow(1.0);
    for (i, o) in options.iter().enumerate() {
        let on = i == current;
        opts = opts.child(row().key(format!("{key}:{i}")).center().height(CONTROL_H - 4.0).pad(10.0, 0.0).radius(RADIUS)
            .fill(if on { Paint::INK } else { Paint::GROUND })
            .border(LINE, if on { Paint::INK } else { Paint::DIM }).border_hover(Paint::INK)
            .on_press(msg(i))
            .child(text(*o, LABEL - 1.0, if on { Paint::GROUND } else { Paint::TEXT })));
    }
    row().key(key).gap(10.0).align(AlignItems::START)
        .child(row().width(76.0).height(CONTROL_H - 4.0).align(AlignItems::CENTER).child(text(label, LABEL, Paint::DIM)))
        .child(opts)
}

/// Segments are keyed `key:i`; `height` is the control's height, logical px.
pub fn segmented<M: Clone + 'static>(key: &str, options: &[&str], current: usize, height: f32, msg: impl Fn(usize) -> M + 'static) -> El<M> {
    // The body is built once per message type, not once for every closure an application passes.
    segmented_in(key, options, current, height, Rc::new(msg))
}
fn segmented_in<M: Clone + 'static>(key: &str, options: &[&str], current: usize, height: f32, msg: Rc<dyn Fn(usize) -> M>) -> El<M> {
    let n = options.len();
    let keys = {
        let msg = msg.clone();
        move |k: Key, _: bool| -> Option<M> {
            let to = match k {
                Key::Left | Key::Up if current > 0 => current - 1,
                Key::Right | Key::Down if current + 1 < n => current + 1,
                Key::Home => 0,
                Key::End => n.saturating_sub(1),
                _ => return None,
            };
            (to != current).then(|| msg(to))
        }
    };
    let mut r = row().key(key).height(height).pad(2.0, 2.0).gap(2.0).radius(RADIUS + 1.0).border(LINE, Paint::FAINT).align(AlignItems::STRETCH).on_key(keys);
    for (i, o) in options.iter().enumerate() {
        let on = i == current;
        let mut seg = row().key(format!("{key}:{i}")).center().pad(10.0, 0.0).radius(RADIUS).on_press(msg(i)).focusable(false)
            .child(text(*o, LABEL - 1.0, if on { Paint::GROUND } else { Paint::TEXT }));
        if on { seg = seg.fill(Paint::INK); }
        r = r.child(seg);
    }
    r
}
