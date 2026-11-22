//! card_stack: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::El;
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Stack(StackMsg) }

/// 合着的一叠 · A closed stack
pub fn ex_0() -> El<Msg> {
    card_stack("cs1", &Stack::default(), 260.0, 170.0, Msg::Stack)
}
