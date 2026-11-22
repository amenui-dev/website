//! image: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 带说明 · With a caption
pub fn ex_0() -> El<Msg> {
    image("im1", amenui::pictures::SKY, 0.6, 0, Some("Sky"))
}
