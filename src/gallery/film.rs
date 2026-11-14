//! The film (DESIGN.md §6): one continuous piece in twelve chapters, from the most ordinary surface to
//! the most material one, then everything together. A 2.39:1 stage is centred in the window and
//! letterboxed by the page ground; the bars around it carry a slate (title, chapter, preset), a hairline
//! timeline with a tick per chapter, the chapter's caption, a hint and the way to the gallery.
//!
//! Three actors are in every chapter with the same flow ids — the sentence, the number and the object —
//! so every cut is a match cut: they travel to their new places while everything else changes by the
//! target preset's transition (a commit). Each chapter is a small, believable interface with a real
//! interaction that shows one feature of the framework. Placement is shared: one 12-column grid with
//! fixed margins in a 1600-unit-wide stage, scaled to the window; three type sizes per chapter.
//!
//! Autoplay moves on every `dwell_ms` and plays a scripted "ghost" interaction in each chapter, so the
//! film plays untouched; any input pauses it in the same frame and it resumes after `idle_ms` (unless
//! the viewer paused it with Space).
use super::{Gallery, Msg, Page, STYLE_SLOT};
use crate::demo::{SKY, TERRAIN};
use crate::el::{Drag, El, Font, Key, Paint, Phase, Response, Scope, col, picture, row, scene3d, space, stack, text, text_lh};
use crate::mesh::{Camera, Light, Mat4, MeshId, Pick, Projection, Scene3d};
use crate::morph::{Cmd, Ease, FlowSpec, MorphSpec};
use crate::style::{Gaze, GazeMode, Look, Material, gaze};
use crate::widgets::{Edit, Orbit, OrbitMsg, Spec, TextEdit, Variant, button_as, dialog, input, number, orbit, switch};
use crate::wrap::{Around, Silhouette};
use taffy::AlignItems;

pub const CHAPTERS: usize = 12;

/// One chapter: the preset it is in (index into `style::styles()`), its name, its caption (what it
/// shows of the framework, in plain words) and its hint (what you can do).
pub struct Chapter { pub style: usize, pub name: &'static str, pub caption: &'static str, pub hint: &'static str }
pub const CH: [Chapter; CHAPTERS] = [
    Chapter { style: 7, name: "Normcore", caption: "一张普通的设置表单。静止时什么都不动；打字立刻出现，没有过渡。", hint: "改标题（它会带到每一章） · 开关让物体转 · 数量" },
    Chapter { style: 8, name: "Cyberminimalism", caption: "命令面板：筛选时，变了的字重新组合，没变的留在原地。", hint: "直接打字筛选 · ↑↓ 选择 · Enter 执行" },
    Chapter { style: 6, name: "Tech-Manual", caption: "正交视图。尺寸线量的是物体此刻投影的真实大小，引线指向真实的顶点。", hint: "拖动转动物体 · 悬停零件表的一行" },
    Chapter { style: 0, name: "Algorithmic Gaze", caption: "检测器看到的画面：框、类型、尺寸和坐标都从排版结果量出来。", hint: "指针靠近一个东西 · 按住识别它" },
    Chapter { style: 1, name: "Neo-Dotmatrix", caption: "点阵出发牌：每一秒，一行接一行地刷新，只有变了的数字重组。", hint: "指针放在物体上让它转" },
    Chapter { style: 2, name: "Lo-Fi Print", caption: "小志的一页：正文绕着物体的真实轮廓排，不绕它的方框。", hint: "拖动物体，正文跟着重排 · 对比度" },
    Chapter { style: 4, name: "Metamorphosis Linocut", caption: "一次变形：物体散成格子，拼成这句话，再刻成一张图，再变回物体。", hint: "按下进入下一形态 · 横向拖动来回拨动变形" },
    Chapter { style: 3, name: "Glitch", caption: "信号监视器：按住撕开指针处的一带，松开把它锁在错开的位置。", hint: "在波形上按住拖动 · 错误对话框用 Tab 和 Esc" },
    Chapter { style: 5, name: "Transparent Electronics", caption: "透明的设备：界面分成几层，随指针错开；物体只画它的棱。", hint: "移动指针看视差 · 拆开 / 合上" },
    Chapter { style: 9, name: "Bauhaus", caption: "控件就是几何形：一条线和一个圆是滑块，两个方块是开关，色块是标签页。", hint: "拖动圆 · 按方块 · 选一个色块" },
    Chapter { style: 10, name: "Neo-Y2K", caption: "金属的物体和液态的菜单：指针下的项鼓起来，和邻居融在一起。", hint: "悬停菜单 · 选一项，它会分开" },
    Chapter { style: 11, name: "Mixed media", caption: "拼贴：前面每一章的一块，各用自己的材质；最后一切散成粒子，合成这句话。", hint: "按“合上” · 粒子数" },
];

/// The film's numbers (named, with defaults, each a number control in its strip of the gallery).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilmSpec {
    /// How long autoplay stays in a chapter, and how long after the last input it waits to resume.
    pub dwell_ms: f32,
    pub idle_ms: f32,
    /// The shared parts' flight between chapters.
    pub flow_ms: f32,
    /// Particles of the finale (up to 1,000,000; the default stays smooth on the WebGL2 tier).
    pub particles: f32,
    /// How fast the object turns when it turns (degrees a second).
    pub spin: f32,
}
impl Default for FilmSpec {
    fn default() -> Self { Self { dwell_ms: 9000.0, idle_ms: 12000.0, flow_ms: 640.0, particles: 60000.0, spin: 24.0 } }
}
pub const COUNT_SPEC: Spec = Spec { min: 1000.0, max: 1_000_000.0, step: 1000.0, decimals: 0, unit: "" };
pub const N_SPEC: Spec = Spec { min: 0.0, max: 99.0, step: 1.0, decimals: 0, unit: "" };
pub const CONTRAST_SPEC: Spec = Spec { min: 30.0, max: 300.0, step: 5.0, decimals: 0, unit: "%" };

/// What the film does.
#[derive(Clone, Debug, PartialEq)]
pub enum FMsg {
    Go(usize),
    Next,
    Prev,
    Play,
    Sentence(TextEdit),
    Motion,
    Num(Edit),
    Query(TextEdit),
    Run(usize),
    Orbit(OrbitMsg),
    Identify(String),
    Pick(Pick),
    Obj(f32, f32),
    Contrast(Edit),
    Form,
    Scrub(Drag),
    Tear(Drag),
    Error(bool),
    Retry,
    Explode,
    Size(f32),
    Squares,
    Plane(usize),
    MenuSel(usize),
    Finale,
    Count(Edit),
}

/// What the ghost does at a moment of a chapter (seconds after it was entered).
#[derive(Clone, Debug)]
enum Ghost { Msg(FMsg), Press(&'static str), Focus(&'static str), Type(&'static str), Hover(Option<&'static str>), Ptr(f32, f32) }

pub struct Film {
    pub ch: usize,
    pub spec: FilmSpec,
    /// Autoplay on; paused by the viewer with Space (it does not resume by itself then).
    pub playing: bool,
    pub held: bool,
    /// When the chapter was entered, the last input, the last tick, and the next ghost step.
    pub entered: f32,
    pub last_input: f32,
    pub last_tick: f32,
    pub ghost: usize,
    /// When input last paused autoplay (so the Space that did it is read as "pause").
    pub yielded: Option<f32>,
    /// The three actors' state: the sentence, the number, the object's turn.
    pub sentence: TextEdit,
    pub n: f32,
    pub yaw: f32,
    /// What the pointer is over, and where it is in the stage (0..1 each way).
    pub hover: Option<String>,
    pub ptr: (f32, f32),
    pub obj_hover: bool,
    pub drafts: Option<(u8, String)>,
    // 1
    pub motion: bool,
    // 2
    pub query: TextEdit,
    pub sel: usize,
    // 3
    pub orbit: Orbit,
    // 4
    pub ident: Option<String>,
    // 5: the rows' numbers as shown (they refresh one after another)
    pub rows: [f32; 4],
    pub board_at: f32,
    // 6
    pub obj: (f32, f32),
    pub contrast: f32,
    // 7
    pub form: usize,
    pub scrub_from: Option<usize>,
    // 8
    pub tear: Option<f32>,
    pub locks: Vec<(f32, f32)>,
    pub error: bool,
    // 9
    pub exploded: bool,
    // 10
    pub size: f32,
    pub squares: bool,
    pub plane: usize,
    // 11
    pub menu_sel: usize,
    // 12
    pub finale: bool,
}
impl Default for Film {
    fn default() -> Self {
        Self {
            ch: 0, spec: FilmSpec::default(), playing: true, held: false, entered: 0.0, last_input: -1e9, last_tick: 0.0, ghost: 0, yielded: None,
            sentence: TextEdit::new("松开，再合上"), n: 12.0, yaw: 24.0, hover: None, ptr: (0.5, 0.5), obj_hover: false, drafts: None,
            motion: false, query: TextEdit::default(), sel: 0, orbit: { let mut o = Orbit::default(); o.yaw = 35.0; o.pitch = 20.0; o }, ident: None,
            rows: [12.0; 4], board_at: 0.0, obj: (340.0, 20.0), contrast: 100.0, form: 0, scrub_from: None, tear: None, locks: vec![], error: false,
            exploded: false, size: 0.55, squares: false, plane: 0, menu_sel: 1, finale: false,
        }
    }
}

// ── Geometry: the stage and its grid ─────────────────────────────────────────────────────────────

/// The stage in the window (logical px) and `k`, logical px per stage unit (the stage is 1600 units wide).
#[derive(Clone, Copy, Debug)]
pub struct Geo { pub sx: f32, pub sy: f32, pub sw: f32, pub sh: f32, pub k: f32, pub bar: f32 }
pub const ASPECT: f32 = 2.39;
pub const UNITS: f32 = 1600.0;
/// The bars' least heights: the slate on top, the timeline, caption and hint below.
const TOP: f32 = 46.0;
const BOTTOM: f32 = 112.0;
pub fn geo(w: f32, h: f32) -> Geo {
    let side = (w * 0.03).clamp(16.0, 48.0);
    let sw = (w - side * 2.0).min((h - TOP - BOTTOM) * ASPECT).max(160.0);
    let sh = sw / ASPECT;
    let sy = ((h - sh) * 0.5).clamp(TOP, (h - BOTTOM - sh).max(TOP));
    // The slate's type: small, a little larger on large windows, never below 11 px.
    let bar = (sw / 1600.0 * 12.0).clamp(11.0, 15.0);
    Geo { sx: (w - sw) * 0.5, sy, sw, sh, k: sw / UNITS, bar }
}
/// Grid: 72-unit margins, 12 columns, 24-unit gutters.
const MARGIN: f32 = 72.0;
const GUTTER: f32 = 24.0;
const CW: f32 = (UNITS - 2.0 * MARGIN - 11.0 * GUTTER) / 12.0;
pub fn colx(c: f32) -> f32 { MARGIN + c * (CW + GUTTER) }
pub fn span(n: f32) -> f32 { n * CW + (n - 1.0).max(0.0) * GUTTER }
/// The stage's height in units.
pub const STAGE_H: f32 = UNITS / ASPECT;

/// The three type sizes of a chapter (units): display, body, caption.
#[derive(Clone, Copy)]
struct Type { d: f32, b: f32, c: f32 }

/// Building one chapter: the state, the geometry, the chapter, its type.
struct Cx<'a> { g: &'a Gallery, f: &'a Film, geo: Geo, c: usize, t: Type }
impl Cx<'_> {
    fn u(&self, v: f32) -> f32 { v * self.geo.k }
    fn key(&self, name: &str) -> String { format!("film/c{:02}/{name}", self.c + 1) }
    /// Place an element at (x, y) units in the stage.
    fn at(&self, el: El<Msg>, x: f32, y: f32) -> El<Msg> { el.absolute(Some(self.u(x)), Some(self.u(y)), None, None) }
    fn display(&self) -> f32 { self.u(self.t.d) }
    fn body(&self) -> f32 { self.u(self.t.b).max(12.0) }
    fn caption(&self) -> f32 { self.u(self.t.c).max(10.5) }
    fn words(&self) -> String { let s = self.f.sentence.text.trim(); if s.is_empty() { "松开，再合上".into() } else { s.to_owned() } }
    /// The sentence actor.
    fn sentence(&self, size: f32, paint: Paint) -> El<Msg> {
        text(self.words(), size, paint).font(Font::Display).key(self.key("sentence")).flow("film-sentence").nowrap()
    }
    /// The number actor (it recomposes when it changes).
    fn number(&self, size: f32, paint: Paint) -> El<Msg> {
        text(format!("{:02}", self.f.n.round() as i64), size, paint).font(Font::Display).key(self.key("number")).flow("film-number")
    }
    /// The object actor: a low-poly solid, `w` × `h` units.
    fn object(&self, sc: Scene3d, w: f32, h: f32) -> El<Msg> {
        scene3d(sc).key(self.key("object")).flow("film-object").width(self.u(w)).height(self.u(h))
    }
    fn label(&self, s: impl Into<String>, size: f32, paint: Paint) -> El<Msg> { text(s, size, paint) }
    fn mono(&self, s: impl Into<String>, size: f32, paint: Paint) -> El<Msg> { text(s, size, paint).font(Font::Mono) }
}

/// The object: an icosahedron subdivided once (80 faces), turned by `yaw`, in `paint`.
pub fn object_scene(yaw: f32, paint: Paint, projection: Projection) -> Scene3d {
    let mut sc = Scene3d::single(MeshId::ICO[1]);
    sc.items[0].transforms = vec![Mat4::rotate_y(yaw).mul(&Mat4::rotate_x(-14.0)).mul(&Mat4::uniform(1.45))];
    sc.items[0].paint = paint;
    sc.camera = Camera { projection, ..Camera::default() };
    sc.spec.resolution_scale = 1.0;
    sc.light = Light::from_angles(-40.0, 50.0, 0.3);
    sc
}
fn persp() -> Projection { Projection::Perspective { fov: 35.0 } }

// ── The page ─────────────────────────────────────────────────────────────────────────────────────

pub fn view(s: &Gallery) -> El<Msg> {
    let f = &s.film;
    let geo = geo(s.width, s.height);
    let th = s.theme();
    let c = f.ch.min(CHAPTERS - 1);
    let t = TYPES[c];
    let cx = Cx { g: s, f, geo, c, t };
    let parts = match c {
        0 => normcore(&cx),
        1 => cyber(&cx),
        2 => manual(&cx),
        3 => detector(&cx),
        4 => board(&cx),
        5 => zine(&cx),
        6 => poster(&cx),
        7 => monitor(&cx),
        8 => device(&cx),
        9 => bauhaus(&cx),
        10 => chrome(&cx),
        _ => collage(&cx),
    };
    // Everything of a chapter lives inside the stage; its own presses reach the stage (DESIGN.md §1.15).
    let mut stage = col().key(format!("film/stage/{:02}", c + 1)).width(geo.sw).height(geo.sh).clip().ripple_scope().gaze_scope()
        .absolute(Some(geo.sx), Some(geo.sy), None, None);
    for p in parts { stage = stage.child(p); }
    col().key("film").fonts(th.roles()).fill_width().fill_height()
        .child(corners(&geo))
        .child(stage)
        .child(slate(s, &geo))
        .child(bottom(s, &geo))
}

/// Four crop marks just outside the stage's corners: the frame, without a box.
fn corners(g: &Geo) -> El<Msg> {
    let (l, w) = (10.0, 1.0);
    let mut c = col().key("film/frame").absolute(Some(0.0), Some(0.0), None, None).width(0.0).height(0.0).respond(Response::Ignore).material(Material::Plain.id());
    for (i, (x, y, sx, sy)) in [(g.sx, g.sy, -1.0, -1.0), (g.sx + g.sw, g.sy, 1.0, -1.0), (g.sx, g.sy + g.sh, -1.0, 1.0), (g.sx + g.sw, g.sy + g.sh, 1.0, 1.0)].into_iter().enumerate() {
        let hx = if sx < 0.0 { x - 5.0 - l } else { x + 5.0 };
        let vy = if sy < 0.0 { y - 5.0 - l } else { y + 5.0 };
        c = c.child(space(l, w).fill(Paint::DIM).key(format!("film/frame/h{i}")).absolute(Some(hx), Some(y - w * 0.5), None, None))
             .child(space(w, l).fill(Paint::DIM).key(format!("film/frame/v{i}")).absolute(Some(x - w * 0.5), Some(vy), None, None));
    }
    c
}

/// The slate above the stage: the work's title on the left; the chapter and its preset on the right.
fn slate(s: &Gallery, g: &Geo) -> El<Msg> {
    let f = &s.film;
    let st = if f.playing { "▶" } else { "||" };
    row().key("film/slate").material(Material::Plain.id()).width(g.sw).absolute(Some(g.sx), Some((g.sy - g.bar * 1.4 - 22.0).max(4.0)), None, None).align(AlignItems::CENTER).calm()
        .child(text("gui-lab · 松开，再合上", g.bar, Paint::TEXT).font(Font::Mono).key("film/title"))
        .child(space(0.0, 0.0).grow(1.0))
        .child(text(format!("{st}  {:02} / {CHAPTERS}   {}", f.ch + 1, CH[f.ch].name), g.bar, Paint::TEXT).font(Font::Mono).key("film/slate/at").recompose(false))
}

/// Below the stage: the timeline (a hairline with a tick per chapter; hover a tick for its name, press
/// it to go there), then the caption, the hint and the way to the gallery.
fn bottom(s: &Gallery, g: &Geo) -> El<Msg> {
    let f = &s.film;
    let y = g.sy + g.sh + 16.0;
    let mut line = col().key("film/timeline").width(g.sw).height(26.0);
    line = line.child(space(g.sw, 1.0).fill(Paint::FAINT).key("film/timeline/rule").absolute(Some(0.0), Some(12.0), None, None));
    for (i, chapter) in CH.iter().enumerate() {
        let x = i as f32 / (CHAPTERS - 1) as f32 * (g.sw - 1.0);
        let on = i == f.ch;
        let k = format!("film/tick/{:02}", i + 1);
        let hot = f.hover.as_deref() == Some(k.as_str());
        let h = if on { 13.0 } else if hot { 10.0 } else { 7.0 };
        let mark = space(if on { 2.0 } else { 1.0 }, h).fill(if on { Paint::ACCENT } else if hot { Paint::INK } else { Paint::DIM });
        let tick = col().key(k.clone()).width(18.0).height(26.0).align(AlignItems::CENTER).justify(taffy::JustifyContent::CENTER)
            .on_press(Msg::Film(FMsg::Go(i))).ripple(Scope::Off).label(format!("{:02} {}", i + 1, chapter.name)).focusable(false)
            .absolute(Some((x - 9.0).clamp(-8.0, g.sw - 10.0)), Some(0.0), None, None)
            .child(mark.key(format!("{k}/mark")));
        line = line.child(tick);
        if hot {
            let name = format!("{:02} {}", i + 1, chapter.name);
            let w = name.chars().count() as f32 * g.bar * 0.62 + 8.0;
            line = line.child(text(name, g.bar, Paint::INK).font(Font::Mono).key("film/tick/name").absolute(Some((x - w * 0.5).clamp(0.0, g.sw - w)), Some(-g.bar * 1.5), None, None));
        }
    }
    let ch = &CH[f.ch];
    col().key("film/bottom").width(g.sw).gap(4.0).absolute(Some(g.sx), Some(y), None, None).calm().material(Material::Plain.id())
        .child(line)
        .child(text(ch.caption, g.bar + 1.0, Paint::TEXT).key("film/caption").recompose(false))
        .child(row().key("film/notes").width(g.sw).gap(18.0).align(AlignItems::CENTER)
            .child(text(ch.hint, g.bar, Paint::DIM).font(Font::Mono).key("film/hint").recompose(false).grow(1.0).shrink())
            .child(button_as("film/gallery", "gallery →", Variant::Quiet, Msg::Go(Page::Comp(0)))))
}

/// Each chapter's three sizes (units): display, body, caption.
const TYPES: [Type; CHAPTERS] = [
    Type { d: 64.0, b: 19.0, c: 13.0 }, Type { d: 30.0, b: 18.0, c: 13.0 }, Type { d: 34.0, b: 16.0, c: 12.0 }, Type { d: 56.0, b: 16.0, c: 12.0 },
    Type { d: 60.0, b: 30.0, c: 14.0 }, Type { d: 70.0, b: 18.0, c: 12.0 }, Type { d: 58.0, b: 120.0, c: 13.0 }, Type { d: 40.0, b: 16.0, c: 12.0 },
    Type { d: 30.0, b: 15.0, c: 12.0 }, Type { d: 84.0, b: 18.0, c: 13.0 }, Type { d: 86.0, b: 20.0, c: 13.0 }, Type { d: 40.0, b: 16.0, c: 12.0 },
];

// ── 1 Normcore: a plain settings form ────────────────────────────────────────────────────────────
fn normcore(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let lab = |s: &str| x.label(s, x.body(), Paint::DIM).width(x.u(140.0));
    let draft = x.g.film.drafts.as_ref().filter(|d| d.0 == 1).map(|d| d.1.as_str());
    let form = col().key(x.key("form")).gap(x.u(22.0)).width(x.u(span(5.0)))
        .child(row().key(x.key("row-title")).align(AlignItems::CENTER).child(lab("标题"))
            .child(input(&x.key("title"), &f.sentence, "标题", x.u(span(5.0) - 140.0).max(160.0), 24, |e| Msg::Film(FMsg::Sentence(e)))))
        .child(row().key(x.key("row-motion")).align(AlignItems::CENTER).child(lab("旋转物体"))
            .child(switch(x.key("motion"), "", f.motion, None, Msg::Film(FMsg::Motion))))
        .child(row().key(x.key("row-count")).align(AlignItems::CENTER).child(lab("数量"))
            .child(number(&x.key("count"), "数量", f.n, N_SPEC, draft, |e| Msg::Film(FMsg::Num(e)))));
    vec![
        x.at(x.label("设置 · 外观与内容", x.caption(), Paint::DIM).key(x.key("eyebrow")), colx(0.0), 76.0),
        x.at(x.sentence(x.display(), Paint::INK), colx(0.0), 104.0),
        x.at(x.label("数量", x.caption(), Paint::DIM).key(x.key("count-label")), colx(10.0), 76.0),
        x.at(x.number(x.display(), Paint::INK), colx(10.0), 104.0),
        x.at(space(x.u(span(12.0)), 1.0).fill(Paint::FAINT).key(x.key("rule")), colx(0.0), 214.0),
        x.at(form, colx(0.0), 262.0),
        x.at(x.object(object_scene(f.yaw, Paint::INK, persp()), span(5.0), 380.0), colx(7.0), 236.0),
    ]
}

// ── 2 Cyberminimalism: a command palette ─────────────────────────────────────────────────────────
/// The palette's commands: the twelve chapters, then two that act here.
pub fn commands() -> Vec<String> {
    let mut v: Vec<String> = CH.iter().enumerate().map(|(i, c)| format!("go {:02} {}", i + 1, c.name.to_lowercase())).collect();
    v.push("spin object".into());
    v.push("count +1".into());
    v
}
/// The commands the query matches (every character of it, in order), as indices.
pub fn matches(q: &str) -> Vec<usize> {
    let q: Vec<char> = q.to_lowercase().chars().filter(|c| !c.is_whitespace()).collect();
    commands().iter().enumerate().filter(|(_, c)| {
        let mut it = c.chars();
        q.iter().all(|ch| it.any(|x| x == *ch))
    }).map(|(i, _)| i).collect()
}
fn cyber(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let found = matches(&f.query.text);
    let cmds = commands();
    let sel = f.sel.min(found.len().saturating_sub(1));
    let w = span(6.0);
    let mut list = col().key(x.key("results")).gap(x.u(4.0)).width(x.u(w));
    // Rows are keyed by place, so a new filter recomposes the characters that changed in each row.
    for r in 0..6 {
        let (s, on) = match found.get(r) { Some(&i) => (cmds[i].clone(), r == sel), None => (String::new(), false) };
        let row_el = row().key(x.key(&format!("row{r}"))).height(x.u(40.0)).gap(x.u(14.0)).align(AlignItems::CENTER).pad(x.u(12.0), 0.0)
            .child(space(2.0, x.u(22.0)).fill(if on { Paint::ACCENT } else { Paint::GROUND }).key(x.key(&format!("row{r}/bar"))))
            .child(x.mono(s, x.body(), if on { Paint::ACCENT } else { Paint::TEXT }).key(x.key(&format!("row{r}/text"))));
        let row_el = match found.get(r) { Some(&i) => row_el.on_press(Msg::Film(FMsg::Run(i))).focusable(false), None => row_el };
        list = list.child(row_el);
    }
    let field = input(&x.key("query"), &f.query, "输入命令…", x.u(w - 40.0), 32, |e| Msg::Film(FMsg::Query(e)));
    vec![
        x.at(x.sentence(x.display(), Paint::TEXT), colx(0.0), 70.0),
        x.at(x.mono("results", x.caption(), Paint::DIM).key(x.key("n-label")), colx(10.0), 52.0),
        x.at(x.number(x.display(), Paint::TEXT).font(Font::Mono), colx(10.0), 70.0),
        x.at(row().key(x.key("prompt")).gap(x.u(12.0)).align(AlignItems::CENTER).child(x.mono("›", x.body() * 1.3, Paint::ACCENT)).child(field), colx(3.0), 168.0),
        x.at(space(x.u(w), 1.0).fill(Paint::FAINT).key(x.key("rule")), colx(3.0), 226.0),
        x.at(list, colx(3.0), 240.0),
        x.at(x.mono(format!("{} / {}    ↑↓ 选择   enter 执行", found.len(), cmds.len()), x.caption(), Paint::DIM).key(x.key("count")).recompose(false), colx(3.0), 520.0),
        x.at(x.object(object_scene(f.yaw, Paint::DIM, persp()), 120.0, 120.0), colx(11.0) - 10.0, 520.0),
    ]
}

// ── 3 Tech-Manual: an orthographic drawing ───────────────────────────────────────────────────────
pub const PARTS: [(&str, &str); 4] = [("外壳", "shell"), ("面", "face × 80"), ("棱", "edge × 120"), ("顶点", "vertex × 42")];
fn manual_scene(f: &Film) -> Scene3d {
    let mut sc = object_scene(0.0, Paint::INK, Projection::Orthographic { height: 2.6 });
    sc.items[0].transforms = vec![Mat4::rotate_x(-14.0).mul(&Mat4::uniform(1.55))];
    sc.camera = f.orbit.camera(sc.camera);
    sc
}
/// The projected bounds of the object in its view (fractions of the view) and four points the callouts
/// point at: the top, the left and the right of its outline, and the vertex nearest the eye.
pub fn measured(sc: &Scene3d, aspect: f32) -> ([f32; 4], [(f32, f32); 4]) {
    let lib = crate::mesh::Library::default();
    let Some(mesh) = lib.get(sc.items[0].mesh) else { return ([0.0, 0.0, 1.0, 1.0], [(0.5, 0.5); 4]) };
    let m = sc.items[0].transforms[0];
    let c = sc.camera.proj(aspect).mul(&sc.camera.view());
    let q = &c.0;
    let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    let (mut top, mut left, mut right, mut near) = ((0.0, f32::MAX), (f32::MAX, 0.0), (f32::MIN, 0.0), (0.5, 0.5, f32::MAX));
    for t in &mesh.tris { for p in t {
        let w = m.point(*p);
        let cl = [q[0] * w[0] + q[4] * w[1] + q[8] * w[2] + q[12], q[1] * w[0] + q[5] * w[1] + q[9] * w[2] + q[13], q[2] * w[0] + q[6] * w[1] + q[10] * w[2] + q[14], q[3] * w[0] + q[7] * w[1] + q[11] * w[2] + q[15]];
        let (fx, fy, z) = ((cl[0] / cl[3] + 1.0) * 0.5, (1.0 - cl[1] / cl[3]) * 0.5, cl[2] / cl[3]);
        b = [b[0].min(fx), b[1].min(fy), b[2].max(fx), b[3].max(fy)];
        if fy < top.1 { top = (fx, fy); }
        if fx < left.0 { left = (fx, fy); }
        if fx > right.0 { right = (fx, fy); }
        if z < near.2 { near = (fx, fy, z); }
    } }
    (b, [top, left, right, (near.0, near.1)])
}
fn manual(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let (vw, vh) = (span(6.0), 470.0);
    let (ox, oy) = (colx(0.0) + 20.0, 74.0);
    let sc = manual_scene(f);
    let (bb, pts) = measured(&sc, vw / vh);
    let mut out = vec![];
    // The view, in a real frame.
    out.push(x.at(space(x.u(vw), x.u(vh)).border(1.0, Paint::INK).key(x.key("frame")).respond(Response::Ignore), ox, oy));
    let view = orbit(&x.key("object"), sc, &f.orbit, |m| Msg::Film(FMsg::Orbit(m))).flow("film-object").width(x.u(vw)).height(x.u(vh));
    out.push(x.at(view, ox, oy));
    // The callouts: a small mark on each measured point, with a leader to its numbered label.
    for (i, (px, py)) in pts.iter().enumerate() {
        let on = f.hover.as_deref().is_some_and(|h| h.ends_with(&format!("part{i}"))) ;
        let s = if on { 9.0 } else { 6.0 };
        let mk = space(x.u(s).max(5.0), x.u(s).max(5.0)).radius(x.u(s)).fill(if on { Paint::ACCENT } else { Paint::INK }).key(x.key(&format!("callout{i}")))
            .annotate(format!("{} {}", PARTS[i].0, PARTS[i].1)).respond(Response::Ignore);
        out.push(x.at(mk, ox + px * vw - s * 0.5, oy + py * vh - s * 0.5));
    }
    // Dimensions, measured from the projection: across under the frame, and up its right side.
    let (dx0, dx1) = (ox + bb[0] * vw, ox + bb[2] * vw);
    let (dy0, dy1) = (oy + bb[1] * vh, oy + bb[3] * vh);
    let wpx = (bb[2] - bb[0]) * x.u(vw);
    let hpx = (bb[3] - bb[1]) * x.u(vh);
    let yd = oy + vh + 26.0;
    out.push(x.at(space(x.u(dx1 - dx0), 1.0).fill(Paint::INK).key(x.key("dim-w")), dx0, yd));
    out.push(x.at(space(1.0, x.u(16.0)).fill(Paint::INK).key(x.key("dim-w0")), dx0, yd - 8.0));
    out.push(x.at(space(1.0, x.u(16.0)).fill(Paint::INK).key(x.key("dim-w1")), dx1, yd - 8.0));
    out.push(x.at(x.mono(format!("{wpx:.1}"), x.caption(), Paint::INK).key(x.key("dim-w-label")), (dx0 + dx1) * 0.5 - 24.0, yd + 6.0));
    let xd = ox + vw + 26.0;
    out.push(x.at(space(1.0, x.u(dy1 - dy0)).fill(Paint::INK).key(x.key("dim-h")), xd, dy0));
    out.push(x.at(space(x.u(16.0), 1.0).fill(Paint::INK).key(x.key("dim-h0")), xd - 8.0, dy0));
    out.push(x.at(space(x.u(16.0), 1.0).fill(Paint::INK).key(x.key("dim-h1")), xd - 8.0, dy1));
    out.push(x.at(x.mono(format!("{hpx:.1}"), x.caption(), Paint::INK).key(x.key("dim-h-label")), xd + 8.0, (dy0 + dy1) * 0.5 - 8.0));
    // The parts list; the row under the pointer marks its callout.
    let mut list = col().key(x.key("parts")).width(x.u(span(4.0))).gap(0.0)
        .child(row().key(x.key("parts/head")).height(x.u(34.0)).align(AlignItems::CENTER).gap(x.u(16.0))
            .child(x.mono("№", x.caption(), Paint::DIM).width(x.u(40.0))).child(x.mono("零件", x.caption(), Paint::DIM).width(x.u(120.0))).child(x.mono("数量", x.caption(), Paint::DIM)));
    for (i, (zh, en)) in PARTS.iter().enumerate() {
        let k = x.key(&format!("part{i}"));
        let on = f.hover.as_deref() == Some(k.as_str());
        list = list.child(space(x.u(span(4.0)), 1.0).fill(Paint::FAINT).key(x.key(&format!("part{i}/rule"))))
            .child(row().key(k).height(x.u(40.0)).align(AlignItems::CENTER).gap(x.u(16.0)).on_press(Msg::Film(FMsg::Num(Edit::Set(f.n)))).focusable(false)
                .child(x.mono(format!("{:02}", i + 1), x.body(), if on { Paint::ACCENT } else { Paint::INK }).width(x.u(40.0)))
                .child(x.label(*zh, x.body(), if on { Paint::ACCENT } else { Paint::INK }).width(x.u(120.0)))
                .child(x.mono(*en, x.caption(), Paint::DIM)));
    }
    out.push(x.at(list, colx(8.0), 84.0));
    // The title block: figure, the sentence, scale with the number.
    let block = col().key(x.key("titleblock")).width(x.u(span(4.0))).pad(x.u(16.0), x.u(12.0)).gap(x.u(6.0)).border(1.0, Paint::INK)
        .child(x.mono("图 03 · 正交视图", x.caption(), Paint::DIM))
        .child(x.sentence(x.display(), Paint::INK))
        .child(row().gap(x.u(10.0)).align(AlignItems::BASELINE).key(x.key("scale")).child(x.mono("比例 1 :", x.caption(), Paint::DIM)).child(x.number(x.body() * 1.4, Paint::INK).font(Font::Mono)));
    out.push(x.at(block, colx(8.0), 420.0));
    out
}

// ── 4 Algorithmic Gaze: the stage as a detector sees it ──────────────────────────────────────────
fn detector(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let ided = |k: &str| f.ident.as_deref() == Some(k);
    let thing = |el: El<Msg>, k: String, kind: &str| {
        let note = if ided(&k) { format!("{kind} · 已识别 · {{xy}}") } else { format!("{kind} · {{xy}}") };
        el.annotate(note).legible(false).respond(Response::Loosen).on_press(Msg::Film(FMsg::Identify(k))).focusable(true)
    };
    let sk = x.key("sentence");
    let nk = x.key("number");
    let ok = x.key("object");
    let pk = x.key("picture");
    let ck = x.key("caption");
    vec![
        x.at(thing(x.sentence(x.display(), Paint::INK), sk, "文字"), colx(0.0), 96.0),
        x.at(thing(x.number(x.display(), Paint::ACCENT), nk, "数字"), colx(10.0) + 40.0, 96.0),
        x.at(thing(x.object(object_scene(f.yaw, Paint::INK, persp()), 440.0, 400.0), ok, "物体"), colx(4.0), 196.0),
        x.at(thing(col().key(pk.clone()).width(x.u(span(3.0))).child(picture(TERRAIN, super::pages::ratio(TERRAIN)).key(x.key("terrain")).pic_fx(STYLE_SLOT)), pk, "图片"), colx(9.0), 300.0),
        x.at(thing(x.mono(format!("frame {:04}", (f.n * 37.0) as i64 % 10000), x.body(), Paint::DIM).key(ck.clone()), ck, "文字"), colx(0.0), 560.0),
    ]
}

// ── 5 Neo-Dotmatrix: a departure board ───────────────────────────────────────────────────────────
pub const DEST: [&str; 4] = ["松开", "合上", "再松开", "回到原处"];
fn board(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let mut b = col().key(x.key("board")).width(x.u(span(7.0))).gap(x.u(10.0))
        .child(row().key(x.key("board/head")).gap(x.u(24.0))
            .child(x.label("时间", x.caption(), Paint::DIM).width(x.u(130.0))).child(x.label("去往", x.caption(), Paint::DIM).width(x.u(300.0))).child(x.label("站台", x.caption(), Paint::DIM)));
    for (i, d) in DEST.iter().enumerate() {
        let v = f.rows[i].round() as i64 + i as i64 * 7;
        b = b.child(row().key(x.key(&format!("row{i}"))).gap(x.u(24.0)).align(AlignItems::CENTER)
            .child(x.label(format!("{:02}:{:02}", (9 + v / 60) % 24, v % 60), x.body(), Paint::ACCENT).key(x.key(&format!("row{i}/time"))).width(x.u(130.0)))
            .child(x.label(*d, x.body(), Paint::INK).key(x.key(&format!("row{i}/dest"))).width(x.u(300.0)))
            .child(x.label(format!("{}", (v % 9) + 1), x.body(), Paint::INK).key(x.key(&format!("row{i}/gate")))));
    }
    let obj = x.object(object_scene(f.yaw, Paint::INK, persp()), 520.0, 460.0).loose(0.8).on_pick(|p| Some(Msg::Film(FMsg::Pick(p))));
    vec![
        x.at(x.label("出发 · DEPARTURES", x.caption(), Paint::DIM).key(x.key("eyebrow")), colx(0.0), 70.0),
        x.at(x.sentence(x.display(), Paint::INK), colx(0.0), 96.0),
        x.at(b, colx(0.0), 240.0),
        x.at(x.label("分", x.caption(), Paint::DIM).key(x.key("n-label")), colx(6.0), 70.0),
        x.at(x.number(x.display(), Paint::ACCENT), colx(6.0), 96.0),
        x.at(obj, colx(7.0) + 40.0, 130.0),
    ]
}

// ── 6 Lo-Fi Print: a zine spread ─────────────────────────────────────────────────────────────────
pub const ZINE: &str = "一样东西松开的时候，它不是消失了，而是把自己摊开：每一个点都还在，只是不再挨着。印在纸上的字也是这样，墨从笔画里渗出来一点，网点之间露出纸，复印一次，再复印一次，边缘就毛了。可你仍然认得它。复印机的光从左边扫到右边，纸上的灰变成了点，点又连成了字。认得一个东西，靠的不是它的方框，而是它的轮廓：这一行字绕着这个物体走，贴着它真实的边，在它凸出来的地方让开，在它收回去的地方又靠过去。把它拖到别处，字就跟过去，重新找自己的位置。纸的边上有一道折痕，是上一期留下的。";
pub const ZINE2: &str = "这一页是为了被拿在手里。它有自己的纸色，有一种专色，套印的时候差了一点点，于是红色的那一层落在黑色旁边。我们不修正它。对比度可以调：调高，灰的地方变成黑；调低，整页褪成一张旧的复印件。";
/// The object's outline in the zine, rows of the text box's coordinates (logical px): its faces as the
/// view projects them, filled row by row.
fn outline(sc: &Scene3d, ox: f32, oy: f32, w: f32, h: f32) -> Silhouette {
    let lib = crate::mesh::Library::default();
    let Some(mesh) = lib.get(sc.items[0].mesh) else { return Silhouette::circle(ox + w * 0.5, oy + h * 0.5, w * 0.4) };
    let m = sc.items[0].transforms[0];
    let c = sc.camera.proj(w / h.max(1.0)).mul(&sc.camera.view());
    let q = &c.0;
    let proj = |p: [f32; 3]| { let w0 = m.point(p); let cl = [q[0] * w0[0] + q[4] * w0[1] + q[8] * w0[2] + q[12], q[1] * w0[0] + q[5] * w0[1] + q[9] * w0[2] + q[13], q[3] * w0[0] + q[7] * w0[1] + q[11] * w0[2] + q[15]];
        (ox + (cl[0] / cl[2] + 1.0) * 0.5 * w, oy + (1.0 - cl[1] / cl[2]) * 0.5 * h) };
    let top = oy.floor();
    let mut rows = vec![(f32::INFINITY, f32::NEG_INFINITY); h.ceil() as usize + 2];
    for t in &mesh.tris {
        let v = [proj(t[0]), proj(t[1]), proj(t[2])];
        let (y0, y1) = (v.iter().map(|p| p.1).fold(f32::MAX, f32::min), v.iter().map(|p| p.1).fold(f32::MIN, f32::max));
        let (x0, x1) = (v.iter().map(|p| p.0).fold(f32::MAX, f32::min), v.iter().map(|p| p.0).fold(f32::MIN, f32::max));
        for r in ((y0 - top).floor().max(0.0) as usize)..((y1 - top).ceil().max(0.0) as usize).min(rows.len()) {
            rows[r] = (rows[r].0.min(x0), rows[r].1.max(x1));
        }
    }
    Silhouette { top, rows }
}
fn zine(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let (tx, ty, tw, th) = (colx(0.0), 196.0, span(6.0) - 30.0, 420.0);
    let (ow, oh) = (300.0, 300.0);
    let (bx, by) = (f.obj.0.clamp(-60.0, tw - ow + 60.0), f.obj.1.clamp(-40.0, th - oh + 40.0));
    let mut sc = object_scene(f.yaw, Paint::hue(0), persp());
    sc.picture = Some(STYLE_SLOT);
    // The obstacle in the text's own box, logical px.
    let sil = outline(&sc, x.u(bx), x.u(by), x.u(ow), x.u(oh));
    let body = text_lh(ZINE, x.body(), x.body() * 1.7, Paint::INK).key(x.key("body")).width(x.u(tw)).around(Around::new(vec![sil], x.u(14.0)));
    let draft = x.g.film.drafts.as_ref().filter(|d| d.0 == 6).map(|d| d.1.as_str());
    let obj = x.object(sc, ow, oh).loose(1.0).cursor(crate::el::Cursor::Grab)
        .on_drag_with({ let (bx, by) = (bx, by); move |d: Drag| Some(Msg::Film(FMsg::Obj(bx + d.dx / d.w.max(1.0) * ow, by + d.dy / d.h.max(1.0) * oh))) });
    vec![
        x.at(x.label("小志 · 第 3 期", x.caption(), Paint::DIM).key(x.key("eyebrow")), colx(0.0), 60.0),
        x.at(x.sentence(x.display(), Paint::INK).weight(1.5), colx(0.0), 84.0),
        x.at(body, tx, ty),
        x.at(obj, tx + bx, ty + by),
        x.at(space(1.0, x.u(500.0)).fill(Paint::FAINT).key(x.key("gutter")), 800.0, 90.0),
        x.at(text_lh(ZINE2, x.body(), x.body() * 1.7, Paint::INK).key(x.key("body2")).width(x.u(span(5.0))), colx(6.5), 110.0),
        x.at(number(&x.key("contrast"), "对比度", f.contrast, CONTRAST_SPEC, draft, |e| Msg::Film(FMsg::Contrast(e))).width(x.u(300.0).max(260.0)), colx(6.5), 470.0),
        x.at(row().key(x.key("folio")).gap(x.u(10.0)).align(AlignItems::BASELINE).child(x.label("—", x.body(), Paint::DIM)).child(x.number(x.body() * 1.2, Paint::INK)).child(x.label("—", x.body(), Paint::DIM)), 1170.0, 570.0),
    ]
}

// ── 7 Metamorphosis Linocut: a poster that becomes itself ────────────────────────────────────────
pub const FORMS: usize = 3;
pub fn form_key(i: usize) -> String { format!("film/c07/form{i}") }
fn poster(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let (w, h) = (span(7.0), 470.0);
    let forms = [
        x.object(object_scene(f.yaw, Paint::INK, persp()), w, h).key(form_key(0)),
        col().key(form_key(1)).width(x.u(w)).height(x.u(h)).align(AlignItems::CENTER).justify(taffy::JustifyContent::CENTER)
            .child(text(x.words(), x.u(x.t.b), Paint::INK).font(Font::Display).weight(x.u(3.0)).key("film/c07/word").nowrap()),
        col().key(form_key(2)).width(x.u(w * 0.9)).child(picture(SKY, super::pages::ratio(SKY)).key("film/c07/print").pic_fx(STYLE_SLOT).loose(1.0)),
    ];
    let current = f.form % FORMS;
    let st = stack(forms.into_iter().enumerate().map(|(i, el)| el.hidden_until_morphed(i != current).respond(Response::Loosen).loose(0.55)))
        .key(x.key("forms")).width(x.u(w)).height(x.u(h)).focusable(true).label("变形").cursor(crate::el::Cursor::Grab)
        .on_drag_with(|d: Drag| Some(Msg::Film(FMsg::Scrub(d))))
        .on_key(|k, _| matches!(k, Key::Enter | Key::Char(' ')).then_some(Msg::Film(FMsg::Form)));
    vec![
        x.at(x.sentence(x.display(), Paint::INK).weight(2.0), colx(0.0), 64.0),
        x.at(x.label("No.", x.caption(), Paint::DIM).key(x.key("n-label")), colx(10.0) + 30.0, 46.0),
        x.at(x.number(x.display(), Paint::hue(0)).weight(2.0), colx(10.0) + 30.0, 64.0),
        x.at(st, colx(2.5), 150.0),
        x.at(x.label("凸版 · 一刀一刀刻出来的形状，按下就变成下一个", x.caption(), Paint::DIM).key(x.key("note")), colx(0.0), 610.0),
    ]
}

// ── 8 Glitch: a signal monitor ───────────────────────────────────────────────────────────────────
pub const BARS: usize = 96;
fn monitor(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let (wx, wy, ww, wh) = (colx(0.0), 190.0, span(12.0), 300.0);
    let bw = ww / BARS as f32;
    let mut wave = col().key(x.key("wave")).width(x.u(ww)).height(x.u(wh)).cursor(crate::el::Cursor::Grab)
        .on_drag_with(|d: Drag| Some(Msg::Film(FMsg::Tear(d)))).focusable(false).respond(Response::Loosen);
    let band = 46.0;
    for i in 0..BARS {
        let ph = i as f32 / BARS as f32 * std::f32::consts::TAU;
        let a = (0.45 * (ph * 3.0 + f.n * 0.3).sin() + 0.3 * (ph * 7.0).sin() + 0.15 * (ph * 17.0 + 1.0).sin()).abs();
        let bh = (a * wh).clamp(6.0, wh);
        let (y0, y1) = (wh * 0.5 - bh * 0.5, wh * 0.5 + bh * 0.5);
        // Cut the bar where a band is torn (loose while held) or locked (moved sideways, at rest).
        let mut cuts: Vec<(f32, f32, f32, f32)> = vec![(y0, y1, 0.0, 0.0)];
        let mut bands: Vec<(f32, f32, f32, f32)> = f.locks.iter().map(|&(c, dx)| (c * wh - band * 0.5, c * wh + band * 0.5, dx, 0.0)).collect();
        if let Some(c) = f.tear { bands.push((c * wh - band * 0.5, c * wh + band * 0.5, 0.0, 1.0)); }
        for (b0, b1, dx, loose) in bands {
            let mut next = vec![];
            for (a0, a1, d0, l0) in cuts {
                if a1 <= b0 || a0 >= b1 { next.push((a0, a1, d0, l0)); continue; }
                if a0 < b0 { next.push((a0, b0, d0, l0)); }
                next.push((a0.max(b0), a1.min(b1), d0 + dx, l0.max(loose)));
                if a1 > b1 { next.push((b1, a1, d0, l0)); }
            }
            cuts = next;
        }
        for (j, (a0, a1, dx, loose)) in cuts.into_iter().enumerate() {
            let hue = if (i * 5) % 11 == 0 { Paint::hue(1) } else { Paint::INK };
            wave = wave.child(space(x.u(bw * 0.55).max(1.0), x.u(a1 - a0).max(1.0)).fill(hue).key(x.key(&format!("bar{i}/{j}"))).loose(loose)
                .absolute(Some(x.u(i as f32 * bw + dx)), Some(x.u(a0)), None, None));
        }
    }
    let close = Msg::Film(FMsg::Error(false));
    let dlg = dialog(&x.key("error"), f.error, "信号丢失", 380.0, 0.35, close.clone(),
        col().gap(14.0).child(x.label("通道 08 有三段被锁在了错开的位置。重试会把它们放回去。", 14.0, Paint::TEXT).width(330.0))
            .child(row().gap(10.0).child(button_as(x.key("retry"), "重试", Variant::Accent, Msg::Film(FMsg::Retry))).child(button_as(x.key("ignore"), "忽略", Variant::Default, close))));
    vec![
        x.at(x.mono("CH 08 · 监视", x.caption(), Paint::DIM).key(x.key("eyebrow")), colx(0.0), 60.0),
        x.at(x.sentence(x.display(), Paint::INK), colx(0.0), 84.0),
        x.at(x.mono("SNR dB", x.caption(), Paint::DIM).key(x.key("n-label")), colx(10.0), 60.0),
        x.at(x.number(x.display(), Paint::hue(1)).font(Font::Mono), colx(10.0), 84.0),
        x.at(space(x.u(ww), 1.0).fill(Paint::FAINT).key(x.key("axis")), wx, wy + wh * 0.5),
        x.at(wave, wx, wy),
        x.at(x.mono(format!("locked {}   ·   hold to tear, release to lock", f.locks.len()), x.caption(), Paint::DIM).key(x.key("status")).recompose(false), colx(0.0), 540.0),
        x.at(button_as(x.key("raise"), "注入错误", Variant::Default, Msg::Film(FMsg::Error(true))), colx(6.0), 532.0),
        x.at(x.object(object_scene(f.yaw, Paint::INK, persp()), 130.0, 130.0), colx(11.0) - 30.0, 510.0),
        dlg,
    ]
}

// ── 9 Transparent Electronics: a device seen through ─────────────────────────────────────────────
pub const LAYERS: [&str; 4] = ["背板", "电路", "屏幕", "玻璃"];
fn device(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let (dw, dh) = (360.0, 470.0);
    let (cx0, cy0) = (colx(4.0) + 20.0, 96.0);
    let (px, py) = (f.ptr.0 - 0.5, f.ptr.1 - 0.5);
    let mut out = vec![];
    for (i, name) in LAYERS.iter().enumerate() {
        let depth = i as f32;
        // Exploded: the layers stand apart to the upper right; the pointer moves the nearer ones more.
        let spread = if f.exploded { 92.0 } else { 0.0 };
        let (lx, ly) = (cx0 + depth * spread * 1.1 + px * depth * 26.0, cy0 - depth * spread * 0.45 + py * depth * 18.0);
        let k = x.key(&format!("layer{i}"));
        let mut layer = col().key(k.clone()).width(x.u(dw)).height(x.u(dh)).radius(x.u(28.0)).annotate(*name).flow(format!("film-layer{i}"));
        layer = match i {
            0 => layer.fill(Paint::FAINT),
            1 => {
                let mut l = layer.border(1.0, Paint::DIM);
                for t in 0..6 { l = l.child(space(x.u(dw * 0.7), 1.0).fill(Paint::hue(0)).key(format!("{k}/trace{t}")).absolute(Some(x.u(dw * 0.15)), Some(x.u(70.0 + t as f32 * 64.0)), None, None)); }
                l.child(x.object(object_scene(f.yaw, Paint::INK, persp()), 250.0, 250.0).absolute(Some(x.u(55.0)), Some(x.u(120.0)), None, None).annotate("物体"))
            }
            2 => layer.border(1.0, Paint::INK).pad(x.u(26.0), x.u(30.0)).gap(x.u(10.0))
                .child(x.mono("NOW PLAYING", x.caption(), Paint::DIM).key(format!("{k}/eyebrow")))
                .child(x.sentence(x.display(), Paint::INK))
                .child(space(0.0, x.u(250.0)))
                .child(row().key(format!("{k}/track")).gap(x.u(8.0)).align(AlignItems::BASELINE).child(x.mono("track", x.caption(), Paint::DIM)).child(x.number(x.display(), Paint::hue(0)).font(Font::Mono)))
                .child(row().key(format!("{k}/controls")).gap(x.u(18.0)).child(x.mono("◀◀", x.body(), Paint::INK)).child(x.mono("▶", x.body(), Paint::hue(0))).child(x.mono("▶▶", x.body(), Paint::INK))),
            _ => layer.border(2.0, Paint::INK).respond(Response::Ignore),
        };
        out.push(x.at(layer, lx, ly));
    }
    out.push(x.at(row().key(x.key("explode-row")).width(x.u(300.0).max(200.0)).child(switch(x.key("explode"), "拆开", f.exploded, None, Msg::Film(FMsg::Explode))), colx(0.0), 560.0));
    out.push(x.at(x.label("每一层只画它的轮廓和走线，所以下面的层透得过来。", x.caption(), Paint::DIM).key(x.key("note")), colx(0.0), 96.0));
    out
}

// ── 10 Bauhaus: controls as geometry ─────────────────────────────────────────────────────────────
fn bauhaus(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let lw = span(4.0);
    let r = 15.0;
    // The slider: a line and a circle on it.
    let slider = col().key(x.key("slider")).width(x.u(lw)).height(x.u(40.0)).focusable(true).label("大小").cursor(crate::el::Cursor::Grab)
        .value_range(f.size, 0.0, 1.0)
        .on_drag_with(|d: Drag| Some(Msg::Film(FMsg::Size(d.fx()))))
        .on_key({ let v = f.size; move |k, sh| { let st = if sh { 0.2 } else { 0.05 }; match k { Key::Left => Some(Msg::Film(FMsg::Size(v - st))), Key::Right => Some(Msg::Film(FMsg::Size(v + st))), _ => None } } })
        .child(space(x.u(lw), 2.0).fill(Paint::INK).key(x.key("slider/line")).absolute(Some(0.0), Some(x.u(20.0) - 1.0), None, None))
        .child(space(x.u(r * 2.0), x.u(r * 2.0)).radius(x.u(r)).fill(Paint::hue(0)).key(x.key("slider/knob")).absolute(Some(x.u((lw - r * 2.0) * f.size)), Some(x.u(20.0 - r)), None, None));
    // The switch: two squares, the one that is on filled.
    let sq = |on: bool, k: &str| space(x.u(34.0), x.u(34.0)).key(x.key(k)).fill(if on { Paint::INK } else { Paint::GROUND }).border(2.0, Paint::INK);
    let toggle = row().key(x.key("toggle")).gap(x.u(6.0)).on_press(Msg::Film(FMsg::Squares)).label("反色").child(sq(!f.squares, "toggle/a")).child(sq(f.squares, "toggle/b"));
    // The tabs: three planes of colour; the chosen one is taller.
    let mut tabs = row().key(x.key("planes")).gap(x.u(8.0)).align(AlignItems::END);
    for i in 0..3usize {
        tabs = tabs.child(space(x.u(70.0), x.u(if i == f.plane { 56.0 } else { 34.0 })).fill(Paint::hue(i as u8)).key(x.key(&format!("plane{i}")))
            .on_press(Msg::Film(FMsg::Plane(i))).label(["圆", "方", "三角"][i]));
    }
    // The composition: the object reduced to the primitive the planes choose, sized by the slider.
    let (ccx, ccy, big) = (colx(7.5), 70.0, 300.0 + 220.0 * f.size);
    let (a, b) = if f.squares { (Paint::hue(1), Paint::hue(0)) } else { (Paint::hue(0), Paint::hue(1)) };
    let prim = match f.plane {
        0 => space(x.u(big), x.u(big)).radius(x.u(big)).fill(a),
        1 => space(x.u(big * 0.86), x.u(big * 0.86)).fill(a),
        _ => {
            let mut sc = Scene3d::single(MeshId::CONE);
            sc.items[0].transforms = vec![Mat4::uniform(2.0)];
            sc.items[0].paint = a;
            sc.camera = Camera { projection: Projection::Orthographic { height: 2.2 }, pitch: 0.0, yaw: 0.0, ..Camera::default() };
            sc.light = Light { dir: [0.0, 0.0, 1.0], ambient: 1.0 };
            scene3d(sc).width(x.u(big)).height(x.u(big))
        }
    };
    vec![
        x.at(x.label("大小", x.caption(), Paint::DIM).key(x.key("l-size")), colx(0.0), 110.0),
        x.at(slider, colx(0.0), 134.0),
        x.at(x.label("反色", x.caption(), Paint::DIM).key(x.key("l-toggle")), colx(0.0), 214.0),
        x.at(toggle, colx(0.0), 238.0),
        x.at(x.label("形", x.caption(), Paint::DIM).key(x.key("l-planes")), colx(0.0), 310.0),
        x.at(tabs, colx(0.0), 334.0),
        x.at(space(x.u(span(3.0)), x.u(100.0)).fill(b).key(x.key("square")), colx(5.0), 330.0),
        x.at(space(x.u(span(12.0)), x.u(14.0)).fill(Paint::INK).key(x.key("bar")), colx(0.0), 430.0),
        x.at(prim.key(x.key(&format!("object{}", f.plane))).flow("film-object"), ccx, ccy),
        x.at(x.sentence(x.display(), Paint::INK).weight(2.0), colx(0.0), 466.0),
        x.at(x.number(x.display() * 0.6, Paint::GROUND), colx(5.0) + 24.0, 340.0),
    ]
}

// ── 11 Neo-Y2K: chrome and a liquid menu ─────────────────────────────────────────────────────────
pub const MENU: [&str; 5] = ["播放", "收藏", "分享", "下载", "设置"];
fn chrome(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let liquid = Material::Liquid.id();
    let mut menu = col().key(x.key("menu")).gap(0.0);
    for (i, name) in MENU.iter().enumerate() {
        let k = x.key(&format!("item{i}"));
        let hot = f.hover.as_deref() == Some(k.as_str());
        let sel = i == f.menu_sel;
        let r = if hot { 40.0 } else { 31.0 };
        let shift = if sel { 54.0 } else { 0.0 };
        let blob = space(x.u(r * 2.0), x.u(r * 2.0)).radius(x.u(r)).fill(if sel { Paint::hue(1) } else { Paint::hue(0) }).material(liquid).key(format!("{k}/blob")).respond(Response::Loosen);
        menu = menu.child(row().key(k).height(x.u(66.0)).gap(x.u(20.0)).align(AlignItems::CENTER).on_press(Msg::Film(FMsg::MenuSel(i))).label(*name)
            .child(space(x.u(shift + 40.0 - r), 1.0))
            .child(blob)
            .child(x.label(*name, x.body(), if sel { Paint::INK } else { Paint::TEXT })));
    }
    // A highlight sweeps over the sentence where the pointer is.
    let hx = colx(0.0) + f.ptr.0 * span(6.0);
    vec![
        x.at(menu, colx(0.0) - 10.0, 70.0),
        x.at(x.object(object_scene(f.yaw, Paint::INK, persp()), 600.0, 520.0), colx(6.0), 40.0),
        x.at(x.sentence(x.display(), Paint::INK).weight(2.0), colx(0.0), 440.0),
        x.at(space(x.u(4.0), x.u(110.0)).fill(Paint::hue(2)).key(x.key("sweep")).respond(Response::Ignore), hx, 430.0),
        x.at(x.number(x.display() * 0.7, Paint::INK), colx(4.0), 96.0),
    ]
}

// ── 12 Mixed media: the collage, and the finale ──────────────────────────────────────────────────
fn collage(x: &Cx) -> Vec<El<Msg>> {
    let f = x.f;
    let m = |mat: Material| mat.id();
    let paper = |el: El<Msg>, k: &str, mat: Material| col().key(x.key(k)).pad(x.u(14.0), x.u(12.0)).fill(Paint::tone(0.08)).material(m(mat)).child(el);
    let frags: Vec<(El<Msg>, f32, f32)> = vec![
        (paper(row().key(x.key("f1/r")).gap(10.0).align(AlignItems::CENTER).child(x.label("旋转", x.body(), Paint::TEXT)).child(switch(x.key("f1/switch"), "", f.motion, None, Msg::Film(FMsg::Motion))), "f1", Material::Plain), colx(0.0), 70.0),
        (paper(x.mono("› go 05 neo-dotmatrix", x.body(), Paint::ACCENT).key(x.key("f2/t")), "f2", Material::Plain), colx(3.0), 60.0),
        (paper(row().gap(6.0).key(x.key("f3/r")).child(space(x.u(90.0), 1.0).fill(Paint::INK)).child(x.mono("312.4", x.caption(), Paint::INK)), "f3", Material::Plain), colx(7.0), 72.0),
        (paper(x.number(x.u(54.0), Paint::ACCENT).loose(0.0), "f5", Material::Dot), colx(0.0), 190.0),
        (paper(text_lh("墨从笔画里渗出来一点，网点之间露出纸。", x.body(), x.body() * 1.5, Paint::INK).width(x.u(220.0)).key(x.key("f6/t")), "f6", Material::Print), colx(2.0), 190.0),
        (paper(text("刻", x.u(90.0), Paint::INK).font(Font::Display).weight(3.0).key(x.key("f7/t")).loose(0.6), "f7", Material::Lino), colx(5.0) + 10.0, 170.0),
        (paper(row().gap(3.0).key(x.key("f8/w")).children((0..18).map(|i| space(3.0, x.u(12.0 + ((i * 7) % 11) as f32 * 4.0)).fill(Paint::INK).key(x.key(&format!("f8/b{i}"))))), "f8", Material::Glitch), colx(7.0), 180.0),
        (paper(col().key(x.key("f9/p")).width(x.u(150.0)).height(x.u(100.0)).radius(x.u(14.0)).fill(Paint::INK), "f9", Material::Xray), colx(9.5), 70.0),
        (paper(row().gap(x.u(8.0)).key(x.key("f10/r")).child(space(x.u(70.0), x.u(70.0)).radius(x.u(70.0)).fill(Paint::hue(0)).loose(0.8)).child(space(x.u(60.0), x.u(60.0)).fill(Paint::hue(1)).loose(0.8)), "f10", Material::Halftone), colx(0.0), 330.0),
        (paper(row().gap(0.0).key(x.key("f11/r")).child(space(x.u(56.0), x.u(56.0)).radius(x.u(56.0)).fill(Paint::hue(0))).child(space(x.u(46.0), x.u(46.0)).radius(x.u(46.0)).fill(Paint::hue(0))), "f11", Material::Liquid), colx(3.0), 340.0),
    ];
    let mut board = col().key("film/c12/board").width(x.u(UNITS)).height(x.u(STAGE_H)).respond(Response::Loosen);
    for (el, fx, fy) in frags { board = board.child(x.at(el, fx, fy)); }
    board = board
        .child(x.at(x.sentence(x.display() * 1.5, Paint::INK).weight(1.5), colx(4.0), 450.0))
        .child(x.at(x.object(object_scene(f.yaw, Paint::INK, persp()), 360.0, 330.0), colx(8.0) + 40.0, 220.0));
    let finale = col().key("film/c12/finale").width(x.u(UNITS)).height(x.u(STAGE_H)).align(AlignItems::CENTER).justify(taffy::JustifyContent::CENTER)
        .child(text(x.words(), x.u(150.0), Paint::INK).font(Font::Display).weight(3.0).key("film/c12/finale/words").nowrap());
    let draft = x.g.film.drafts.as_ref().filter(|d| d.0 == 12).map(|d| d.1.as_str());
    let mut out = vec![];
    if f.finale {
        out.push(x.at(finale, 0.0, 0.0));
        out.push(x.at(board.hidden_until_morphed(true), 0.0, 0.0));
    } else {
        out.push(x.at(board, 0.0, 0.0));
        out.push(x.at(finale.hidden_until_morphed(true), 0.0, 0.0));
    }
    out.push(x.at(row().key(x.key("controls")).gap(x.u(18.0)).align(AlignItems::CENTER)
        .child(button_as(x.key("close"), if f.finale { "再松开" } else { "合上" }, Variant::Accent, Msg::Film(FMsg::Finale)))
        .child(number(&x.key("count"), "粒子数", f.spec.particles, COUNT_SPEC, draft, |e| Msg::Film(FMsg::Count(e))).width(360.0)), colx(0.0), 560.0));
    out
}

// ── Update, the clock, keys ──────────────────────────────────────────────────────────────────────

/// Go to chapter `i` (wrapping): the target preset's transition from `at` (logical px; the stage's
/// centre when `None`), and the actors flow.
pub fn go(g: &mut Gallery, i: usize, at: Option<(f32, f32)>) {
    let i = i % CHAPTERS;
    let geo = geo(g.width, g.height);
    let (x, y) = at.unwrap_or((geo.sx + geo.sw * 0.5, geo.sy + geo.sh * 0.5));
    g.cmds.push(Cmd::Commit { x, y });
    g.cmds.push(Cmd::Flow(FlowSpec { flow_ms: g.film.spec.flow_ms, flow_loose: 0.2, out_ms: 160.0 }));
    let f = &mut g.film;
    f.ch = i;
    f.entered = g.clock;
    f.ghost = 0;
    f.hover = None;
    f.tear = None;
    f.error = false;
    f.finale = false;
    f.scrub_from = None;
    f.ident = None;
    f.drafts = None;
    g.apply_style(CH[i].style);
    // Keyboard first: the palette has the focus when it arrives.
    if i == 1 { g.cmds.push(Cmd::Focus { key: "film/c02/query".into() }); }
}

pub fn update(g: &mut Gallery, m: FMsg) {
    let now = g.clock;
    match m {
        FMsg::Go(i) => { let at = tick_at(g, i); go(g, i, at); }
        FMsg::Next => { let n = g.film.ch + 1; go(g, n, None); }
        FMsg::Prev => { let n = g.film.ch + CHAPTERS - 1; go(g, n, None); }
        FMsg::Play => {
            // Space pauses what was playing (even when this key's own input has just paused it) and plays
            // what was paused.
            let f = &mut g.film;
            let was = f.playing || f.yielded == Some(now);
            f.playing = !was;
            f.held = was;
            f.yielded = None;
            if f.playing { f.entered = now; f.ghost = 0; }
        }
        FMsg::Sentence(t) => g.film.sentence = t,
        FMsg::Motion => g.film.motion = !g.film.motion,
        FMsg::Num(Edit::Draft(d)) => g.film.drafts = Some((1, d)),
        FMsg::Num(Edit::Set(v)) => { g.film.n = N_SPEC.clamp(v); g.film.drafts = None; }
        FMsg::Num(Edit::Cancel) | FMsg::Contrast(Edit::Cancel) | FMsg::Count(Edit::Cancel) => g.film.drafts = None,
        FMsg::Query(t) => { g.film.query = t; g.film.sel = 0; }
        FMsg::Run(i) => run_command(g, i),
        FMsg::Orbit(o) => g.film.orbit.update(o, now),
        FMsg::Identify(k) => g.film.ident = Some(k),
        FMsg::Pick(Pick::Hover(h)) => g.film.obj_hover = h.is_some(),
        FMsg::Pick(Pick::Press(_)) => {}
        FMsg::Obj(x, y) => g.film.obj = (x, y),
        FMsg::Contrast(Edit::Draft(d)) => g.film.drafts = Some((6, d)),
        FMsg::Contrast(Edit::Set(v)) => { g.film.contrast = CONTRAST_SPEC.clamp(v); g.film.drafts = None; }
        FMsg::Form => next_form(g, false),
        FMsg::Scrub(d) => scrub(g, d),
        FMsg::Tear(d) => {
            let y = d.fy();
            match d.phase {
                Phase::Start | Phase::Move => g.film.tear = Some(y),
                Phase::End => {
                    g.film.tear = None;
                    let dx = ((y * 997.0).sin() * 0.5 + 0.5) * 60.0 - 30.0;
                    g.film.locks.push((y, if dx.abs() < 12.0 { dx.signum() * 18.0 } else { dx }));
                    if g.film.locks.len() > 3 { g.film.locks.remove(0); }
                    if g.film.locks.len() == 3 { g.film.error = true; }
                }
            }
        }
        FMsg::Error(on) => g.film.error = on,
        FMsg::Retry => { g.film.locks.clear(); g.film.error = false; }
        FMsg::Explode => { g.cmds.push(Cmd::Flow(FlowSpec { flow_ms: 520.0, flow_loose: 0.15, out_ms: 0.0 })); g.film.exploded = !g.film.exploded; g.cmds.push(Cmd::FrontAt { key: "film/c09/object".into(), strength: 0.8 }); }
        FMsg::Size(v) => g.film.size = v.clamp(0.0, 1.0),
        FMsg::Squares => g.film.squares = !g.film.squares,
        FMsg::Plane(i) => { g.cmds.push(Cmd::Flow(FlowSpec { flow_ms: 420.0, flow_loose: 0.3, out_ms: 120.0 })); g.film.plane = i.min(2); }
        FMsg::MenuSel(i) => { g.cmds.push(Cmd::Flow(FlowSpec { flow_ms: 380.0, flow_loose: 0.3, out_ms: 0.0 })); g.film.menu_sel = i.min(MENU.len() - 1); }
        FMsg::Finale => {
            let f = &g.film;
            let spec = MorphSpec { duration_ms: 1800.0, loosen_ms: 420.0, tighten_ms: 520.0, count_max: f.spec.particles, curl: 90.0, stagger: 0.55, ease: Ease::InOutCubic, lattice: 2.0, ..MorphSpec::default() };
            let (from, to) = if f.finale { ("film/c12/finale", "film/c12/board") } else { ("film/c12/board", "film/c12/finale") };
            g.cmds.push(Cmd::Morph { from: from.into(), to: to.into(), spec });
            g.film.finale = !g.film.finale;
        }
        FMsg::Count(Edit::Draft(d)) => g.film.drafts = Some((12, d)),
        FMsg::Count(Edit::Set(v)) => { g.film.spec.particles = COUNT_SPEC.clamp(v); g.film.drafts = None; }
    }
}

/// Where a tick of the timeline is (its centre), for the transition to start there.
fn tick_at(g: &Gallery, i: usize) -> Option<(f32, f32)> {
    let geo = geo(g.width, g.height);
    Some((geo.sx + i as f32 / (CHAPTERS - 1) as f32 * geo.sw, geo.sy + geo.sh + 28.0))
}

fn run_command(g: &mut Gallery, i: usize) {
    if i < CHAPTERS { go(g, i, None); return; }
    match i - CHAPTERS { 0 => g.film.motion = !g.film.motion, _ => g.film.n = N_SPEC.clamp(g.film.n + 1.0) }
}

/// The poster's next form: a morph from what it shows to the next one.
fn next_form(g: &mut Gallery, scrubbed: bool) {
    let f = &mut g.film;
    let (a, b) = (f.form % FORMS, (f.form + 1) % FORMS);
    let spec = MorphSpec { duration_ms: 1300.0, loosen_ms: 260.0, tighten_ms: 420.0, curl: 70.0, stagger: 0.5, lattice: 4.0, count_max: 16000.0, ..MorphSpec::default() };
    g.cmds.push(Cmd::Morph { from: form_key(a), to: form_key(b), spec });
    if scrubbed { g.cmds.push(Cmd::Scrub(Some(0.0))); }
    f.scrub_from = Some(a);
    f.form = b;
}

/// Dragging across the poster: the drag's start starts the morph held at 0, moving holds it where the
/// drag is, letting go runs it on (past half way) or back.
fn scrub(g: &mut Gallery, d: Drag) {
    let u = (d.dx / d.w.max(1.0) * 1.6).clamp(0.0, 1.0);
    match d.phase {
        Phase::Start => next_form(g, true),
        Phase::Move => g.cmds.push(Cmd::Scrub(Some(u))),
        Phase::End => {
            g.cmds.push(Cmd::Scrub(None));
            // A press without a drag is a press: the morph runs on. A short drag goes back.
            if d.dx.abs() > 6.0 && u < 0.5 {
                let f = &mut g.film;
                let (a, b) = (f.form, f.scrub_from.unwrap_or((f.form + FORMS - 1) % FORMS));
                g.cmds.push(Cmd::Morph { from: form_key(a), to: form_key(b), spec: MorphSpec { duration_ms: 700.0, ..MorphSpec::default() } });
                f.form = b;
                f.scrub_from = Some(a);
            }
        }
    }
}

/// The ghost's script for chapter `c`: (seconds after entering, what it does).
fn script(c: usize) -> Vec<(f32, Ghost)> {
    use Ghost::*;
    match c {
        0 => vec![(1.4, Focus("film/c01/title")), (2.6, Press("film/c01/motion")), (4.2, Press("film/c01/count/more")), (4.8, Press("film/c01/count/more")), (6.8, Press("film/c01/motion"))],
        1 => vec![(1.0, Type("g")), (1.4, Type("o")), (1.8, Type("0")), (2.4, Type("5")), (3.4, Msg(FMsg::Query(TextEdit::new("go")))), (4.2, Type(" n")), (5.0, Msg(FMsg::Query(TextEdit::default())))],
        2 => vec![(1.0, Hover(Some("film/c03/part0"))), (2.2, Hover(Some("film/c03/part2"))), (3.4, Hover(Some("film/c03/part3"))), (4.6, Hover(None))],
        3 => vec![(0.8, Focus("film/c04/sentence")), (2.0, Focus("film/c04/object")), (3.2, Press("film/c04/object")), (4.6, Focus("film/c04/picture")), (5.8, Focus("film/c04/number"))],
        5 => vec![(1.2, Msg(FMsg::Obj(90.0, 120.0))), (3.0, Msg(FMsg::Obj(380.0, 60.0))), (5.0, Msg(FMsg::Contrast(Edit::Set(160.0)))), (6.6, Msg(FMsg::Contrast(Edit::Set(100.0)))), (7.2, Msg(FMsg::Obj(340.0, 20.0)))],
        6 => vec![(1.0, Msg(FMsg::Form)), (3.6, Msg(FMsg::Form)), (6.2, Msg(FMsg::Form))],
        7 => vec![(1.2, Msg(FMsg::Tear(drag(Phase::Start, 0.3)))), (1.8, Msg(FMsg::Tear(drag(Phase::End, 0.3)))), (2.8, Msg(FMsg::Tear(drag(Phase::Start, 0.62)))), (3.4, Msg(FMsg::Tear(drag(Phase::End, 0.62)))),
                 (4.4, Msg(FMsg::Tear(drag(Phase::Start, 0.45)))), (5.0, Msg(FMsg::Tear(drag(Phase::End, 0.45)))), (6.8, Press("film/c08/retry"))],
        8 => vec![(0.5, Ptr(0.3, 0.4)), (1.5, Ptr(0.7, 0.6)), (2.4, Msg(FMsg::Explode)), (4.0, Ptr(0.35, 0.3)), (5.2, Ptr(0.65, 0.7)), (6.4, Msg(FMsg::Explode)), (7.4, Ptr(0.5, 0.5))],
        9 => vec![(1.0, Msg(FMsg::Size(0.9))), (2.4, Press("film/c10/toggle")), (3.6, Press("film/c10/plane1")), (5.0, Press("film/c10/plane2")), (6.4, Msg(FMsg::Size(0.4))), (7.4, Press("film/c10/plane0"))],
        10 => vec![(0.8, Hover(Some("film/c11/item2"))), (1.6, Ptr(0.2, 0.8)), (2.4, Hover(Some("film/c11/item3"))), (3.2, Press("film/c11/item3")), (4.4, Ptr(0.8, 0.8)), (5.0, Hover(Some("film/c11/item0"))), (6.0, Press("film/c11/item1")), (6.6, Hover(None))],
        11 => vec![(2.2, Msg(FMsg::Finale))],
        _ => vec![],
    }
}
fn drag(phase: Phase, fy: f32) -> Drag { Drag { phase, x: 100.0, y: fy * 300.0, w: 1000.0, h: 300.0, dx: 0.0, dy: 0.0 } }

/// What the ghost does now.
fn act(g: &mut Gallery, a: Ghost) {
    match a {
        Ghost::Msg(m) => update(g, m),
        Ghost::Press(k) => g.cmds.push(Cmd::Press { key: k.into() }),
        Ghost::Focus(k) => g.cmds.push(Cmd::Focus { key: k.into() }),
        Ghost::Type(s) => { let mut t = g.film.query.clone(); t.text.push_str(s); t.caret = t.text.chars().count(); t.anchor = t.caret; g.film.query = t; g.film.sel = 0; }
        Ghost::Hover(k) => g.film.hover = k.map(str::to_owned),
        Ghost::Ptr(x, y) => g.film.ptr = (x, y),
    }
}

/// Whether the object turns by itself now: while the ghost plays the chapter, or (Normcore) while its
/// switch is on, or (dot matrix) while the pointer is on it.
fn spinning(f: &Film) -> bool {
    match f.ch {
        0 => f.motion,
        2 => false,
        4 => f.playing || f.obj_hover,
        _ => f.playing || (f.ch == 11 && f.motion),
    }
}

/// The film's clock (`Ui::on_tick`): autoplay, the ghost, the board's rows, the turning object.
pub fn tick(g: &mut Gallery, t: f32) -> bool {
    if g.page != Page::Film { return false; }
    let dt = (t - g.film.last_tick).clamp(0.0, 0.1);
    g.film.last_tick = t;
    let mut changed = false;
    let f = &mut g.film;
    if !f.playing && !f.held && t - f.last_input >= f.spec.idle_ms / 1000.0 { f.playing = true; f.entered = t; f.ghost = 0; changed = true; }
    if g.film.playing {
        let el = t - g.film.entered;
        let steps = script(g.film.ch);
        while let Some((at, a)) = steps.get(g.film.ghost).cloned() && el >= at {
            g.film.ghost += 1;
            act(g, a);
            changed = true;
        }
        if el >= g.film.spec.dwell_ms / 1000.0 { let n = g.film.ch + 1; go(g, n, None); return true; }
    }
    let f = &mut g.film;
    if spinning(f) { f.yaw = (f.yaw + f.spec.spin * dt) % 360.0; changed = true; }
    if f.ch == 2 && f.orbit.step(t) { changed = true; }
    // The board: once a second the number moves on, and the rows follow one after another.
    if f.ch == 4 && f.playing {
        if t - f.board_at >= 1.0 { f.board_at = t; f.n = (f.n + 1.0) % 100.0; changed = true; }
        for i in 0..4 { if f.rows[i] != f.n && t - f.board_at >= 0.12 * (i + 1) as f32 { f.rows[i] = f.n; changed = true; } }
    }
    changed
}

/// When the film wants its next tick (`Ui::on_wake`).
pub fn wake(g: &Gallery, t: f32) -> Option<f32> {
    if g.page != Page::Film { return None; }
    let f = &g.film;
    if spinning(f) || (f.ch == 2 && f.orbit.moving()) { return Some(t); }
    if f.playing {
        let mut w = f.entered + f.spec.dwell_ms / 1000.0;
        if let Some((at, _)) = script(f.ch).get(f.ghost) { w = w.min(f.entered + at); }
        if f.ch == 4 { w = w.min(f.board_at + 1.0).min(f.board_at + 0.5); }
        return Some(w.max(t));
    }
    if !f.held { return Some(f.last_input + f.spec.idle_ms / 1000.0); }
    None
}

/// A key nothing on the page took: chapters, play, the gallery; the palette's selection.
pub fn key(g: &mut Gallery, k: Key, _shift: bool) -> bool {
    if g.page != Page::Film { return false; }
    let ch = g.film.ch;
    if ch == 1 {
        let n = matches(&g.film.query.text).len();
        match k {
            Key::Up => { g.film.sel = g.film.sel.saturating_sub(1); return true; }
            Key::Down => { g.film.sel = (g.film.sel + 1).min(n.saturating_sub(1)); return true; }
            Key::Enter => { if let Some(&i) = matches(&g.film.query.text).get(g.film.sel) { run_command(g, i); } return true; }
            _ => {}
        }
    }
    match k {
        Key::Right => update(g, FMsg::Next),
        Key::Left => update(g, FMsg::Prev),
        Key::Char(' ') => update(g, FMsg::Play),
        Key::Char(c @ '1'..='9') => go(g, c as usize - '1' as usize, None),
        Key::Char('0') => go(g, 9, None),
        Key::Char('-') => go(g, 10, None),
        Key::Char('=') => go(g, 11, None),
        Key::Escape => { super::update(g, Msg::Go(Page::Comp(0))); }
        _ => return false,
    }
    true
}

/// A person's input (a press, the wheel, a key): autoplay yields at once.
pub fn heard(g: &mut Gallery) -> bool {
    if g.page != Page::Film { return false; }
    let f = &mut g.film;
    f.last_input = g.clock;
    let was = f.playing;
    if was { f.yielded = Some(g.clock); }
    f.playing = false;
    was
}

/// The pointer over the window: where it is in the stage, and it counts as input.
pub fn pointer(g: &mut Gallery, x: f32, y: f32) -> bool {
    if g.page != Page::Film { return false; }
    let geo = geo(g.width, g.height);
    let p = (((x - geo.sx) / geo.sw).clamp(0.0, 1.0), ((y - geo.sy) / geo.sh).clamp(0.0, 1.0));
    let was = heard(g);
    let follows = matches!(g.film.ch, 8 | 10);
    let moved = follows && p != g.film.ptr;
    g.film.ptr = p;
    was || moved
}

/// What is under the pointer (ticks, parts, menu items).
pub fn hovered(g: &mut Gallery, k: Option<&str>) -> bool {
    if g.page != Page::Film { return false; }
    let k = k.map(str::to_owned);
    if k == g.film.hover { return false; }
    g.film.hover = k;
    true
}

/// The film's own changes to the look: the annotation each chapter wants, the poster's picture contrast.
pub fn look(g: &Gallery, l: &mut Look) {
    if g.page != Page::Film { return; }
    let f = &g.film;
    match f.ch {
        // The detector boxes what is focused while the ghost plays, what the pointer is near otherwise.
        3 => l.gaze = Gaze { mode: if f.playing { GazeMode::Focus } else if f.hover.is_some() { GazeMode::Hover } else { GazeMode::All }, kinds: gaze::FRAME | gaze::KIND | gaze::SIZE | gaze::CROSS, density: 6.0, ..l.gaze },
        5 => l.pics[STYLE_SLOT as usize].gamma = (100.0 / f.contrast.max(1.0)).clamp(0.3, 3.0),
        _ => {}
    }
}
