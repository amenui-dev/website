//! The mark atlas, palettes, pixel faces and the compare wipe, without a GPU.
use amenui::demo;
use amenui::marks::{self, Atlas, LEVELS, TILE};
use amenui::style::set;
use amenui::text::{MONO, PIXEL, SANS, Text, load_bundled};

fn text() -> Text { Text::new(load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts")) }
fn atlas() -> Atlas { Atlas::build(&mut text(), MONO) }
fn ink(a: &Atlas, s: u8, level: usize) -> f32 {
    (0..TILE * TILE).map(|i| a.at(s as usize, level, i % TILE, i / TILE).0).sum()
}

#[test]
fn bayer_is_a_permutation() {
    let mut seen = [false; 64];
    for y in 0..8 { for x in 0..8 { seen[marks::bayer8(x, y) as usize] = true; } }
    assert!(seen.iter().all(|&b| b));
}

#[test]
fn level_zero_is_empty_where_marks_carry_tone() {
    let a = atlas();
    // Pixel-art, mosaic and brick are solid at every level: their tone is the colour, not the mark.
    // The LED matrix keeps a faint seat for an unlit lamp, so the matrix reads in the dark.
    for s in (0..set::COUNT as u8).filter(|s| ![set::PIXEL, set::MOSAIC, set::BRICK, set::LED].contains(s)) {
        assert_eq!(ink(&a, s, 0), 0.0, "set {} level 0 is not empty", set::NAMES[s as usize]);
        assert!(ink(&a, s, LEVELS - 1) > ink(&a, s, 0), "set {} has no ink at its last level", set::NAMES[s as usize]);
    }
    for s in [set::PIXEL, set::MOSAIC, set::BRICK] {
        for l in 0..LEVELS { assert!(ink(&a, s, l) > (TILE * TILE) as f32 * 0.8, "{} is not solid", set::NAMES[s as usize]); }
    }
}

#[test]
fn dither_fills_up_in_order() {
    let a = atlas();
    assert_eq!(ink(&a, set::DITHER, LEVELS - 1), (TILE * TILE) as f32, "last dither level is not full");
    // Every level lights what the one before lit, and more.
    for l in 1..LEVELS {
        for y in 0..TILE { for x in 0..TILE {
            assert!(a.at(set::DITHER as usize, l, x, y).0 >= a.at(set::DITHER as usize, l - 1, x, y).0);
        } }
        assert!(ink(&a, set::DITHER, l) > ink(&a, set::DITHER, l - 1));
    }
}

#[test]
fn geometric_marks_grow_with_level() {
    let a = atlas();
    for s in [set::DOTS, set::CROSS, set::DIAGONAL, set::DIAMOND, set::LINES, set::GLOW, set::STROKE] {
        for l in 1..LEVELS { assert!(ink(&a, s, l) > ink(&a, s, l - 1), "{} level {l}", set::NAMES[s as usize]); }
    }
}

#[test]
fn characters_are_ordered_by_measured_ink() {
    let a = atlas();
    assert_eq!(a.chars[0], ' ');
    let chars: std::collections::HashSet<char> = a.chars.iter().copied().collect();
    assert_eq!(chars.len(), LEVELS, "a character was picked twice: {:?}", a.chars);
    for l in 1..LEVELS {
        assert!(a.ink[l] > a.ink[l - 1], "{:?} ink {:?}", a.chars, a.ink);
        // The ink is what is really in the atlas, not an estimate.
        assert!((ink(&a, set::CHARS, l) - a.ink[l]).abs() < (TILE * TILE) as f32 * 0.002);
    }
}

#[test]
fn tiles_wrap_without_seams() {
    let a = atlas();
    for s in 0..set::COUNT {
        for l in 0..LEVELS {
            for ch in 0..2 {
                let v = |x: usize, y: usize| { let p = a.at(s, l, x, y); if ch == 0 { p.0 } else { p.1 } };
                // The step across the wrap is no larger than the largest step inside the tile.
                let mut inside: f32 = 0.0;
                for y in 0..TILE { for x in 0..TILE - 1 { inside = inside.max((v(x + 1, y) - v(x, y)).abs()).max((v(y, x + 1) - v(y, x)).abs()); } }
                let mut seam: f32 = 0.0;
                for i in 0..TILE { seam = seam.max((v(0, i) - v(TILE - 1, i)).abs()).max((v(i, 0) - v(i, TILE - 1)).abs()); }
                assert!(seam <= inside + 1.0 / 255.0, "set {} level {l} channel {ch}: seam {seam} > inside {inside}", set::NAMES[s]);
            }
        }
    }
}

#[test]
fn mips_halve_down_to_one_texel_per_tile() {
    let m = atlas().mips();
    assert_eq!(m.len(), marks::MIPS);
    let (w, h, _) = m.last().unwrap();
    assert_eq!((*w, *h), (LEVELS, set::COUNT));
}

#[test]
fn palettes_are_full_rows() {
    assert_eq!(marks::palette_pixels().len(), 16 * 4 * marks::PALETTES.len());
    assert!(marks::PALETTES.iter().any(|p| p.1 == 2) && marks::PALETTES.iter().any(|p| p.1 == 3));
}

#[test]
fn pixel_faces_snap_and_cut_at_half() {
    let mut t = text();
    t.set_scale(1.0);
    assert_eq!(t.size_for(PIXEL, 16.0), 12.0);
    assert_eq!(t.size_for(PIXEL, 20.0), 24.0);
    assert_eq!(t.size_for(MONO, 30.0), 33.0);
    assert_eq!(t.size_for(SANS, 15.5), 15.5);
    t.set_scale(2.0);
    // At 2× the native size is reached with half the logical size.
    assert_eq!(t.size_for(PIXEL, 6.0), 6.0);
    assert_eq!(t.size_for(PIXEL, 12.0), 12.0);
    for (face, px) in [(PIXEL, 12.0), (PIXEL, 24.0), (MONO, 11.0), (MONO, 22.0)] {
        let g = t.raster(face, '像', px).or_else(|| t.raster(face, 'A', px)).expect("glyph");
        assert!(g.data.iter().all(|&c| c == 0 || c == 255), "face {face} at {px}: grey coverage");
    }
    let g = t.raster(SANS, '像', 16.0).expect("glyph");
    assert!(g.data.iter().any(|&c| c != 0 && c != 255), "the outline face keeps its anti-aliasing");
}

#[test]
fn dragging_the_picture_moves_the_compare_line() {
    let mut ui = demo::new_ui(load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts"), 1200, 6000, 1.0);
    let r = ui.node("marks-picture").expect("marks picture").rect;
    ui.press(r.x + r.w * 0.25, r.y + 10.0);
    assert_eq!(ui.state.wipe, 0.25);
    ui.pointer_move(r.x + r.w * 0.8, r.y + 30.0);
    assert_eq!(ui.state.wipe, 0.8);
    ui.release(r.x + r.w * 0.8, r.y + 30.0);
    ui.pointer_move(r.x + r.w * 0.1, r.y + 30.0);
    assert_eq!(ui.state.wipe, 0.8, "moving after release does not drag");
    // The line is a half-plane source in the field, over the picture.
    let f = ui.field();
    assert_eq!(f.wipe[0], r.x + r.w * 0.8);
    assert_eq!(f.wipe[1], 1.0);
    assert_eq!(f.wipe_rect, [r.x, r.y, r.right(), r.bottom()]);
}

#[test]
fn theme_picks_faces_for_the_page() {
    let mut ui = demo::new_ui(load_bundled(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("fonts"), 1200, 900, 1.0);
    let face = |ui: &demo::DemoUi, k: &str| match &ui.node(k).expect("node").content {
        amenui::el::Content::Text { font, .. } => *font,
        _ => panic!("not text"),
    };
    assert_eq!(face(&ui, "count"), amenui::el::Font::Face(SANS));
    let (x, y) = ui.node("theme:2").expect("theme choice").rect.center();
    ui.press(x, y);
    assert_eq!(face(&ui, "count"), amenui::el::Font::Face(PIXEL));
    // Every theme can be chosen, including the first one again.
    let (x, y) = ui.node("theme:0").expect("theme choice").rect.center();
    ui.press(x, y);
    assert_eq!(ui.state.theme, 0);
}


#[test]
fn characters_are_cut_sharp_at_the_pitch_they_are_shown() {
    let c = marks::char_tiles(&mut text(), MONO);
    for p in marks::CHAR_MIN..=marks::CHAR_MAX {
        let chosen = c.chosen(p);
        // Strictly more ink level by level, and level 0 is empty.
        for l in 1..LEVELS { assert!(chosen[l].2 > chosen[l - 1].2, "pitch {p} level {l}: {chosen:?}"); }
        assert!((0..p * p).all(|i| c.at(p, 0, i % p, i / p) == 0), "pitch {p} level 0 is not empty");
        if p >= 8 {
            // Whole texels only: nothing resampled, so nothing grey.
            for l in 0..LEVELS { for i in 0..p * p { let v = c.at(p, l, i % p, i / p); assert!(v == 0 || v == 255, "pitch {p} level {l} has grey"); } }
        }
    }
    // The pitches the owner reads at: every one of the 16 levels is a different character.
    for p in 8..=10 {
        let mut seen: Vec<(char, bool)> = c.chosen(p).iter().map(|x| (x.0, x.1)).collect();
        seen.dedup();
        assert_eq!(seen.len(), LEVELS, "pitch {p}");
    }
}
