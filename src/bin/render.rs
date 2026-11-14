//! One deterministic frame of any gallery page or recipe, and its tree (DESIGN.md §1.17). Fixed 60 Hz
//! clock from 0, fixed seeds: the same arguments give byte-identical PNGs.
//!
//! usage: render --page <hash | recipe-name> [--size 1280x800] [--scale 1] [--material <id>] [--theme <id>]
//!               [--at <ms>] [--events <file.json>] [--out out.png] [--tree out.json] [--strict]
//!
//! `--events` is a JSON list such as `[{"at_ms": 200, "kind": "press", "target": "button/live"},
//! {"at_ms": 260, "kind": "release", "target": "button/live"}]` (kinds: press, release, move, key, wheel,
//! text; see `lab::drive::Input`). Problems in the view (duplicate keys…) are printed; `--strict` makes
//! them an error.
use lab::drive::{RenderSpec, parse_events, render};

const USAGE: &str = "usage: render --page <hash | recipe-name> [--size 1280x800] [--scale 1] [--material <id>] [--theme <id>] [--at <ms>] [--events <file.json>] [--out out.png] [--tree out.json] [--strict]";

fn fail(msg: impl std::fmt::Display) -> ! {
    eprintln!("render: {msg}");
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut spec = RenderSpec { page: String::new(), w: 1280, h: 800, scale: 1.0, material: None, theme: None, at_ms: 1000.0, events: vec![] };
    let (mut out, mut tree, mut strict) = ("out.png".to_owned(), None, false);
    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        if flag == "--strict" { strict = true; i += 1; continue; }
        if flag == "--help" || flag == "-h" { println!("{USAGE}"); return; }
        let Some(v) = args.get(i + 1).cloned() else { fail(format!("`{flag}` needs a value.\n{USAGE}")) };
        match flag {
            "--page" => spec.page = v,
            "--size" => {
                let (w, h) = v.split_once('x').unwrap_or_else(|| fail(format!("`--size {v}`: write width x height in device px, e.g. `--size 1280x800`.")));
                spec.w = w.parse().unwrap_or_else(|_| fail(format!("`--size {v}`: `{w}` is not a whole number of px.")));
                spec.h = h.parse().unwrap_or_else(|_| fail(format!("`--size {v}`: `{h}` is not a whole number of px.")));
            }
            "--scale" => spec.scale = v.parse().unwrap_or_else(|_| fail(format!("`--scale {v}`: write device px per logical px, e.g. 1 or 2."))),
            "--material" => spec.material = Some(v),
            "--theme" => spec.theme = Some(v),
            "--at" => spec.at_ms = v.parse().unwrap_or_else(|_| fail(format!("`--at {v}`: write the time of the frame in ms, e.g. 1000."))),
            "--events" => {
                let json = std::fs::read_to_string(&v).unwrap_or_else(|e| fail(format!("`--events {v}` could not be read ({e}).")));
                spec.events = parse_events(&json, &v).unwrap_or_else(|e| fail(e));
            }
            "--out" => out = v,
            "--tree" => tree = Some(v),
            _ => fail(format!("unknown flag `{flag}`.\n{USAGE}")),
        }
        i += 2;
    }
    if spec.page.is_empty() { fail(format!("`--page` is required.\n{USAGE}")); }
    let faces = lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).unwrap_or_else(|e| fail(e));
    let r = render(&spec, faces, None).unwrap_or_else(|e| fail(e));
    for p in &r.problems { eprintln!("render: problem: {p}"); }
    std::fs::write(&out, &r.png).unwrap_or_else(|e| fail(format!("`--out {out}` could not be written ({e}).")));
    if let Some(t) = tree {
        let json = serde_json::to_string_pretty(&r.tree).unwrap_or_else(|e| fail(e));
        std::fs::write(&t, json).unwrap_or_else(|e| fail(format!("`--tree {t}` could not be written ({e}).")));
    }
    eprintln!("render: {} {}x{} at {} ms -> {out}", spec.page, spec.w, spec.h, spec.at_ms);
    if strict && !r.problems.is_empty() { std::process::exit(1); }
}
