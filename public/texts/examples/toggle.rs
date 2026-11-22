//! toggle: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 一个 · One
pub fn ex_0() -> El<Msg> {
    toggle("tg1", "B", true, Msg::Nothing)
}

/// 一组 · A group
pub fn ex_1() -> El<Msg> {
    toggle_group("tg2", &["Left", "Center", "Right"], &[true, false, false], Msg::Pick)
}
