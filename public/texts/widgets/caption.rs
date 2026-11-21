//! caption
//!
//! Purpose: The formula caption under a live demo: one small monospaced line that prints the actual
//!   numbers driving it, and those numbers as small number controls to change them.
//! Props: `key`, `formula` (the line, written from the same fields the demo reads), `params`
//!   (`CaptionParam`: name, value, spec, draft), `msg`.
//! States: rest; a number being typed (see `number`); a changed value recomposes its digits.
//! Messages: `msg(i, Edit)` from parameter `i`'s control.
//! Usage: `caption("sheet/cap", format!("k={k}"), &params, |i, e| Msg::Num(i, e))`, or
//!   `demo.caption_params(...)` to put it under the demo.
//!
//! The rule (DESIGN.md §5.2): the caption is never typed by hand. Its line is formatted from the fields
//! the demo uses, and its controls set those same fields, so what it says is what drives the demo.
use super::{Edit, Spec, number};
use crate::el::{El, Font, Paint, col, row, text};
use std::rc::Rc;

/// One number in a caption: its name in the formula, its value now, its range and what is being typed.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptionParam { pub name: String, pub value: f32, pub spec: Spec, pub draft: Option<String> }
impl CaptionParam {
    pub fn new(name: impl Into<String>, value: f32, spec: Spec) -> Self { Self { name: name.into(), value, spec, draft: None } }
    pub fn draft(mut self, d: Option<&str>) -> Self { self.draft = d.map(str::to_owned); self }
}

/// The caption: the formula line, then one small number control per parameter.
pub fn caption<M: Clone + 'static>(key: &str, formula: impl Into<String>, params: &[CaptionParam], msg: impl Fn(usize, Edit) -> M + 'static) -> El<M> {
    // The caption is built once, sending `(i, edit)`, and mapped to `M`: only the mapping is built per message type.
    let msg = Rc::new(msg);
    lines(key, formula.into(), params).map(Rc::new(move |(i, e)| msg(i, e)))
}
/// The caption, sending `(i, edit)` for parameter `i`.
fn lines(key: &str, formula: String, params: &[CaptionParam]) -> El<(usize, Edit)> {
    let mut nums = row().key(format!("{key}/nums")).wrap().gap(6.0);
    for (i, p) in params.iter().enumerate() {
        nums = nums.child(number(&format!("{key}/p{i}"), &p.name, p.value, p.spec, p.draft.as_deref(), move |e| (i, e)).width(208.0).zoom(0.8));
    }
    col().key(key.to_owned()).gap(6.0).fill_width().min_width(0.0)
        .child(text(formula, 11.5, Paint::DIM).font(Font::Mono).key(format!("{key}/formula")).recompose(false))
        .child(nums)
}
