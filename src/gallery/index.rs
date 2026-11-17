//! The section index pages: the Components grid (`/components/`, the six groups of the rail, a card
//! per component) and the parts the Lab's index shares with it (the section header, the grid, a card).
use super::{Gallery, Msg, Page, docs, pages};
use amenui::el::{El, Font, Paint, Role, col, row, space, text, text_lh};
use taffy::AlignItems;

/// A section header (D.4): a hairline across the width with an index and a name set into it.
pub fn header(key: &str, n: usize, name: &str) -> El<Msg> {
    let rule = |k: &str, w: Option<f32>| { let r = space(0.0, 1.0).fill(Paint::FAINT).key(format!("{key}/{k}")); match w { Some(w) => r.width(w), None => r.grow(1.0) } };
    row().key(key.to_owned()).fill_width().gap(10.0).align(AlignItems::CENTER)
        .child(rule("a", Some(16.0)))
        .child(text(format!("{n:02}"), 11.0, Paint::DIM).font(Font::Mono))
        .child(rule("b", Some(16.0)))
        .child(text(name.to_uppercase(), 11.0, Paint::INK).font(Font::Mono).heading(2))
        .child(rule("c", None))
}

/// A grid of `count` columns of at least `min` px in `room` px, 16 px apart.
pub fn grid(key: &str, room: f32, min: f32) -> El<Msg> {
    let cols = ((room + 16.0) / (min + 16.0)).floor().max(1.0) as usize;
    let mut g = col().key(key.to_owned()).fill_width().min_width(0.0);
    g.style.display = taffy::Display::Grid;
    g.style.grid_template_columns = vec![taffy::prelude::minmax(taffy::prelude::length(0.0), taffy::prelude::fr(1.0)); cols];
    g.style.gap = taffy::prelude::Size { width: taffy::prelude::length(16.0), height: taffy::prelude::length(16.0) };
    g
}

/// A card: what it shows of its page (`preview`, if any), its index and name, one line under them;
/// pressing it opens the page.
pub fn card(key: &str, n: usize, name: String, line: String, preview: Option<El<Msg>>, to: Page) -> El<Msg> {
    let mut c = col().key(key.to_owned()).fill_width().min_width(0.0).gap(10.0).pad(16.0, 14.0).radius(6.0).border(1.0, Paint::FAINT)
        .on_press(Msg::Go(to)).role(Role::Link).label(&name);
    if let Some(p) = preview {
        c = c.child(col().key(format!("{key}/preview")).fill_width().min_width(0.0).height(112.0).align(AlignItems::START).justify(taffy::JustifyContent::CENTER).clip().scroll_x().child(p));
    }
    c.child(row().key(format!("{key}/name")).gap(10.0).align(AlignItems::BASELINE)
            .child(text(format!("{n:02}"), 11.0, Paint::DIM).font(Font::Mono))
            .child(text(name, 16.0, Paint::INK).flow(format!("card-name-{}", to.hash()))))
        .child(text_lh(line, 12.0, 19.0, Paint::DIM).key(format!("{key}/line")))
}

/// A card whose line is its pages, each a link (a Looks card on the Lab's index: every preset, mark
/// set or material is one press away).
pub fn card_links(key: &str, n: usize, name: String, links: Vec<Page>, to: Page) -> El<Msg> {
    let mut line = row().key(format!("{key}/links")).wrap().gap(10.0).fill_width().min_width(0.0);
    for p in links {
        line = line.child(row().key(format!("{key}/to/{}", p.hash())).height(24.0).align(AlignItems::CENTER).on_press(Msg::Go(p)).role(Role::Link)
            .child(text(p.name(), 12.0, Paint::TEXT)));
    }
    col().key(key.to_owned()).fill_width().min_width(0.0).gap(10.0).pad(16.0, 14.0).radius(6.0).border(1.0, Paint::FAINT)
        .child(row().key(format!("{key}/name")).gap(10.0).align(AlignItems::BASELINE).on_press(Msg::Go(to)).role(Role::Link)
            .child(text(format!("{n:02}"), 11.0, Paint::DIM).font(Font::Mono))
            .child(text(name, 16.0, Paint::INK)))
        .child(line)
}

/// The room the index has for its grids.
pub fn room(s: &Gallery) -> f32 { s.width - super::side(s) - if s.width < super::NARROW { 36.0 } else { 80.0 } }

/// `/components/`: the six groups of the rail, each a header and a grid of its components' cards.
pub fn components(s: &Gallery) -> El<Msg> {
    let en = s.locale == amenui::i18n::Locale::En;
    let all = docs::items();
    let mut page = col().key("index/components").fill_width().min_width(0.0).gap(28.0)
        .child(text(amenui::i18n::tr("nav.components"), 34.0, Paint::INK).font(Font::Display).heading(1).key("index/components/title").flow("page-title"));
    let mut n = 0;
    for (gi, (name, members)) in super::comp_groups().into_iter().enumerate() {
        let mut g = grid(&format!("index/g{gi}/grid"), room(s), 240.0);
        for p in members {
            n += 1;
            let id = match p { Page::Comp(c) => docs::COMP_NAMES[c].replace('-', "_"), Page::Widget(i) => docs::widget_names()[i as usize].clone(), _ => continue };
            let it = all.iter().find(|i| i.name == id || i.name == id.replace('_', "-"));
            let line = it.map(|i| if en && !i.desc_en.is_empty() { i.desc_en.clone() } else { i.desc.clone() }).unwrap_or_default();
            let line = line.split_inclusive(['.', '。']).next().unwrap_or("").trim().to_owned();
            let preview = match p { Page::Comp(c) => Some(pages::live(s, c).rekey(&format!("index/{}", p.hash()))), _ => docs::first_example(&id).map(|e| e.rekey(&format!("index/{}", p.hash()))) };
            g = g.child(card(&format!("index/{}", p.hash()), n, p.name(), line, preview, p));
        }
        page = page.child(col().key(format!("index/g{gi}")).fill_width().min_width(0.0).gap(16.0).child(header(&format!("index/g{gi}/head"), gi + 1, &name)).child(g));
    }
    page
}
