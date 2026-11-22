//! orbit
//!
//! Purpose: Turn a 3D view: drag to orbit, the wheel to dolly, arrow keys when focused; let go and it
//! coasts to a stop.
//! Props: `key`, `scene` (its camera is replaced by the orbit's), `orbit`, `msg`.
//! States: rest, focused, dragging (captured until release), coasting (after a fast drag).
//! Messages: `msg(OrbitMsg)` for every drag step, wheel turn and arrow key; the application calls
//! `Orbit::update` with it and `Orbit::step` on every tick while `Orbit::moving()`.
//! Usage: `orbit("view", scene, &s.orbit, Msg::Orbit)`
//!
//! The camera is state: the widget only reports input. A drag turns the camera by `per_px` degrees a
//! pixel from where it was at the press (no drift on a long drag); letting go keeps the last 0.1 s's
//! speed and lets it decay as `v e^(−friction t)` — the same exact step as a finger's fling (DESIGN.md
//! §1.8.2) — until it is below `stop` degrees a second. The wheel dollies by a factor per pixel; the
//! arrows turn by `step_deg` (Shift × 4).
use crate::el::{Drag, El, Key, Phase, scene3d};
use crate::mesh::{Camera, Scene3d};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OrbitMsg { Drag(Drag), Wheel(f32), Key(Key, bool) }

/// An orbiting camera's state and its numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct Orbit {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    /// Degrees a second, while coasting.
    pub vyaw: f32,
    pub vpitch: f32,
    /// Degrees per pixel of drag; degrees per arrow key; how fast coasting dies (1/s) and below what
    /// speed it stops (degrees a second); the fastest it may start (degrees a second).
    pub per_px: f32,
    pub step_deg: f32,
    pub friction: f32,
    pub stop: f32,
    pub max_speed: f32,
    /// Distance factor per pixel of wheel, and the nearest and furthest the camera may be.
    pub dolly: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    /// The press (yaw and pitch then) and the last steps of the drag: time, yaw, pitch.
    grab: Option<(f32, f32)>,
    samples: Vec<(f32, f32, f32)>,
    last: f32,
}
impl Default for Orbit {
    fn default() -> Self {
        Self { yaw: 30.0, pitch: 22.0, distance: 3.2, vyaw: 0.0, vpitch: 0.0, per_px: 0.4, step_deg: 5.0, friction: 3.5, stop: 2.0, max_speed: 720.0,
               dolly: 0.0015, min_distance: 0.8, max_distance: 30.0, grab: None, samples: vec![], last: 0.0 }
    }
}
impl Orbit {
    /// Whether it is coasting (call `step` on every tick, and ask for frames, while it is).
    pub fn moving(&self) -> bool { self.grab.is_none() && (self.vyaw != 0.0 || self.vpitch != 0.0) }
    /// The camera the orbit gives a scene: `base` with this yaw, pitch and distance.
    pub fn camera(&self, base: Camera) -> Camera { Camera { yaw: self.yaw, pitch: self.pitch, distance: self.distance, ..base } }
    pub fn update(&mut self, m: OrbitMsg, now: f32) {
        match m {
            OrbitMsg::Drag(d) => match d.phase {
                Phase::Start => { self.grab = Some((self.yaw, self.pitch)); self.vyaw = 0.0; self.vpitch = 0.0; self.samples.clear(); self.samples.push((now, self.yaw, self.pitch)); }
                Phase::Move | Phase::End => {
                    let Some((y0, p0)) = self.grab else { return };
                    self.yaw = y0 - d.dx * self.per_px;
                    self.pitch = (p0 + d.dy * self.per_px).clamp(-85.0, 85.0);
                    self.samples.push((now, self.yaw, self.pitch));
                    self.samples.retain(|s| now - s.0 <= 0.1);
                    if d.phase == Phase::End {
                        self.grab = None;
                        let (a, b) = (self.samples[0], self.samples[self.samples.len() - 1]);
                        let dt = b.0 - a.0;
                        let (vy, vp) = if dt > 1e-3 { ((b.1 - a.1) / dt, (b.2 - a.2) / dt) } else { (0.0, 0.0) };
                        let sp = (vy * vy + vp * vp).sqrt();
                        let k = if sp > self.max_speed { self.max_speed / sp } else { 1.0 };
                        (self.vyaw, self.vpitch) = if sp * k > self.stop * 4.0 { (vy * k, vp * k) } else { (0.0, 0.0) };
                        self.last = now;
                    }
                }
            },
            OrbitMsg::Wheel(dy) => self.distance = (self.distance * (dy * self.dolly).exp()).clamp(self.min_distance, self.max_distance),
            OrbitMsg::Key(k, shift) => {
                let s = self.step_deg * if shift { 4.0 } else { 1.0 };
                match k {
                    Key::Left => self.yaw += s, Key::Right => self.yaw -= s,
                    Key::Up => self.pitch = (self.pitch + s).clamp(-85.0, 85.0), Key::Down => self.pitch = (self.pitch - s).clamp(-85.0, 85.0),
                    _ => {}
                }
            }
        }
    }
    /// Coast to `now`: the exact integral of the decaying speed. Returns whether the camera moved.
    pub fn step(&mut self, now: f32) -> bool {
        if !self.moving() { self.last = now; return false; }
        let dt = (now - self.last).max(0.0);
        self.last = now;
        let f = self.friction.max(1e-3);
        let (g, decay) = ((1.0 - (-f * dt).exp()) / f, (-f * dt).exp());
        self.yaw += self.vyaw * g;
        self.pitch = (self.pitch + self.vpitch * g).clamp(-85.0, 85.0);
        self.vyaw *= decay;
        self.vpitch *= decay;
        if (self.vyaw * self.vyaw + self.vpitch * self.vpitch).sqrt() < self.stop { self.vyaw = 0.0; self.vpitch = 0.0; }
        dt > 0.0
    }
}

/// The scene, seen through the orbit's camera; drag, wheel and arrows report `OrbitMsg`s.
pub fn orbit<M: 'static>(key: &str, mut scene: Scene3d, o: &Orbit, msg: impl Fn(OrbitMsg) -> M + 'static) -> El<M> {
    scene.camera = o.camera(scene.camera);
    let msg = Rc::new(msg);
    let (a, b, c) = (msg.clone(), msg.clone(), msg);
    scene3d(scene).key(key).focusable(true).cursor(crate::el::Cursor::Grab).label(crate::i18n::tr("widget.orbit.label"))
        .on_drag_with(move |d| Some(a(OrbitMsg::Drag(d))))
        .on_wheel(move |_, dy| Some(b(OrbitMsg::Wheel(dy))))
        .on_key(move |k, shift| matches!(k, Key::Left | Key::Right | Key::Up | Key::Down).then(|| c(OrbitMsg::Key(k, shift))))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn drag(phase: Phase, dx: f32) -> OrbitMsg { OrbitMsg::Drag(Drag { phase, x: 0.0, y: 0.0, w: 100.0, h: 100.0, dx, dy: 0.0, t: 0.0 }) }

    #[test]
    fn drags_from_the_press_and_coasts_to_a_stop() {
        let mut o = Orbit::default();
        let y0 = o.yaw;
        o.update(drag(Phase::Start, 0.0), 0.0);
        for i in 1..=6 { o.update(drag(Phase::Move, -20.0 * i as f32), i as f32 / 60.0); }
        assert!((o.yaw - (y0 + 120.0 * o.per_px)).abs() < 1e-3);
        o.update(drag(Phase::End, -120.0), 6.0 / 60.0);
        assert!(o.moving() && o.vyaw > 0.0);
        // It travels v / friction in all (the exact integral), and stops.
        let (v, y1) = (o.vyaw, o.yaw);
        let mut t = 0.1;
        while o.moving() && t < 10.0 { t += 1.0 / 60.0; o.step(t); }
        assert!(!o.moving());
        let expect = (v - o.stop.min(v)) / o.friction;
        assert!((o.yaw - y1 - v / o.friction).abs() <= expect.max(1.0) * 0.05 + o.stop / o.friction + 0.5, "{} vs {}", o.yaw - y1, v / o.friction);
        // A slow release does not coast.
        o.update(drag(Phase::Start, 0.0), 20.0);
        o.update(drag(Phase::End, -0.1), 20.5);
        assert!(!o.moving());
    }

    #[test]
    fn wheel_and_keys() {
        let mut o = Orbit::default();
        let d = o.distance;
        o.update(OrbitMsg::Wheel(100.0), 0.0);
        assert!(o.distance > d);
        o.update(OrbitMsg::Wheel(-1e6), 0.0);
        assert_eq!(o.distance, o.min_distance);
        let y = o.yaw;
        o.update(OrbitMsg::Key(Key::Left, true), 0.0);
        assert_eq!(o.yaw, y + o.step_deg * 4.0);
        o.update(OrbitMsg::Key(Key::Up, false), 0.0);
        for _ in 0..40 { o.update(OrbitMsg::Key(Key::Up, true), 0.0); }
        assert_eq!(o.pitch, 85.0);
    }
}
