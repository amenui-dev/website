//! number
//!
//! Purpose: Set a number exactly.
//! Props: `key`, `label`, `value`, `spec: Spec` (min, max, step, decimals, unit), `draft` (what is
//!   being typed), `msg`.
//! States: rest, focused, typing (ink border, accent caret), disabled; a new value recomposes its
//!   digits (`{key}/shown`).
//! Messages: `msg(Edit::Draft(s))` while typing, `Edit::Set(v)` on Enter / steps / arrows / scrub,
//!   `Edit::Cancel` on Esc or blur.
//! Usage: `number("copies", "Copies", s.copies, SPEC, s.draft.as_deref(), Msg::Copies)`
//!
//! A numeric control for setting a value exactly: step buttons on either side, and once focused the
//! value can be typed (digits, `.`, `-`, Backspace; Enter commits, Esc cancels) or stepped with
//! Up / Down (Shift for ×10). Dragging sideways on the value scrubs it, one step per `SCRUB` px; a
//! press that does not move only focuses it, so typing stays the main way.
//!
//! It is never narrower than its steps and field: given a width that leaves its label no room beside
//! them, the steps go on a line under the label, and a label wider than the control ends in an
//! ellipsis (read whole by assistive technology).
use super::{CONTROL_H, LABEL, LINE, RADIUS};
use crate::el::{Drag, El, Font, Key, Paint, Phase, row, space, text};
use std::rc::Rc;
use taffy::AlignItems;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spec { pub min: f32, pub max: f32, pub step: f32, pub decimals: usize, pub unit: &'static str }
impl Spec {
    /// Into range, rounded to the shown precision, so what is stored is what is displayed.
    pub fn clamp(&self, v: f32) -> f32 {
        let k = 10f32.powi(self.decimals as i32);
        (v.clamp(self.min, self.max) * k).round() / k
    }
    pub fn show(&self, v: f32) -> String { format!("{:.*}", self.decimals, v) }
}

/// Logical px of sideways drag per step.
pub const SCRUB: f32 = 6.0;
/// The steps and the field with their gaps: the least a number control is wide.
const CONTROLS: f32 = 2.0 * CONTROL_H + 96.0 + 2.0 * 6.0;

/// What the control asks the application to do with its value.
#[derive(Clone, Debug, PartialEq)]
pub enum Edit { Draft(String), Set(f32), Cancel }

/// `draft` is the text being typed, if any; the application keeps it in its state.
pub fn number<M: Clone + 'static>(key: &str, label: &str, value: f32, spec: Spec, draft: Option<&str>, msg: impl Fn(Edit) -> M + 'static) -> El<M> {
    // The control is built once, sending `Edit`, and mapped to `M`: only the mapping is built per message type.
    control(key, label, value, spec, draft).map(Rc::new(msg))
}
/// The control, sending what it asks for as it is.
fn control(key: &str, label: &str, value: f32, spec: Spec, draft: Option<&str>) -> El<Edit> {
    let draft: Option<String> = draft.map(str::to_owned);
    let editing = draft.is_some();
    let step = |dir: f32, glyph: &str, k: &str| row().key(format!("{key}/{k}")).center().width(CONTROL_H).height(CONTROL_H).radius(RADIUS)
        .fill(Paint::GROUND).border(LINE, Paint::FAINT).on_press(Edit::Set(spec.clamp(value + dir * spec.step))).label(crate::i18n::tr(&format!("widget.number.{k}")))
        // The mono face: the pixel CJK face has no minus sign, and the steps then match in every theme.
        .child(text(glyph, LABEL + 2.0, Paint::TEXT).font(Font::Mono));
    let handler = {
        let draft = draft.clone();
        move |k: Key, shift: bool| -> Option<Edit> {
            let e = match k {
                Key::Char(c) if c.is_ascii_digit() || c == '.' || c == '-' => {
                    let mut s = draft.clone().unwrap_or_default();
                    s.push(c);
                    Edit::Draft(s)
                }
                Key::Backspace => {
                    let mut s = draft.clone().unwrap_or_else(|| spec.show(value));
                    s.pop();
                    Edit::Draft(s)
                }
                Key::Enter => match draft.as_deref()?.parse::<f32>() { Ok(v) => Edit::Set(spec.clamp(v)), Err(_) => Edit::Cancel },
                Key::Escape => { draft.as_ref()?; Edit::Cancel }
                Key::Up | Key::Down => {
                    let n = if shift { 10.0 } else { 1.0 } * if k == Key::Up { 1.0 } else { -1.0 };
                    Edit::Set(spec.clamp(value + n * spec.step))
                }
                _ => return None,
            };
            Some(e)
        }
    };
    // The handler is captured at the press, so `value` here is the value the scrub started from.
    let scrub = {
        let moved = std::cell::Cell::new(false);
        move |d: Drag| -> Option<Edit> {
            if d.phase == Phase::Start { return None; }
            moved.set(moved.get() || d.dx.abs() >= SCRUB * 0.5);
            if !moved.get() { return None; }
            Some(Edit::Set(spec.clamp(value + (d.dx / SCRUB).round() * spec.step)))
        }
    };
    let shown = draft.clone().unwrap_or_else(|| spec.show(value));
    let mut field = row().key(format!("{key}/value")).height(CONTROL_H).width(96.0).pad(10.0, 0.0).gap(4.0).radius(RADIUS)
        .align(AlignItems::CENTER).fill(Paint::GROUND)
        .border(LINE, if editing { Paint::INK } else { Paint::DIM }).border_hover(Paint::INK)
        .on_key(handler)
        .on_drag_with(scrub)
        // A new value recomposes the changed digits (DESIGN.md §1.16); digits being typed appear at once.
        .child({
            let t = text(shown, LABEL, Paint::INK).key(format!("{key}/shown"));
            // Typing shows digits at once; otherwise it follows its subtree (on, unless turned off around it).
            if editing { t.recompose(false) } else { t }
        });
    if editing {
        field = field.child(space(1.5, 16.0).fill(Paint::ACCENT)).on_blur(Edit::Cancel);
    }
    field = field.child(space(0.0, 0.0).grow(1.0)).child(text(spec.unit, LABEL - 2.0, Paint::DIM));
    // The steps and the field never shrink, and the control is never narrower than they are: a width
    // that does not leave the label room puts the steps on a line of their own under it, and a label
    // wider than the whole control ends in an ellipsis (read whole).
    let controls = row().gap(6.0).align(AlignItems::CENTER)
        .child(step(-1.0, "−", "less"))
        .child(field)
        .child(step(1.0, "+", "more"));
    // As wide as it is (not grown from nothing), so a label too wide beside the steps wraps them under it;
    // never wider than the control.
    let mut label = text(label, LABEL, Paint::DIM).grow(1.0).min_width(0.0).clamp(1);
    label.style.flex_basis = taffy::prelude::auto();
    label.style.max_size.width = taffy::prelude::percent(1.0);
    row().key(key).wrap().gap(6.0).align(AlignItems::CENTER).min_width(CONTROLS)
        .child(label)
        .child(controls)
}
