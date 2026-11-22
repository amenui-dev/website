//! floating
//!
//! Purpose: A small box that floats over an area (a picture-in-picture player): dragged anywhere, and
//!   let go it glides to the corner its throw is heading for.
//! Props: `key`, `floating` (`Floating`: `size`, `area`, `corner_inset`, `projection_decel`, `spring`,
//!   `rubber`), `content`, `msg`.
//! States: resting in a corner, dragged (captured until release), springing to a corner; arrow keys
//!   move it to the neighbouring corner when focused.
//! Messages: `msg(FloatMsg)` for every drag step and arrow key; the application calls
//!   `Floating::update` with it and `Floating::step` on every tick while `Floating::moving()`.
//! Usage: `floating("pip", &s.pip, player, Msg::Pip)`
//!
//! The position is state; the widget only reports input and moves the box with `El::shift`. The corner
//! is chosen by the projected end of the throw (`gesture::project`), not by where the finger let go, so
//! a short flick toward the far side gets there. Past the area's edges the drag is rubber-banded.
use super::RADIUS;
use crate::el::{Drag, El, Key, Paint, Phase, Role, col};
use crate::gesture::{Animated, RubberBand, Spring, Velocity, project};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatMsg { Drag(Drag), Key(Key) }

/// A floating box's state and its numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct Floating {
    /// The box (logical px) and the area it floats in (the application sets it).
    pub size: (f32, f32),
    pub area: (f32, f32),
    /// How far a resting box stays from the area's edges (default 16).
    pub corner_inset: f32,
    /// How fast a throw decays, per millisecond, for the projection (default 0.998).
    pub projection_decel: f32,
    pub spring: Spring,
    pub rubber: RubberBand,
    x: Animated,
    y: Animated,
    grab: Option<(f32, f32)>,
    vel: Velocity,
    corner: usize,
}
impl Default for Floating {
    fn default() -> Self {
        let spring = Spring::default();
        let mut f = Self { size: (160.0, 96.0), area: (480.0, 320.0), corner_inset: 16.0, projection_decel: 0.998, spring, rubber: RubberBand::default(),
                           x: Animated::new(0.0, spring), y: Animated::new(0.0, spring), grab: None, vel: Velocity::default(), corner: 3 };
        let (x, y) = f.corner_at(3);
        f.x.hold(x);
        f.y.hold(y);
        f
    }
}
impl Floating {
    /// Its top-left corner in the area (logical px).
    pub fn at(&self) -> (f32, f32) { (self.x.value, self.y.value) }
    pub fn velocity(&self) -> (f32, f32) { (self.x.velocity, self.y.velocity) }
    /// Which corner it rests in or goes to: 0 top left, 1 top right, 2 bottom left, 3 bottom right.
    pub fn corner(&self) -> usize { self.corner }
    /// Corner `i`'s resting position.
    pub fn corner_at(&self, i: usize) -> (f32, f32) {
        let (l, t) = (self.corner_inset, self.corner_inset);
        let (r, b) = (self.area.0 - self.size.0 - self.corner_inset, self.area.1 - self.size.1 - self.corner_inset);
        (if i.is_multiple_of(2) { l } else { r.max(l) }, if i < 2 { t } else { b.max(t) })
    }
    pub fn moving(&self) -> bool { self.x.moving() || self.y.moving() }
    pub fn dragging(&self) -> bool { self.grab.is_some() }
    /// Put it in corner `i` at once (after the area changed, say).
    pub fn place(&mut self, i: usize) { self.corner = i.min(3); let (x, y) = self.corner_at(self.corner); self.x.hold(x); self.y.hold(y); }
    /// The corner nearest to where a throw from (x, y) at (vx, vy) px/s would come to rest.
    pub fn aim(&self, x: f32, y: f32, vx: f32, vy: f32) -> usize {
        let end = (x + project(vx, self.projection_decel), y + project(vy, self.projection_decel));
        (0..4).min_by(|&a, &b| {
            let d = |i| { let c = self.corner_at(i); (c.0 - end.0).powi(2) + (c.1 - end.1).powi(2) };
            d(a).total_cmp(&d(b))
        }).unwrap_or(3)
    }
    fn go(&mut self, i: usize, v: (f32, f32), now: f32) {
        self.corner = i;
        let (cx, cy) = self.corner_at(i);
        (self.x.spring, self.y.spring) = (self.spring, self.spring);
        self.x.to(cx, v.0, now);
        self.y.to(cy, v.1, now);
    }
    pub fn update(&mut self, m: FloatMsg, now: f32) {
        match m {
            FloatMsg::Drag(d) => match d.phase {
                Phase::Start => { self.grab = Some(self.at()); self.vel.clear(); self.vel.push(now, d.dx, d.dy); self.x.hold(self.x.value); self.y.hold(self.y.value); }
                Phase::Move | Phase::End => {
                    let Some((gx, gy)) = self.grab else { return };
                    self.vel.push(now, d.dx, d.dy);
                    let (mx, my) = (self.area.0 - self.size.0, self.area.1 - self.size.1);
                    self.x.hold(self.rubber.clamp(gx + d.dx, 0.0, mx.max(0.0), self.area.0));
                    self.y.hold(self.rubber.clamp(gy + d.dy, 0.0, my.max(0.0), self.area.1));
                    if d.phase == Phase::End {
                        self.grab = None;
                        let v = self.vel.get();
                        let i = self.aim(self.x.value, self.y.value, v.0, v.1);
                        self.go(i, v, now);
                    }
                }
            },
            FloatMsg::Key(k) => {
                let c = self.corner;
                let i = match k { Key::Left => c & !1, Key::Right => c | 1, Key::Up => c & 1, Key::Down => c | 2, _ => return };
                self.go(i, (0.0, 0.0), now);
            }
        }
    }
    /// Spring to `now`; returns whether it moved.
    pub fn step(&mut self, now: f32) -> bool { let a = self.x.step(now); let b = self.y.step(now); a || b }
}

/// The box over a page: put it after the page in a `layers` box `floating.area` big. `content` inside
/// a rounded, draggable frame moved to `Floating::at`.
pub fn floating<M: Clone + 'static>(key: &str, f: &Floating, content: El<M>, msg: impl Fn(FloatMsg) -> M + 'static) -> El<M> {
    let msg = Rc::new(msg);
    let (a, b) = (msg.clone(), msg);
    let (x, y) = f.at();
    col().key(key.to_owned()).width(f.size.0).height(f.size.1).radius(RADIUS * 2.0).clip()
        .fill(Paint::GROUND).border(1.0, Paint::HAIRLINE).focusable(true).role(Role::Group).label(crate::i18n::tr("widget.floating.label"))
        .cursor(if f.dragging() { crate::el::Cursor::Grabbing } else { crate::el::Cursor::Grab })
        .shift(x, y)
        // Dragged, thrown or gliding, it leaves a trail in the field map (DESIGN.md §1.29).
        .stirr(crate::el::Stirr::default())
        .on_drag_with(move |d| Some(a(FloatMsg::Drag(d))))
        .on_key(move |k, _| matches!(k, Key::Left | Key::Right | Key::Up | Key::Down).then(|| b(FloatMsg::Key(k))))
        .child(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn drag(phase: Phase, dx: f32, dy: f32) -> FloatMsg { FloatMsg::Drag(Drag { phase, x: 0.0, y: 0.0, w: 160.0, h: 96.0, dx, dy, t: 0.0 }) }
    fn throw(f: &mut Floating, t0: f32, path: &[(f32, f32)]) -> f32 {
        f.update(drag(Phase::Start, 0.0, 0.0), t0);
        for (i, &(dx, dy)) in path.iter().enumerate() { f.update(drag(if i + 1 == path.len() { Phase::End } else { Phase::Move }, dx, dy), t0 + (i + 1) as f32 / 60.0); }
        t0 + path.len() as f32 / 60.0
    }
    fn settle(f: &mut Floating, mut t: f32) { let end = t + 10.0; while f.moving() && t < end { t += 1.0 / 60.0; f.step(t); } }

    #[test]
    fn the_corner_is_chosen_by_projection_not_by_the_release_point() {
        let mut f = Floating::default();
        f.place(2); // bottom left: (16, 208)
        // A slow drag 100 px to the right lets go nearer the left: it goes back left.
        let slow: Vec<(f32, f32)> = (1..=120).map(|i| (100.0 * i as f32 / 120.0, 0.0)).collect();
        let t = throw(&mut f, 0.0, &slow);
        assert_eq!(f.corner(), 2);
        settle(&mut f, t);
        assert_eq!(f.at(), f.corner_at(2));
        // A quick flick of only 60 px to the right is let go just as near the left, but heads right.
        let t = throw(&mut f, 5.0, &[(20.0, 0.0), (40.0, 0.0), (60.0, 0.0)]);
        let release = f.at();
        assert!(release.0 - f.corner_at(2).0 < (f.corner_at(3).0 - f.corner_at(2).0) * 0.5, "let go on the left half");
        assert_eq!(f.corner(), 3, "the throw's projection picks the right corner");
        assert!(f.velocity().0 > 500.0, "and its velocity carries on into the spring");
        settle(&mut f, t);
        assert_eq!(f.at(), f.corner_at(3));
    }
    #[test]
    fn dragged_past_the_edge_it_resists_and_keys_move_it() {
        let mut f = Floating::default();
        f.place(0);
        f.update(drag(Phase::Start, 0.0, 0.0), 0.0);
        f.update(drag(Phase::Move, -400.0, 0.0), 0.5);
        assert!(f.at().0 < 0.0 && f.at().0 > -f.area.0, "banded: {}", f.at().0);
        assert!(f.at().0 > -400.0 * 0.55);
        f.update(drag(Phase::End, -400.0, 0.0), 1.0);
        settle(&mut f, 1.0);
        assert_eq!(f.corner(), 0);
        f.update(FloatMsg::Key(Key::Down), 3.0);
        settle(&mut f, 3.0);
        assert_eq!((f.corner(), f.at()), (2, f.corner_at(2)));
    }
}
