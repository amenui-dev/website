//! Every page that has a Markdown form, written out: `md <out_dir>` writes `<out_dir>/<locale>/<hash>.md`
//! for both languages, and `md --llms [<amenui>/llms.txt]` rewrites the page list in amenui's `llms.txt` (between its `## Pages`
//! heading and the next heading) from the same pages, one line each with its first sentence.
use website::gallery::{Page, docs};
use amenui::i18n::Locale;

/// A page's Markdown in a language.
type Md = Box<dyn Fn(Locale) -> String>;

fn first_line(md: &str) -> String {
    let line = md.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('|') && !l.starts_with("```") && !l.starts_with('-') && !l.starts_with('>')).unwrap_or("");
    let s = line.split_inclusive(['.', '。']).next().unwrap_or(line).trim();
    let s = amenui::markdown::inline(s).0;
    if s.chars().count() > 160 { format!("{}…", s.chars().take(157).collect::<String>()) } else { s }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    // The prose pages by their older hash names, then every page with a Markdown form.
    let prose: Vec<(String, Md)> = (0..docs::DOCS.len()).map(|i| (format!("docs-{}", docs::DOCS[i]), Box::new(move |l| docs::source(i, l).to_owned()) as Md))
        .chain(docs::AI.iter().map(|&(h, _, src)| (h.to_owned(), Box::new(move |_| src.to_owned()) as Md))).collect();
    let pages: Vec<(String, Md)> = prose.into_iter()
        .chain(Page::all().into_iter().filter(|p| docs::markdown(*p, Locale::En).is_some()).map(|p| (p.hash(), Box::new(move |l| docs::markdown(p, l).unwrap_or_default()) as Md))).collect();
    let title = |md: &str, h: &str| md.lines().find_map(|x| x.strip_prefix("# ")).unwrap_or(h).to_owned();
    if a.get(1).map(String::as_str) == Some("--llms") {
        amenui::i18n::set_current(Locale::En);
        let mut list = String::from("## Pages\n\nEvery documentation page as Markdown, at `md/en/<page>.md` and `md/zh-Hans/<page>.md` next to the web build (`cargo run --release --bin md -- dist/md`).\n\n");
        for (h, md) in &pages {
            let md = md(Locale::En);
            list.push_str(&format!("- [{}](md/en/{h}.md): {}\n", title(&md, h), first_line(&md)));
        }
        // amenui's llms.txt, in a checkout of the framework (next to this one by default).
        let path = a.get(2).map_or("../amenui/llms.txt", String::as_str);
        let src = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let out = match src.find("## Pages") {
            Some(i) => { let rest = &src[i + 8..]; let j = rest.find("\n## ").map_or(src.len(), |j| i + 8 + j + 1); format!("{}{}\n{}", &src[..i], list, &src[j..]) }
            None => match src.find("## Optional") { Some(i) => format!("{}{}\n{}", &src[..i], list, &src[i..]), None => format!("{src}\n{list}") },
        };
        std::fs::write(path, out).expect("write llms.txt");
        eprintln!("llms.txt: {} pages", pages.len());
        return;
    }
    let out = a.get(1).cloned().unwrap_or_else(|| "dist/md".into());
    for l in website::LOCALES {
        amenui::i18n::set_current(l);
        let dir = format!("{out}/{}", l.tag());
        std::fs::create_dir_all(&dir).expect("out dir");
        for (h, md) in &pages { std::fs::write(format!("{dir}/{h}.md"), md(l)).expect("write"); }
    }
    eprintln!("{} pages × {} languages in {out}", pages.len(), website::LOCALES.len());
}
