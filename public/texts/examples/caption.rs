//! caption: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El, Paint, space};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Num(usize, Edit) }

/// 演示下面的公式注 · A formula caption under a demo
pub fn ex_0() -> El<Msg> {
    let (w, r) = (120.0_f32, 8.0_f32);
    let spec = Spec { min: 0.0, max: 400.0, step: 1.0, decimals: 0, unit: "px" };
    let params = [CaptionParam::new("width", w, spec), CaptionParam::new("radius", r, spec)];
    space(w, 40.0).radius(r).fill(Paint::RAISED).border(1.0, Paint::HAIRLINE).key("cap1/demo")
        .caption_params("cap1", format!("rect(w={w:.0}, r={r:.0})"), &params, Msg::Num)
}
