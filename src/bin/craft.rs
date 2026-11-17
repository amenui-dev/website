//! Filmstrips of the interaction previews (DESIGN.md §1.8.4, §5.2), drawn off-screen with a fixed 60 Hz
//! clock and scripted pointer input, one sheet per study: `sheet` (bottom sheet: present, a slow drag that
//! returns, a pull past the top, a flick that dismisses), `floating` (a slow drag back, a flick to the far
//! corner), `stirr` (the field map's trail, four materials), `scribblr` (a title written out and back), and the later studies as they are added.
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
    fn new(page: &str, region: &str) -> Self { Self::with(page, region, &std::env::var("LAB_MATERIAL").unwrap_or_else(|_| "plain".into())) }
    fn with(page: &str, region: &str, m: &str) -> Self {
        let mut ui = gallery::new_ui(faces(), W, H, 1.0);
        let p = Page::from_hash(page).unwrap_or_else(|| panic!("no page {page}"));
        gallery::update(&mut ui.state, Msg::Go(p));
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
        // No region: the whole window.
        let r = if self.region.is_empty() { lab::ui::Rect::new(0.0, 0.0, self.o.w as f32, self.o.h as f32) } else { self.ui.node(&self.region).unwrap_or_else(|| panic!("no {}", self.region)).rect };
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
/// The field map (DESIGN.md §1.29): the window dragged across the decoration, the picture and the text,
/// let go, gliding, and the place settled; plain, dot, print and glitch, eight frames each.
fn study_stirr(out: &str) {
    for m in ["plain", "dot", "print", "glitch"] {
        let mut r = Rig::with("stirr", "stirr/stage", m);
        r.shot();
        // Across the row of shapes, then down over the picture and the text.
        r.drag("stirr/float", 300.0, 30.0, 20, &[10, 20]);
        let (x, y) = r.center("stirr/float");
        r.ui.press(x, y);
        for i in 1..=14 { r.ui.pointer_move(x + 6.0 * i as f32, y + 14.0 * i as f32); r.frame(); if i == 14 { r.shot(); } }
        r.ui.release(x + 90.0, y + 200.0);
        r.run(0.12); r.shot();
        r.run(0.3); r.shot();
        r.run(0.5); r.shot();
        r.run(2.5); r.shot();
        r.save(out, &format!("stirr-{m}"), 4);
    }
}

/// scribblr (DESIGN.md §1.30): a title and a line of body text written out and back, from looseness 1
/// to 0 in eight frames, on the dark and the light theme. Each frame stirs the field map through the
/// title by as much as it is loose, so the knots (where the map has energy) and the tail (along its
/// flow) show; the body text stays within the legibility cap.
fn study_scribblr(out: &str) {
    use lab::el::{Paint, col, text};
    use lab::morph::{Cmd, StirAt};
    use lab::style::{DARK, LIGHT, Look, Material};
    struct St { k: f32, light: bool, cmds: Vec<Cmd> }
    fn page(st: &St) -> lab::el::El<()> {
        col().key("page").pad(40.0, 36.0).gap(18.0)
            .child(text("Loose thread", 64.0, Paint::INK).font(lab::el::Font::Display).key("title").legible(false).loose(st.k).respond(lab::el::Response::Loosen))
            .child(text("One line writes the ink out and back.", 18.0, Paint::TEXT).key("body").loose(st.k.min(0.15)))
    }
    for (name, light) in [("dark", false), ("light", true)] {
        let mut ui = lab::ui::Ui::new(St { k: 1.0, light, cmds: vec![] }, page, |_, _| {}, faces(), 760, 260, 1.0)
            .on_look(|s| Look { theme: if s.light { LIGHT } else { DARK }, material: Material::Scribblr, ..Look::default() })
            .on_commands(|s| std::mem::take(&mut s.cmds));
        let mut o = Offscreen::new(760, 260);
        ui.pointer_leave();
        ui.tick(5.0);
        let mut frames = vec![];
        let mut t = 5.0;
        for k in [1.0f32, 0.85, 0.7, 0.5, 0.35, 0.2, 0.1, 0.0] {
            ui.state.k = k;
            if k > 0.0 { ui.state.cmds.push(Cmd::Stir { at: StirAt::Key("title".into()), v: [1.0, -0.25], energy: k, radius: 200.0, scope: lab::el::Scope::Global }); }
            ui.relayout();
            // A fifth of a second at 60 Hz between frames, so the map steps as it would; the last frame
            // after the map has settled (1.5 s), at rest.
            for _ in 0..if k > 0.0 { 11 } else { 90 } { t += 1.0 / 60.0; ui.tick(t); }
            // Topped up to the frame's looseness over the whole title just before it is drawn, so the
            // knots have the energy they are drawn by (at 1, at least 0.8 everywhere on the title).
            if k > 0.0 {
                let r = ui.node("title").expect("title").rect;
                let (cx, cy) = r.center();
                let low = |ui: &lab::ui::Ui<St, ()>| [r.x, cx, r.right()].iter().map(|&x| ui.fieldmap().energy_at(x, cy)).fold(1.0, f32::min);
                for _ in 0..8 {
                    let e = low(&ui);
                    if e >= 0.8 * k { break; }
                    ui.state.cmds.push(Cmd::Stir { at: StirAt::Point([cx, cy]), v: [0.0, 0.0], energy: k - e, radius: 900.0, scope: lab::el::Scope::Global });
                    ui.relayout();
                    t += 1.0 / 60.0;
                    ui.tick(t);
                }
                let low = low(&ui);
                eprintln!("scribblr-{name}: looseness {k}, the map's energy on the title at least {low:.2}");
            }
            let look = ui.look();
            let px = o.render(&mut ui, &look);
            frames.push((760, 260, px));
        }
        let (w, h, px) = sheet(&frames, 2);
        save_png(&format!("{out}/scribblr-{name}.png"), w, h, &px);
        eprintln!("scribblr-{name}: {} frames", frames.len());
    }
}

/// Loose Thread (DESIGN.md §4.4): the style page at rest (its tangle drawn), with the pointer passing
/// through the title and down across the text and the controls (the title's thread, the edge notes and
/// corner marks), a quarter second and a second later, settled again; then on paper, in black and
/// white only, and inverted.
fn study_loose_thread(out: &str) {
    let mut r = Rig::with("style-loose-thread", "style-loose-thread/comp", "scribblr");
    r.pad = 24.0;
    r.run(1.0);
    r.shot();
    let b = r.ui.node("style-loose-thread/comp").expect("comp").rect;
    let t = r.ui.node("style-loose-thread/title").expect("title").rect;
    let sweep = |r: &mut Rig| {
        // Through the title, then down across the text and the controls.
        for i in 0..=30 { let k = i as f32 / 30.0; r.ui.pointer_move(b.x + 10.0 + (t.w + 40.0) * k, t.y + t.h * 0.5 + 220.0 * k * k); r.frame(); }
    };
    let loose = |r: &Rig| r.ui.looseness("style-loose-thread/title", r.ui.now());
    sweep(&mut r);
    r.shot();
    eprintln!("loose-thread: the title {:.2} loose after the sweep", loose(&r));
    r.run(0.25); r.shot();
    r.ui.pointer_leave();
    r.run(0.75); r.shot();
    eprintln!("loose-thread: {:.2} a second after", loose(&r));
    r.run(2.0); r.shot();
    eprintln!("loose-thread: {:.2} three seconds after, {} threads", loose(&r), r.ui.strands().len());
    r.save(out, "loose-thread", 2);
    for (name, flips) in [("paper", vec![Msg::Theme(lab::style::theme_index(lab::style::THREAD_PAPER.name))]), ("binary", vec![Msg::PostFlag(0)]), ("invert", vec![Msg::PostFlag(1)])] {
        let mut r = Rig::with("style-loose-thread", "style-loose-thread/comp", "scribblr");
        r.pad = 24.0;
        for m in flips { gallery::update(&mut r.ui.state, m); }
        r.ui.relayout();
        r.run(1.0);
        sweep(&mut r);
        r.shot();
        r.ui.pointer_leave();
        r.run(3.0); r.shot();
        r.save(out, &format!("loose-thread-{name}"), 2);
    }
}

/// Against the references: the style page with its title held fully loose (the map stirred over it
/// every few frames), on black and on paper.
fn study_thread_ref(out: &str) {
    use lab::morph::{Cmd, StirAt};
    for (name, paper) in [("black", false), ("paper", true)] {
        let mut r = Rig::with("style-loose-thread", "style-loose-thread/comp", "scribblr");
        r.pad = 0.0;
        if paper { gallery::update(&mut r.ui.state, Msg::Theme(lab::style::theme_index(lab::style::THREAD_PAPER.name))); r.ui.relayout(); }
        r.run(1.0);
        let c = r.ui.node("style-loose-thread/title").expect("title").rect.center();
        for _ in 0..12 {
            r.ui.state.cmds.push(Cmd::Stir { at: StirAt::Point([c.0, c.1]), v: [1.0, 0.35], energy: 1.0, radius: 260.0, scope: lab::el::Scope::Global });
            r.ui.relayout();
            r.run(4.0 / 60.0);
        }
        eprintln!("thread-ref-{name}: the title {:.2} loose", r.ui.looseness("style-loose-thread/title", r.ui.now()));
        r.shot();
        r.save(out, &format!("thread-ref-{name}"), 1);
    }
}

/// The field map's readouts (DESIGN.md §4.2, §4.3): edge notes, the waveform and the level bar over a
/// Loose Thread page while a stir runs along a path across it, four frames, then settled.
fn study_readouts(out: &str) {
    use lab::el::{Paint, col, text};
    use lab::morph::{Cmd, StirAt};
    use lab::style::{Gaze, GazeMode, Look, gaze};
    struct St { cmds: Vec<Cmd>, look: Look }
    fn page(_: &St) -> lab::el::El<()> {
        col().key("page").pad(48.0, 40.0).child(col().key("scope").gaze_scope().width(840.0).height(420.0).pad(40.0, 36.0).gap(22.0)
            .child(text("readouts", 12.0, Paint::TEXT).whisper())
            .child(text("Field", 72.0, Paint::INK).font(lab::el::Font::Display).key("title").respond(lab::el::Response::Loosen).legible(false))
            .child(text("The waveform reads one row of the field map; the bar its mean.", 16.0, Paint::TEXT)))
    }
    let p = lab::preset::presets().into_iter().find(|p| p.hash == "loose-thread").expect("preset");
    let look = Look { gaze: Gaze { mode: GazeMode::All, kinds: gaze::EDGE | gaze::OSC | gaze::LEVEL, ..p.gaze }, ..p.apply(&Look::default()) };
    let (w, h) = (940u32, 500u32);
    let mut ui = lab::ui::Ui::new(St { cmds: vec![], look }, page, |_, _| {}, faces(), w, h, 1.0)
        .on_look(|s| s.look).on_gaze(|s| s.look.gaze).on_commands(|s| std::mem::take(&mut s.cmds));
    let mut o = Offscreen::new(w, h);
    ui.pointer_leave();
    ui.tick(2.0);
    let path = std::rc::Rc::new(vec![[100.0, 160.0], [380.0, 120.0], [620.0, 260.0], [860.0, 200.0]]);
    let mut frames = vec![];
    let mut t = 2.0;
    for f in 0..4 {
        let (a, b) = (f as f32 / 4.0, (f + 1) as f32 / 4.0);
        ui.state.cmds.push(Cmd::Stir { at: StirAt::Path { points: path.clone(), from: a, to: b }, v: [1.0, 0.2], energy: 1.0, radius: 50.0, scope: lab::el::Scope::Global });
        ui.relayout();
        for _ in 0..8 { t += 1.0 / 60.0; ui.tick(t); }
        let l = ui.look();
        frames.push((w, h, o.render(&mut ui, &l)));
    }
    for _ in 0..240 { t += 1.0 / 60.0; ui.tick(t); }
    let l = ui.look();
    frames.push((w, h, o.render(&mut ui, &l)));
    let (sw, sh, px) = sheet(&frames, 2);
    save_png(&format!("{out}/readouts.png"), sw, sh, &px);
    eprintln!("readouts: 5 frames");
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let out = a.get(1).cloned().unwrap_or_else(|| "out".into());
    std::fs::create_dir_all(&out).expect("out dir");
    let all = ["sheet", "floating", "menu", "loading", "stack", "chart", "stirr", "scribblr", "loose-thread", "readouts", "thread-ref"];
    let pick: Vec<&str> = if a.len() > 2 { a[2..].iter().map(String::as_str).collect() } else { all.to_vec() };
    for s in pick {
        match s {
            "sheet" => study_sheet(&out),
            "floating" => study_floating(&out),
            "menu" => study_menu(&out),
            "loading" => study_loading(&out),
            "stack" => study_stack(&out),
            "chart" => study_chart(&out),
            "stirr" => study_stirr(&out),
            "scribblr" => study_scribblr(&out),
            "loose-thread" => study_loose_thread(&out),
            "readouts" => study_readouts(&out),
            "thread-ref" => study_thread_ref(&out),
            o => eprintln!("no study {o} (studies: {all:?})"),
        }
    }
}
