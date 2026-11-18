//! The demo application, written with the framework only: a counter, a picture, proportional mixed
//! text, a scrolling list, pictures turning into marks with a compare wipe, pixel type, and a panel
//! that sets every field parameter exactly and picks the theme and material.
use amenui::el::{El, Font, Paint, col, picture, row, space, text, text_lh};
use amenui::scene::Fx;
use amenui::style::{Look, MATERIALS, Material, Markr, THEMES, Theme, colour, set};
use amenui::text::{Face, MONO, PIXEL, SANS};
use amenui::widgets::{Edit, Spec, button, choice, number};
use taffy::AlignItems;

pub struct Demo {
    pub count: i32,
    pub fx: Fx,
    /// Slot 0: the picture at the top (loosened by the pointer and fronts). Slot 1: the marks section.
    pub pics: [Markr; 8],
    pub theme: usize,
    pub material: Material,
    /// The parameter being typed into and its text so far.
    pub edit: Option<(usize, String)>,
    pub picked: Option<usize>,
    /// Which picture the marks section shows, and where its compare line is (fraction of its width).
    pub shown: u32,
    pub wipe: f32,
}
impl Default for Demo {
    fn default() -> Self {
        let mut pics = [Markr::default(); 8];
        pics[1] = Markr { pitch: 16.0, levels: 16.0, set: set::CHARS as f32, ground: 0.0, gamma: 0.6, ..Markr::default() };
        Self { count: 0, fx: Fx::default(), pics, theme: 0, material: Material::Dottr, edit: None, picked: None, shown: TREE, wipe: 0.42 }
    }
}
impl Demo {
    pub fn theme(&self) -> Theme { THEMES[self.theme.min(THEMES.len() - 1)] }
    pub fn look(&self) -> Look { Look { fx: self.fx, theme: self.theme(), material: self.material, pics: self.pics, ..Look::default() } }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Msg {
    Inc, Param(usize, Edit), Square, Theme(usize), Material(usize), Pick(usize), Reset,
    Show(u32), Set(u8), Colour(u8), Palette(usize), Wipe(f32),
}

pub struct Param { pub name: &'static str, pub spec: Spec }
const fn p(name: &'static str, min: f32, max: f32, step: f32, decimals: usize, unit: &'static str) -> Param {
    Param { name, spec: Spec { min, max, step, decimals, unit } }
}
/// Every number in `Fx`, in the order of `slot`.
pub const PARAMS: [Param; 10] = [
    p("字的点距", 2.0, 16.0, 0.5, 1, "px"),
    p("不规则度", 0.0, 1.0, 0.05, 2, ""),
    p("错位", 0.0, 2.0, 0.05, 2, ""),
    p("波速", 100.0, 3000.0, 10.0, 0, "px/s"),
    p("波的寿命", 0.2, 6.0, 0.1, 1, "s"),
    p("波宽", 4.0, 120.0, 1.0, 0, "px"),
    p("尾长", 0.0, 600.0, 10.0, 0, "px"),
    p("推力", 0.0, 30.0, 0.5, 1, "px"),
    p("透镜半径", 0.0, 300.0, 2.0, 0, "px"),
    p("凝结时长", 0.0, 3.0, 0.05, 2, "s"),
];
pub fn slot(fx: &mut Fx, i: usize) -> &mut f32 {
    match i {
        0 => &mut fx.pitch, 1 => &mut fx.jitter, 2 => &mut fx.glitch,
        3 => &mut fx.wave_speed, 4 => &mut fx.wave_life, 5 => &mut fx.wave_width, 6 => &mut fx.wave_tail,
        7 => &mut fx.push, 8 => &mut fx.lens, _ => &mut fx.enter,
    }
}
fn value(fx: &Fx, i: usize) -> f32 { *slot(&mut fx.clone(), i) }

/// Every number in `Markr`, in the order of `pic_slot`. Ids in messages are `PIC + index`.
pub const PIC: usize = 100;
pub const PIC_PARAMS: [Param; 26] = [
    p("点距", 1.0, 64.0, 1.0, 0, "px"),
    p("亮度级数", 2.0, 16.0, 1.0, 0, ""),
    p("记号表", 0.0, 11.0, 1.0, 0, ""),
    p("上色方式", 0.0, 3.0, 1.0, 0, ""),
    p("底的亮度", 0.0, 1.0, 0.05, 2, ""),
    p("曲线", 0.3, 3.0, 0.05, 2, ""),
    p("过渡宽度", 0.0, 1.0, 0.05, 2, ""),
    p("朝向跟随", 0.0, 1.0, 0.05, 2, ""),
    p("纵深", 0.0, 24.0, 0.5, 1, "px"),
    p("辉光", 0.0, 2.0, 0.05, 2, ""),
    p("调色板", 0.0, 5.0, 1.0, 0, ""),
    p("颜色数", 0.0, 16.0, 1.0, 0, ""),
    p("浮雕", 0.0, 3.0, 0.05, 2, ""),
    p("分区大小", 0.0, 32.0, 1.0, 0, "格"),
    p("分区依据", 0.0, 1.0, 1.0, 0, ""),
    p("分区表数", 1.0, 4.0, 1.0, 0, ""),
    p("分区表 1", 0.0, 11.0, 1.0, 0, ""),
    p("分区表 2", 0.0, 11.0, 1.0, 0, ""),
    p("分区表 3", 0.0, 11.0, 1.0, 0, ""),
    p("分区表 4", 0.0, 11.0, 1.0, 0, ""),
    p("色 A 红", 0.0, 255.0, 1.0, 0, ""),
    p("色 A 绿", 0.0, 255.0, 1.0, 0, ""),
    p("色 A 蓝", 0.0, 255.0, 1.0, 0, ""),
    p("色 B 红", 0.0, 255.0, 1.0, 0, ""),
    p("色 B 绿", 0.0, 255.0, 1.0, 0, ""),
    p("色 B 蓝", 0.0, 255.0, 1.0, 0, ""),
];
/// Tints are kept 0..1 and shown 0..255.
pub fn pic_slot(f: &mut Markr, i: usize) -> (&mut f32, f32) {
    match i {
        0 => (&mut f.pitch, 1.0), 1 => (&mut f.levels, 1.0), 2 => (&mut f.set, 1.0), 3 => (&mut f.colour, 1.0),
        4 => (&mut f.ground, 1.0), 5 => (&mut f.gamma, 1.0), 6 => (&mut f.spread, 1.0), 7 => (&mut f.follow, 1.0),
        8 => (&mut f.depth, 1.0), 9 => (&mut f.glow, 1.0), 10 => (&mut f.palette, 1.0), 11 => (&mut f.colours, 1.0),
        12 => (&mut f.relief, 1.0), 13 => (&mut f.regions, 1.0), 14 => (&mut f.region_by, 1.0), 15 => (&mut f.region_count, 1.0),
        16..=19 => (&mut f.region_sets[i - 16], 1.0),
        20..=22 => (&mut f.tint_a[i - 20], 255.0),
        _ => (&mut f.tint_b[(i - 23).min(2)], 255.0),
    }
}
fn pic_value(f: &Markr, i: usize) -> f32 { let mut c = *f; let (v, k) = pic_slot(&mut c, i); *v * k }

/// The compare line's position.
pub const WIPE: usize = 200;
const WIPE_SPEC: Spec = Spec { min: 0.0, max: 1.0, step: 0.01, decimals: 2, unit: "" };

pub fn update(s: &mut Demo, m: Msg) {
    match m {
        Msg::Inc => s.count += 1,
        Msg::Param(i, Edit::Draft(d)) => s.edit = Some((i, d)),
        Msg::Param(i, Edit::Set(v)) => {
            if i == WIPE { s.wipe = v; }
            else if i >= PIC { let (f, k) = pic_slot(&mut s.pics[1], i - PIC); *f = v / k; }
            else { *slot(&mut s.fx, i) = v; }
            s.edit = None;
        }
        Msg::Param(_, Edit::Cancel) => s.edit = None,
        Msg::Square => s.fx.square = !s.fx.square,
        Msg::Theme(i) => s.theme = i,
        Msg::Material(i) => s.material = MATERIALS[i.min(MATERIALS.len() - 1)].0,
        Msg::Pick(i) => s.picked = if s.picked == Some(i) { None } else { Some(i) },
        Msg::Reset => { s.fx = Fx { square: s.fx.square, ..Fx::default() }; s.pics = Demo::default().pics; s.edit = None; }
        Msg::Show(id) => s.shown = id,
        Msg::Set(k) => s.pics[1].set = k as f32,
        Msg::Colour(k) => s.pics[1].colour = k as f32,
        Msg::Palette(i) => {
            // 0 is "no palette"; otherwise use the palette with its own number of colours.
            if i == 0 { s.pics[1].colours = 0.0; }
            else { s.pics[1].palette = (i - 1) as f32; s.pics[1].colours = amenui::marks::PALETTES[i - 1].1 as f32; }
        }
        Msg::Wipe(x) => s.wipe = WIPE_SPEC.clamp(x),
    }
}

const PARAGRAPHS: [(&str, Paint); 3] = [
    ("在空白处按一下，会有一圈波从按下的位置传出去。波经过的字和图会松开成点阵，然后合回原样。新出现的字也会从松开的状态凝结出来。", Paint::TEXT),
    ("指针移到图上，图会在指针附近松开；指针经过文字时，文字不变。Tab 键在控件之间移动焦点，Enter 或空格键触发。", Paint::TEXT),
    ("Layout is computed by taffy and text is set by parley, in proportional type at any size. Every glyph, shape and picture is drawn by its own outline, never by a box around it.", Paint::DIM),
];

pub fn view(s: &Demo) -> El<Msg> {
    col().key("page").fonts(s.theme().roles()).scroll().fill_width().fill_height().pad(24.0, 28.0)
        .child(row().key("columns").wrap().gap(28.0).align(AlignItems::START).child(content(s)).child(panel(s)))
}

fn content(s: &Demo) -> El<Msg> {
    let th = s.theme();
    let sz = th.sizes;
    let card = row().key("card").gap(16.0).pad(18.0, 14.0).radius(6.0).border(th.hairline, Paint::FAINT).align(AlignItems::CENTER)
        .child(text(format!("计数 {}", s.count), sz.title + 2.0, Paint::INK).key("count").font(Font::Display).grow(1.0))
        .child(button("inc", "+1", Msg::Inc));
    let mut list = col().key("list").scroll().height(232.0).radius(6.0).border(th.hairline, Paint::FAINT).pad(0.0, 4.0);
    for i in 0..40 {
        let on = s.picked == Some(i);
        let mut item = row().key(format!("item/{i}")).height(36.0).pad(14.0, 0.0).gap(12.0).align(AlignItems::CENTER).on_press(Msg::Pick(i))
            .child(text(format!("第 {} 项", i + 1), 15.0, if on { Paint::ACCENT } else { Paint::TEXT }).grow(1.0))
            .child(text(format!("Item {:02}", i + 1), 13.0, Paint::DIM).font(Font::Mono));
        if on { item = item.fill(Paint { tone: 0.0, accent: 1.0, alpha: 0.12, hue: 0 }); }
        list = list.child(item);
    }
    let body_line = (sz.body * 1.75).round();
    col().key("content").grow(1.0).shrink().basis(460.0).min_width(0.0).max_width(760.0).gap(18.0)
        .child(text_lh("界面骨架", sz.display, (sz.display * 1.3).round(), Paint::INK).font(Font::Display))
        .child(text_lh("这一页的每个字、每条线和每张图都由框架自己画出：布局、文字排版、点击、焦点和滚动都在框架里。", sz.small + 2.0, (sz.small * 1.8).round(), Paint::DIM))
        .child(card)
        .child(picture(SPHERE, 0.5).key("picture"))
        .children(PARAGRAPHS.iter().map(|(t, p)| text_lh(*t, sz.body, body_line, *p)))
        .child(text("列表", sz.small + 1.0, Paint::DIM))
        .child(list)
        .child(marks_section(s))
        .child(type_section(s))
        .child(space(1.0, 8.0))
}

/// A picture turning into marks, with the compare line and every `Markr` number of slot 1.
fn marks_section(s: &Demo) -> El<Msg> {
    let th = s.theme();
    let f = &s.pics[1];
    let draft = |i: usize| s.edit.as_ref().filter(|(j, _)| *j == i).map(|(_, d)| d.as_str());
    let palette_names: Vec<&str> = std::iter::once("不限色").chain(amenui::marks::PALETTES.iter().map(|p| p.0)).collect();
    let palette_now = if f.colours < 1.0 { 0 } else { f.palette as usize + 1 };
    let mut numbers = row().key("pic-params").wrap().gap(10.0);
    for (i, prm) in PIC_PARAMS.iter().enumerate() {
        numbers = numbers.child(number(&format!("pic/{i}"), prm.name, pic_value(f, i), prm.spec, draft(PIC + i), move |e| Msg::Param(PIC + i, e)).width(330.0));
    }
    col().key("marks").gap(12.0)
        .child(space(1.0, 10.0))
        .child(text_lh("图片的记号", th.sizes.title, (th.sizes.title * 1.5).round(), Paint::INK).font(Font::Display))
        .child(text_lh("线的右侧是图片完全松开后的样子：先变成无缝的色块，再在每一格里刻记号。拖动图片可以移动这条线。", th.sizes.small + 1.0, (th.sizes.small * 1.8).round(), Paint::DIM))
        .child(choice("show", "图片", &PICTURE_NAMES, s.shown as usize, |i| Msg::Show(i as u32)))
        .child(picture(s.shown, PICTURE_SIZES[s.shown as usize].1 as f32 / PICTURE_SIZES[s.shown as usize].0 as f32)
            .key("marks-picture").markr(1).wipr(s.wipe).on_drag(|x, _| Msg::Wipe(x)))
        .child(number("wipe", "对比线位置", s.wipe, WIPE_SPEC, draft(WIPE), |e| Msg::Param(WIPE, e)).width(330.0))
        .child(choice("set", "记号表", &set::NAMES, f.set as usize, |i| Msg::Set(i as u8)))
        .child(choice("colour", "上色", &colour::NAMES, f.colour as usize, |i| Msg::Colour(i as u8)))
        .child(choice("palette", "限色", &palette_names, palette_now, Msg::Palette))
        .child(numbers)
}

const PIXEL_ZH: &str = "像素字体按字面的整数倍显示，笔画落在整像素上，边缘只有墨色和底色。";
const PIXEL_EN: &str = "Pixel faces are drawn at whole multiples of their size, so every edge is ink or ground.";

/// The same words in the pixel faces and in the proportional face, and a heading printed through a
/// coarse lattice by the material.
fn type_section(s: &Demo) -> El<Msg> {
    let th = s.theme();
    let column = |key: &str, title: &str, zh: Font, en: Font, zs: f32, es: f32| col().key(key).grow(1.0).basis(280.0).gap(10.0)
        .child(text(title, 12.0, Paint::DIM).font(Font::Face(MONO)))
        .child(text_lh(PIXEL_ZH, zs, (zs * 1.75).round(), Paint::TEXT).font(zh))
        .child(text_lh(PIXEL_EN, es, (es * 1.6).round(), Paint::DIM).font(en));
    col().key("type").gap(14.0)
        .child(space(1.0, 10.0))
        .child(text_lh("像素字 Pixel Type", 36.0, 48.0, Paint::INK).font(Font::Face(PIXEL)).key("pixel-heading"))
        .child(row().key("type-compare").wrap().gap(24.0).align(AlignItems::START)
            .child(column("type-pixel", "FUSION PIXEL 12 / DEPARTURE MONO 11", Font::Face(PIXEL), Font::Face(MONO), 12.0, 11.0))
            .child(column("type-sans", "NOTO SANS SC", Font::Face(SANS), Font::Face(SANS), 13.0, 12.0)))
        .child(text_lh("2× 像素字：24 / 22", 24.0, 34.0, Paint::TEXT).font(Font::Face(PIXEL)))
        .child(text_lh("SIGNAL 0042 / 22PX", 22.0, 30.0, Paint::ACCENT).font(Font::Face(MONO)))
        .child(text("印成像素的标题（点距 3）", 12.0, Paint::DIM).font(Font::Face(PIXEL)))
        .child(text_lh("任何字体都能印成像素", (th.sizes.display * 1.2).round(), (th.sizes.display * 1.6).round(), Paint::INK).font(Font::Face(SANS)).print(3.0).key("printed"))
}

fn panel(s: &Demo) -> El<Msg> {
    let theme_names: Vec<&str> = THEMES.iter().map(|t| t.name).collect();
    let material_names: Vec<&str> = MATERIALS.iter().map(|m| m.1).collect();
    let material_now = MATERIALS.iter().position(|m| m.0 == s.material).unwrap_or(0);
    let mut panel = col().key("panel").grow(1.0).basis(330.0).max_width(380.0).gap(10.0).pad(18.0, 16.0).radius(6.0).border(s.theme().hairline, Paint::FAINT)
        .child(text("参数", 16.0, Paint::INK).font(Font::Display));
    for (i, prm) in PARAMS.iter().enumerate() {
        let draft = s.edit.as_ref().filter(|(j, _)| *j == i).map(|(_, d)| d.as_str());
        panel = panel.child(number(&format!("param/{i}"), prm.name, value(&s.fx, i), prm.spec, draft, move |e| Msg::Param(i, e)));
    }
    panel
        .child(space(1.0, 2.0))
        .child(amenui::widgets::switch("square", "点的形状", s.fx.square, Some(("圆点", "方块")), Msg::Square))
        .child(choice("theme", "主题", &theme_names, s.theme, Msg::Theme))
        .child(choice("material", "材质", &material_names, material_now, Msg::Material))
        .child(row().child(button("reset", "恢复默认", Msg::Reset)))
}

// The pictures it shows are the framework's sample pictures.
pub use amenui::pictures::*;

pub type DemoUi = amenui::ui::Ui<Demo, Msg>;
pub fn new_ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> DemoUi { amenui::ui::Ui::new(Demo::default(), view, update, faces, w, h, scale) }
