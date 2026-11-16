//! The documentation site (DESIGN.md §5): prose pages from `docs/site/<locale>/*.md`, the For-AI pages,
//! the changelog, and one page per component in the same order every time — title and description,
//! a preview with 预览 / 代码 tabs, installation (`lab add`), usage, examples (each a preview and its
//! code), the API from `registry.json`, keyboard and accessibility, and the component under every
//! material. A right-hand outline follows the scroll; prev / next links close every page; ⌘K / Ctrl+K
//! or `/` searches every page.
use super::{Gallery, Msg, Page};
use crate::el::{El, Font, Paint, Role, col, row, space, text, text_lh};
use crate::i18n::{Locale, tr};
use crate::morph::Cmd;
use crate::style::{MATERIAL_COUNT, Material};
use crate::widgets::{SELECTED, TextEdit, Variant, button_as, command, command_matches, dialog};
use taffy::AlignItems;

/// The prose pages, by slug, in reading order: the first six are 开始 (with the command line and the
/// forms recipe), the rest 概念.
pub const DOCS: [&str; 23] = ["introduction", "installation", "cli", "first-view", "project-structure", "forms", "tree-and-messages", "layout", "text", "materials", "ripples",
    "transforms", "presets", "annotation", "three-d", "particles", "i18n", "accessibility", "performance", "testing", "theming", "rtl", "registry-schema"];
pub const START: usize = 6;
const ZH: [&str; 23] = [
    include_str!("../../docs/site/zh-Hans/introduction.md"), include_str!("../../docs/site/zh-Hans/installation.md"), include_str!("../../docs/site/zh-Hans/cli.md"), include_str!("../../docs/site/zh-Hans/first-view.md"),
    include_str!("../../docs/site/zh-Hans/project-structure.md"), include_str!("../../docs/site/zh-Hans/forms.md"), include_str!("../../docs/site/zh-Hans/tree-and-messages.md"), include_str!("../../docs/site/zh-Hans/layout.md"),
    include_str!("../../docs/site/zh-Hans/text.md"), include_str!("../../docs/site/zh-Hans/materials.md"), include_str!("../../docs/site/zh-Hans/ripples.md"),
    include_str!("../../docs/site/zh-Hans/transforms.md"), include_str!("../../docs/site/zh-Hans/presets.md"), include_str!("../../docs/site/zh-Hans/annotation.md"),
    include_str!("../../docs/site/zh-Hans/three-d.md"), include_str!("../../docs/site/zh-Hans/particles.md"), include_str!("../../docs/site/zh-Hans/i18n.md"),
    include_str!("../../docs/site/zh-Hans/accessibility.md"), include_str!("../../docs/site/zh-Hans/performance.md"), include_str!("../../docs/site/zh-Hans/testing.md"),
    include_str!("../../docs/site/zh-Hans/theming.md"), include_str!("../../docs/site/zh-Hans/rtl.md"), include_str!("../../docs/site/zh-Hans/registry-schema.md"),
];
const EN: [&str; 23] = [
    include_str!("../../docs/site/en/introduction.md"), include_str!("../../docs/site/en/installation.md"), include_str!("../../docs/site/en/cli.md"), include_str!("../../docs/site/en/first-view.md"),
    include_str!("../../docs/site/en/project-structure.md"), include_str!("../../docs/site/en/forms.md"), include_str!("../../docs/site/en/tree-and-messages.md"), include_str!("../../docs/site/en/layout.md"),
    include_str!("../../docs/site/en/text.md"), include_str!("../../docs/site/en/materials.md"), include_str!("../../docs/site/en/ripples.md"),
    include_str!("../../docs/site/en/transforms.md"), include_str!("../../docs/site/en/presets.md"), include_str!("../../docs/site/en/annotation.md"),
    include_str!("../../docs/site/en/three-d.md"), include_str!("../../docs/site/en/particles.md"), include_str!("../../docs/site/en/i18n.md"),
    include_str!("../../docs/site/en/accessibility.md"), include_str!("../../docs/site/en/performance.md"), include_str!("../../docs/site/en/testing.md"),
    include_str!("../../docs/site/en/theming.md"), include_str!("../../docs/site/en/rtl.md"), include_str!("../../docs/site/en/registry-schema.md"),
];
/// The For-AI pages and the changelog: (hash name, title key, source).
pub const AI: [(&str, &str, &str); 3] = [("ai-agents", "docs.ai.agents", include_str!("../../AGENTS.md")), ("ai-llms", "docs.ai.llms", include_str!("../../llms.txt")), ("changelog", "docs.changelog", include_str!("../../CHANGELOG.md"))];
const REGISTRY: &str = include_str!("../../registry.json");

/// A prose page's source in a language (English when the language has none).
pub fn source(i: usize, l: Locale) -> &'static str { match l { Locale::ZhHans => ZH[i], Locale::En => EN[i] } }
/// A prose page's title: its first heading.
pub fn title(i: usize, l: Locale) -> String { source(i, l).lines().find_map(|x| x.strip_prefix("# ")).unwrap_or(DOCS[i]).to_owned() }

/// The components the registry lists (not the shared sizes), in its order: (name, title, description,
/// functions, files, dependencies, response, recompose).
pub struct Item { pub name: String, pub title: String, pub group: String, pub desc: String, pub desc_en: String, pub functions: Vec<String>, pub files: Vec<String>, pub deps: Vec<String>, pub response: String, pub recompose: String }
pub fn items() -> Vec<Item> {
    let reg: serde_json::Value = serde_json::from_str(REGISTRY).unwrap_or_default();
    let s = |v: &serde_json::Value| v.as_str().unwrap_or_default().to_owned();
    let list = |v: &serde_json::Value| v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()).unwrap_or_default();
    reg["items"].as_array().into_iter().flatten().filter(|i| i["name"] != "base").map(|i| Item {
        name: s(&i["name"]), title: s(&i["title"]), group: s(&i["group"]), desc: s(&i["description"]), desc_en: s(&i["description_en"]), functions: list(&i["functions"]), files: list(&i["files"]),
        deps: list(&i["dependencies"]), response: s(&i["response"]), recompose: s(&i["recompose"]),
    }).collect()
}
/// The component groups the docs list, in order (`registry.json`'s `group`), with their title keys.
pub const GROUPS: [(&str, &str); 6] = [("inputs", "nav.group.inputs"), ("display", "nav.group.display"), ("feedback", "nav.group.feedback"), ("navigation", "nav.group.navigation"), ("layout", "nav.group.layout"), ("data", "nav.group.data")];
/// A component's group by its name (`scroll-area` and `scroll_area` alike).
pub fn group_of(name: &str) -> String {
    static GROUP: std::sync::OnceLock<Vec<(String, String)>> = std::sync::OnceLock::new();
    let all = GROUP.get_or_init(|| items().into_iter().map(|i| (i.name.replace('_', "-"), i.group)).collect());
    let n = name.replace('_', "-");
    all.iter().find(|(k, _)| *k == n).map(|(_, g)| g.clone()).unwrap_or_default()
}
/// A widget file's header line (`Props:`, `States:`, `Messages:`, with its continuation lines).
fn header_line(src: &str, part: &str) -> String {
    let mut out = String::new();
    let mut on = false;
    for l in src.lines().take_while(|l| l.starts_with("//!")) {
        let t = l.trim_start_matches("//!").trim();
        if t.starts_with(part) { on = true; out.push_str(t.trim_start_matches(part).trim()); continue; }
        if on && l.starts_with("//!   ") { out.push(' '); out.push_str(t); continue; }
        on = false;
    }
    out
}
/// The source of a widget file by its path.
fn widget_source(path: &str) -> &'static str {
    match path.rsplit('/').next().unwrap_or("") {
        "button.rs" => include_str!("../widgets/button.rs"), "number.rs" => include_str!("../widgets/number.rs"), "input.rs" => include_str!("../widgets/input.rs"),
        "switch.rs" => include_str!("../widgets/switch.rs"), "slider.rs" => include_str!("../widgets/slider.rs"), "tabs.rs" => include_str!("../widgets/tabs.rs"),
        "choice.rs" => include_str!("../widgets/choice.rs"), "list.rs" => include_str!("../widgets/list.rs"), "card.rs" => include_str!("../widgets/card.rs"),
        "dialog.rs" => include_str!("../widgets/dialog.rs"), "image.rs" => include_str!("../widgets/image.rs"), "divider.rs" => include_str!("../widgets/divider.rs"),
        "scroll.rs" => include_str!("../widgets/scroll.rs"), "menu.rs" => include_str!("../widgets/menu.rs"), "orbit.rs" => include_str!("../widgets/orbit.rs"),
        "checkbox.rs" => include_str!("../widgets/checkbox.rs"), "radio.rs" => include_str!("../widgets/radio.rs"), "select.rs" => include_str!("../widgets/select.rs"),
        "tooltip.rs" => include_str!("../widgets/tooltip.rs"), "popover.rs" => include_str!("../widgets/popover.rs"), "toast.rs" => include_str!("../widgets/toast.rs"),
        "progress.rs" => include_str!("../widgets/progress.rs"), "accordion.rs" => include_str!("../widgets/accordion.rs"), "table.rs" => include_str!("../widgets/table.rs"),
        "badge.rs" => include_str!("../widgets/badge.rs"), "avatar.rs" => include_str!("../widgets/avatar.rs"), "skeleton.rs" => include_str!("../widgets/skeleton.rs"),
        "breadcrumb.rs" => include_str!("../widgets/breadcrumb.rs"), "pagination.rs" => include_str!("../widgets/pagination.rs"), "sheet.rs" => include_str!("../widgets/sheet.rs"),
        "context_menu.rs" => include_str!("../widgets/context_menu.rs"), "command.rs" => include_str!("../widgets/command.rs"), "toggle.rs" => include_str!("../widgets/toggle.rs"),
        "bottom_sheet.rs" => include_str!("../widgets/bottom_sheet.rs"), "floating.rs" => include_str!("../widgets/floating.rs"), "caption.rs" => include_str!("../widgets/caption.rs"), "nested_menu.rs" => include_str!("../widgets/nested_menu.rs"),
        _ => "",
    }
}
/// The registry names of the components the gallery has its own live page for (`COMPONENTS`).
pub const COMP_NAMES: [&str; 13] = ["button", "number", "input", "switch", "slider", "tabs", "choice", "list", "card", "dialog", "image", "divider", "scroll-area"];
/// The other components: their pages are made from their examples (`Page::Widget(i)`).
pub fn widget_names() -> Vec<String> { items().into_iter().map(|i| i.name).filter(|n| !COMP_NAMES.contains(&n.as_str())).collect() }
/// The examples file of a component (registry name → `examples/components/<name>.rs`).
fn example_name(name: &str) -> &str { match name { "scroll_area" | "scroll-area" => "scroll", n => n } }
pub fn example_source(name: &str) -> Option<&'static str> { crate::recipes::components::all().into_iter().find(|(n, _)| *n == example_name(name)).map(|(_, s)| s) }
/// The examples in a file: (label, the function's source).
pub fn examples_of(src: &str) -> Vec<(String, String)> {
    let mut out = vec![];
    let lines: Vec<&str> = src.lines().collect();
    for (i, l) in lines.iter().enumerate() {
        if let Some(label) = l.strip_prefix("/// ") && lines.get(i + 1).is_some_and(|n| n.starts_with("pub fn ex_")) {
            let end = (i + 1..lines.len()).find(|&j| lines[j] == "}").unwrap_or(lines.len() - 1);
            out.push((label.to_owned(), lines[i + 1..=end].join("\n")));
        }
    }
    out
}
/// The part of a "中文 · English" label for a language.
fn pick(label: &str, l: Locale) -> String { match label.split_once(" · ") { Some((z, e)) => if l == Locale::En { e.into() } else { z.into() }, None => label.into() } }

/// A block of code, selectable, with a copy button.
fn code(k: &str, src: &str) -> El<Msg> {
    // Code keeps its lines: a narrow window scrolls them sideways rather than widening the page.
    let mut b = col().key(format!("{k}/code")).fill_width().min_width(0.0).scroll_x().pad(16.0, 12.0).radius(6.0).border(1.0, Paint::FAINT).selectable();
    for (j, line) in src.lines().enumerate() { b = b.child(text(if line.is_empty() { " ".to_owned() } else { line.to_owned() }, 12.5, Paint::TEXT).font(Font::Mono).key(format!("{k}/code/{j}"))); }
    col().key(k.to_owned()).gap(6.0).fill_width()
        .child(row().key(format!("{k}/bar")).child(space(0.0, 0.0).grow(1.0)).child(button_as(format!("{k}/copy"), &tr("docs.copy"), Variant::Quiet, Msg::Copy(src.to_owned())).height(24.0)))
        .child(b)
}
/// One section of a page: its heading (in the outline) and body.
fn section(page: &str, i: usize, title: &str, body: El<Msg>) -> El<Msg> {
    col().key(format!("doc/{page}/sec{i}")).gap(12.0).fill_width().min_width(0.0)
        .child(text(title, 20.0, Paint::INK).font(Font::Display).heading(2).key(format!("doc/{page}/sec{i}/h")))
        .child(body)
}
/// The right-hand outline: the page's sections, the one in view marked; pressing one scrolls to it.
fn outline(s: &Gallery, page: &str, titles: &[(String, String)]) -> El<Msg> {
    let mut c = col().key(format!("doc/{page}/outline")).width(180.0).gap(6.0).role(Role::Navigation)
        .child(text(tr("docs.on-this-page"), 12.0, Paint::DIM).key(format!("doc/{page}/outline/title")));
    for (i, (t, k)) in titles.iter().enumerate() {
        let on = s.outline_at == i;
        c = c.child(text(t.clone(), 13.0, if on { Paint::ACCENT } else { Paint::TEXT }).key(format!("doc/{page}/outline/{i}"))
            .on_press(Msg::Outline(k.clone())).focusable(false).role(Role::Link));
    }
    c
}
/// The links to the page before and after this one.
fn prev_next(s: &Gallery) -> El<Msg> {
    let all = super::Page::all();
    let at = all.iter().position(|p| *p == s.page).unwrap_or(0);
    let mut r = row().key("doc/prevnext").fill_width().gap(12.0).pad(0.0, 24.0);
    if at > 0 { let p = all[at - 1]; r = r.child(button_as("doc/prev", &format!("← {}", p.name()), Variant::Default, Msg::Go(p))); }
    r = r.child(space(0.0, 0.0).grow(1.0));
    if at + 1 < all.len() { let p = all[at + 1]; r = r.child(button_as("doc/next", &format!("{} →", p.name()), Variant::Default, Msg::Go(p))); }
    r
}
/// The width a documentation page's text is set in: the window less the rail, the margins and (on a
/// wide window) the outline; on a narrow window, the window less its two margins.
pub fn doc_w(s: &Gallery, max: f32) -> f32 {
    if s.width < super::NARROW { return (s.width - 36.0).max(200.0); }
    (s.width - 220.0 - 80.0 - if s.width > 1180.0 { 220.0 } else { 0.0 }).clamp(280.0, max)
}
/// A page as raw Markdown (`<hash>.md`), for agents and for reading without the framework: a prose page
/// is its source; a component's page is written out in the same order as it is shown (title and
/// description, installation, usage, every example's code, the API table). `None` for pages that are
/// drawn only (the film, the mark and material pages, the scenes).
pub fn markdown(page: Page, l: Locale) -> Option<String> {
    match page {
        Page::Doc(i) => Some(source(i as usize, l).to_owned()),
        Page::Ai(i) => Some(AI[i as usize].2.to_owned()),
        Page::Widget(_) | Page::Comp(_) => {
            let name = match page { Page::Widget(i) => widget_names()[i as usize].clone(), Page::Comp(c) => COMP_NAMES[c].replace('-', "_"), _ => unreachable!() };
            let all = items();
            let it = all.iter().find(|i| i.name == name || i.name == name.replace('_', "-"))?;
            let en = l == Locale::En;
            let title = if en { it.name.replace('_', " ") } else { it.title.clone() };
            let desc = if en && !it.desc_en.is_empty() { &it.desc_en } else { &it.desc };
            let src = it.files.first().map(|f| widget_source(f)).unwrap_or("");
            let ex = examples_of(example_source(&it.name).unwrap_or(""));
            let mut md = format!("# {title}\n\n{desc}\n\n## {}\n\n```sh\ncargo run --release --bin lab -- add {} --to ../my-app\n```\n\n{}: {}\n\n## {}\n\n```rust\nuse lab::widgets::{{{}}};\n\n{}\n```\n",
                tr("docs.sec.install"), it.name, tr("docs.install.files"), it.files.join(", "), tr("docs.sec.usage"), it.functions.join(", "), ex.first().map(|e| e.1.clone()).unwrap_or_default());
            for (label, code) in &ex { md.push_str(&format!("\n## {}\n\n```rust\n{code}\n```\n", pick(label, l))); }
            md.push_str(&format!("\n## {}\n\n| | |\n|---|---|\n| {} | `{}` |\n| {} | {} |\n| {} | `{}` |\n| {} | {} |\n",
                tr("docs.sec.api"), tr("docs.api.functions"), it.functions.join("`, `"), tr("docs.api.props"), header_line(src, "Props:"), tr("docs.api.files"), it.files.join("`, `"), tr("docs.kb.keys"), header_line(src, "Messages:")));
            Some(md)
        }
        Page::Registry => Some(REGISTRY.to_owned()),
        _ => None,
    }
}
/// A page and its outline side by side, prev / next under it.
fn frame(s: &Gallery, page: &str, body: El<Msg>, titles: &[(String, String)]) -> El<Msg> {
    let wide = s.width > 1180.0;
    let mut r = row().key(format!("doc/{page}/frame")).fill_width().gap(40.0).align(AlignItems::START).child(col().key(format!("doc/{page}/col")).grow(1.0).shrink().min_width(0.0).gap(30.0).max_width(860.0).child(body).child(prev_next(s)));
    if wide { r = r.child(outline(s, page, titles)); }
    r
}

/// A prose page (开始, 概念).
pub fn prose(s: &Gallery, i: usize) -> El<Msg> {
    let src = source(i, s.locale);
    let w = doc_w(s, 820.0);
    let (body, out) = crate::markdown::render(src, &format!("doc/{}/md", DOCS[i]), w, &|t| link(t));
    // The outline lists the page's second-level headings, by their keys.
    frame(s, DOCS[i], body, &out.into_iter().filter(|o| o.0 == 2).map(|o| (o.1, o.2)).collect::<Vec<_>>())
}
/// A For-AI page, the registry, or the changelog.
pub fn ai(s: &Gallery, i: usize) -> El<Msg> {
    let (hash, _, src) = AI[i];
    let w = doc_w(s, 820.0);
    // llms.txt is plain text: shown as a block of code.
    let body = if hash == "ai-llms" { col().key("doc/ai-llms/md").gap(14.0).child(text(tr("docs.ai.llms"), 30.0, Paint::INK).font(Font::Display).heading(1)).child(code("doc/ai-llms/txt", src)) }
        else { crate::markdown::render(src, &format!("doc/{hash}/md"), w, &|t| link(t)).0 };
    frame(s, hash, body, &[])
}
/// The registry as a table, and the recipes.
pub fn registry(s: &Gallery) -> El<Msg> {
    let mut md = format!("# {}\n\n{}\n\n| {} | {} | {} |\n|---|---|---|\n", tr("docs.ai.registry"), tr("docs.registry.lead"), tr("docs.api.name"), tr("docs.api.functions"), tr("docs.api.files"));
    for it in items() { md.push_str(&format!("| [{}](#{}) | `{}` | `{}` |\n", it.name, page_hash(&it.name), it.functions.join(", "), it.files.join(", "))); }
    md.push_str(&format!("\n## {}\n\n", tr("docs.recipes")));
    for (n, _, what) in crate::recipes::RECIPES { md.push_str(&format!("- [{n}](#recipe-{n}): {what}\n")); }
    let w = doc_w(s, 900.0);
    let (body, _) = crate::markdown::render(&md, "doc/ai-registry/md", w, &|t| link(t));
    frame(s, "ai-registry", body, &[])
}
/// A link target as a message: `#hash` goes to that page.
fn link(t: &str) -> Msg {
    match Page::from_hash(t) { Some(p) => Msg::Go(p), None => Msg::Nothing }
}
/// The hash of a component's page.
pub fn page_hash(name: &str) -> String { name.replace('_', "-") }

/// A component's page, in the same order every time. `live` is the gallery's own interactive preview
/// when it has one (else the first example), `params` its numbers.
pub fn component(s: &Gallery, name: &str, live: Option<El<Msg>>, params: Option<El<Msg>>) -> El<Msg> {
    let l = s.locale;
    let all = items();
    let Some(it) = all.iter().find(|i| i.name == name) else { return text(format!("⟨{name}⟩"), 14.0, Paint::DIM) };
    let h = page_hash(name);
    let src = it.files.first().map(|f| widget_source(f)).unwrap_or("");
    let ex_src = example_source(name).unwrap_or("");
    let examples = examples_of(ex_src);
    let mut previews = crate::recipes::components::examples(example_name(name));
    let as_msg = |e: El<()>| e.map(std::rc::Rc::new(|_| Msg::Nothing));
    let titles = [tr("docs.sec.preview"), tr("docs.sec.install"), tr("docs.sec.usage"), tr("docs.sec.examples"), tr("docs.sec.api"), tr("docs.sec.keyboard"), tr("docs.sec.rtl"), tr("docs.sec.materials")];
    let title = if l == Locale::En { name.replace('_', " ") } else { it.title.clone() };
    let desc = if l == Locale::En && !it.desc_en.is_empty() { it.desc_en.clone() } else { it.desc.clone() };
    let head = col().key(format!("doc/{h}/head")).gap(10.0)
        .child(row().wrap().gap(14.0).align(AlignItems::BASELINE).key(format!("doc/{h}/title-row"))
            .child(text(title, 34.0, Paint::INK).font(Font::Display).heading(1).key(format!("{h}/title")).flow("page-title"))
            .child(text(it.functions.join(" / "), 13.0, Paint::DIM).font(Font::Mono)))
        .child(text_lh(desc, 16.0, 27.0, Paint::DIM).key(format!("{h}/desc")));
    // 1. Preview / code.
    let first = if previews.is_empty() { None } else { Some(as_msg(previews.remove(0))) };
    let shown = live.or_else(|| crate::recipes::components::examples(example_name(name)).into_iter().next().map(|e| as_msg(e).rekey(&format!("doc/{h}/live")))).unwrap_or_else(|| text("—", 14.0, Paint::DIM));
    let tabs = row().key(format!("doc/{h}/tabs")).gap(4.0).role(Role::TabList)
        .child(tab(&h, 0, &tr("docs.tab.preview"), s.doc_tab == 0)).child(tab(&h, 1, &tr("docs.tab.code"), s.doc_tab == 1));
    let pane = if s.doc_tab == 1 { code(&format!("doc/{h}/source"), ex_src) }
        else { col().key(format!("doc/{h}/preview")).fill_width().min_width(0.0).scroll_x().pad(if s.width < super::NARROW { 16.0 } else { 40.0 }, 36.0).gap(16.0).radius(6.0).border(1.0, Paint::FAINT).align(AlignItems::START).child(shown) };
    let mut preview = col().key(format!("doc/{h}/preview-col")).gap(10.0).fill_width().min_width(0.0).child(tabs).child(pane);
    if let Some(p) = params { preview = preview.child(p); }
    // 2. Installation: the add command and the files it copies.
    let files = crate::add::files(REGISTRY, &[name]).unwrap_or_default();
    let install = col().key(format!("doc/{h}/install")).gap(10.0).fill_width().min_width(0.0)
        .child(text_lh(tr("docs.install.lead"), 15.0, 26.0, Paint::TEXT))
        .child(code(&format!("doc/{h}/add"), &format!("cargo run --release --bin lab -- add {name} --to ../my-app")))
        .child(text_lh(format!("{}: {}", tr("docs.install.files"), files.join(", ")), 13.0, 22.0, Paint::DIM));
    // 3. Usage: the first example's code.
    let usage = col().key(format!("doc/{h}/usage")).gap(10.0).fill_width().min_width(0.0)
        .child(code(&format!("doc/{h}/use"), &format!("use lab::widgets::{{{}}};\n\n{}", it.functions.join(", "), examples.first().map(|e| e.1.clone()).unwrap_or_default())));
    // 4. Examples: every one, a preview and its code.
    let mut exs = col().key(format!("doc/{h}/examples")).gap(18.0).fill_width().min_width(0.0);
    let all_previews: Vec<El<Msg>> = first.into_iter().chain(previews.into_iter().map(as_msg)).collect();
    for (j, ((label, c), p)) in examples.iter().zip(all_previews).enumerate() {
        let p = p.rekey(&format!("doc/{h}/ex{j}/k"));
        exs = exs.child(col().key(format!("doc/{h}/ex{j}")).gap(8.0).fill_width()
            .child(text(pick(label, l), 15.0, Paint::INK).key(format!("doc/{h}/ex{j}/label")))
            .child(col().key(format!("doc/{h}/ex{j}/box")).fill_width().min_width(0.0).scroll_x().pad(if s.width < super::NARROW { 12.0 } else { 24.0 }, 20.0).radius(6.0).border(1.0, Paint::FAINT).align(AlignItems::START).child(p))
            .child(code(&format!("doc/{h}/ex{j}/src"), c)));
    }
    // 5. API: from the registry and the widget's own header.
    let api_md = format!("| | |\n|---|---|\n| {} | `{}` |\n| {} | {} |\n| {} | `{}` |\n| {} | {} |\n| {} | {} |\n| {} | {} |\n",
        tr("docs.api.functions"), it.functions.join("`, `"), tr("docs.api.props"), header_line(src, "Props:"), tr("docs.api.files"), it.files.join("`, `"),
        tr("docs.api.deps"), it.deps.join(", "), tr("docs.api.response"), it.response, tr("docs.api.recompose"), it.recompose);
    let api = crate::markdown::render(&api_md, &format!("doc/{h}/api"), doc_w(s, 780.0), &|t| link(t)).0;
    // 6. Keyboard and accessibility.
    let kb_md = format!("| | |\n|---|---|\n| {} | {} |\n| {} | {} |\n", tr("docs.kb.states"), header_line(src, "States:"), tr("docs.kb.keys"), header_line(src, "Messages:"));
    let kb = col().key(format!("doc/{h}/kb-col")).gap(10.0).fill_width().min_width(0.0)
        .child(crate::markdown::render(&kb_md, &format!("doc/{h}/kb"), doc_w(s, 780.0), &|t| link(t)).0)
        .child(text_lh(tr("docs.kb.mirror"), 14.0, 24.0, Paint::DIM).width(doc_w(s, 780.0)).key(format!("doc/{h}/kb-mirror")));
    // 7. Right to left: the first example in a subtree that runs right to left, and what follows.
    let rtl_ex = crate::recipes::components::examples(example_name(name)).into_iter().next().map(as_msg).unwrap_or_else(|| space(0.0, 0.0)).rekey(&format!("doc/{h}/rtl/k"));
    let rtl = col().key(format!("doc/{h}/rtl")).gap(10.0).fill_width().min_width(0.0)
        .child(text_lh(tr("docs.rtl.note"), 14.0, 24.0, Paint::TEXT).width(doc_w(s, 780.0)).key(format!("doc/{h}/rtl-note")))
        .child(row().key(format!("doc/{h}/rtl/box")).fill_width().min_width(0.0).pad(24.0, 20.0).radius(6.0).border(1.0, Paint::FAINT).rtl(true).child(rtl_ex));
    // 8. Under every material: the first example, small, once per material.
    let mut strip = row().key(format!("doc/{h}/mats")).wrap().gap(10.0).fill_width();
    let first_ex = crate::recipes::components::examples(example_name(name)).into_iter().next();
    if let Some(e0) = first_ex {
        let _ = e0;
        for m in 0..MATERIAL_COUNT as u8 {
            let mat = Material::from_id(m);
            let e = crate::recipes::components::examples(example_name(name)).into_iter().next().map(as_msg).unwrap_or_else(|| space(0.0, 0.0)).rekey(&format!("doc/{h}/mat{m}/k"));
            strip = strip.child(col().key(format!("doc/{h}/mat{m}")).gap(6.0).width(220.0)
                .child(col().key(format!("doc/{h}/mat{m}/box")).width(220.0).height(120.0).pad(12.0, 12.0).radius(6.0).border(1.0, Paint::FAINT).clip().material(mat.id()).align(AlignItems::START).child(e))
                .child(text(mat.name(), 12.0, Paint::DIM).font(Font::Mono)));
        }
    }
    let secs = [preview, install, usage, exs, api, kb, rtl, strip];
    let mut body = col().key(format!("doc/{h}")).gap(34.0).fill_width().min_width(0.0).child(head);
    for (i, (t, b)) in titles.iter().zip(secs).enumerate() { body = body.child(section(&h, i, t, b)); }
    let outline: Vec<(String, String)> = titles.iter().enumerate().map(|(i, t)| (t.clone(), format!("doc/{h}/sec{i}"))).collect();
    frame(s, &h, body, &outline)
}
fn tab(h: &str, i: usize, label: &str, on: bool) -> El<Msg> {
    let mut t = row().key(format!("doc/{h}/tab{i}")).height(30.0).pad(12.0, 0.0).radius(4.0).center().on_press(Msg::DocTab(i)).role(Role::Tab).selected(on)
        .child(text(label, 13.0, if on { Paint::ACCENT } else { Paint::TEXT }));
    if on { t = t.fill(SELECTED); }
    t
}

/// The search over every page (⌘K / Ctrl+K, or `/`): a palette in a dialog.
pub fn search(s: &Gallery) -> El<Msg> {
    let names: Vec<String> = Page::all().iter().map(|p| p.name().to_owned()).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    dialog("search", s.search, &tr("docs.search"), 460.0, 0.35, Msg::Search(false),
        command("search/palette", &s.search_q, &refs, s.search_active, Msg::SearchQuery, Msg::SearchRun))
}
/// The page a search row runs.
pub fn search_run(s: &mut Gallery, i: usize) {
    let all = Page::all();
    if let Some(p) = all.get(i) { s.search = false; s.search_q = TextEdit::default(); super::update(s, Msg::Go(*p)); }
}
/// Up / Down / Enter in the search's field.
pub fn search_key(s: &mut Gallery, k: crate::el::Key) -> bool {
    if !s.search { return false; }
    let names: Vec<String> = Page::all().iter().map(|p| p.name().to_owned()).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let found = command_matches(&s.search_q.text, &refs);
    match k {
        crate::el::Key::Down => { s.search_active = (s.search_active + 1).min(found.len().saturating_sub(1)); true }
        crate::el::Key::Up => { s.search_active = s.search_active.saturating_sub(1); true }
        crate::el::Key::Enter => { if let Some(&i) = found.get(s.search_active) { search_run(s, i); } true }
        _ => false,
    }
}
/// Where the outline is: the last section whose top has scrolled above the window's first fifth.
pub fn scrolled(s: &mut Gallery, key: &str, _off: f32, at: &dyn Fn(&str) -> Option<f32>) -> bool {
    if !key.starts_with("main/") { return false; }
    let page = s.page;
    let keys: Vec<String> = match page {
        Page::Comp(_) | Page::Widget(_) => (0..7).map(|i| format!("doc/{}/sec{i}", page.hash())).collect(),
        Page::Doc(i) => crate::markdown::parse(source(i as usize, s.locale)).iter().enumerate().filter(|(_, b)| matches!(b, crate::markdown::Block::Heading(2, _))).map(|(j, _)| format!("doc/{}/md/h{j}", DOCS[i as usize])).collect(),
        _ => return false,
    };
    let mut now = 0;
    for (i, k) in keys.iter().enumerate() { if let Some(y) = at(k) && y < s.height * 0.2 { now = i; } }
    let changed = now != s.outline_at;
    s.outline_at = now;
    changed
}
pub fn outline_go(s: &mut Gallery, key: String) { s.cmds.push(Cmd::ScrollTo { key }); }
