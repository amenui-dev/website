//! progress
//!
//! Purpose: Show how far a task has come (0..1), or that it is going on (indeterminate).
//! Props: `key`, `value` (`None`: indeterminate), `width`.
//! States: a filled share of the track; indeterminate shows a short segment.
//! Messages: none.
//! Usage: `progress("upload", Some(0.4), 240.0)`
//!
//! A track and its fill; the progressbar role carries the value (0–100).
use super::RADIUS;
use crate::el::{El, Paint, Role, row, space};

pub fn progress<M>(key: &str, value: Option<f32>, width: f32) -> El<M> {
    let (v, at) = match value { Some(v) => (v.clamp(0.0, 1.0), 0.0), None => (0.3, 0.35) };
    let mut r = row().key(key).width(width).height(6.0).radius(RADIUS).fill(Paint::FAINT).role(Role::ProgressBar)
        .child(space(width * at, 6.0)).child(space(width * v, 6.0).radius(RADIUS).fill(Paint::INK).key(format!("{key}/fill")));
    if let Some(x) = value { r = r.value_range(x.clamp(0.0, 1.0) * 100.0, 0.0, 100.0); }
    r
}
