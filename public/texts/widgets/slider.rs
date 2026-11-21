//! slider
//!
//! Purpose: Set a value roughly by dragging (the number control is the exact way).
//! Props: `key`, `value`, `spec`, `width`, `msg`.
//! States: rest, focused, dragging (captured until release; the thumb turns to the accent).
//! Messages: `msg(v)` while dragging and on Left / Right / Home / End.
//! Usage: `slider("level", s.level, SPEC, 240.0, Msg::Level)`
//!
//! A slider for a value in a range. Drag anywhere on it, or with focus use Left / Right (Shift for
//! ×10) and Home / End. It is the secondary way to set a value; `number` is the exact one.
use super::Spec;
use crate::el::{Drag, El, Key, Paint, Role, row, space};
use taffy::AlignItems;

const KNOB: f32 = 14.0;

/// The value for a drag position: the knob's centre follows the pointer, snapped to the step.
pub fn slider_value(d: &Drag, spec: Spec) -> f32 {
    let f = ((d.x - KNOB * 0.5) / (d.w - KNOB).max(1.0)).clamp(0.0, 1.0);
    let steps = ((spec.max - spec.min) * f / spec.step).round();
    spec.clamp(spec.min + steps * spec.step)
}

pub fn slider<M: Clone + 'static>(key: &str, value: f32, spec: Spec, width: f32, msg: impl Fn(f32) -> M + 'static) -> El<M> {
    // The body is built once per message type, not once for every closure an application passes.
    slider_in(key, value, spec, width, std::rc::Rc::new(msg))
}
fn slider_in<M: Clone + 'static>(key: &str, value: f32, spec: Spec, width: f32, msg: std::rc::Rc<dyn Fn(f32) -> M>) -> El<M> {
    let f = ((value - spec.min) / (spec.max - spec.min).max(1e-6)).clamp(0.0, 1.0);
    let run = (width - KNOB).max(0.0);
    let keys = {
        let msg = msg.clone();
        move |k: Key, shift: bool| -> Option<M> {
            let n = if shift { 10.0 } else { 1.0 };
            let v = match k {
                Key::Left | Key::Down => value - spec.step * n,
                Key::Right | Key::Up => value + spec.step * n,
                Key::Home => spec.min,
                Key::End => spec.max,
                _ => return None,
            };
            let v = spec.clamp(v);
            (v != value).then(|| msg(v))
        }
    };
    let drag = move |d: Drag| Some(msg(slider_value(&d, spec)));
    row().key(key).width(width).height(24.0).align(AlignItems::CENTER).on_key(keys).on_drag_with(drag).role(Role::Slider).value_range(value, spec.min, spec.max)
        // The filled run and the thumb in ink, the rest in the hairline grey; the thumb turns to the
        // accent while it is dragged.
        .child(space(run * f, 2.0).fill(Paint::INK))
        .child(space(KNOB, KNOB).radius(KNOB * 0.5).fill(Paint::INK).fill_held(Paint::ACCENT))
        .child(space(0.0, 2.0).grow(1.0).fill(Paint::HAIRLINE))
}
