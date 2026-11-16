//! Every page at a phone's width (330 logical px): no text runs past the window's right edge or its
//! visible region, unless it sits in something that scrolls sideways.
use lab::el::Content;
use lab::gallery::{self, Msg, Page};

fn fonts() -> Vec<lab::text::Face> { lab::text::load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts") }

#[test]
fn no_text_is_cut_at_330_px() {
    let mut ui = gallery::new_ui(fonts(), 330, 700, 1.0);
    ui.pointer_leave();
    let mut bad = vec![];
    for p in Page::all() {
        gallery::update(&mut ui.state, Msg::Go(p));
        ui.relayout();
        let t = ui.now();
        for i in 1..=20 { ui.tick(t + i as f32 / 30.0); }
        let ns = ui.nodes();
        for n in ns {
            let Content::Text { s, .. } = &n.content else { continue };
            if n.hidden || s.trim().is_empty() || n.rect.w <= 0.0 { continue; }
            // Inside a sideways scroller, running past is what scrolling is for.
            let mut a = n.parent;
            let mut side = false;
            while let Some(i) = a { if ns[i].horizontal { side = true; break; } a = ns[i].parent; }
            if side { continue; }
            let edge = n.clip.right().min(330.0);
            if n.rect.x < edge - 1.0 && n.rect.right() > edge + 1.0 && n.rect.y < n.clip.bottom() && n.rect.bottom() > n.clip.y {
                bad.push(format!("{}: {} ({:.0}..{:.0} > {:.0}) {:?}", p.hash(), n.key, n.rect.x, n.rect.right(), edge, s.chars().take(24).collect::<String>()));
            }
        }
    }
    assert!(bad.is_empty(), "{} cut texts:\n{}", bad.len(), bad.join("\n"));
}
