//! pagination: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 第三页 · Page three
pub fn ex_0() -> El<Msg> {
    pagination("pg1", 2, 12, Msg::Pick)
}
