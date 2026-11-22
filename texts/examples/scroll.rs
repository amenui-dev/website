//! scroll: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El, Paint, col, text};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 滚动区 · Scroll area
pub fn ex_0() -> El<Msg> {
    scroll_area("sa1", 120.0, col().gap(6.0).children((0..12).map(|i| text(format!("Line {}", i + 1), 13.0, Paint::TEXT))))
}
