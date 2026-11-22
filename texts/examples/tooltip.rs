//! tooltip: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 显示 · Shown
pub fn ex_0() -> El<Msg> {
    tooltip("tt1", button("tb", "Save", Msg::Nothing), "Save to this computer", true)
}
