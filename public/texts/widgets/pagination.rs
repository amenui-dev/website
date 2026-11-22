//! pagination
//!
//! Purpose: Move between pages of results.
//! Props: `key`, `page` (0-based), `pages`, `msg`.
//! States: the current page marked; previous / next disabled at the ends.
//! Messages: `msg(i)`; numbers are keyed `key:i`, the steps `key/prev`, `key/next`.
//! Usage: `pagination("results", s.page, 12, Msg::Page)`
//!
//! Previous, up to seven page numbers around the current one (with the first and last), next.
use super::{CONTROL_H, LABEL, LINE, RADIUS};
use crate::el::{El, Paint, Role, row, text};
use taffy::AlignItems;

pub fn pagination<M: Clone>(key: &str, page: usize, pages: usize, msg: impl Fn(usize) -> M) -> El<M> {
    let pages = pages.max(1);
    let page = page.min(pages - 1);
    let cell = |k: String, label: String, on: bool| row().key(k).height(CONTROL_H).min_width(CONTROL_H).pad(8.0, 0.0).radius(RADIUS).center()
        .fill(if on { Paint::INK } else { Paint::GROUND }).border(LINE, if on { Paint::INK } else { Paint::DIM }).border_hover(Paint::INK)
        .child(text(label, LABEL - 1.0, if on { Paint::GROUND } else { Paint::TEXT }));
    let mut r = row().key(key).gap(6.0).align(AlignItems::CENTER).role(Role::Navigation)
        .child(cell(format!("{key}/prev"), "‹".into(), false).on_press(msg(page.saturating_sub(1))).disabled(page == 0).label(crate::i18n::tr("widget.pagination.prev")));
    let lo = page.saturating_sub(3).min(pages.saturating_sub(7));
    for i in lo..(lo + 7).min(pages) {
        r = r.child(cell(format!("{key}:{i}"), format!("{}", i + 1), i == page).on_press(msg(i)).selected(i == page));
    }
    r.child(cell(format!("{key}/next"), "›".into(), false).on_press(msg((page + 1).min(pages - 1))).disabled(page + 1 >= pages).label(crate::i18n::tr("widget.pagination.next")))
}
