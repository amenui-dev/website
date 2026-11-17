//! The section index pages: the Components grid (`/components/`, the six groups of the rail, a card
//! per component) and the parts the Lab's index shares with it (the section header, the grid, a card).
use super::{Gallery, Msg, Page, docs, pages};
use amenui::el::{El, Font, Paint, Role, col, row, space, text, text_lh};
use amenui::style::Material;
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

/// How many columns of at least `min` px fit in `room` px, 16 px apart.
pub fn columns(room: f32, min: f32) -> usize { ((room + 16.0) / (min + 16.0)).floor().max(1.0) as usize }

/// A grid of `count` columns of at least `min` px in `room` px, 16 px apart.
pub fn grid(key: &str, room: f32, min: f32) -> El<Msg> {
    let cols = columns(room, min);
    let mut g = col().key(key.to_owned()).fill_width().min_width(0.0);
    g.style.display = taffy::Display::Grid;
    g.style.grid_template_columns = vec![taffy::prelude::minmax(taffy::prelude::length(0.0), taffy::prelude::fr(1.0)); cols];
    g.style.gap = taffy::prelude::Size { width: taffy::prelude::length(16.0), height: taffy::prelude::length(16.0) };
    g
}

/// How tall a card's preview is: the same in every card that has one, so the names in a row sit on
/// one baseline.
pub const PREVIEW_H: f32 = 112.0;

/// A card: what it shows of its page (`preview`, centred in the preview's height at its natural
/// size), its index and name, one line under them; pressing it opens the page. A card is opaque and
/// plain: the ambient dots are not drawn under it.
pub fn card(key: &str, n: usize, name: String, line: String, preview: Option<El<Msg>>, to: Page) -> El<Msg> {
    let mut c = col().key(key.to_owned()).fill_width().min_width(0.0).gap(10.0).pad(16.0, 14.0).radius(6.0).border(1.0, Paint::FAINT)
        .fill(Paint::GROUND).material(Material::Plain.id())
        .on_press(Msg::Go(to)).role(Role::Link).label(&name);
    if let Some(e) = preview { c = c.child(col().key(format!("{key}/preview")).fill_width().min_width(0.0).height(PREVIEW_H).align(AlignItems::START).justify(taffy::JustifyContent::CENTER).child(e)); }
    c.child(row().key(format!("{key}/name")).gap(10.0).align(AlignItems::BASELINE)
            .child(text(format!("{n:02}"), 11.0, Paint::DIM).font(Font::Mono))
            .child(text(name, 16.0, Paint::INK).flow(format!("card-name-{}", to.hash()))))
        .child(text_lh(line, 12.0, 19.0, Paint::DIM).key(format!("{key}/line")))
}

/// A description's first sentence, kept to two lines of a card's text `width`: a sentence that runs
/// longer ends at its first colon or semicolon, else at the last word that fits, with an ellipsis.
pub fn two_lines(desc: &str, width: f32) -> String {
    let first = desc.split_inclusive(['.', '。']).next().unwrap_or("").trim();
    // A conservative advance at 12 px: Latin about 6.2 px, CJK a full em.
    let wide = |t: &str| t.chars().map(|c| if (c as u32) >= 0x2E80 { 12.0 } else { 6.2 }).sum::<f32>();
    let room = 2.0 * width - 12.0;
    if wide(first) <= room { return first.to_owned(); }
    if let Some(i) = first.find([':', ';', '：', '；']) {
        let head = &first[..i];
        let end = if head.chars().any(|c| (c as u32) >= 0x2E80) { "。" } else { "." };
        if wide(head) + 12.0 <= room { return format!("{}{end}", head.trim_end()); }
    }
    let mut out = String::new();
    for w in first.split_inclusive(' ') { if wide(&out) + wide(w) + 12.0 > room { break; } out.push_str(w); }
    format!("{}…", out.trim_end().trim_end_matches([',', '，']))
}

/// A card with its name on top and what it shows of its page under it, as the cards of links beside
/// it in a row (their names on one baseline); pressing it opens the page.
pub fn card_named(key: &str, n: usize, name: String, preview: Option<El<Msg>>, to: Page) -> El<Msg> {
    let mut c = col().key(key.to_owned()).fill_width().min_width(0.0).gap(10.0).pad(16.0, 14.0).radius(6.0).border(1.0, Paint::FAINT)
        .fill(Paint::GROUND).material(Material::Plain.id())
        .on_press(Msg::Go(to)).role(Role::Link).label(&name)
        .child(row().key(format!("{key}/name")).gap(10.0).align(AlignItems::BASELINE)
            .child(text(format!("{n:02}"), 11.0, Paint::DIM).font(Font::Mono))
            .child(text(name, 16.0, Paint::INK).flow(format!("card-name-{}", to.hash()))));
    if let Some(e) = preview { c = c.child(col().key(format!("{key}/preview")).fill_width().min_width(0.0).height(PREVIEW_H).align(AlignItems::START).justify(taffy::JustifyContent::CENTER).child(e)); }
    c
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
        .fill(Paint::GROUND).material(Material::Plain.id())
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
        .child(col().key("index/components/head").gap(10.0).fill_width().min_width(0.0)
            .child(text(amenui::i18n::tr("nav.components"), 34.0, Paint::INK).font(Font::Display).heading(1).key("index/components/title").flow("page-title"))
            .child(text_lh(amenui::i18n::tr("index.components.lead"), 14.0, 22.0, Paint::DIM).key("index/components/lead").fill(Paint::GROUND).material(Material::Plain.id())));
    let (room, cols) = (room(s), columns(room(s), 240.0));
    let text_w = (room - 16.0 * (cols as f32 - 1.0)) / cols as f32 - 32.0;
    let mut n = 0;
    for (gi, (name, members)) in super::comp_groups().into_iter().enumerate() {
        let mut g = grid(&format!("index/g{gi}/grid"), room, 240.0);
        for p in members {
            n += 1;
            let id = match p { Page::Comp(c) => docs::COMP_NAMES[c].replace('-', "_"), Page::Widget(i) => docs::widget_names()[i as usize].clone(), _ => continue };
            let it = all.iter().find(|i| i.name == id || i.name == id.replace('_', "-"));
            let line = it.map(|i| if en && !i.desc_en.is_empty() { i.desc_en.clone() } else { i.desc.clone() }).unwrap_or_default();
            let line = two_lines(&line, text_w);
            let key = format!("index/{}", p.hash());
            let preview = match p { Page::Comp(c) => Some(pages::index_preview(s, c)), _ => widget_preview(&id, &key).or_else(|| docs::first_example(&id).map(|e| e.rekey(&key))) };
            g = g.child(card(&format!("index/{}", p.hash()), n, p.name(), line, preview, p));
        }
        page = page.child(col().key(format!("index/g{gi}")).fill_width().min_width(0.0).gap(16.0).child(header(&format!("index/g{gi}/head"), gi + 1, &name)).child(g));
    }
    page
}

/// A compact instance of a widget whose first example is wider than a card or taller than its
/// preview (at most 200 × 104 px): the same widget, smaller where it has a size, fewer items where it
/// has a list.
fn widget_preview(name: &str, key: &str) -> Option<El<Msg>> {
    use amenui::mesh::{MeshId, Scene3d};
    use amenui::widgets::*;
    let k = |n: &str| format!("{key}/{n}");
    Some(match name.replace('-', "_").as_str() {
        "orbit" => orbit(&k("o"), Scene3d::single(MeshId::CUBE), &Orbit::default(), |_| Msg::Nothing).width(150.0).height(100.0),
        "card_stack" => card_stack(&k("cs"), &Stack::default(), 150.0, 96.0, |_| Msg::Nothing),
        // The tip and the toast are placed by insets from their boxes' far edges; here from the near
        // ones, so they stay in the card at any width.
        "tooltip" => {
            let mut t = tooltip(&k("tt"), button(k("tb"), "Save", Msg::Nothing), "Save to this computer", true);
            t.style.align_items = AlignItems::START;
            if let Some(tip) = t.children.last_mut() { tip.style.inset = taffy::Rect { left: taffy::prelude::length(0.0), top: taffy::prelude::length(36.0), right: taffy::prelude::auto(), bottom: taffy::prelude::auto() }; }
            col().width(160.0).height(70.0).child(t)
        }
        "toast" => col().width(200.0).height(76.0).child(toast(&k("to"), "Saved", "Three changes written", true, Msg::Nothing).width(200.0).absolute(Some(0.0), Some(0.0), None, None)),
        "bottom_sheet" => {
            let body = col().gap(6.0).child(text("One line", 13.0, Paint::TEXT));
            amenui::el::layers([col().fill_width().fill_height(), bottom_sheet(&k("bs"), &Sheet::resting(104.0, 1), "Share to", body, |_| Msg::Nothing)])
                .width(160.0).height(104.0).radius(12.0).border(1.0, Paint::DIM).clip()
        }
        "pagination" => pagination(&k("pg"), 2, 5, |_| Msg::Nothing),
        // One match under the query; the query field as wide as the card allows.
        "command" => {
            let mut c = command(&k("cp"), &TextEdit::new("sa"), &["save all", "open file"], 0, |_| Msg::Nothing, |_| Msg::Nothing).width(200.0);
            if let Some(q) = c.children.first_mut() { q.style.size.width = taffy::prelude::length(200.0); }
            c
        }
        "nested_menu" => {
            let mut m = Menu::new(vec![MenuItem::sub("New", vec![MenuItem::leaf("Document"), MenuItem::leaf("Folder")]), MenuItem::leaf("Close")]);
            (m.width, m.row_h) = (104.0, 26.0);
            m.update(MenuMsg::Press(0, 0), 0.0);
            nested_menu(&k("nm"), &m, |_| Msg::Nothing)
        }
        "accordion" => accordion(&k("a"), vec![("What is a material", text("The second pass.", 13.0, Paint::TEXT)), ("What is a preset", text("A set of numbers.", 13.0, Paint::TEXT))], &[false, false], |_| Msg::Nothing).width(200.0),
        "floating" => {
            let mut f = Floating::default();
            f.area = (160.0, 100.0);
            f.size = (72.0, 40.0);
            f.place(3);
            amenui::el::layers([floating(&k("fl"), &f, text("00:42", 12.0, Paint::TEXT).key(k("t")), |_| Msg::Nothing)])
                .width(160.0).height(100.0).radius(8.0).border(1.0, Paint::FAINT).clip()
        }
        // The open sheet's panel and the open context menu, without the modal layer each puts over
        // the page; the skip link as it shows when it has focus.
        "sheet" => {
            let open = sheet(&k("sh"), true, "Filter", 150.0, Msg::Nothing, text("Conditions", 13.0, Paint::TEXT));
            let panel = open.children.into_iter().nth(1)?;
            row().width(200.0).height(100.0).radius(8.0).border(1.0, Paint::FAINT).clip().justify(taffy::JustifyContent::END).child(panel)
        }
        "context_menu" => {
            let open = context_menu(&k("cm"), Some((0.0, 0.0)), &["Open", "Rename"], 0, |_| Msg::Nothing, |_| Msg::Nothing, Msg::Nothing);
            let mut m = open.children.into_iter().next()?;
            m.style.position = taffy::Position::Relative;
            m
        }
        "skip_link" => row().key(k("sl")).height(32.0).pad(12.0, 0.0).radius(6.0).center().fill(Paint::ACCENT)
            .child(text("Skip to content", 13.0, Paint::GROUND)),
        _ => return None,
    })
}
