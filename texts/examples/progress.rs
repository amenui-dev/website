//! progress: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 四成 · Forty per cent
pub fn ex_0() -> El<Msg> {
    progress("p1", Some(0.4), 240.0)
}

/// 进行中 · Going on
pub fn ex_1() -> El<Msg> {
    progress("p2", None, 240.0)
}
