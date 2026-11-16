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

fn study_menu(out: &str) {
    let mut r = Rig::new("nested-menu", "craft/menu/demo");
    r.tap("craft/menu/zone");
    let o = { let b = r.ui.node("craft/menu").unwrap().rect; (b.x, b.y) };
    let go = |r: &mut Rig, x: f32, y: f32| { r.ui.pointer_move(o.0 + x, o.1 + y); r.frame(); };
    go(&mut r, 100.0, 18.0);
    r.run(0.2); r.shot();
    // Diagonal toward the submenu across B and C: A stays open, the triangle follows.
    for i in 1..=10 { go(&mut r, 100.0 + 9.5 * i as f32, 18.0 + 7.0 * i as f32); if i % 3 == 0 { r.shot(); } }
    go(&mut r, 215.0, 92.0); r.run(0.2); r.shot();
    // Back to the first menu and straight down: outside the triangle, the row takes over after the delay.
    go(&mut r, 100.0, 18.0); r.run(0.3);
    for y in [40.0, 60.0, 78.0] { go(&mut r, 100.0, y); }
    r.shot();
    r.run(0.15); r.shot();
    r.save(out, "menu", 4);
}

fn study_loading(out: &str) {
    let mut r = Rig::new("skeleton", "craft/load/demo");
    r.shot();
    r.tap("craft/load/reload");
    r.ui.pointer_move(5.0, 900.0);
    for secs in [0.15, 0.25, 0.3, 0.25, 0.03, 0.05, 0.05, 0.6] { r.run(secs); r.shot(); }
    r.save(out, "loading", 3);
}

fn study_stack(out: &str) {
    // The same fan in the three looks: closed, fanning (three frames), fanned.
    for look in 1..=3 {
        let mut r = Rig::new("card-stack", "craft/stack/cards");
        gallery::update(&mut r.ui.state, Msg::Craft(gallery::craft::CMsg::StackLook(look)));
        r.ui.relayout();
        r.run(0.5);
        r.shot();
        let b = r.ui.node("craft/stack/cards").unwrap().rect;
        // Onto the top card: everything beneath fans out.
        r.ui.pointer_move(b.x + b.w * 0.55, b.y + b.h * 0.6);
        for _ in 0..3 { r.run(0.05); r.shot(); }
        r.run(0.8); r.shot();
        // Then onto the second card where it shows: the top two stay, the rest fan from there.
        let (cx, cy) = (b.x + b.w * 0.5, b.y + b.h * 0.6);
        let mut hit = None;
        for k in 0..60 { let (x, y) = (cx - 60.0 - k as f32 * 3.0, cy - 80.0 - k as f32 * 2.0); r.ui.pointer_move(x, y); if r.ui.state.craft.stack.under() == Some(1) { hit = Some((x, y)); break; } }
        if hit.is_some() { r.run(0.06); r.shot(); r.run(0.8); r.shot(); } else { eprintln!("look {look}: the second card was not found under the pointer"); r.shot(); r.shot(); }
        r.save(out, &format!("stack-{look}"), 7);
    }
    // Stitch the three rows into one sheet.
    let rows: Vec<(u32, u32, Vec<u8>)> = (1..=3).map(|l| read_png(&format!("{out}/stack-{l}.png"))).collect();
    let (w, h, px) = sheet(&rows, 1);
    save_png(&format!("{out}/stack.png"), w, h, &px);
    for l in 1..=3 { let _ = std::fs::remove_file(format!("{out}/stack-{l}.png")); }
}
fn read_png(path: &str) -> (u32, u32, Vec<u8>) {
    let d = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).expect("png")));
    let mut r = d.read_info().expect("png info");
    let mut buf = vec![0; r.output_buffer_size()];
    let info = r.next_frame(&mut buf).expect("png frame");
    buf.truncate(info.buffer_size());
    (info.width, info.height, buf)
}

fn study_chart(out: &str) {
    let mut r = Rig::new("dot-chart", "craft/chart/demo");
    r.shot();
    let b = r.ui.node("craft/chart/dots").unwrap().rect;
    // Scrub from left to right.
    for f in [0.15, 0.45, 0.8] { r.ui.pointer_move(b.x + b.w * f, b.y + 40.0); r.frame(); r.shot(); }
    // Drag the fourth control point down, then the second up.
    r.drag("craft/chart/dots/knot3", 0.0, 70.0, 10, &[10]);
    r.drag("craft/chart/dots/knot1", 0.0, -60.0, 10, &[10]);
    r.save(out, "chart", 3);
}
fn study_zine(out: &str) {
    let mut ui = gallery::new_ui(faces(), W, 640, 1.0);
    ui.pointer_leave();
    assert!(ui.state.route("film"));
    if ui.state.film.playing { gallery::update(&mut ui.state, Msg::Film(lab::gallery::film::FMsg::Play)); }
    lab::gallery::film::go(&mut ui.state, 6, None);
    ui.relayout();
    let mut r = Rig { o: Offscreen::new(W, 640), ui, frames: vec![], last: vec![], region: "film/c06/body".into(), pad: 40.0 };
    r.run(3.0);
    r.shot();
    let (x, y) = r.center("film/c06/sign");
    r.ui.press(x, y);
    for i in 1..=36 {
        let k = i as f32;
        r.ui.pointer_move(x + (k * 0.17).sin() * 150.0, y + (k * 0.11).cos() * 60.0 - 60.0);
        r.frame();
        if i % 7 == 0 { r.shot(); }
    }
    r.ui.release(x, y);
    r.run(0.3); r.shot();
    r.save(out, "zine", 4);
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| "out".into());
    std::fs::create_dir_all(&out).expect("out dir");
    let all = ["sheet", "floating", "menu", "loading", "stack", "chart", "zine"];
    let pick: Vec<&str> = if a.len() > 2 { a[2..].iter().map(String::as_str).collect() } else { all.to_vec() };
    for s in pick {
        match s {
            "sheet" => study_sheet(&out),
            "floating" => study_floating(&out),
            "menu" => study_menu(&out),
            "loading" => study_loading(&out),
            "stack" => study_stack(&out),
            "chart" => study_chart(&out),
            "zine" => study_zine(&out),
            o => eprintln!("no study {o} (studies: {all:?})"),
        }
    }
}
