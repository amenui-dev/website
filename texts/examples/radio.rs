//! radio: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 选中第二个 · The second chosen
pub fn ex_0() -> El<Msg> {
    radio_group("r1", &["Small", "Medium", "Large"], 1, Msg::Pick)
}
