//! accordion: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El, Paint, text};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 一节打开 · One open
pub fn ex_0() -> El<Msg> {
    accordion("a1", vec![("What is a material", text("How the second pass draws.", 13.0, Paint::TEXT)), ("What is a preset", text("A set of numbers.", 13.0, Paint::TEXT))], &[true, false], Msg::Pick).width(320.0)
}
