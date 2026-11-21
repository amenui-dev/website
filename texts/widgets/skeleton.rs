//! skeleton
//!
//! Purpose: Hold the place of content that is still loading, in its shape.
//! Props: `key`, `width`, `height`, `round` (a disc rather than a bar); for a list, `Loading`
//!   (`skeleton_after_ms`, `swap_stagger_ms`) and `El::skeleton()` on the real rows.
//! States: nothing yet (the first `skeleton_after_ms`), skeleton, content (row by row); it stays still:
//!   no shimmer, loading is not an animation here.
//! Messages: none.
//! Usage: `skeleton("title", 220.0, 16.0, false)`, or `row_el.skeleton()` while
//!   `loading.show(i, now) == Show::Skeleton`
//!
//! A faint block; replaced by the content when it arrives (keyed text then condenses in). Better: draw
//! the real row with `El::skeleton()` — the same tree, laid out the same, its text drawn as bars of the
//! measured line widths and its pictures as blocks — so swapping it for the content moves nothing.
use super::RADIUS;
use crate::el::{El, Paint, space};

pub fn skeleton<M>(key: &str, width: f32, height: f32, round: bool) -> El<M> {
    space(width, height).key(key).radius(if round { width.min(height) * 0.5 } else { RADIUS }).fill(Paint::FAINT)
}

/// What a loading row shows now (`Loading::show`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Show {
    /// Nothing yet: the place is kept, nothing is drawn (the first `skeleton_after_ms`).
    Nothing,
    /// The row's own skeleton (`El::skeleton` on the real row).
    Skeleton,
    /// The content.
    Content,
}

/// The rules of a loading list, as numbers (DESIGN.md §1.23): what does not depend on the data draws at
/// once; a skeleton appears only if the data takes longer than `skeleton_after_ms` (default 300), so a
/// quick answer never flashes one; when the data arrives, row `i` replaces its skeleton in place
/// `i × swap_stagger_ms` (default 40) after it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Loading {
    pub skeleton_after_ms: f32,
    pub swap_stagger_ms: f32,
    started: Option<f32>,
    arrived: Option<f32>,
}
impl Default for Loading {
    fn default() -> Self { Self { skeleton_after_ms: 300.0, swap_stagger_ms: 40.0, started: None, arrived: None } }
}
impl Loading {
    /// A request went out at `now` (s).
    pub fn start(&mut self, now: f32) { self.started = Some(now); self.arrived = None; }
    /// The data came at `now`.
    pub fn arrive(&mut self, now: f32) { if self.started.is_some() && self.arrived.is_none() { self.arrived = Some(now); } }
    pub fn started(&self) -> Option<f32> { self.started }
    pub fn arrived(&self) -> Option<f32> { self.arrived }
    /// What row `i` shows at `now`.
    pub fn show(&self, i: usize, now: f32) -> Show {
        let Some(s) = self.started else { return Show::Content };
        let shown_at = s + self.skeleton_after_ms / 1000.0;
        match self.arrived {
            Some(a) if now >= a + i as f32 * self.swap_stagger_ms / 1000.0 => Show::Content,
            // A skeleton only once it is due, and only if the data was not already here by then.
            _ if now >= shown_at && self.arrived.is_none_or(|a| a > shown_at) => Show::Skeleton,
            _ => Show::Nothing,
        }
    }
    /// The next time anything changes for `rows` rows after `now`, if any (for `Ui::on_wake`).
    pub fn next_change(&self, rows: usize, now: f32) -> Option<f32> {
        let s = self.started?;
        let mut times = vec![s + self.skeleton_after_ms / 1000.0];
        if let Some(a) = self.arrived { times.extend((0..rows).map(|i| a + i as f32 * self.swap_stagger_ms / 1000.0)); }
        times.into_iter().filter(|&t| t > now).reduce(f32::min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_skeleton_only_after_the_wait_and_rows_swap_in_turn() {
        let mut l = Loading::default();
        l.start(0.0);
        assert_eq!(l.show(0, 0.29), Show::Nothing);
        assert_eq!(l.show(0, 0.31), Show::Skeleton);
        l.arrive(1.0);
        assert_eq!((l.show(0, 1.0), l.show(1, 1.0), l.show(1, 1.041), l.show(3, 1.1), l.show(3, 1.121)), (Show::Content, Show::Skeleton, Show::Content, Show::Skeleton, Show::Content));
        assert_eq!(l.next_change(4, 1.05), Some(1.08));
        // A quick answer never shows a skeleton.
        let mut q = Loading::default();
        q.start(0.0);
        q.arrive(0.2);
        for t in [0.2, 0.25, 0.31, 0.4] { for i in 0..4 { assert_ne!(q.show(i, t), Show::Skeleton); } }
    }
}
