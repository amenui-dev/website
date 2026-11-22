//! divider: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El, Paint, col, text};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 横线 · Across
pub fn ex_0() -> El<Msg> {
    col().width(240.0).gap(8.0).child(text("Above", 13.0, Paint::TEXT)).child(divider(1.0)).child(text("Below", 13.0, Paint::TEXT))
}
