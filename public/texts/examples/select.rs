//! select: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 关着 · Closed
pub fn ex_0() -> El<Msg> {
    select("se1", &["Beijing", "Shanghai", "Guangzhou"], Some(0), false, 0, Msg::Nothing, Msg::Nothing, Msg::Pick, Msg::Pick)
}
