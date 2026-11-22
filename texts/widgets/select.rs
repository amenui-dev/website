//! select
//!
//! Purpose: Choose one option from a list that opens on demand (a combobox with a listbox).
//! Props: `key`, `options`, `current`, `open`, `active` (the option the keyboard is on while open),
//!   `toggle`, `close`, `pick`, `move_to` (the keyboard moves the active option).
//! States: closed (shows the choice), open (a listbox in a layer below), an active option.
//! Messages: `toggle` on press, Enter, Space or Down (closed); while open Up / Down / Home / End send
//!   `move_to(i)`, Enter / Space `pick(active)`, Esc or a press outside `close` (WAI-ARIA select-only
//!   combobox).
//! Usage: `select("city", &cities, s.city, s.open, s.active, Msg::Toggle, Msg::Close, Msg::City, Msg::Active)`
//!
//! The button shows the current option; the list opens in a layer right below it. The application
//! keeps whether it is open and which option is active.
use super::{CONTROL_H, LABEL, LINE, MENU_SURFACE, RADIUS, selected_on};
use crate::el::{El, Key, Paint, Role, col, row, space, text};
use std::rc::Rc;
use taffy::AlignItems;

#[allow(clippy::too_many_arguments)]
pub fn select<M: Clone + 'static>(key: &str, options: &[&str], current: Option<usize>, open: bool, active: usize, toggle: M, close: M, pick: impl Fn(usize) -> M + 'static, move_to: impl Fn(usize) -> M + 'static) -> El<M> {
    let pick: Rc<dyn Fn(usize) -> M> = Rc::new(pick);
    let n = options.len();
    let shown = current.and_then(|i| options.get(i)).copied().unwrap_or("—");
    let keys = { let (pick, toggle) = (pick.clone(), toggle.clone()); Rc::new(move |k: Key, _: bool| -> Option<M> {
        if !open { return matches!(k, Key::Enter | Key::Char(' ') | Key::Down).then(|| toggle.clone()); }
        match k {
            Key::Down => Some(move_to((active + 1).min(n.saturating_sub(1)))), Key::Up => Some(move_to(active.saturating_sub(1))),
            Key::Home => Some(move_to(0)), Key::End => Some(move_to(n.saturating_sub(1))),
            Key::Enter | Key::Char(' ') => Some(pick(active)), _ => None,
        }
    }) };
    let mut r = col().key(format!("{key}/select"))
        .child(row().key(key).height(CONTROL_H).min_width(160.0).pad(10.0, 0.0).gap(8.0).radius(RADIUS).align(AlignItems::CENTER).focusable(true)
            .fill(Paint::GROUND).border(LINE, if open { Paint::INK } else { Paint::DIM }).border_hover(Paint::INK).on_press(toggle).on_key({ let k = keys.clone(); move |a, b| k(a, b) })
            .role(Role::Combobox).expanded(open).text_value(shown)
            .child(text(shown, LABEL, Paint::TEXT).grow(1.0))
            .child(col().gap(1.0).align(AlignItems::CENTER).child(space(7.0, 1.5).fill(Paint::DIM)).child(space(4.0, 1.5).fill(Paint::DIM)).child(space(1.5, 1.5).fill(Paint::DIM))));
    if open {
        let mut list = col().key(format!("{key}/list")).anchor(key).min_width(160.0).pad(4.0, 4.0).gap(1.0).radius(RADIUS + 2.0).fill(MENU_SURFACE).border(LINE, Paint::HAIRLINE).role(Role::Listbox)
            // Open, the keyboard is in the list (the layer takes the focus): the same keys work there.
            .focusable(true).on_key(move |a, b| keys(a, b));
        for (i, o) in options.iter().enumerate() {
            let (on, act) = (Some(i) == current, i == active);
            let mut item = row().key(format!("{key}:{i}")).height(CONTROL_H - 2.0).pad(10.0, 0.0).radius(RADIUS).align(AlignItems::CENTER).on_press(pick(i)).focusable(false)
                .role(Role::Option).selected(on).child(text(*o, LABEL, if on { Paint::INK } else { Paint::TEXT }));
            if act { item = item.fill(selected_on(MENU_SURFACE)); }
            list = list.child(item);
        }
        r = r.child(col().key(format!("{key}/layer")).modal(close.clone(), 0.0).on_press(close).focusable(false).child(list));
    }
    r
}
