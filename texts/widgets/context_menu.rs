//! context_menu
//!
//! Purpose: Commands for what is under the pointer, opened by a secondary press (or Shift+F10 /
//!   the menu key), at that place.
//! Props: `key`, `at` (where it opens: logical px, `None` closed), `items`, `active`, `pick`, `move_to`, `close`.
//! States: closed, open (a menu in a layer at `at`, an active item).
//! Messages: `pick(i)` on an item or Enter; Up / Down / Home / End `move_to(i)`; Esc or a press
//!   outside `close` (WAI-ARIA menu).
//! Usage: `context_menu("file-menu", s.menu_at, &["Open", "Rename"], s.active, Msg::Pick, Msg::Active, Msg::Close)`
//!
//! The application opens it (from a long press, a secondary button or a key) and keeps where.
use super::{CONTROL_H, LABEL, LINE, MENU_SURFACE, RADIUS, selected_on};
use crate::el::{El, Key, Paint, Role, col, row, space, text};
use std::rc::Rc;
use taffy::AlignItems;

pub fn context_menu<M: Clone + 'static>(key: &str, at: Option<(f32, f32)>, items: &[&str], active: usize, pick: impl Fn(usize) -> M + 'static, move_to: impl Fn(usize) -> M + 'static, close: M) -> El<M> {
    let Some((x, y)) = at else { return space(0.0, 0.0) };
    let pick: Rc<dyn Fn(usize) -> M> = Rc::new(pick);
    let n = items.len();
    let keys = { let pick = pick.clone(); move |k: Key, _: bool| -> Option<M> { match k {
        Key::Down => Some(move_to((active + 1) % n.max(1))), Key::Up => Some(move_to((active + n.max(1) - 1) % n.max(1))),
        Key::Home => Some(move_to(0)), Key::End => Some(move_to(n.saturating_sub(1))), Key::Enter | Key::Char(' ') => Some(pick(active)), _ => None } } };
    let mut m = col().key(format!("{key}/menu")).min_width(180.0).pad(4.0, 4.0).gap(1.0).radius(RADIUS + 2.0).fill(MENU_SURFACE).border(LINE, Paint::HAIRLINE)
        .role(Role::Menu).focusable(true).on_key(keys).absolute(Some(x), Some(y), None, None);
    for (i, it) in items.iter().enumerate() {
        let mut r = row().key(format!("{key}:{i}")).height(CONTROL_H - 2.0).pad(10.0, 0.0).radius(RADIUS).align(AlignItems::CENTER).on_press(pick(i)).focusable(false).role(Role::MenuItem)
            .child(text(*it, LABEL, if i == active { Paint::INK } else { Paint::TEXT }));
        if i == active { r = r.fill(selected_on(MENU_SURFACE)); }
        m = m.child(r);
    }
    col().key(key).fill_width().fill_height().modal(close.clone(), 0.0).on_press(close).focusable(false).child(m)
}
