//! Filmstrips of the seven showcase scenes (DESIGN.md §6): for each, a scripted interaction (synthetic
//! pointer and key events on the real gallery) shot at six or more moments, in its default style, in
//! two other styles, and in a narrow 420 × 800 window; and `scenes.png`, one moment of each.
//! `scenes <out> perf` instead times a frame of every scene in the middle of its interaction.
//!
//! usage: scenes <out_dir> [perf]
use lab::el::Key;
use lab::gallery::{self, GalleryUi, Msg, Page, SCENES};
use lab::offscreen::{Offscreen, save_png, sheet};
use lab::style::styles;

const W: u32 = 1280;
const H: u32 = 800;

struct Dir<'a> { ui: GalleryUi, o: &'a mut Offscreen, frames: Vec<(u32, u32, Vec<u8>)>, labels: Vec<String> }

impl Dir<'_> {
    fn wait(&mut self, secs: f32) {
        let to = self.ui.now() + secs;
        let mut t = self.ui.now();
        while t < to { t = (t + 1.0 / 60.0).min(to); self.ui.tick(t); }
    }
    fn at(&self, key: &str) -> (f32, f32) { self.ui.node(key).unwrap_or_else(|| panic!("no node {key}")).rect.center() }
    fn click(&mut self, key: &str) { let (x, y) = self.at(key); self.ui.press(x, y); self.ui.release(x, y); self.ui.pointer_leave(); }
    fn hover(&mut self, x: f32, y: f32) { self.ui.pointer_move(x, y); }
    /// Press `key`, move by (dx, dy) over `secs`, and let go (unless `hold`).
    fn drag(&mut self, key: &str, dx: f32, dy: f32, secs: f32, hold: bool) {
        let (x, y) = self.at(key);
        self.ui.press(x, y);
        let n = (secs * 30.0).max(1.0) as u32;
        for i in 1..=n {
            let k = i as f32 / n as f32;
            self.wait(secs / n as f32);
            self.ui.pointer_move(x + dx * k, y + dy * k);
        }
        if !hold { self.ui.release(x + dx, y + dy); }
    }
    fn send(&mut self, m: Msg) { gallery::update(&mut self.ui.state, m); self.ui.relayout(); }
    fn snap(&mut self, label: &str) {
        let look = self.ui.state.look();
        let px = self.o.render(&mut self.ui, &look);
        self.frames.push((self.o.w, self.o.h, px));
        self.labels.push(label.to_owned());
    }
}

/// A smaller copy (box filter, whole factor).
fn shrink(f: &(u32, u32, Vec<u8>), k: u32) -> (u32, u32, Vec<u8>) {
    let (w, h, px) = (f.0, f.1, &f.2);
    let (sw, sh) = (w / k, h / k);
    let mut o = vec![0u8; (sw * sh * 4) as usize];
    for y in 0..sh { for x in 0..sw { for c in 0..4 {
        let mut acc = 0u32;
        for j in 0..k { for i in 0..k { acc += px[(((y * k + j) * w + x * k + i) * 4 + c) as usize] as u32; } }
        o[((y * sw + x) * 4 + c) as usize] = (acc / (k * k)) as u8;
    } } }
    (sw, sh, o)
}
/// The scripted interaction of scene `i`; returns which frame is its most striking.
fn script(d: &mut Dir, i: usize) -> usize {
    let narrow = d.ui.state.width < 720.0;
    match i {
        0 => {
            d.snap("rest");
            d.click("op/start");
            d.wait(0.75); d.snap("the dots fly");
            d.wait(1.1); d.snap("the title");
            d.wait(1.0); d.snap("the title scatters into the page");
            d.wait(1.95); d.snap("the operator's mark flies to the search field");
            d.wait(0.45); d.snap("it types; a ripple where it acts");
            d.wait(2.7); d.snap("the switch, the number");
            // The person takes over: any press stops the operator.
            let r = d.ui.node("op/stage").expect("stage").rect;
            d.ui.press(r.x + 8.0, r.y + 8.0); d.ui.release(r.x + 8.0, r.y + 8.0); d.ui.pointer_leave();
            d.wait(0.6); d.snap("taken over: 继续演示");
            d.click("op/resume");
            d.wait(3.5); d.snap("resumed, then done");
            4
        }
        1 => {
            d.snap("rest");
            d.drag("read/bust", -240.0, 60.0, 0.5, true); d.snap("dragging the bust left");
            d.drag("read/bust", 0.0, 0.0, 0.05, false);
            d.drag("read/bust", 300.0, -20.0, 0.6, false); d.wait(0.6); d.snap("dropped to the right");
            d.click("read/which:2"); d.wait(0.8); d.snap("two obstacles");
            let (dx, dy) = if narrow { (120.0, -120.0) } else { (230.0, -150.0) };
            d.drag("read/disc", dx, dy, 0.6, true); d.snap("the disc dragged into the text");
            d.ui.release(0.0, 0.0);
            d.send(Msg::Num(lab::gallery::Target::Scene(1, 0), lab::widgets::Edit::Set(36.0)));
            d.wait(1.0); d.snap("gap 36 px");
            4
        }
        2 => {
            d.snap("rest");
            let r = d.ui.node("pr/wall").expect("wall").rect;
            d.hover(r.x + r.w * 0.1, r.y + r.h * 0.45); d.wait(0.3); d.snap("the lens over the characters");
            d.hover(r.x + r.w * 0.4, r.y + r.h * 0.4); d.wait(0.3); d.snap("across pixels and strokes");
            d.hover(r.x + r.w * 0.72, r.y + r.h * 0.55); d.wait(0.3); d.snap("glow");
            d.ui.pointer_leave();
            d.drag("pr/seam1", -r.w * 0.12, 0.0, 0.5, false);
            d.send(Msg::Num(lab::gallery::Target::Scene(2, 3), lab::widgets::Edit::Set(1.0)));
            d.wait(1.0); d.snap("seam moved, marks everywhere");
            d.send(Msg::Num(lab::gallery::Target::Scene(2, 3), lab::widgets::Edit::Set(0.0)));
            d.click("pr/mode:1"); d.wait(1.0); d.snap("one strip: glow against the original");
            d.drag("pr/compare", -r.w * 0.2, 0.0, 0.4, false);
            d.send(Msg::Num(lab::gallery::Target::Scene(2, 4), lab::widgets::Edit::Set(0.0)));
            d.send(Msg::Num(lab::gallery::Target::Scene(2, 5), lab::widgets::Edit::Set(2.0)));
            d.wait(1.0); d.snap("characters, three-colour palette");
            d.send(Msg::Num(lab::gallery::Target::Scene(2, 4), lab::widgets::Edit::Set(2.0)));
            d.send(Msg::Num(lab::gallery::Target::Scene(2, 5), lab::widgets::Edit::Set(6.0)));
            d.click("pr/which:2"); d.wait(1.0); d.snap("pixels, PICO-8, the bust");
            1
        }
        3 => {
            d.snap("rest: a heavy word");
            d.click("mo/next"); d.wait(0.75); d.snap("the word in flight");
            d.wait(1.5); d.snap("a picture");
            d.click("mo/next"); d.wait(2.4); d.snap("the picture as marks");
            d.click("mo/next"); d.wait(0.9); d.snap("marks into an interface");
            d.wait(1.6); d.snap("the interface");
            d.click("mo/ui/fav"); d.click("mo/ui/n/more"); d.wait(0.6); d.snap("it works: switch off, 3 to 4");
            d.click("mo/ui/go"); d.wait(0.9); d.snap("出发: back into the word");
            d.wait(2.0); d.snap("the word again");
            5
        }
        4 => {
            d.snap("rest");
            d.ui.key(Key::Char(' '), false);
            d.wait(0.25); d.snap("held: the top lifts off");
            d.wait(1.2); d.snap("apart: boxes, numbers, the nearness");
            let (x, y) = d.at("xr/top-true");
            d.hover(x, y); d.wait(0.6); d.snap("apart, the pointer on the top");
            d.ui.pointer_leave();
            d.ui.key_up(Key::Char(' '));
            d.wait(0.25); d.snap("let go: back together");
            d.wait(1.5); d.snap("rest again");
            2
        }
        5 => {
            d.snap("rest");
            d.click("snd/play");
            d.wait(0.55); d.snap("a beat");
            d.wait(0.9); d.snap("the low end pushes");
            d.wait(1.3); d.snap("later");
            d.send(Msg::Num(lab::gallery::Target::Scene(5, 1), lab::widgets::Edit::Set(1.0)));
            d.wait(2.1); d.snap("intensity 1");
            d.ui.key(Key::Char(' '), false); d.wait(1.5); d.snap("paused");
            4
        }
        _ => {
            d.snap("rest (Normcore)");
            d.click("st/s3"); d.wait(0.35); d.snap("Glitch spreading from the press");
            d.wait(1.5); d.snap("Glitch");
            d.ui.key(Key::Right, false); d.wait(0.3); d.snap("→: Mixed media");
            3
        }
    }
}

fn faces() -> Vec<lab::text::Face> { lab::text::load_bundled("assets").expect("fonts") }

fn open(o: &mut Offscreen, i: usize, style: Option<usize>, w: u32, h: u32) -> Dir<'_> {
    let mut ui = gallery::new_ui(faces(), w, h, 1.0);
    ui.pointer_leave();
    gallery::update(&mut ui.state, Msg::Go(Page::Scene(i as u8)));
    if let Some(st) = style { gallery::update(&mut ui.state, Msg::Style(st)); }
    ui.relayout();
    ui.restart(0.0);
    let mut d = Dir { ui, o, frames: vec![], labels: vec![] };
    d.wait(2.0);
    d
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| "scenes".into());
    if a.get(2).is_some_and(|s| s == "perf") { perf(); return; }
    std::fs::create_dir_all(&out).expect("out dir");
    let only: Option<usize> = a.get(2).and_then(|s| s.parse().ok());
    let mut wide = Offscreen::new(W, H);
    let mut narrow = Offscreen::new(420, 800);
    let mut best = vec![];
    // Two other styles each scene is shown in, besides its default.
    let others: [[usize; 2]; 7] = [[0, 9], [4, 7], [1, 3], [1, 9], [5, 0], [1, 10], [8, 11]];
    for i in 0..SCENES.len() {
        if only.is_some_and(|o| o != i) { continue; }
        let name = SCENES[i].0;
        let def = SCENES[i].3;
        for st in [None, Some(others[i][0]), Some(others[i][1])] {
            let mut d = open(&mut wide, i, st, W, H);
            let pick = script(&mut d, i);
            let tag = styles()[st.unwrap_or(def)].hash;
            let small: Vec<_> = d.frames.iter().map(|f| shrink(f, 2)).collect();
            let (w, h, px) = sheet(&small, 3);
            save_png(&format!("{out}/scene-{name}-{tag}.png"), w, h, &px);
            if st.is_none() {
                for (k, f) in d.frames.iter().enumerate() { save_png(&format!("{out}/scene-{name}-{k}.png"), f.0, f.1, &f.2); }
                best.push(shrink(&d.frames[pick], 2));
                eprintln!("{name}: {}", d.labels.join(" | "));
            }
        }
        let mut d = open(&mut narrow, i, None, 420, 800);
        script(&mut d, i);
        let (w, h, px) = sheet(&d.frames, 6);
        save_png(&format!("{out}/scene-{name}-narrow.png"), w, h, &px);
        eprintln!("{name} done");
    }
    if only.is_none() {
        let (w, h, px) = sheet(&best, 2);
        save_png(&format!("{out}/scenes.png"), w, h, &px);
        // The list of scenes, and a card opening into its scene.
        let mut ui = gallery::new_ui(faces(), W, H, 1.0);
        ui.pointer_leave();
        gallery::update(&mut ui.state, Msg::Go(Page::Scenes));
        ui.relayout();
        let mut d = Dir { ui, o: &mut wide, frames: vec![], labels: vec![] };
        d.wait(2.0);
        d.snap("the list");
        d.click("scenes/card3");
        d.wait(0.2); d.snap("a card opening: the title flies");
        d.wait(1.5); d.snap("the scene");
        d.ui.key(Key::Escape, false);
        d.wait(0.2); d.snap("Esc: back, the title flies home");
        let small: Vec<_> = d.frames.iter().map(|f| shrink(f, 2)).collect();
        let (w, h, px) = sheet(&small, 2);
        save_png(&format!("{out}/scenes-list.png"), w, h, &px);
    }
}

/// A frame of every scene in the middle of its interaction, at 1280 × 800 (llvmpipe here).
fn perf() {
    let mut o = Offscreen::new(W, H);
    for (i, scene) in SCENES.iter().enumerate() {
        let mut d = open(&mut o, i, None, W, H);
        // Into the busy part of each scene.
        match i {
            0 => { d.click("op/start"); d.wait(0.7); }
            1 => { d.drag("read/bust", -120.0, 40.0, 0.2, true); }
            2 => { let r = d.ui.node("pr/wall").unwrap().rect; d.hover(r.x + r.w * 0.5, r.y + r.h * 0.5); }
            3 => { d.click("mo/next"); d.wait(0.7); }
            4 => { d.ui.key(Key::Char(' '), false); d.wait(1.0); }
            5 => { d.click("snd/play"); d.wait(1.0); }
            _ => { d.click("st/s3"); d.wait(0.3); }
        }
        let q = std::env::var("QUALITY").ok().and_then(|v| v.parse().ok()).unwrap_or(1.0);
        let mut look = d.ui.state.look();
        look.quality = q;
        d.o.draw(&mut d.ui, &look);
        let n = 8;
        let t0 = std::time::Instant::now();
        let mut t = d.ui.now();
        for _ in 0..n { t += 1.0 / 60.0; d.ui.tick(t); let mut look = d.ui.state.look(); look.quality = q; d.o.draw(&mut d.ui, &look); }
        eprintln!("{:<9} {:6.1} ms per frame (1280×800, layout and draw)", scene.0, t0.elapsed().as_secs_f64() * 1000.0 / n as f64);
    }
}
