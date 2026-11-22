//! skip_link
//!
//! Purpose: A link that keyboard users meet first and that moves focus past the navigation to the
//! main content.
//! Props: `key`, `label`, `target` (the key of the element to jump to; focus goes to it, or to the
//! first element inside it that takes focus).
//! States: shown only while it has focus.
//! Messages: none (the runtime moves focus itself).
//! Usage: `stack([page, skip_link("skip", "Skip to content", "main")])`, with
//! `.tab_order(1)` on it when it is not first in the tree.
//!
//! A small filled tag at the top left, over the page, drawn only while focused: Tab reaches it
//! first, Enter jumps. Screen readers list it as a link.
use super::{LABEL, RADIUS};
use crate::el::{El, Paint, Role, row, text};

pub fn skip_link<M>(key: &str, label: &str, target: &str) -> El<M> {
    row().key(key).absolute(Some(8.0), Some(8.0), None, None).height(32.0).pad(12.0, 0.0).radius(RADIUS).center()
        .fill(Paint::INK).skip_to(target).focus_shown().role(Role::Link).label(label)
        .child(text(label, LABEL, Paint::GROUND))
}
