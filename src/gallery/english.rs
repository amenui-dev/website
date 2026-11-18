//! The demos in English whatever the interface language: the words the site's and the framework's
//! examples are written with (`assets/english.txt`, which the web module fetches: `texts`) put in English on the way to the screen. A text
//! is looked up whole (numbers in it stand for `{n}`), then by its quoted strings (a line of code),
//! then by the longer phrases in it. Editable text is left as the person typed it.
use amenui::el::{Content, El};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::OnceLock;

struct Table { whole: HashMap<String, String>, phrases: Vec<(String, String)> }

fn table() -> &'static Table {
    // `assets/english.txt` is one of the texts the web module fetches (`texts`): until it has come the
    // table is the catalogs' alone, and it is not kept.
    static T: OnceLock<Table> = OnceLock::new();
    static PART: OnceLock<Table> = OnceLock::new();
    match crate::texts::get("english.txt") {
        Some(words) => T.get_or_init(|| make(words)),
        None => PART.get_or_init(|| make("")),
    }
}
fn make(words: &str) -> Table {
    {
        let mut whole = HashMap::new();
        for line in words.lines() {
            if line.starts_with('#') { continue; }
            let Some((k, v)) = line.split_once(" = ") else { continue };
            whole.insert(k.replace("\\n", "\n"), v.replace("\\n", "\n"));
        }
        // And every message of the site's and the framework's catalogs, from its Chinese to its English
        // (named arguments stand for numbers, as `{n}`).
        let files: Vec<(amenui::i18n::Locale, &str)> = amenui::i18n::files().iter().map(|f| (f.0, f.2)).chain(crate::CATALOGS.iter().map(|f| (f.0, f.2))).collect();
        let parse = |l: amenui::i18n::Locale| -> HashMap<String, String> {
            let mut m = HashMap::new();
            for (_, src) in files.iter().filter(|f| f.0 == l) {
                for line in src.lines() {
                    if line.starts_with('#') { continue; }
                    if let Some((k, v)) = line.split_once(" = ") { m.insert(k.trim().to_owned(), v.to_owned()); }
                }
            }
            m
        };
        let (zh, en) = (parse(amenui::i18n::Locale::ZhHans), parse(amenui::i18n::Locale::En));
        let args = |v: &str| { let mut out = String::new(); let mut rest = v; while let Some(a) = rest.find('{') { let Some(b) = rest[a..].find('}') else { break }; out.push_str(&rest[..a]); out.push_str("{n}"); rest = &rest[a + b + 1..]; } out.push_str(rest); out };
        for (k, z) in &zh {
            let Some(e) = en.get(k).or_else(|| k.strip_suffix(".other").and_then(|b| en.get(&format!("{b}.other")))) else { continue };
            let (z, e) = (args(z).replace("\\n", "\n"), args(e).replace("\\n", "\n"));
            if cjk(&z) && z.matches("{n}").count() == e.matches("{n}").count() { whole.entry(z).or_insert(e); }
        }
        // Phrases of two or more characters without numbers, longest first, for texts made of several.
        let mut phrases: Vec<(String, String)> = whole.iter().filter(|(k, _)| k.chars().count() >= 2 && !k.contains('{')).map(|(k, v)| (k.clone(), v.clone())).collect();
        phrases.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(a.0.cmp(&b.0)));
        Table { whole, phrases }
    }
}

fn cjk(s: &str) -> bool { s.chars().any(|c| ('\u{2e80}'..='\u{9fff}').contains(&c) || ('\u{f900}'..='\u{faff}').contains(&c) || ('\u{ff00}'..='\u{ffef}').contains(&c) || ('\u{3000}'..='\u{303f}').contains(&c)) }

/// The whole text's English: numbers in it stand for `{n}` and are put back in order.
fn whole(s: &str) -> Option<String> {
    let t = table();
    if let Some(v) = t.whole.get(s) { return Some(v.clone()); }
    let (mut key, mut nums, mut digits) = (String::new(), vec![], String::new());
    for c in s.chars().chain(std::iter::once('\0')) {
        if c.is_ascii_digit() { digits.push(c); continue; }
        if !digits.is_empty() { key.push_str("{n}"); nums.push(std::mem::take(&mut digits)); }
        if c != '\0' { key.push(c); }
    }
    let v = t.whole.get(&key)?;
    let mut out = String::new();
    let mut it = nums.into_iter();
    let mut rest = v.as_str();
    while let Some(i) = rest.find("{n}") { out.push_str(&rest[..i]); out.push_str(&it.next().unwrap_or_default()); rest = &rest[i + 3..]; }
    out.push_str(rest);
    Some(out)
}

/// A text in English, as far as the table knows it.
pub fn english(s: &str) -> String {
    if !cjk(s) { return s.to_owned(); }
    if let Some(v) = whole(s) { return v; }
    // Several lines: each on its own.
    if s.contains('\n') { return s.split('\n').map(english).collect::<Vec<_>>().join("\n"); }
    let mut out = s.to_owned();
    // A line of code: its quoted strings.
    if out.contains('"') {
        let mut done = String::new();
        let mut rest = out.as_str();
        while let Some(a) = rest.find('"') {
            done.push_str(&rest[..=a]);
            let tail = &rest[a + 1..];
            let Some(b) = tail.find('"') else { rest = tail; break };
            let lit = &tail[..b];
            // (A string literal writes a line break as `\n`.)
            let en = || whole(lit).or_else(|| whole(&lit.replace("\\n", "\n")).map(|v| v.replace('\n', "\\n")));
            done.push_str(&if cjk(lit) { en().unwrap_or_else(|| lit.to_owned()) } else { lit.to_owned() });
            done.push('"');
            rest = &tail[b + 1..];
        }
        done.push_str(rest);
        out = done;
    }
    // Texts made of several: the longer phrases in them.
    if cjk(&out) { for (k, v) in &table().phrases { if out.contains(k.as_str()) { out = out.replace(k.as_str(), v); } } }
    out
}

/// The tree with its words in English; `skip` keeps a subtree as it is (chrome, prose).
pub fn tree<M: 'static>(mut e: El<M>, skip: Rc<dyn Fn(&str) -> bool>) -> El<M> {
    if skip(&e.key) { return e; }
    if e.on_text.is_none() && let Content::Text { s, .. } = &mut e.content && cjk(s) { *s = english(s); }
    if let Some(a) = &mut e.access {
        if let Some(l) = &mut a.label && cjk(l) { *l = english(l); }
        if e.on_text.is_none() && let Some(t) = &mut a.text && cjk(t) { *t = english(t); }
    }
    if let Some(v) = &mut e.virt {
        let (b, k) = (v.build.clone(), skip.clone());
        v.build = Rc::new(move |i| tree(b(i), k.clone()));
    }
    e.children = std::mem::take(&mut e.children).into_iter().map(|c| tree(c, skip.clone())).collect();
    e
}

/// What a key does to the English of the subtree under it (`demos`).
#[derive(Clone, Copy, PartialEq)]
pub enum Part {
    /// No change: as its parent.
    Same,
    /// The page under the chrome: in English, unless chrome inside it says otherwise.
    Page,
    /// Chrome or prose: as written, unless a demo inside it says otherwise.
    Chrome,
    /// A demo: in English, all of it.
    Demo,
}

/// The tree with its demos in English and its chrome and prose as written, by what `part` says of
/// each key. (0: as written, 1: in English under a page, 2: in English inside a demo.)
pub fn demos<M: 'static>(e: El<M>, part: Rc<dyn Fn(&str) -> Part>) -> El<M> { walk(e, 0, part) }
fn walk<M: 'static>(mut e: El<M>, mode: u8, part: Rc<dyn Fn(&str) -> Part>) -> El<M> {
    let mode = if mode == 2 { 2 } else { match part(&e.key) { Part::Same => mode, Part::Page => 1, Part::Chrome => 0, Part::Demo => 2 } };
    if mode == 2 { return tree(e, Rc::new(|_| false)); }
    if mode == 1 {
        if e.on_text.is_none() && let Content::Text { s, .. } = &mut e.content && cjk(s) { *s = english(s); }
        if let Some(a) = &mut e.access {
            if let Some(l) = &mut a.label && cjk(l) { *l = english(l); }
            if e.on_text.is_none() && let Some(t) = &mut a.text && cjk(t) { *t = english(t); }
        }
    }
    if let Some(v) = &mut e.virt {
        let (b, p) = (v.build.clone(), part.clone());
        v.build = Rc::new(move |i| walk(b(i), mode, p.clone()));
    }
    e.children = std::mem::take(&mut e.children).into_iter().map(|c| walk(c, mode, part.clone())).collect();
    e
}
