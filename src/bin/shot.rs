//! Render the demo off-screen to PNG files through a short scripted sequence. No window, no surface.
//! usage: shot <out_dir> <width> <height> <scale>     (device px)
use amenui::demo::{self, DemoUi};
use amenui::el::Key;
use amenui::offscreen::{Offscreen, save_png};

struct Shooter { o: Offscreen, out: String }

impl Shooter {
    fn shoot(&mut self, ui: &mut DemoUi, name: &str) {
        let look = ui.state.look();
        let px = self.o.render(ui, &look);
        save_png(&format!("{}/{name}.png", self.out), self.o.w, self.o.h, &px);
        eprintln!("{name}");
    }
}

/// Step the clock in frames so that eased things (the corner marks) move as they would on screen.
fn run(ui: &mut DemoUi, to: f32) {
    let mut t = ui.now();
    while t < to { t = (t + 1.0 / 60.0).min(to); ui.tick(t); }
}

/// Scroll the page until `key` sits in the upper part of the window, then return its centre.
fn centre(ui: &mut DemoUi, key: &str) -> (f32, f32) {
    let r = ui.node(key).unwrap_or_else(|| panic!("no node {key}")).rect;
    let (_, lh) = ui.logical_size();
    if r.y < 0.0 || r.bottom() > lh { ui.wheel(4.0, 4.0, r.y - lh * 0.3); }
    ui.node(key).expect("node").rect.center()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| "shots".into());
    let w: u32 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(1280);
    let h: u32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(900);
    let scale: f32 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(1.0);
    std::fs::create_dir_all(&out).expect("out dir");
    let o = Offscreen::new(w, h);
    let mut ui = demo::new_ui(amenui::text::load_bundled("assets").expect("fonts"), w, h, scale);
    let mut s = Shooter { o, out };
    let (lw, lh) = (w as f32 / scale, h as f32 / scale);

    run(&mut ui, 0.1);
    s.shoot(&mut ui, "0-condensing");
    run(&mut ui, 10.0);
    s.shoot(&mut ui, "1-rest");

    let (px, py) = (lw * 0.3, lh * 0.55);
    ui.press(px, py);
    ui.release(px, py);
    run(&mut ui, 10.2);
    s.shoot(&mut ui, "2-wave-early");
    run(&mut ui, 10.5);
    s.shoot(&mut ui, "3-wave-mid");

    let (bx, by) = centre(&mut ui, "inc");
    ui.pointer_move(bx, by);
    run(&mut ui, 12.0);
    s.shoot(&mut ui, "4-hover-button");

    ui.press(bx, by);
    run(&mut ui, 12.1);
    s.shoot(&mut ui, "5-held");
    ui.release(bx, by);
    run(&mut ui, 12.25);
    s.shoot(&mut ui, "6-released");
    run(&mut ui, 14.0);
    s.shoot(&mut ui, "7-after-click");

    let (lx, ly) = centre(&mut ui, "list");
    ui.pointer_move(lx, ly);
    ui.wheel(lx, ly, 330.0);
    run(&mut ui, 15.0);
    s.shoot(&mut ui, "8-list-scrolled");

    // Keyboard: focus the first parameter and type a new value without committing it.
    let (fx, fy) = centre(&mut ui, "param/0/value");
    ui.press(fx, fy);
    ui.release(fx, fy);
    for c in "6.5".chars() { ui.key(Key::Char(c), false); }
    run(&mut ui, 17.0);
    s.shoot(&mut ui, "9-typing");
    ui.key(Key::Enter, false);
    ui.key(Key::Tab, false);
    run(&mut ui, 18.0);
    s.shoot(&mut ui, "10-committed-tab");

    let (tx, ty) = centre(&mut ui, "theme:1");
    ui.press(tx, ty);
    ui.release(tx, ty);
    run(&mut ui, 18.35);
    s.shoot(&mut ui, "11-light-wave");
    let (_, picy) = centre(&mut ui, "picture");
    ui.pointer_move(lw * 0.3, picy);
    run(&mut ui, 21.0);
    s.shoot(&mut ui, "12-light-lens");

    // Pictures into marks: the compare line dragged across the bust, then the pixel type.
    let (_, my) = centre(&mut ui, "marks-picture");
    let r = ui.node("marks-picture").expect("marks picture").rect;
    ui.pointer_leave();
    ui.press(r.x + r.w * 0.3, my);
    ui.pointer_move(r.x + r.w * 0.35, my);
    ui.release(r.x + r.w * 0.35, my);
    ui.pointer_leave();
    run(&mut ui, 23.0);
    s.shoot(&mut ui, "13-marks-wipe");
    let (tx, ty) = centre(&mut ui, "theme:0");
    ui.press(tx, ty);
    ui.release(tx, ty);
    let _ = centre(&mut ui, "pixel-heading");
    ui.pointer_leave();
    run(&mut ui, 26.0);
    s.shoot(&mut ui, "14-pixel-type");
    let (tx, ty) = centre(&mut ui, "theme:2");
    ui.press(tx, ty);
    ui.release(tx, ty);
    ui.pointer_leave();
    run(&mut ui, 29.0);
    s.shoot(&mut ui, "15-pixel-theme");

    // The whole page scrolls when it is taller than the window.
    let (cx, cy) = (8.0, lh * 0.5);
    ui.wheel(cx, cy, 100_000.0);
    ui.pointer_leave();
    run(&mut ui, 31.0);
    s.shoot(&mut ui, "16-page-end");
}
