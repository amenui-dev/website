//! menu
//!
//! Purpose: A drop-down of options.
//! Props: `key`, `label`, `options`, `current`, `open`, `commit` (the choice is a commit: theme,
//!   material), `toggle`, `close`, `pick`.
//! States: closed (shows the current option) or open (a layer under the button).
//! Messages: `toggle` on the button, `pick(i)` on an option, `close` outside or on Esc.
//! Usage: `menu("theme", "Theme", &names, Some(s.theme), s.menu, true, Msg::Menu, Msg::Close,
//!   Msg::Theme)`
//!
//! A drop-down menu: a button that shows the current option; pressed, a list of the options opens
//! just below it in a layer over everything else. Pressing an option sends `pick(i)`; pressing
//! outside the list or Esc sends `close`. The application keeps whether it is open.
use super::{CONTROL_H, LABEL, LINE, MENU_SURFACE, RADIUS, SELECTED, selected_on};
use crate::el::{El, Paint, col, row, space, text};
use taffy::AlignItems;

/// Options are keyed `key:i`; the button is `key`. `commit` makes each option a commit (a theme or
/// material switch that spreads from where it was pressed).
#[allow(clippy::too_many_arguments)]
pub fn menu<M: Clone>(key: &str, label: &str, options: &[&str], current: Option<usize>, open: bool, commit: bool, toggle: M, close: M, pick: impl Fn(usize) -> M) -> El<M> {
    let shown = current.and_then(|i| options.get(i)).copied().unwrap_or("—");
    let button = row().key(key).height(CONTROL_H).pad(10.0, 0.0).gap(8.0).radius(RADIUS).align(AlignItems::CENTER)
        .fill(if open { SELECTED } else { Paint::GROUND }).border(LINE, if open { Paint::INK } else { Paint::DIM }).border_hover(Paint::INK)
        .on_press(toggle)
        .child(text(shown, LABEL - 1.0, Paint::TEXT))
        // A small chevron drawn as two strokes would need rotation; a narrow triangle of bars reads the same.
        .child(col().gap(1.0).align(AlignItems::CENTER)
            .child(space(7.0, 1.5).fill(Paint::DIM)).child(space(4.0, 1.5).fill(Paint::DIM)).child(space(1.5, 1.5).fill(Paint::DIM)));
    let mut r = row().key(format!("{key}/menu")).gap(8.0).align(AlignItems::CENTER)
        .child(text(label, LABEL - 1.0, Paint::DIM))
        .child(button);
    if open {
        let mut list = col().key(format!("{key}/list")).anchor(key).min_width(160.0).pad(4.0, 4.0).gap(1.0).radius(RADIUS + 2.0)
            .fill(MENU_SURFACE).border(LINE, Paint::HAIRLINE);
        for (i, o) in options.iter().enumerate() {
            let on = Some(i) == current;
            let mut item = row().key(format!("{key}:{i}")).height(CONTROL_H - 2.0).pad(10.0, 0.0).radius(RADIUS).align(AlignItems::CENTER).on_press(pick(i))
                .child(text(*o, LABEL - 1.0, if on { Paint::INK } else { Paint::TEXT }));
            if on { item = item.fill(selected_on(MENU_SURFACE)); }
            if commit { item = item.commit(); }
            list = list.child(item);
        }
        // The layer covers the window; pressing it anywhere outside the list closes the menu.
        r = r.child(col().key(format!("{key}/layer")).modal(close.clone(), 0.0).on_press(close).focusable(false).child(list));
    }
    r
}
