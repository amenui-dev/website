//! bottom_sheet
//!
//! Purpose: A panel that rises from the bottom edge for a short task, dragged by its handle or body
//!   between resting heights, and swiped down to dismiss.
//! Props: `key`, `sheet` (`Sheet`: `detents`, `dismiss_velocity`, `projection_decel`, `backdrop`,
//!   `spring`, `rubber`, `height`), `title`, `body`, `msg`.
//! States: closed, presenting, resting on a detent, dragged (captured until release), springing to a
//!   detent or away; Esc dismisses; role dialog.
//! Messages: `msg(SheetMsg)` for every drag step, Esc and a press on the backdrop; the application
//!   calls `Sheet::update` with it and `Sheet::step` on every tick while `Sheet::moving()`.
//! Usage: `bottom_sheet("share", &s.sheet, "Share", body, Msg::Sheet)` (and `Cmd::Focus` on `share/handle`
//!   when presenting)
//!
//! The sheet's height is state (`Sheet::shown`, logical px above the bottom edge); the widget only
//! reports input and moves the panel with `El::shift`, so a drag lays nothing out. Letting go hands the
//! finger's velocity to the spring; where it rests is decided by where that velocity would carry it
//! (`gesture::project` with `projection_decel`): the nearest detent, or away when the projection ends
//! below half the lowest detent, or below the lowest one while moving down faster than
//! `dismiss_velocity`. Above the top detent the drag is rubber-banded (`gesture::RubberBand`). A scroll
//! area in the body scrolls first; a finger pulling it down at its top drags the sheet instead
//! (DESIGN.md §1.8.2, hand-off).
use super::{LABEL, LINE, MENU_SURFACE, RADIUS};
use crate::el::{Drag, El, Font, Key, Paint, Phase, Response, Role, col, layers, space, text};
use crate::gesture::{Animated, RubberBand, Spring, Velocity, project};
use std::rc::Rc;
use taffy::AlignItems;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SheetMsg { Drag(Drag), Key(Key), Backdrop }

/// A bottom sheet's state and its numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct Sheet {
    /// Resting heights as fractions of `height`, lowest first (default 0.45 and 0.9).
    pub detents: Vec<f32>,
    /// Moving down at least this fast (px/s) when let go below the lowest detent dismisses (default 900).
    pub dismiss_velocity: f32,
    /// How fast a flick decays, per millisecond, for the projection (default 0.998).
    pub projection_decel: f32,
    /// How far the backdrop covers the page at the lowest detent and above (ground alpha, default 0.5).
    pub backdrop: f32,
    pub spring: Spring,
    pub rubber: RubberBand,
    /// The height of the area the sheet rises in (logical px; the application sets it).
    pub height: f32,
    shown: Animated,
    open: bool,
    /// While dragged: how high it was at the press.
    grab: Option<f32>,
    vel: Velocity,
}
impl Default for Sheet {
    fn default() -> Self {
        let spring = Spring::default();
        Self { detents: vec![0.45, 0.9], dismiss_velocity: 900.0, projection_decel: 0.998, backdrop: 0.5, spring, rubber: RubberBand::default(), height: 640.0,
               shown: Animated::new(0.0, spring), open: false, grab: None, vel: Velocity::default() }
    }
}
impl Sheet {
    /// Open and at rest on detent `i`, in an area `height` tall (for a first frame, or a still).
    pub fn resting(height: f32, i: usize) -> Self { Self::default().rest_on(height, i) }
    /// This sheet, open and at rest on detent `i` in an area `height` tall.
    pub fn rest_on(mut self, height: f32, i: usize) -> Self {
        self.height = height;
        self.open = true;
        let h = self.detent(i);
        self.shown.hold(h);
        self
    }
    /// How high it shows now (logical px above the bottom edge).
    pub fn shown(&self) -> f32 { self.shown.value }
    /// Where it is going (px), and its velocity (px/s, up is positive).
    pub fn target(&self) -> f32 { self.shown.target }
    pub fn velocity(&self) -> f32 { self.shown.velocity }
    /// Detent `i` in px.
    pub fn detent(&self, i: usize) -> f32 { self.detents.get(i).copied().unwrap_or(0.5) * self.height }
    fn top(&self) -> f32 { self.detents.iter().copied().fold(0.0, f32::max) * self.height }
    fn lowest(&self) -> f32 { self.detents.iter().copied().fold(f32::MAX, f32::min).min(1.0) * self.height }
    /// Present or on screen (dismissing included).
    pub fn visible(&self) -> bool { self.open || self.shown.value > 0.0 }
    /// Presented (not dismissed).
    pub fn is_open(&self) -> bool { self.open }
    pub fn dragging(&self) -> bool { self.grab.is_some() }
    /// Springing (call `step` on every tick, and ask for frames, while it is).
    pub fn moving(&self) -> bool { self.shown.moving() }
    /// 0 closed … 1 at the lowest detent or above: how far the backdrop has dimmed.
    pub fn progress(&self) -> f32 { (self.shown.value / self.lowest().max(1.0)).clamp(0.0, 1.0) }
    /// Rise to the lowest detent.
    pub fn present(&mut self, now: f32) { self.open = true; self.grab = None; self.shown.spring = self.spring; self.shown.to(self.lowest(), 0.0, now); }
    /// Spring away with velocity `v` (px/s, up positive).
    pub fn dismiss(&mut self, v: f32, now: f32) { self.open = false; self.grab = None; self.shown.spring = self.spring; self.shown.to(0.0, v.min(0.0), now); }
    /// Where a release at height `at` moving at `v` (px/s, up positive) comes to rest: `None` dismisses.
    pub fn settle(&self, at: f32, v: f32) -> Option<f32> {
        let end = at + project(v, self.projection_decel);
        let low = self.lowest();
        if end < low * 0.5 || (end < low && -v >= self.dismiss_velocity) { return None; }
        let mut best = (f32::MAX, low);
        for &d in &self.detents { let h = d * self.height; if (h - end).abs() < best.0 { best = ((h - end).abs(), h); } }
        Some(best.1)
    }
    pub fn update(&mut self, m: SheetMsg, now: f32) {
        match m {
            SheetMsg::Drag(d) => match d.phase {
                Phase::Start => { self.grab = Some(self.shown.value); self.vel.clear(); self.vel.push(now, 0.0, d.dy); self.shown.hold(self.shown.value); }
                Phase::Move | Phase::End => {
                    let Some(g) = self.grab else { return };
                    self.vel.push(now, 0.0, d.dy);
                    // The finger says where; above the top detent it resists; it cannot go below the edge.
                    let raw = g - d.dy;
                    let top = self.top();
                    self.shown.hold(self.rubber.clamp(raw, 0.0, top, self.height).max(0.0));
                    if d.phase == Phase::End {
                        self.grab = None;
                        let v = -self.vel.get().1;
                        match self.settle(self.shown.value.min(top), v) {
                            Some(h) => { self.shown.spring = self.spring; self.shown.to(h, v, now); }
                            None => self.dismiss(v, now),
                        }
                    }
                }
            },
            SheetMsg::Key(Key::Escape) | SheetMsg::Backdrop => self.dismiss(0.0, now),
            SheetMsg::Key(_) => {}
        }
    }
    /// Spring to `now`; returns whether it moved.
    pub fn step(&mut self, now: f32) -> bool { self.shown.step(now) }
}

/// The sheet over a page: put it after the page in a `layers` box the size of the area (clip the box).
/// A backdrop that dims with `Sheet::progress` and the panel, moved down to show `Sheet::shown`. The
/// panel is twice `sheet.height` tall so a pull past the top never shows a gap under it. Nothing when
/// it is closed.
pub fn bottom_sheet<M: Clone + 'static>(key: &str, s: &Sheet, title: &str, body: El<M>, msg: impl Fn(SheetMsg) -> M + 'static) -> El<M> {
    if !s.visible() { return space(0.0, 0.0).key(key); }
    let msg = Rc::new(msg);
    let (a, b, c) = (msg.clone(), msg.clone(), msg.clone());
    let h = s.height;
    let top = s.top();
    // The page's own ground over it, more opaque as the sheet rises: it dims on a dark theme and pales
    // on a light one, and is never pushed by fronts.
    let backdrop = col().key(format!("{key}/backdrop")).fill_width().fill_height()
        .fill(Paint::GROUND.alpha(s.backdrop * s.progress())).respond(Response::Ignore).on_press(msg(SheetMsg::Backdrop)).focusable(false).unframed();
    // The handle takes focus when the sheet is presented (`Cmd::Focus { key: "<key>/handle" }`), so Esc
    // reaches it; a press on the body focuses the panel, which hears Esc too.
    let handle = col().key(format!("{key}/handle")).fill_width().height(22.0).center().role(Role::Button).label(crate::i18n::tr("widget.sheet.handle"))
        .on_key(move |k, _| (k == Key::Escape).then(|| c(SheetMsg::Key(k)))).child(space(36.0, 4.0).radius(2.0).fill(Paint::DIM).key(format!("{key}/grabber")));
    let head = text(title, LABEL + 2.0, Paint::INK).font(Font::Display).key(format!("{key}/title"));
    // The body is as tall as what shows at the top detent.
    let body = col().key(format!("{key}/body")).fill_width().height((top - 22.0 - 40.0).max(40.0)).scroll().child(body);
    let panel = col().key(format!("{key}/panel")).fill_width().height(h * 2.0).pad(18.0, 0.0).gap(8.0)
        .radius(RADIUS * 3.0).fill(MENU_SURFACE).border(LINE, Paint::HAIRLINE).align(AlignItems::START)
        .role(Role::Dialog).label(title).focusable(true).unframed()
        .shift(0.0, h - s.shown())
        // Rising, dragged or springing, its edge stirs the field map (DESIGN.md §1.29).
        .stirr(crate::el::Stirr::default())
        .on_drag_with(move |d| Some(a(SheetMsg::Drag(d))))
        .on_key(move |k, _| (k == Key::Escape).then(|| b(SheetMsg::Key(k))))
        .child(handle).child(head).child(body);
    layers([backdrop, panel]).key(key.to_owned()).fill_width().fill_height()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn drag(phase: Phase, dy: f32) -> SheetMsg { SheetMsg::Drag(Drag { phase, x: 0.0, y: 0.0, w: 360.0, h: 640.0, dx: 0.0, dy, t: 0.0 }) }
    fn settle(s: &mut Sheet, mut t: f32) -> f32 { let end = t + 10.0; while s.moving() && t < end { t += 1.0 / 60.0; s.step(t); } t }
    /// Drag by `total` px over `secs`, in 60 Hz steps, and let go.
    fn swipe(s: &mut Sheet, t0: f32, total: f32, secs: f32) -> f32 {
        s.update(drag(Phase::Start, 0.0), t0);
        let n = (secs * 60.0).ceil().max(1.0) as usize;
        for i in 1..=n { s.update(drag(if i == n { Phase::End } else { Phase::Move }, total * i as f32 / n as f32), t0 + i as f32 / 60.0); }
        t0 + n as f32 / 60.0
    }
    fn opened() -> Sheet { let mut s = Sheet::default(); s.present(0.0); settle(&mut s, 0.0); s }

    #[test]
    fn presents_to_the_lowest_detent() {
        let s = opened();
        assert_eq!(s.shown(), s.detent(0));
        assert!(!s.moving() && s.progress() == 1.0);
    }
    #[test]
    fn a_slow_drag_below_the_threshold_returns() {
        let mut s = opened();
        let t = swipe(&mut s, 1.0, 80.0, 1.2);
        assert!(s.is_open() && s.target() == s.detent(0), "returns to the detent, target {}", s.target());
        settle(&mut s, t);
        assert_eq!(s.shown(), s.detent(0));
    }
    #[test]
    fn a_fast_flick_dismisses() {
        let mut s = opened();
        // 60 px in 0.05 s: 1200 px/s down — far less distance than the slow drag above.
        let t = swipe(&mut s, 1.0, 60.0, 0.05);
        assert!(!s.is_open() && s.target() == 0.0);
        assert!(s.velocity() < -900.0, "the release velocity carries on into the spring: {}", s.velocity());
        settle(&mut s, t);
        assert!(!s.visible());
    }
    #[test]
    fn a_flick_up_goes_to_the_next_detent_and_overshoot_is_banded() {
        let mut s = opened();
        let t = swipe(&mut s, 1.0, -120.0, 0.1);
        assert_eq!(s.target(), s.detent(1));
        settle(&mut s, t);
        // Dragged far above the top detent: banded, and never beyond top + height.
        s.update(drag(Phase::Start, 0.0), 20.0);
        let mut last = s.shown();
        for i in 1..=200 {
            s.update(drag(Phase::Move, -20.0 * i as f32), 20.0 + i as f32 / 60.0);
            assert!(s.shown() >= last && s.shown() < s.detent(1) + s.height);
            last = s.shown();
        }
        assert!(last - s.detent(1) < 4000.0 * 0.55);
        s.update(drag(Phase::End, -4000.0), 24.0);
        settle(&mut s, 24.0);
        assert_eq!(s.shown(), s.detent(1));
    }
    #[test]
    fn esc_and_the_backdrop_dismiss() {
        let mut s = opened();
        s.update(SheetMsg::Key(Key::Escape), 1.0);
        assert!(!s.is_open());
        let mut s = opened();
        s.update(SheetMsg::Backdrop, 1.0);
        settle(&mut s, 1.0);
        assert!(!s.visible());
    }
}
