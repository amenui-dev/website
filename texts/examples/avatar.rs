//! avatar: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El, row};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 首字母 · Initials
pub fn ex_0() -> El<Msg> {
    row().gap(8.0).child(avatar("av1", "Lin Mo", 36.0, 0)).child(avatar("av2", "Grace Hopper", 36.0, 1)).child(avatar("av3", "Ada", 36.0, 2))
}
