//! toast
//!
//! Purpose: A brief notice that something happened, in a corner, that goes away by itself.
//! Props: `key`, `title`, `body`, `shown`, `dismiss` (its close button).
//! States: hidden, shown (at the window's bottom right, polite live region).
//! Messages: `dismiss` on its close button; the application hides it after a few seconds (`Ui::on_wake`).
//! Usage: `toast("saved", "Saved", "Three changes written", s.toast, Msg::HideToast)`
//!
//! The toast has the status role (announced politely) and is never modal: input goes on around it.
use super::{LABEL, LINE, RADIUS, Variant, button_as};
use crate::el::{El, Font, Paint, Role, col, row, space, text};
use taffy::AlignItems;

pub fn toast<M: Clone>(key: &str, title: &str, body: &str, shown: bool, dismiss: M) -> El<M> {
    if !shown { return space(0.0, 0.0).key(key); }
    col().key(key).width(320.0).pad(16.0, 12.0).gap(4.0).radius(RADIUS + 3.0).fill(Paint::GROUND).border(LINE, Paint::HAIRLINE).role(Role::Status).label(title)
        .absolute(None, None, Some(24.0), Some(24.0))
        .child(row().gap(8.0).align(AlignItems::CENTER).child(text(title, LABEL, Paint::INK).font(Font::Display).grow(1.0))
            .child(button_as(format!("{key}/close"), &crate::i18n::tr("widget.dialog.close"), Variant::Quiet, dismiss)))
        .child(text(body, LABEL - 1.0, Paint::DIM))
}
