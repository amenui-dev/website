//! floating: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El, Paint, layers, text};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Float(FloatMsg) }

/// 停在右下角 · Resting in the bottom-right corner
pub fn ex_0() -> El<Msg> {
    let mut f = Floating::default();
    f.area = (300.0, 180.0);
    f.size = (120.0, 72.0);
    f.place(3);
    layers([floating("fl1", &f, text("00:42", 12.0, Paint::TEXT).key("fl1/t"), Msg::Float)])
        .width(300.0).height(180.0).radius(8.0).border(1.0, Paint::FAINT).clip()
}
