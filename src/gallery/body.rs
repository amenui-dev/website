//! The Body scene's page (`#body`, the first scene; DESIGN.md §6.2): the example in
//! `examples/body.rs` (`recipes::scenes::body`) filling the window under the top bar, in its own look.
use super::{Gallery, Msg, Page};
use crate::el::El;
use crate::experiment::Experiment;
use crate::recipes::scenes::body::{self as scene, Body};
use crate::style::Look;
use std::rc::Rc;

pub fn update(s: &mut Gallery, m: scene::Msg) { let now = s.clock; s.body.update(m, now); }
pub fn tick(s: &mut Gallery, t: f32) -> bool { s.page == Page::Body && s.body.step(t) }
pub fn wake(s: &Gallery, t: f32) -> Option<f32> { (s.page == Page::Body && s.body.moving()).then_some(t) }
/// The scene's look on its page: the lab's dark ground and one hue, a soft bloom.
pub fn look(s: &Gallery, l: &mut Look) {
    if s.page != Page::Body { return; }
    let b = scene::look();
    l.theme = b.theme;
    l.material = b.material;
    l.post = b.post;
}
pub fn page(s: &Gallery) -> El<Msg> { s.body.view("body").map(Rc::new(Msg::Body)) }
/// A fresh scene state (the page is opened on the text).
pub fn fresh(reduced: bool) -> Body { let mut b = Body::default(); b.reduced = reduced; b }
