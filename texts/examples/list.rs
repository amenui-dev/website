//! list: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 带选中 · With a selection
pub fn ex_0() -> El<Msg> {
    list("l1", &[("Apples", "3"), ("Pears", "5"), ("Peaches", "1")], Some(1), 36.0, 140.0, Msg::Pick)
}
