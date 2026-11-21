//! dialog: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El, Paint, col, text};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 关着，一个打开它的按钮 · Closed, with a button that opens it
pub fn ex_0() -> El<Msg> {
    col().gap(8.0).child(button("rename", "Rename…", Msg::Nothing)).child(dialog("d1", false, "Rename", 320.0, 0.35, Msg::Nothing, text("A new name", 14.0, Paint::TEXT)))
}
