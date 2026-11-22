//! The style pages' display lines are set by `styles::fit` from widths written down beside them
//! (`styles::DISPLAY`, `styles::LED`): measured again here in each style's own faces, as shown.
use website::gallery::english::english;
use website::gallery::styles::{BOARD, DISPLAY, LED, fit, units};

fn text() -> amenui::text::Text { amenui::text::Text::new(amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts")) }

#[test]
fn the_display_lines_are_as_wide_as_written() {
    let mut t = text();
    let presets = amenui::preset::presets();
    for (i, line, size, wide, keep) in DISPLAY {
        let face = presets[i].theme().display;
        let shown = english(line);
        assert!(keep.is_empty() || shown.starts_with(keep), "{line:?} ({shown:?}) does not start with {keep:?}");
        let pieces = if keep.is_empty() { units(&shown) } else { vec![keep.to_owned()] };
        let widest = pieces.iter().map(|u| t.measure(u, face, size, size, None).0).fold(0.0f32, f32::max);
        assert!((widest - wide).abs() <= 1.0, "{line:?} ({shown:?}) at {size} px: its widest piece is {widest} px, written {wide}");
    }
    let mono = presets[1].theme().mono;
    for (s, size, wide) in LED {
        let w = t.measure(s, mono, size, size, None).0;
        assert!((w - wide).abs() <= 1.0, "{s:?} at {size} px is {w} px, written {wide}");
    }
    for (s, is_mono, size, wide) in BOARD {
        let face = if is_mono { mono } else { presets[1].theme().body };
        let shown = english(s);
        let w = t.measure(&shown, face, size, size, None).0;
        assert!((w - wide).abs() <= 1.0, "{s:?} ({shown:?}) at {size} px is {w} px, written {wide}");
    }
}

#[test]
fn a_display_line_fits_in_steps_of_two_px_and_40_at_least() {
    // As designed when it fits; else the largest even size that does, never more than designed.
    assert_eq!(fit(140.0, 463.0, 600.0), 140.0);
    assert_eq!(fit(140.0, 463.0, 354.0), 106.0);
    assert_eq!(fit(64.0, 366.0, 354.0), 60.0);
    assert_eq!(fit(120.0, 348.0, 100.0), 40.0);
    // `&` and `·` stay with the word before them.
    assert_eq!(units("Knife & wood"), ["Knife &", "wood"]);
    assert_eq!(units("Shape · color · line"), ["Shape ·", "color ·", "line"]);
    assert_eq!(units("Shape\ncolor\nline"), ["Shape", "color", "line"]);
}
