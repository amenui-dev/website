//! Coming back to the Lab's index (site-design Z.5, review items 2 and 3): Back puts the cell that
//! was opened where it was, and `/lab/#<step>` opens at that step's header with the brackets on it,
//! from the first frame.
use website::gallery::{self, GalleryUi, Msg, Page, lab};

fn fonts() -> Vec<amenui::text::Face> { amenui::text::load_bundled(amenui::text::ASSETS).expect("fonts") }

fn frames(ui: &mut GalleryUi, n: usize) {
    let mut t = ui.now();
    for _ in 0..n { t += 1.0 / 60.0; ui.tick(t); ui.frame_drawn(); }
}

fn top(ui: &GalleryUi, key: &str) -> f32 { ui.node(key).unwrap_or_else(|| panic!("{key}")).rect.y }

/// The header's distance under the path line's lower edge.
fn under_line(ui: &GalleryUi, step: &str) -> f32 {
    let line = ui.node("lab/path").expect("the path line").rect;
    top(ui, &format!("labx/{step}/head")) - (line.y + line.h)
}

#[test]
fn back_puts_the_opened_cell_where_it_was() {
    for (w, h) in [(1440, 900), (390, 844)] {
        let mut ui = gallery::new_ui(fonts(), w, h, 1.0);
        assert!(ui.state.route("lab/"));
        ui.relayout();
        frames(&mut ui, 10);
        // Down the page to the 12th cell (02.5 on four columns, further on one).
        let cell = format!("labi/{}", Page::Experiment(0).hash());
        for _ in 0..200 {
            if top(&ui, &cell) < h as f32 * 0.5 { break; }
            ui.wheel(w as f32 * 0.5, h as f32 * 0.5, 120.0);
            frames(&mut ui, 2);
        }
        frames(&mut ui, 30);
        let y = top(&ui, &cell);
        // Its name under the frame opens it.
        let r = ui.node(&format!("labx/{}/name", Page::Experiment(0).hash())).unwrap().rect;
        ui.pointer_move(r.x + 4.0, r.y + r.h * 0.5);
        ui.press(r.x + 4.0, r.y + r.h * 0.5);
        frames(&mut ui, 90);
        assert_eq!(ui.state.page, Page::Experiment(0), "the cell opens its piece at {w}");
        // Back: the host routes to the index's address.
        assert!(ui.state.route("lab/"));
        ui.relayout();
        frames(&mut ui, 1);
        let back = top(&ui, &cell);
        println!("back at {w}: the cell was at {y:.1}, is at {back:.1} in the first frame");
        assert!((back - y).abs() <= 8.0, "at {w} the cell was at {y}, is at {back} after Back");
    }
}

#[test]
fn an_address_with_a_step_opens_at_its_header() {
    for w in [1440, 390] {
        for (i, step) in lab::STEPS.iter().enumerate() {
            // Opened cold.
            let mut ui = gallery::new_ui(fonts(), w, 900, 1.0);
            assert!(ui.state.route(&format!("lab/#{step}")));
            ui.relayout();
            frames(&mut ui, 1);
            let d = under_line(&ui, step);
            println!("cold {step} at {w}: {d:.1} px under the line, brackets on {}", lab::here(&ui.state));
            assert!((0.0..=8.0).contains(&d), "cold {step} at {w}: the header {d} px under the line");
            assert_eq!(lab::here(&ui.state), i, "cold {step} at {w}: the brackets");
            // From a piece's strip: Kern's page, then its step's link.
            let mut ui = gallery::new_ui(fonts(), w, 900, 1.0);
            assert!(ui.state.route(&format!("lab/{}/", lab::NAMES[7])));
            ui.relayout();
            frames(&mut ui, 10);
            gallery::update(&mut ui.state, Msg::Lab(lab::LMsg::GoStep(i)));
            ui.relayout();
            frames(&mut ui, 1);
            let d = under_line(&ui, step);
            println!("strip {step} at {w}: {d:.1} px under the line, brackets on {}", lab::here(&ui.state));
            assert!((0.0..=8.0).contains(&d), "strip {step} at {w}: the header {d} px under the line");
            assert_eq!(lab::here(&ui.state), i, "strip {step} at {w}: the brackets");
        }
    }
}
