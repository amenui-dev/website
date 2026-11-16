//! The Body scene's page (`#body`, the landing page and the first scene; DESIGN.md §6.2): the example
//! in `examples/body.rs` (`recipes::scenes::body`) filling the whole window, full-bleed, in its own
//! look. Its markings ask the host for the language and for the Lab or the components.
use super::{Gallery, Msg, Page};
use crate::el::El;
use crate::experiment::Experiment;
use crate::recipes::scenes::body::{self as scene, Ask, Body};
use crate::style::Look;
use std::rc::Rc;

pub fn update(s: &mut Gallery, m: scene::Msg) {
    let now = s.clock;
    s.body.update(m, now);
    match s.body.asked.take() {
        Some(Ask::Language) => { let l = if s.locale == crate::i18n::Locale::En { crate::i18n::Locale::ZhHans } else { crate::i18n::Locale::En }; super::update(s, Msg::Locale(l)); }
        Some(Ask::Go("lab")) => super::update(s, Msg::Go(Page::Lab)),
        Some(Ask::Go(_)) => super::update(s, Msg::Go(Page::Comp(0))),
        None => {}
    }
}
pub fn tick(s: &mut Gallery, t: f32) -> bool { s.page == Page::Body && s.body.step(t) }
pub fn wake(s: &Gallery, t: f32) -> Option<f32> { (s.page == Page::Body && s.body.moving()).then_some(t) }
/// The scene's look on its page: its form's palette on a dark ground, a bloom, a vignette.
pub fn look(s: &Gallery, l: &mut Look) {
    if s.page != Page::Body { return; }
    let b = scene::look(s.body.form);
    l.theme = b.theme;
    l.material = b.material;
    l.post = b.post;
}
pub fn page(s: &Gallery) -> El<Msg> { s.body.view_in("body").map(Rc::new(Msg::Body)) }
/// A fresh scene, sized to the window, arriving now (the page is shown, then gathers).
pub fn fresh(s: &Gallery) -> Body {
    let mut b = Body::default();
    b.reduced = s.lab.reduced;
    b.resize(s.width, s.height);
    b.entered = s.clock;
    b.t = s.clock;
    b
}
/// Keys on the Body page that no control used: Space and Enter gather or release, ← → turn it.
pub fn key(s: &mut Gallery, k: crate::el::Key) -> bool {
    if s.page != Page::Body { return false; }
    use crate::el::Key;
    if !matches!(k, Key::Char(' ') | Key::Enter | Key::Left | Key::Right) { return false; }
    let now = s.clock;
    s.body.update(scene::Msg::Key(k), now);
    true
}
/// The pointer anywhere over the window (logical px): the scene's parallax follows it.
pub fn pointer(s: &mut Gallery, x: f32, y: f32) -> bool {
    if s.page != Page::Body { return false; }
    s.body.look_at(x / s.width.max(1.0) - 0.5, y / s.height.max(1.0) - 0.5);
    false
}
