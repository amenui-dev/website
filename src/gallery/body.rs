//! The home page's canvas: the Body scene (`examples/body.rs`) filling the whole window, full-bleed, in
//! its own look. The headline, the actions, the navigation and the rail row are the web page's HTML
//! over it; the host tells the scene where its stage is (`Body::set_stage`) and which state to open
//! (`Body::open`).
use super::{Gallery, Msg, Page};
use amenui::el::El;
use amenui::experiment::Experiment;
use crate::showcase::scenes::body::{self as scene, Body};
use amenui::style::Look;
use std::rc::Rc;

pub fn update(s: &mut Gallery, m: scene::Msg) {
    let now = s.clock;
    s.body.update(m, now);
}
/// The scene's clock; the front across its listing, when it asks for one, goes out from where it says.
pub fn tick(s: &mut Gallery, t: f32) -> bool {
    if s.page != Page::Body { return false; }
    let mut changed = s.body.step(t);
    if let Some((x, y)) = s.body.take_front() { s.cmds.push(amenui::morph::Cmd::Front { x, y, strength: 1.0 }); changed = true; }
    // The scene's morphs, and the walk's focus and presses.
    if !s.body.cmds.is_empty() { s.cmds.append(&mut s.body.cmds); changed = true; }
    changed
}
pub fn wake(s: &Gallery, t: f32) -> Option<f32> { (s.page == Page::Body && s.body.moving()).then_some(t) }
/// The scene's look on its page: its form's palette on a dark ground, a bloom, a vignette.
pub fn look(s: &Gallery, l: &mut Look) {
    if s.page != Page::Body { return; }
    let b = scene::look(s.body.form, s.body.gather.value);
    l.theme = b.theme;
    l.material = b.material;
    l.post = b.post;
    if s.body.fronting() { l.fx.wave_speed = scene::FRONT_SPEED; }
    // The specimen's lattice pitch (the morphs' cells too).
    l.fx.pitch = s.body.spec.pitch;
}
/// A person acted: the walk stops where it is.
pub fn heard(s: &mut Gallery) -> bool { s.page == Page::Body && s.body.heard() }
pub fn page(s: &Gallery) -> El<Msg> { s.body.view_in("body").map(Rc::new(Msg::Body)) }
/// A fresh scene, sized to the window, in the stage the host gave the last one.
pub fn fresh(s: &Gallery) -> Body {
    let mut b = Body::default();
    (b.reduced, b.stage, b.text_centre, b.form, b.seed) = (s.lab.reduced, s.body.stage, s.body.text_centre, s.body.form, s.body.seed);
    b.resize(s.width, s.height);
    b.entered = s.clock;
    b.t = s.clock;
    b
}
/// Keys on the Body page that no control used: 1 and 4 the states, Space and Enter between them, F the
/// form, [ and ] the seed, ← → a turn.
pub fn key(s: &mut Gallery, k: amenui::el::Key) -> bool {
    if s.page != Page::Body { return false; }
    if !Body::takes(k) { return false; }
    let now = s.clock;
    s.body.update(scene::Msg::Key(k), now);
    true
}
