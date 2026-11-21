//! card_stack
//!
//! Purpose: A stack of cards that fans open under the pointer: hovering or pressing a card swings the
//!   cards beneath it out, each a little less and a little later than the one above.
//! Props: `key`, `stack` (`Stack`: `count`, `angle0`, `falloff`, `delay_ms`, `spring`, `look`,
//!   `card`, `paints`, `outline`, `outlines`), `msg`.
//! States: closed, fanning (cards springing toward their angles, each after its delay), fanned under
//!   card k, held (pressed: stays fanned when the pointer leaves); arrow keys move the fan when
//!   focused, Enter or Space holds it.
//! Messages: `msg(StackMsg)` for hover and presses on a card and for keys; the application calls
//!   `Stack::update` and `Stack::step` on every tick while `Stack::moving()`.
//! Usage: `card_stack("deck", &s.deck, Msg::Deck)`
//!
//! Card n below the hovered one turns to `angle(n) = angle0 · falloff^n` (degrees; n = 0 is the first
//! card beneath it) after `delay(n) = delay_ms · n`, about its bottom-left corner; cards above it stay
//! closed. The cards are a 3D element (`scene3d`, one cube per card), so the look follows the material:
//! `StackLook::Plain` flat cards seen straight on, `Thick` cards with depth seen at an angle so their
//! edges show (outlined, two tones), `Marks` flat cards drawn by a mark material (dot or ASCII) and held
//! loose so the marks show at rest.
use crate::el::{El, Key, Paint, Role, scene3d};
use crate::gesture::{Animated, Spring};
use crate::mesh::{Camera, Light, Mat4, MeshHit, MeshId, MeshInstances, Mesh3d, Pick, Projection, Scene3d};
use crate::style::Material;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StackMsg { Hover(Option<usize>), Press(Option<usize>), Key(Key) }

/// How the cards are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackLook { Plain, Thick, Marks(Material) }
impl StackLook {
    /// The look a material asks for: the mark materials draw marks, plain is plain, every other one
    /// (print, lino, glitch…) has thick cards.
    pub fn of(m: Material) -> Self {
        match m {
            Material::Plain | Material::Near => StackLook::Plain,
            Material::Dottr | Material::Charr | Material::Halftonr => StackLook::Marks(m),
            _ => StackLook::Thick,
        }
    }
}

/// A stack's state and its numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct Stack {
    pub count: usize,
    /// The first card beneath turns `angle0` degrees (default 42); each next one `falloff` of the one
    /// before (default 0.7); each starts `delay_ms` after the one before (default 28).
    pub angle0: f32,
    pub falloff: f32,
    pub delay_ms: f32,
    pub spring: Spring,
    pub look: StackLook,
    /// A card's size (world units: 1 is the view's height).
    pub card: (f32, f32),
    /// Card n's colour (cycled), and an outline (logical px) round flat cards (0: none).
    pub paints: Vec<Paint>,
    pub outline: f32,
    /// Card n's outline grey (card 0 is the top one; the last repeats for the rest; empty: the ink). By
    /// default every card is raised, the top one in an ink outline and the ones under it in hairline.
    pub outlines: Vec<Paint>,
    /// Which card the fan is under (hovered, or held by a press).
    hover: Option<usize>,
    held: Option<usize>,
    angles: Vec<Animated>,
    /// Card n's next target and when it starts.
    queue: Vec<Option<(f32, f32)>>,
}
impl Default for Stack {
    fn default() -> Self {
        let spring = Spring { stiffness: 260.0, damping: 24.0, ..Spring::default() };
        let count = 5;
        Self { count, angle0: 42.0, falloff: 0.7, delay_ms: 28.0, spring, look: StackLook::Plain, card: (0.8, 0.52), paints: vec![Paint::RAISED], outline: 1.0, outlines: vec![Paint::INK, Paint::HAIRLINE], hover: None, held: None,
               angles: vec![Animated::new(0.0, spring); count], queue: vec![None; count] }
    }
}
impl Stack {
    /// The fanned angle of the card `n` below the one the fan is under (degrees).
    pub fn angle(&self, n: usize) -> f32 { self.angle0 * self.falloff.powi(n as i32) }
    /// When card `n` below starts to move after the fan changes (ms).
    pub fn delay(&self, n: usize) -> f32 { self.delay_ms * n as f32 }
    /// The card the fan is under.
    pub fn under(&self) -> Option<usize> { self.held.or(self.hover) }
    /// Card `i`'s angle now (degrees); card 0 is on top.
    pub fn angle_of(&self, i: usize) -> f32 { self.angles.get(i).map_or(0.0, |a| a.value) }
    /// Card `i`'s angle when the fan has settled.
    pub fn target_of(&self, i: usize) -> f32 {
        match self.under() { Some(k) if i > k => self.angle(i - k - 1), _ => 0.0 }
    }
    pub fn moving(&self) -> bool { self.angles.iter().any(Animated::moving) || self.queue.iter().any(Option::is_some) }
    fn refan(&mut self, now: f32) {
        if self.angles.len() != self.count { self.angles = vec![Animated::new(0.0, self.spring); self.count]; self.queue = vec![None; self.count]; }
        let k = self.under();
        for i in 0..self.count {
            let target = self.target_of(i);
            let n = match k { Some(k) if i > k => i - k - 1, _ => 0 };
            self.queue[i] = Some((now + self.delay(n) / 1000.0, target));
        }
    }
    pub fn update(&mut self, m: StackMsg, now: f32) {
        let before = self.under();
        match m {
            StackMsg::Hover(h) => self.hover = h.filter(|&i| i < self.count),
            StackMsg::Press(p) => self.held = if p.is_some() && p == self.held { None } else { p.filter(|&i| i < self.count) },
            StackMsg::Key(k) => {
                let c = self.under().unwrap_or(0);
                match k {
                    Key::Down | Key::Right => self.held = Some((c + 1).min(self.count.saturating_sub(1))),
                    Key::Up | Key::Left => self.held = Some(c.saturating_sub(1)),
                    Key::Enter | Key::Char(' ') => self.held = if self.held.is_some() { None } else { Some(c) },
                    Key::Escape => self.held = None,
                    _ => {}
                }
            }
        }
        if self.under() != before { self.refan(now); }
    }
    /// Start what is due and spring to `now`; returns whether anything moved.
    pub fn step(&mut self, now: f32) -> bool {
        let mut moved = false;
        for i in 0..self.angles.len() {
            if let Some((at, target)) = self.queue[i] && now >= at {
                self.queue[i] = None;
                let v = self.angles[i].velocity;
                self.angles[i].spring = self.spring;
                self.angles[i].to(target, v, at);
                moved = true;
            }
            moved |= self.angles[i].step(now);
        }
        moved
    }
    /// The cards as a 3D scene: card 0 on top (nearest), each turned about its bottom-left corner.
    pub fn scene(&self) -> Scene3d {
        let colours = &self.paints;
        let (w, h) = self.card;
        let thick = self.look == StackLook::Thick;
        let depth = if thick { 0.028 } else { 0.004 };
        let items = (0..self.count).map(|i| {
            // The pivot: the bottom-left corner, a little left and down of the centre of the view.
            let pivot = Mat4::translate(-w * 0.4, -h * 0.7, -(i as f32) * (depth + 0.006));
            let t = pivot.mul(&Mat4::rotate_z(self.angle_of(i))).mul(&Mat4::translate(w * 0.5, h * 0.5, 0.0)).mul(&Mat4::scale(w, h, depth));
            let one = MeshInstances::one(MeshId::CUBE, t, colours[i % colours.len().max(1)], i as u32 + 1);
            match self.outlines.get(i.min(self.outlines.len().saturating_sub(1))) { Some(&p) => one.outlined(p), None => one }
        }).collect();
        let camera = if thick { Camera { projection: Projection::Perspective { fov: 34.0 }, yaw: -22.0, pitch: 24.0, distance: 2.3, target: [0.0, 0.0, -0.06], ..Camera::default() } }
            else { Camera { projection: Projection::Orthographic { height: 1.25 }, yaw: 0.0, pitch: 0.0, distance: 3.0, target: [0.0, 0.0, 0.0], ..Camera::default() } };
        // Supersampled twice: a turned card's outline is a smooth line, not steps of whole texels.
        let spec = Mesh3d { tones: 2.0, outline: if thick { 1.2 } else { self.outline }, resolution_scale: 2.0, ..Mesh3d::default() };
        Scene3d { items, camera, light: Light::from_angles(-30.0, 50.0, 0.55), spec, picture: None }
    }
}

/// The stack in a box `width` × `height`: hover and presses pick a card.
pub fn card_stack<M: Clone + 'static>(key: &str, s: &Stack, width: f32, height: f32, msg: impl Fn(StackMsg) -> M + 'static) -> El<M> {
    let msg = Rc::new(msg);
    let (a, b) = (msg.clone(), msg);
    let card = |h: Option<MeshHit>| h.map(|h| h.object.saturating_sub(1) as usize);
    let mut e = scene3d(s.scene()).key(key.to_owned()).width(width).height(height).focusable(true).role(Role::Group).label(crate::i18n::tr("widget.stack.label"))
        .on_pick(move |p| Some(a(match p { Pick::Hover(h) => StackMsg::Hover(card(h)), Pick::Press(h) => StackMsg::Press(card(h)) })))
        .on_key(move |k, _| matches!(k, Key::Up | Key::Down | Key::Left | Key::Right | Key::Enter | Key::Char(' ') | Key::Escape).then(|| b(StackMsg::Key(k))));
    // Marks: the material draws the cards as its marks at rest too (held loose, like a picture).
    if let StackLook::Marks(m) = s.look { e = e.material(m.id()).loose(1.0).legible(false); }
    // The cards swing inside one 3D element: a point at each card's far corner (where it is in the
    // view, as the straight-on camera shows it) stirs the field map as the card turns (DESIGN.md
    // §1.29). The stack is a stirring element itself, so its cards do not read their own trail.
    let (w, h) = s.card;
    let k = height / 1.25;
    for i in 0..s.count {
        let (sn, cs) = s.angle_of(i).to_radians().sin_cos();
        let (px, py) = (-w * 0.4, -h * 0.7);
        let (x, y) = (px + w * cs - h * sn, py + w * sn + h * cs);
        let at = (width * 0.5 + x * k, height * 0.5 - y * k);
        e = e.child(crate::el::space(0.0, 0.0).key(format!("{key}/corner/{i}")).absolute(Some(at.0), Some(at.1), None, None).stirr(crate::el::Stirr::default()));
    }
    e.stirr(crate::el::Stirr::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn settle(s: &mut Stack, mut t: f32) -> f32 { let end = t + 5.0; while s.moving() && t < end { t += 1.0 / 60.0; s.step(t); } t }

    #[test]
    fn the_fan_follows_the_formula_with_its_delays() {
        let mut s = Stack::default();
        s.update(StackMsg::Hover(Some(1)), 0.0);
        // Delays: card 2 (n 0) at once, card 3 after 28 ms, card 4 after 56 ms; card 0 and 1 stay.
        s.step(0.02);
        assert!(s.angle_of(2) > 0.0 && s.angle_of(3) == 0.0 && s.angle_of(4) == 0.0);
        s.step(0.04);
        assert!(s.angle_of(3) > 0.0 && s.angle_of(4) == 0.0);
        settle(&mut s, 0.04);
        assert_eq!((s.angle_of(0), s.angle_of(1)), (0.0, 0.0));
        for (i, n) in [(2, 0), (3, 1), (4, 2)] { assert!((s.angle_of(i) - 42.0 * 0.7f32.powi(n)).abs() < 1e-3, "card {i}: {}", s.angle_of(i)); }
        // Leaving closes it; a press holds it open.
        s.update(StackMsg::Press(Some(0)), 1.0);
        s.update(StackMsg::Hover(None), 1.0);
        settle(&mut s, 1.0);
        assert!((s.angle_of(1) - 42.0).abs() < 1e-3);
        s.update(StackMsg::Key(Key::Escape), 3.0);
        settle(&mut s, 3.0);
        assert!(s.angle_of(1) == 0.0 && !s.moving());
    }
    #[test]
    fn looks_follow_the_material() {
        assert_eq!(StackLook::of(Material::Plain), StackLook::Plain);
        assert_eq!(StackLook::of(Material::Dottr), StackLook::Marks(Material::Dottr));
        assert_eq!(StackLook::of(Material::Smudgr), StackLook::Thick);
    }
}
