//! number: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 整数 · Whole
pub fn ex_0() -> El<Msg> {
    number("n1", "Copies", 3.0, Spec { min: 0.0, max: 9.0, step: 1.0, decimals: 0, unit: "" }, None, Msg::Num).width(320.0)
}

/// 带单位 · With a unit
pub fn ex_1() -> El<Msg> {
    number("n2", "Width", 12.5, Spec { min: 0.0, max: 64.0, step: 0.5, decimals: 1, unit: "px" }, None, Msg::Num).width(320.0)
}
