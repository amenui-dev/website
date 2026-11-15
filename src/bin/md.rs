//! Every page that has a Markdown form, written out: `md <out_dir>` writes `<out_dir>/<locale>/<hash>.md`
//! for both languages, and `md --llms` rewrites the page list in `llms.txt` (between its `## Pages`
//! heading and the next heading) from the same pages, one line each with its first sentence.
use lab::gallery::{Page, docs};
use lab::i18n::Locale;

fn first_line(md: &str) -> String {
    let line = md.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('|') && !l.starts_with("```") && !l.starts_with('-') && !l.starts_with('>')).unwrap_or("");
    let s = line.split_inclusive(['.', '。']).next().unwrap_or(line).trim();
    let s = lab::markdown::inline(s).0;
    if s.chars().count() > 160 { format!("{}…", s.chars().take(157).collect::<String>()) } else { s }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let pages: Vec<Page> = Page::all().into_iter().filter(|p| docs::markdown(*p, Locale::En).is_some()).collect();
    if a.get(1).map(String::as_str) == Some("--llms") {
        lab::i18n::set_current(Locale::En);
        let mut list = String::from("## Pages\n\nEvery documentation page as Markdown, at `md/en/<page>.md` and `md/zh-Hans/<page>.md` next to the web build (`cargo run --release --bin md -- dist/md`).\n\n");
        for p in &pages {
            let md = docs::markdown(*p, Locale::En).unwrap_or_default();
            list.push_str(&format!("- [{}](md/en/{}.md): {}\n", p.name(), p.hash(), first_line(&md)));
        }
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/llms.txt");
        let src = std::fs::read_to_string(path).expect("llms.txt");
        let out = match src.find("## Pages") {
            Some(i) => { let rest = &src[i + 8..]; let j = rest.find("\n## ").map_or(src.len(), |j| i + 8 + j + 1); format!("{}{}\n{}", &src[..i], list, &src[j..]) }
            None => match src.find("## Optional") { Some(i) => format!("{}{}\n{}", &src[..i], list, &src[i..]), None => format!("{src}\n{list}") },
        };
        std::fs::write(path, out).expect("write llms.txt");
        eprintln!("llms.txt: {} pages", pages.len());
        return;
    }
    let out = a.get(1).cloned().unwrap_or_else(|| "dist/md".into());
    for l in Locale::ALL {
        lab::i18n::set_current(l);
        let dir = format!("{out}/{}", l.tag());
        std::fs::create_dir_all(&dir).expect("out dir");
        for p in &pages { std::fs::write(format!("{dir}/{}.md", p.hash()), docs::markdown(*p, l).unwrap_or_default()).expect("write"); }
    }
    eprintln!("{} pages × {} languages in {out}", pages.len(), Locale::ALL.len());
}
