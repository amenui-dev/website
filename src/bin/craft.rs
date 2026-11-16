//! Filmstrips of the interaction previews (DESIGN.md §1.8.4, §5.2), drawn off-screen with a fixed 60 Hz
//! clock and scripted pointer input, one sheet per study: `sheet` (bottom sheet: present, a slow drag that
//! returns, a pull past the top, a flick that dismisses), `floating` (a slow drag back, a flick to the far
//! corner), and the later studies as they are added.
//!
//! usage: craft <out_dir> [<study> …]   (every study when none is named); plain unless LAB_MATERIAL names another (dot, ascii…)
use lab::gallery::{self, GalleryUi, Msg, Page};
use lab::offscreen::{Offscreen, crop, save_png, sheet};

const W: u32 = 1280;
const H: u32 = 1000;

fn faces() -> Vec<lab::text::Face> { lab::text::load_bundled("assets").expect("fonts") }

/// A gallery page, the renderer, the clock, and the frames kept so far.
struct Rig { o: Offscreen, ui: GalleryUi, frames: Vec<(u32, u32, Vec<u8>)>, last: Vec<u8>, region: String, pad: f32 }
impl Rig {
    fn new(page: &str, region: &str) -> Self {
        let mut ui = gallery::new_ui(faces(), W, H, 1.0);
        let p = Page::from_hash(page).unwrap_or_else(|| panic!("no page {page}"));
        gallery::update(&mut ui.state, Msg::Go(p));
        let m = std::env::var("LAB_MATERIAL").unwrap_or_else(|_| "plain".into());
        if let Some(i) = lab::style::MATERIALS.iter().position(|x| x.1 == m) { gallery::update(&mut ui.state, Msg::Material(i)); }
        ui.relayout();
        ui.pointer_leave();
        let mut r = Rig { o: Offscreen::new(W, H), ui, frames: vec![], last: vec![], region: region.into(), pad: 16.0 };
        r.run(1.2);
        r
    }
    /// Tick at 60 Hz for `secs`, drawing what changed.
    fn run(&mut self, secs: f32) {
        let to = self.ui.now() + secs;
        let mut t = self.ui.now();
        while t < to - 1e-4 {
            t = (t + 1.0 / 60.0).min(to);
            self.ui.tick(t);
            if self.ui.needs_frame(t) || self.last.is_empty() { let look = self.ui.state.look(); self.last = self.o.render(&mut self.ui, &look); }
        }
    }
    fn frame(&mut self) { self.run(1.0 / 60.0); }
    /// Keep the current frame, cropped to the region's element.
    fn shot(&mut self) {
        if self.last.is_empty() { self.frame(); }
        let r = self.ui.node(&self.region).unwrap_or_else(|| panic!("no {}", self.region)).rect;
        let p = self.pad;
        let (x, y) = ((r.x - p).max(0.0), (r.y - p).max(0.0));
        let (w, h) = ((r.w + 2.0 * p).min(W as f32 - x), (r.h + 2.0 * p).min(H as f32 - y));
        self.frames.push(crop(&self.last, self.o.w, self.o.h, [x as u32, y as u32, (x + w) as u32, (y + h) as u32]));
    }
    fn center(&self, key: &str) -> (f32, f32) { self.ui.node(key).unwrap_or_else(|| panic!("no {key}")).rect.center() }
    fn tap(&mut self, key: &str) { let (x, y) = self.center(key); self.ui.press(x, y); self.ui.release(x, y); }
    /// A mouse drag from the centre of `key` by (dx, dy) over `n` frames, with a shot at the frames in `at`.
    fn drag(&mut self, key: &str, dx: f32, dy: f32, n: usize, at: &[usize]) {
        let (x, y) = self.center(key);
        self.ui.press(x, y);
        for i in 1..=n {
            let k = i as f32 / n as f32;
            self.ui.pointer_move(x + dx * k, y + dy * k);
            self.frame();
            if at.contains(&i) { self.shot(); }
        }
        self.ui.release(x + dx, y + dy);
    }
    fn save(&self, out: &str, name: &str, cols: usize) {
        let (w, h, px) = sheet(&self.frames, cols);
        save_png(&format!("{out}/{name}.png"), w, h, &px);
        eprintln!("{name}: {} frames", self.frames.len());
    }
}

fn study_sheet(out: &str) {
    let mut r = Rig::new("bottom-sheet", "craft/sheet/frame");
    r.shot();
    r.tap("craft/sheet/open");
    // Off the panel, so no hover marks frame what it covers.
    r.ui.pointer_move(5.0, 600.0);
    for _ in 0..3 { r.run(0.05); r.shot(); }
    r.run(0.8);
    // Slow drag down 60 px (it returns), then a pull up past the top detent (banded) and back.
    r.drag("craft/sheet/handle", 0.0, 60.0, 50, &[50]);
    r.run(0.12); r.shot();
    r.run(0.8);
    r.drag("craft/sheet/handle", 0.0, -180.0, 12, &[]);
    r.run(0.8); r.shot();
    r.drag("craft/sheet/handle", 0.0, -400.0, 30, &[30]);
    r.run(0.1); r.shot();
    r.run(0.8);
    // A fast flick down: dismissed.
    r.drag("craft/sheet/handle", 0.0, 70.0, 3, &[3]);
    r.ui.pointer_move(5.0, 600.0);
    for _ in 0..3 { r.run(0.06); r.shot(); }
    r.run(1.0); r.shot();
    r.save(out, "sheet", 7);
}

fn study_floating(out: &str) {
    let mut r = Rig::new("floating", "craft/pip/area");
    r.shot();
    // Slow drag 110 px left: its projection stays on the right half, it springs back right.
    r.drag("craft/pip/box", -110.0, 30.0, 60, &[60]);
    for _ in 0..2 { r.run(0.08); r.shot(); }
    r.run(1.0); r.shot();
    // A flick of 70 px left and up: let go near where it was, it heads for the top-left corner.
    r.drag("craft/pip/box", -70.0, -40.0, 3, &[3]);
    for _ in 0..3 { r.run(0.07); r.shot(); }
    r.run(1.0); r.shot();
    r.save(out, "floating", 5);
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| "out".into());
    std::fs::create_dir_all(&out).expect("out dir");
    let all = ["sheet", "floating"];
    let pick: Vec<&str> = if a.len() > 2 { a[2..].iter().map(String::as_str).collect() } else { all.to_vec() };
    for s in pick {
        match s {
            "sheet" => study_sheet(&out),
            "floating" => study_floating(&out),
            o => eprintln!("no study {o} (studies: {all:?})"),
        }
    }
}
