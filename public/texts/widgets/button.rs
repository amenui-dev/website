//! button
//!
//! Purpose: Run one action.
//! Props: `key`, `label`, `v: Variant` (Default outlined, Quiet, Primary an ink plate, Accent filled),
//!   `msg`.
//! States: rest, hovered, focused, pressed (held at the legibility cap, condenses back from it),
//!   disabled (`.disabled(true)`).
//! Messages: sends `msg` on press, Enter or Space; its front reaches only itself unless
//!   `.ripplr(..)` says otherwise.
//! Usage: `button_as("save", "Save", Variant::Primary, Msg::Save)`
//!
//! A button: a label that sends a message when pressed (pointer, Enter or Space). To make it
//! unavailable, call `.disabled(true)` on what this returns: it then takes no input and is drawn faint.
use super::{CONTROL_H, LABEL, LINE, RADIUS};
use crate::el::{El, Paint, Role, row, text};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    /// Outlined, for most actions.
    Default,
    /// No outline, for actions that should not draw the eye.
    Quiet,
    /// An ink plate with its label in the ground's colour, turning to the scheme's extreme under the
    /// pointer: the one main action of a view (one in each group of actions).
    Primary,
    /// Filled with the accent colour: an explicitly coloured action.
    Accent,
}

pub fn button<M: Clone>(key: impl Into<String>, label: &str, msg: M) -> El<M> { button_as(key, label, Variant::Default, msg) }

pub fn button_as<M: Clone>(key: impl Into<String>, label: &str, v: Variant, msg: M) -> El<M> {
    let b = row().key(key).center().height(CONTROL_H).pad(14.0, 0.0).radius(RADIUS).on_press(msg).role(Role::Button);
    match v {
        Variant::Default => b.fill(Paint::GROUND).border(LINE, Paint::DIM).border_hover(Paint::INK).child(text(label, LABEL, Paint::TEXT)),
        Variant::Primary => b.fill(Paint::INK).fill_hover(Paint::EXTREME).child(text(label, LABEL, Paint::GROUND)),
        Variant::Quiet => b.child(text(label, LABEL, Paint::TEXT)),
        // The label is set in the page's ground colour, so it reads on the accent in every theme.
        Variant::Accent => b.fill(Paint::ACCENT).child(text(label, LABEL, Paint::GROUND)),
    }
}
