//! The texts the site shows that the web module does not carry (site-design R.5): the Lab's Markdown,
//! the Lab reader's page, the sources the components' pages show (the framework's examples, its widgets
//! and `src/morph.rs`) and the demos' English (`assets/english.txt`). Natively they are compiled in. On
//! the web the page fetches those a route's first frame shows beside the module and gives them to it
//! (`put_text`) before that frame; one the module asks for later it names (`wanted_texts`), the page
//! fetches it and the view is laid out again. A text is named by its path under the site's `texts/`.

/// A text by name, if the module has it.
#[cfg(not(target_arch = "wasm32"))]
pub fn get(name: &str) -> Option<&'static str> {
    macro_rules! lab { ($($n:literal),*) => { match name { $(
        concat!("en/lab-", $n, ".md") => return Some(include_str!(concat!("../docs/site/en/lab-", $n, ".md"))),
        concat!("zh-Hans/lab-", $n, ".md") => return Some(include_str!(concat!("../docs/site/zh-Hans/lab-", $n, ".md"))),
    )* _ => {} } } }
    lab!("lumen", "contours", "silk", "grown", "weld", "flow", "loupe", "orbit", "spring", "stack");
    match name {
        "english.txt" => Some(include_str!("../assets/english.txt")),
        "en/layout.md" => Some(include_str!("../docs/site/en/layout.md")),
        "morph.rs" => Some(amenui::sources::MORPH),
        _ if name.starts_with("widgets/") => Some(match &name[8..] {
            "button.rs" => amenui::sources::widget::BUTTON, "number.rs" => amenui::sources::widget::NUMBER, "input.rs" => amenui::sources::widget::INPUT,
            "switch.rs" => amenui::sources::widget::SWITCH, "slider.rs" => amenui::sources::widget::SLIDER, "tabs.rs" => amenui::sources::widget::TABS,
            "choice.rs" => amenui::sources::widget::CHOICE, "list.rs" => amenui::sources::widget::LIST, "card.rs" => amenui::sources::widget::CARD,
            "dialog.rs" => amenui::sources::widget::DIALOG, "image.rs" => amenui::sources::widget::IMAGE, "divider.rs" => amenui::sources::widget::DIVIDER,
            "scroll.rs" => amenui::sources::widget::SCROLL, "menu.rs" => amenui::sources::widget::MENU, "orbit.rs" => amenui::sources::widget::ORBIT,
            "checkbox.rs" => amenui::sources::widget::CHECKBOX, "radio.rs" => amenui::sources::widget::RADIO, "select.rs" => amenui::sources::widget::SELECT,
            "tooltip.rs" => amenui::sources::widget::TOOLTIP, "popover.rs" => amenui::sources::widget::POPOVER, "toast.rs" => amenui::sources::widget::TOAST,
            "progress.rs" => amenui::sources::widget::PROGRESS, "accordion.rs" => amenui::sources::widget::ACCORDION, "table.rs" => amenui::sources::widget::TABLE,
            "badge.rs" => amenui::sources::widget::BADGE, "avatar.rs" => amenui::sources::widget::AVATAR, "skeleton.rs" => amenui::sources::widget::SKELETON,
            "breadcrumb.rs" => amenui::sources::widget::BREADCRUMB, "pagination.rs" => amenui::sources::widget::PAGINATION, "sheet.rs" => amenui::sources::widget::SHEET,
            "context_menu.rs" => amenui::sources::widget::CONTEXT_MENU, "command.rs" => amenui::sources::widget::COMMAND, "toggle.rs" => amenui::sources::widget::TOGGLE,
        "bottom_sheet.rs" => amenui::sources::widget::BOTTOM_SHEET, "floating.rs" => amenui::sources::widget::FLOATING, "caption.rs" => amenui::sources::widget::CAPTION, "nested_menu.rs" => amenui::sources::widget::NESTED_MENU, "card_stack.rs" => amenui::sources::widget::CARD_STACK, "dot_chart.rs" => amenui::sources::widget::DOT_CHART,
            _ => return None,
        }),
        _ => {
            let ex = name.strip_prefix("examples/")?.strip_suffix(".rs")?;
            amenui::recipes::components::all().into_iter().find(|(n, _)| *n == ex).map(|(_, s)| s)
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::RefCell;
    use std::collections::HashMap;
    thread_local! {
        static HAVE: RefCell<HashMap<String, &'static str>> = RefCell::new(HashMap::new());
        static WANT: RefCell<(Vec<String>, Vec<String>)> = const { RefCell::new((Vec::new(), Vec::new())) };
    }
    /// A text by name, if the page has given it; otherwise it is asked for, once.
    pub fn get(name: &str) -> Option<&'static str> {
        if let Some(t) = HAVE.with(|h| h.borrow().get(name).copied()) { return Some(t); }
        WANT.with(|w| { let (asked, want) = &mut *w.borrow_mut(); if !asked.iter().any(|a| a == name) { asked.push(name.to_owned()); want.push(name.to_owned()); } });
        None
    }
    /// The page gives a text (kept for the life of the page).
    pub fn put(name: &str, body: String) { HAVE.with(|h| { h.borrow_mut().entry(name.to_owned()).or_insert_with(|| Box::leak(body.into_boxed_str())); }); }
    /// The texts asked for since the last call.
    pub fn wanted() -> Vec<String> { WANT.with(|w| std::mem::take(&mut w.borrow_mut().1)) }
    /// Forget what has been asked for: the layouts made before the first route are not the page's.
    pub fn forget() { WANT.with(|w| *w.borrow_mut() = (Vec::new(), Vec::new())); }
}
#[cfg(target_arch = "wasm32")]
pub use web::{forget, get, put, wanted};
