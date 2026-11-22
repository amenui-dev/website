//! scroll
//!
//! Purpose: Scroll what does not fit in a fixed height.
//! Props: `key`, `height`, `body`.
//! States: scroll offset kept by the runtime by key; a thin bar shows where it is.
//! Messages: none.
//! Usage: `scroll_area("code", 380.0, lines)`
//!
//! A scroll area: a fixed height that clips what it holds and scrolls it with the wheel, with a thin
//! bar for where it is.
use super::LINE;
use crate::el::{El, Paint, col};

pub fn scroll_area<M>(key: &str, height: f32, body: El<M>) -> El<M> {
    col().key(key).scroll().bar().height(height).radius(4.0).border(LINE, Paint::FAINT).pad(12.0, 10.0).child(body)
}
