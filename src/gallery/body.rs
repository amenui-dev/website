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
pub fn tick(s: &mut Gallery, t: f32) -> bool { s.page == Page::Body && s.body.step(t) }
pub fn wake(s: &Gallery, t: f32) -> Option<f32> { (s.page == Page::Body && s.body.moving()).then_some(t) }
/// The scene's look on its page: its form's palette on a dark ground, a bloom, a vignette.
pub fn look(s: &Gallery, l: &mut Look) {
    if s.page != Page::Body { return; }
    let b = scene::look(s.body.form, s.body.gather.value);
    l.theme = b.theme;
    l.material = b.material;
    l.post = b.post;
}
pub fn page(s: &Gallery) -> El<Msg> { s.body.view_in("body").map(Rc::new(Msg::Body)) }
/// A fresh scene, sized to the window, in the stage the host gave the last one.
pub fn fresh(s: &Gallery) -> Body {
    let mut b = Body::default();
    (b.reduced, b.stage, b.form, b.seed) = (s.lab.reduced, s.body.stage, s.body.form, s.body.seed);
    // With reduced motion the body starts still (no idle turn to ease out of).
    if b.reduced { b.spin = 0.0; }
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
