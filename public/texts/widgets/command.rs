//! command
//!
//! Purpose: Find and run a command by typing (a command palette, ⌘K / Ctrl+K).
//! Props: `key`, `query` (a `TextEdit`), `items` (labels), `active`, `on_query`, `run`.
//! States: the query; the matching items (every typed character in order); one active.
//! Messages: `on_query(edit)` while typing; Up / Down move `active` (through `on_query` of the same
//!   text with the arrow's step, see `command_step`); Enter `run(i)` (the item's index in `items`).
//! Usage: `command("palette", &s.q, &names, s.active, Msg::Query, Msg::Run)`
//!
//! The film's palette, promoted: a field, then the matching rows keyed by place (so a new filter
//! recomposes only the characters that changed). `command_matches` is the filter.
use super::{LABEL, RADIUS, SELECTED, TextEdit, input};
use crate::el::{El, Font, Paint, Role, col, row, text};
use taffy::AlignItems;

/// The items matching `q` (every character of it, in order, ignoring case and spaces), as indices.
pub fn command_matches(q: &str, items: &[&str]) -> Vec<usize> {
    let q: Vec<char> = q.to_lowercase().chars().filter(|c| !c.is_whitespace()).collect();
    items.iter().enumerate().filter(|(_, c)| { let mut it = c.to_lowercase().chars().collect::<Vec<_>>().into_iter(); q.iter().all(|ch| it.any(|x| x == *ch)) }).map(|(i, _)| i).collect()
}
pub fn command<M: Clone + 'static>(key: &str, query: &TextEdit, items: &[&str], active: usize, on_query: impl Fn(TextEdit) -> M + 'static, run: impl Fn(usize) -> M) -> El<M> {
    let found = command_matches(&query.text, items);
    let mut list = col().key(format!("{key}/list")).gap(2.0).role(Role::Listbox);
    for (r, &i) in found.iter().take(8).enumerate() {
        let on = r == active.min(found.len().saturating_sub(1));
        let mut row_el = row().key(format!("{key}/row{r}")).height(34.0).pad(10.0, 0.0).gap(10.0).radius(RADIUS).align(AlignItems::CENTER).on_press(run(i)).focusable(false).role(Role::Option).selected(on)
            .child(text(items[i], LABEL, if on { Paint::INK } else { Paint::TEXT }).font(Font::Mono).key(format!("{key}/row{r}/text")));
        if on { row_el = row_el.fill(SELECTED); }
        list = list.child(row_el);
    }
    if found.is_empty() { list = list.child(text(crate::i18n::tr("widget.command.none"), LABEL - 1.0, Paint::DIM)); }
    col().key(key).gap(10.0).width(420.0)
        .child(input(&format!("{key}/query"), query, &crate::i18n::tr("widget.command.placeholder"), 420.0, 64, on_query).role(Role::Combobox))
        .child(list)
}
