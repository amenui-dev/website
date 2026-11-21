//! orbit: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::El;
use amenui::mesh::{MeshId, Scene3d};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Pick(usize), Edit(TextEdit), Num(Edit), Value(f32) }

/// 一个立方体 · A cube
pub fn ex_0() -> El<Msg> {
    orbit("o1", Scene3d::single(MeshId::CUBE), &Orbit::default(), |_| Msg::Nothing).width(240.0).height(160.0)
}
