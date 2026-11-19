//! The Lab's reading path (site-design Z.5): five steps, 29 cells in the table's order, no cell alone
//! in a row, previous and next through the whole path, every old address leading on, and the path
//! line taking a press to its step.
use website::gallery::{self, GalleryUi, Msg, Page, lab, routes};

/// A row of cells: its top, and each cell's left, width and height.
type Row = (f32, Vec<(f32, f32, f32)>);

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }

fn index(w: u32, h: u32, l: amenui::i18n::Locale) -> GalleryUi {
    let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
    gallery::update(&mut ui.state, Msg::Locale(l));
    gallery::update(&mut ui.state, Msg::Go(Page::Lab));
    ui.relayout();
    let t = ui.now();
    for k in 1..=30 { ui.tick(t + k as f32 / 60.0); ui.frame_drawn(); }
    ui
}

#[test]
fn the_index_is_five_steps_of_29_cells_none_alone_in_a_row() {
    let path = lab::path();
    assert_eq!(path.iter().map(Vec::len).collect::<Vec<_>>(), vec![7, 6, 6, 3, 7]);
    for l in [amenui::i18n::Locale::En, amenui::i18n::Locale::from_tag("zh-Hans").expect("zh")] {
        for w in [1440, 1146, 1024, 390, 320] {
            let ui = index(w, 900, l);
            let mut cells = vec![];
            for (s, pages) in path.iter().enumerate() {
                let head = ui.node(&format!("labx/{}/head", lab::STEPS[s])).unwrap_or_else(|| panic!("step {s}'s header at {w}"));
                let _ = head;
                let mut rows: Vec<Row> = vec![];
                for p in pages {
                    let r = ui.node(&format!("labi/{}", p.hash())).unwrap_or_else(|| panic!("{p:?}'s cell at {w}")).rect;
                    cells.push((r.y, r.x, *p));
                    match rows.iter_mut().find(|(y, _)| (*y - r.y).abs() < 1.0) { Some((_, v)) => v.push((r.x, r.w, r.h)), None => rows.push((r.y, vec![(r.x, r.w, r.h)])) }
                }
                let cols = rows.iter().map(|(_, v)| v.len()).max().unwrap_or(1);
                let one = rows.iter().flat_map(|(_, v)| v.iter()).map(|c| c.1).fold(f32::MAX, f32::min);
                for (_, v) in &rows {
                    if cols > 1 { assert!(v.len() > 1 || v[0].1 > one * 1.5, "a cell alone in a row of step {s} at {w}: {rows:?}"); }
                    for c in v {
                        let wide = c.1 > one * 1.5;
                        if wide { assert!((c.1 - (2.0 * one + 16.0)).abs() <= 1.0, "a wide cell is two cells and a gap at {w}: {c:?} {one}"); }
                        else { assert!((c.1 - one).abs() <= 1.0, "cells of one width at {w}: {c:?} {one}"); }
                        assert!((c.2 - v[0].2).abs() <= 1.0 && (c.2 - rows[0].1[0].2).abs() <= 1.0, "cells of one height at {w}: {rows:?}");
                    }
                }
            }
            // Read row by row, the cells are in the table's order.
            let flat: Vec<Page> = path.iter().flatten().copied().collect();
            let mut by_place = cells.clone();
            by_place.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(a.1.partial_cmp(&b.1).unwrap()));
            assert_eq!(by_place.iter().map(|c| c.2).collect::<Vec<_>>(), flat, "the order at {w}");
        }
    }
}

#[test]
fn next_runs_through_all_29_and_previous_back() {
    let flat: Vec<Page> = lab::path().iter().flatten().copied().collect();
    let mut at = flat[0];
    let mut seen = vec![at];
    assert!(lab::neighbours(at).unwrap().0.is_none(), "01.1 has no previous");
    loop {
        let (_, (next, label)) = lab::neighbours(at).unwrap();
        if next == Page::Lab { assert!(label.ends_with('↑'), "{label}"); break; }
        seen.push(next);
        at = next;
    }
    assert_eq!(seen, flat);
    let mut back = vec![at];
    while let Some((p, _)) = lab::neighbours(at).unwrap().0 { back.push(p); at = p; }
    back.reverse();
    assert_eq!(back, flat);
    let names: std::collections::HashSet<String> = flat.iter().map(|&p| lab::cell_name(p).to_lowercase()).collect();
    assert_eq!(names.len(), 29, "no two cells share a name");
}

#[test]
fn every_old_address_leads_to_a_page() {
    let all = Page::all();
    let moved = routes::moved();
    for old in ["lab/scenes/", "scenes/", "lab/scenes/reading/", "scenes/xray/", "lab/looks/presets/", "lab/looks/marks/braille/", "lab/looks/materials/dottr/", "lab/looks/annotation/", "lab/looks/type/", "lab/motion/morph/", "lab/motion/particles/", "lab/motion/flow/", "lab/motion/three/"] {
        assert!(moved.iter().any(|(p, _)| p == old), "{old} is in the table");
    }
    for (old, p) in &moved {
        assert!(all.contains(p), "{old} leads to {p:?}, which is a page");
        assert_eq!(Page::from_path(old), Some(*p));
        assert!(all.iter().all(|q| q.path() != *old), "{old} is no page's own address");
    }
}

#[test]
fn a_press_on_the_path_line_brings_its_step_under_it() {
    let mut ui = index(1440, 900, amenui::i18n::Locale::En);
    let r = ui.node("lab/path/material").expect("the path line").rect;
    let line = ui.node("lab/path").expect("the path line").rect;
    let mut t = ui.now();
    ui.pointer_move(r.x + r.w * 0.5, r.y + r.h * 0.5);
    ui.press(r.x + r.w * 0.5, r.y + r.h * 0.5);
    for _ in 0..120 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
    let h = ui.node("labx/material/head").expect("the header").rect;
    let below = h.y - (line.y + line.h);
    assert!((0.0..=9.0).contains(&below), "the header {below} px under the line");
    assert_eq!(lab::here(&ui.state), 2, "the brackets move to it");
}

#[test]
fn a_family_strip_shows_every_member_once() {
    for w in [1440u32, 1024, 390] {
        for f in 0..3 {
            let members = lab::family(f);
            for at in [0, members.len() / 2, members.len() - 1] {
                let p = members[at];
                let mut ui = gallery::new_ui(fonts(), w, 900, 1.0);
                gallery::update(&mut ui.state, Msg::Go(p));
                ui.relayout();
                let mut t = ui.now();
                for _ in 0..20 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
                let line = ui.node("strip/members").unwrap_or_else(|| panic!("the members at {w}")).rect;
                for q in &members {
                    let r = ui.node(&format!("strip/m/{}", q.hash())).unwrap_or_else(|| panic!("{q:?} in the strip at {w}")).rect;
                    if w >= 720 { assert!(r.x >= line.x - 0.5 && r.right() <= line.right() + 0.5, "{q:?} cut at {w}: {r:?} in {line:?}"); }
                    if *q == p { assert!(r.x >= 0.0 && r.right() <= w as f32, "{p:?}, the page's own, in view at {w}: {r:?}"); }
                }
                // A run of marks is never broken across lines.
                if f == 1 && w >= 720 {
                    for (a, b) in lab::MARK_RUNS.iter().zip(lab::MARK_RUNS.iter().skip(1).chain([&members.len()])) {
                        let ys: Vec<f32> = members[*a..*b].iter().map(|q| ui.node(&format!("strip/m/{}", q.hash())).unwrap().rect.y).collect();
                        assert!(ys.iter().all(|y| (y - ys[0]).abs() < 1.0), "a run broken at {w}: {ys:?}");
                    }
                }
                // The strip is the one list of members.
                assert!(ui.node("strip/m-prev").is_none() && ui.node("strip/m-next").is_none(), "no previous and next of the family's own");
                assert!(!ui.nodes().iter().any(|n| n.key.contains("/chip")), "no picker of marks under the picture");
                assert!(ui.focus().is_none(), "nothing keeps the focus at {w}");
            }
        }
    }
}

#[test]
fn the_keyboard_s_focus_runs_a_cell_and_its_stage_alone_is_made_again() {
    let mut ui = index(1440, 900, amenui::i18n::Locale::En);
    let orbit = lab::path()[1][2];
    let key = format!("labi/{}/stage", orbit.hash());
    // Brought to its still, nothing runs.
    let mut t = ui.now();
    for _ in 0..120 { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
    assert!(lab::step(&mut ui.state, t + 1.0 / 60.0).is_empty(), "a still steps no more");
    ui.focus_key(&format!("labx/{}/name", orbit.hash()));
    assert_eq!(ui.state.lab.focus, Some(7), "its name has the focus: Orbit runs");
    let stepped = lab::step(&mut ui.state, t + 2.0 / 60.0);
    assert_eq!(stepped, vec![key.clone()], "only its stage changed");
    let st = lab::stage_of(&ui.state, &key).expect("its stage alone");
    assert_eq!(st.key, key);
    ui.focus_key("lab/path/ink");
    assert_eq!(ui.state.lab.focus, None, "the focus moved on: it rests");
}

#[test]
#[ignore = "prints the next chain: cargo test --profile quick --test lab_path chain -- --ignored --nocapture"]
fn chain() {
    let _ui = index(390, 400, amenui::i18n::Locale::En);
    for p in lab::path().iter().flatten() {
        let (s, i) = lab::place(*p).unwrap();
        let (prev, (_, next)) = lab::neighbours(*p).unwrap();
        println!("| {} | {} | /{} | {} | {} |", lab::number(s, i), lab::cell_name(*p).to_lowercase(), p.path(), prev.map(|x| x.1).unwrap_or_default(), next);
    }
}
