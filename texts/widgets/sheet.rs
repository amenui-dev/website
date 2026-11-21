//! sheet
//!
//! Purpose: A panel that slides over one side of the window for a task that keeps the page in view
//!   (a drawer).
//! Props: `key`, `open`, `title`, `width`, `close`, `body`.
//! States: closed, open (modal: focus inside, Esc closes; the page beneath held a little loose).
//! Messages: `close` on Esc, a press on the page beside it, or its close button.
//! Usage: `sheet("filters", s.open, "Filters", 360.0, Msg::Close, body)`
//!
//! A modal panel along the window's right edge; it arrives as a flow when the view keys it the same
//! (no animation of its own).
use super::{LABEL, LINE, MENU_SURFACE, Variant, button_as};
use crate::el::{El, Font, Paint, Role, col, row, space, text};
use taffy::AlignItems;

pub fn sheet<M: Clone>(key: &str, open: bool, title: &str, width: f32, close: M, body: El<M>) -> El<M> {
    if !open { return space(0.0, 0.0); }
    let panel = col().key(format!("{key}/panel")).width(width).fill_height().pad(22.0, 18.0).gap(14.0).fill(MENU_SURFACE).border(LINE, Paint::HAIRLINE)
        .role(Role::Dialog).label(title)
        .child(row().gap(12.0).align(AlignItems::CENTER).child(text(title, LABEL + 3.0, Paint::INK).font(Font::Display).grow(1.0))
            .child(button_as(format!("{key}/close"), &crate::i18n::tr("widget.dialog.close"), Variant::Quiet, close.clone())))
        .child(body);
    row().key(key).fill_width().fill_height().modal(close.clone(), 0.25)
        .child(col().key(format!("{key}/beside")).grow(1.0).fill_height().on_press(close).focusable(false))
        .child(panel)
}
