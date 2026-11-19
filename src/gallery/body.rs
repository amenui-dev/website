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
    if let Some((x, y)) = s.body.take_front() { s.cmds.push(amenui::morph::Cmd::Ripplr { x, y, strength: 1.0 }); changed = true; }
    // The scene's morphs, and the walk's focus and presses.
    if !s.body.cmds.is_empty() { s.cmds.append(&mut s.body.cmds); changed = true; }
    changed
}
pub fn wake(s: &Gallery, t: f32) -> Option<f32> { if s.page == Page::Body { s.body.wake(t) } else { None } }
/// The scene's look on its page: the palette's flat ground, a bloom.
pub fn look(s: &Gallery, l: &mut Look) {
    if s.page != Page::Body { return; }
    let b = scene::look(s.body.palette, s.body.gather.value);
    l.theme = b.theme;
    l.material = b.material;
    l.post = b.post;
    // The full readout is a plate inside the stage (AA.6): its right edge on the stage's, its foot 8 px
    // above the caption, clear of the console above it and of the rail and the status line under it.
    let st = s.body.stage();
    l.overlay_at = amenui::style::Corner::BottomRight;
    l.overlay_inset = [(s.width - st[2]).max(0.0), (s.height - s.body.caption_y() + 8.0).max(0.0)];
    // The word's lattice (the morphs' cells too), the fronts' speed and the marks' colour.
    s.body.tune(l);
}
/// A person acted: the walk stops where it is.
pub fn heard(s: &mut Gallery) -> bool { s.page == Page::Body && s.body.heard() }
pub fn page(s: &Gallery) -> El<Msg> { s.body.view_in("body").map(Rc::new(Msg::Body)) }
/// A fresh scene, sized to the window, in the stage the host gave the last one.
pub fn fresh(s: &Gallery) -> Body {
    let mut b = Body::default();
    (b.reduced, b.stage, b.text_centre, b.seed, b.palette, b.away) = (s.lab.reduced, s.body.stage, s.body.text_centre, s.body.seed, s.palette, s.body.away);
    b.resize(s.width, s.height);
    b.entered = s.clock;
    b.t = s.clock;
    b
}
/// Keys on the Body page that no control used: 1 to 4 the states, Space and Enter between 02 and 04,
/// [ and ] the seed, ← → a turn, 0 the opening view.
pub fn key(s: &mut Gallery, k: amenui::el::Key) -> bool {
    if s.page != Page::Body { return false; }
    if !Body::takes(k) { return false; }
    let now = s.clock;
    s.body.update(scene::Msg::Key(k), now);
    true
}
