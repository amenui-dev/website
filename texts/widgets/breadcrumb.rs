//! breadcrumb
//!
//! Purpose: Where this page is in a hierarchy, each level a link back.
//! Props: `key`, `items` (labels), `msg` (go to level `i`).
//! States: the last item is the current page (not a link).
//! Messages: `msg(i)` on a level (press, Enter); links are keyed `key:i`.
//! Usage: `breadcrumb("crumbs", &["Docs", "Components", "Button"], Msg::Up)`
//!
//! A navigation landmark of links separated by slashes.
use super::LABEL;
use crate::el::{El, Paint, Role, row, text};
use taffy::AlignItems;

pub fn breadcrumb<M: Clone>(key: &str, items: &[&str], msg: impl Fn(usize) -> M) -> El<M> {
    let mut r = row().key(key).gap(8.0).align(AlignItems::CENTER).role(Role::Navigation);
    for (i, it) in items.iter().enumerate() {
        if i > 0 { r = r.child(text("/", LABEL - 1.0, Paint::DIM)); }
        let last = i + 1 == items.len();
        let t = text(*it, LABEL - 1.0, if last { Paint::INK } else { Paint::DIM }).key(format!("{key}:{i}"));
        r = r.child(if last { t } else { t.on_press(msg(i)).focusable(true).role(Role::Link) });
    }
    r
}
