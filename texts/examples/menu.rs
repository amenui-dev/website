//! menu: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::El;
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 关着 · Closed
pub fn ex_0() -> El<Msg> {
    menu("m1", "Theme", &["Dark", "Light", "Paper"], Some(0), false, false, Msg::Nothing, Msg::Nothing, Msg::Pick)
}
