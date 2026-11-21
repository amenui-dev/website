//! slider: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 中间 · Middle
pub fn ex_0() -> El<Msg> {
    slider("sl1", 40.0, Spec { min: 0.0, max: 100.0, step: 1.0, decimals: 0, unit: "" }, 240.0, Msg::Value)
}
