//! button: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 默认 · Default
pub fn ex_0() -> El<Msg> {
    button_as("b1", "Save", Variant::Default, Msg::Nothing)
}

/// 安静 · Quiet
pub fn ex_1() -> El<Msg> {
    button_as("b2", "Cancel", Variant::Quiet, Msg::Nothing)
}

/// 强调 · Accent
pub fn ex_2() -> El<Msg> {
    button_as("b3", "Publish", Variant::Accent, Msg::Nothing)
}

/// 不可用 · Disabled
pub fn ex_3() -> El<Msg> {
    button_as("b4", "Publish", Variant::Primary, Msg::Nothing).disabled(true)
}

/// 主要 · Primary
pub fn ex_4() -> El<Msg> {
    button_as("b5", "Save", Variant::Primary, Msg::Nothing)
}
