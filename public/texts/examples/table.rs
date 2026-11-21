//! table: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 两行 · Two rows
pub fn ex_0() -> El<Msg> {
    table("tb1", &[("Name", 120.0), ("Size", 60.0)], &[vec!["a.png".into(), "12 KB".into()], vec!["b.png".into(), "3 KB".into()]], Some(0), Msg::Pick)
}
