//! switch: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 关 · Off
pub fn ex_0() -> El<Msg> {
    switch("s1", "Notifications", false, None, Msg::Nothing).width(220.0)
}

/// 开 · On
pub fn ex_1() -> El<Msg> {
    switch("s2", "Notifications", true, None, Msg::Nothing).width(220.0)
}
