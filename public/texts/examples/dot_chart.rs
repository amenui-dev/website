//! dot_chart: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::El;
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Chart(ChartMsg) }

/// 一片点 · A field of dots
pub fn ex_0() -> El<Msg> {
    let mut c = DotChart::default();
    (c.cols, c.rows, c.pitch) = (28, 10, 7.0);
    dot_chart("dc1", &c, Msg::Chart)
}
