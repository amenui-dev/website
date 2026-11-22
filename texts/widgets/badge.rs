//! badge
//!
//! Purpose: A small label for a status or a count.
//! Props: `key`, `label`, `variant` (`Variant::Default` outlined, `Quiet` faint, `Primary` ink, `Accent` filled).
//! States: none.
//! Messages: none.
//! Usage: `badge("new", "New", Variant::Primary)`
//!
//! A pill of small type; filled with the accent for what needs attention.
use super::{LABEL, LINE, Variant};
use crate::el::{El, Paint, row, text};

pub fn badge<M>(key: &str, label: &str, v: Variant) -> El<M> {
    let r = row().key(key).height(20.0).pad(8.0, 0.0).radius(10.0).center();
    let (r, ink) = match v { Variant::Accent => (r.fill(Paint::ACCENT), Paint::GROUND), Variant::Primary => (r.fill(Paint::INK), Paint::GROUND), Variant::Quiet => (r.fill(Paint::FAINT), Paint::TEXT), Variant::Default => (r.border(LINE, Paint::DIM), Paint::TEXT) };
    r.child(text(label, LABEL - 3.0, ink))
}
