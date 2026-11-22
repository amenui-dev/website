//! skeleton: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El, col, row};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 一张卡片 · A card
pub fn ex_0() -> El<Msg> {
    row().gap(12.0).child(skeleton("sk1", 40.0, 40.0, true)).child(col().gap(8.0).child(skeleton("sk2", 180.0, 12.0, false)).child(skeleton("sk3", 120.0, 12.0, false)))
}
