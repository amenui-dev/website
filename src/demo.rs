//! The demo application, written with the framework only: a counter, a picture, proportional mixed
//! text, a scrolling list, pictures turning into marks with a compare wipe, pixel type, and a panel
//! that sets every field parameter exactly and picks the theme and material.
use crate::el::{El, Font, Paint, col, picture, row, space, text, text_lh};
use crate::scene::Fx;
use crate::style::{Look, MATERIALS, Material, PicFx, THEMES, Theme, colour, set};
use crate::text::{Face, MONO, PIXEL, SANS};
use crate::widgets::{Edit, Spec, button, choice, number};
use taffy::AlignItems;

pub struct Demo {
    pub count: i32,
    pub fx: Fx,
    /// Slot 0: the picture at the top (loosened by the pointer and fronts). Slot 1: the marks section.
    pub pics: [PicFx; 8],
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
        let mut pics = [PicFx::default(); 8];
        pics[1] = PicFx { pitch: 16.0, levels: 16.0, set: set::CHARS as f32, ground: 0.0, gamma: 0.6, ..PicFx::default() };
        Self { count: 0, fx: Fx::default(), pics, theme: 0, material: Material::Dot, edit: None, picked: None, shown: BUST, wipe: 0.42 }
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

/// Every number in `PicFx`, in the order of `pic_slot`. Ids in messages are `PIC + index`.
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
pub fn pic_slot(f: &mut PicFx, i: usize) -> (&mut f32, f32) {
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
fn pic_value(f: &PicFx, i: usize) -> f32 { let mut c = *f; let (v, k) = pic_slot(&mut c, i); *v * k }

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
            else { s.pics[1].palette = (i - 1) as f32; s.pics[1].colours = crate::marks::PALETTES[i - 1].1 as f32; }
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
    col().key("content").grow(1.0).basis(460.0).max_width(760.0).gap(18.0)
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

/// A picture turning into marks, with the compare line and every `PicFx` number of slot 1.
fn marks_section(s: &Demo) -> El<Msg> {
    let th = s.theme();
    let f = &s.pics[1];
    let draft = |i: usize| s.edit.as_ref().filter(|(j, _)| *j == i).map(|(_, d)| d.as_str());
    let palette_names: Vec<&str> = std::iter::once("不限色").chain(crate::marks::PALETTES.iter().map(|p| p.0)).collect();
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
            .key("marks-picture").pic_fx(1).wipe(s.wipe).on_drag(|x, _| Msg::Wipe(x)))
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
        .child(crate::widgets::switch("square", "点的形状", s.fx.square, Some(("圆点", "方块")), Msg::Square))
        .child(choice("theme", "主题", &theme_names, s.theme, Msg::Theme))
        .child(choice("material", "材质", &material_names, material_now, Msg::Material))
        .child(row().child(button("reset", "恢复默认", Msg::Reset)))
}

// ── Pictures, computed in code ────────────────────────────────────────────────────────────────────
pub const SPHERE: u32 = 0;
pub const SKY: u32 = 1;
pub const TERRAIN: u32 = 2;
pub const BUST: u32 = 3;
/// The bust cut out of its ground: transparent around its silhouette, so it has a true outline.
pub const CUTOUT: u32 = 4;
pub const PICTURE_NAMES: [&str; 4] = ["球", "天空", "地形", "胸像"];
pub const PICTURE_SIZES: [(u32, u32); 5] = [(640, 320), (640, 400), (640, 400), (640, 400), (520, 400)];

/// Every demo picture: id, width, height, RGBA8.
pub fn pictures() -> Vec<(u32, u32, u32, Vec<u8>)> {
    let f: [fn(u32, u32) -> Vec<u8>; 5] = [picture_pixels, sky_pixels, terrain_pixels, bust_pixels, cutout_pixels];
    f.iter().enumerate().map(|(i, f)| { let (w, h) = PICTURE_SIZES[i]; (i as u32, w, h, f(w, h)) }).collect()
}

fn rgba(w: u32, h: u32, f: impl Fn(f32, f32) -> [f32; 3]) -> Vec<u8> {
    let mut px = vec![0u8; (w * h * 4) as usize];
    for y in 0..h { for x in 0..w {
        let c = f(x as f32 + 0.5, y as f32 + 0.5);
        let o = ((y * w + x) * 4) as usize;
        for k in 0..3 { px[o + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8; }
        px[o + 3] = 255;
    } }
    px
}

/// A lit sphere over a horizon. RGBA8, `w` × `h`.
pub fn picture_pixels(w: u32, h: u32) -> Vec<u8> {
    let (cx, cy, r) = (w as f32 * 0.6, h as f32 * 0.46, h as f32 * 0.33);
    let light = norm([-0.5, -0.62, 0.6]);
    rgba(w, h, |fx, fy| {
        let v = fy / h as f32;
        let horizon = 0.64;
        let mut c = if v < horizon { let k = v / horizon; [0.05 + 0.13 * k, 0.07 + 0.17 * k, 0.09 + 0.2 * k] }
                    else { let k = (v - horizon) / (1.0 - horizon); let glow = (-((fx - cx) / (w as f32 * 0.3)).powi(2)).exp() * (1.0 - k) * 0.4; [0.08 + glow * 0.8, 0.1 + glow * 0.9, 0.12 + glow * 0.9] };
        let (dx, dy) = ((fx - cx) / r, (fy - cy) / r);
        let d2 = dx * dx + dy * dy;
        if d2 < 1.0 {
            let dz = (1.0 - d2).sqrt();
            let lam = (dx * light[0] + dy * light[1] + dz * light[2]).max(0.0);
            let spec = lam.powf(24.0) * 0.5;
            let k = 0.07 + lam * 0.85;
            c = [(0.62 * k + spec).min(1.0), (0.76 * k + spec).min(1.0), (0.78 * k + spec).min(1.0)];
        }
        if (fy - h as f32 * 0.2).abs() < 1.5 && fx > w as f32 * 0.07 && fx < w as f32 * 0.28 { c = [1.0, 0.42, 0.17]; }
        c
    })
}

/// Evening sky: a gradient, a low sun with its glow, layered clouds lit from below, and ridges that
/// fade into the haze with distance.
pub fn sky_pixels(w: u32, h: u32) -> Vec<u8> {
    let (sw, sh) = (w as f32, h as f32);
    let sun = (sw * 0.68, sh * 0.6);
    rgba(w, h, |x, y| {
        let v = y / sh;
        let top = [0.10, 0.14, 0.32];
        let mid = [0.55, 0.38, 0.52];
        let low = [1.0, 0.62, 0.32];
        let mut c = if v < 0.45 { mix3(top, mid, v / 0.45) } else { mix3(mid, low, ((v - 0.45) / 0.25).min(1.0)) };
        let d = ((x - sun.0).powi(2) + (y - sun.1).powi(2)).sqrt() / sh;
        let glow = (-d * 5.0).exp() * 0.8;
        c = add3(c, scale3([1.0, 0.75, 0.45], glow));
        if d < 0.055 { c = mix3(c, [1.0, 0.95, 0.8], smooth(0.055, 0.045, d)); }
        // Clouds: two layers of drifting noise, brighter on their sunward underside.
        for (layer, (yc, thick, sc, dens)) in [(0.22f32, 0.16f32, 0.006f32, 0.55f32), (0.42, 0.10, 0.011, 0.6)].into_iter().enumerate() {
            let band = (-((v - yc) / thick).powi(2)).exp();
            let n = fbm(x * sc * 1.0 + layer as f32 * 31.0, y * sc * 2.6 + layer as f32 * 7.0, 5);
            let a = smooth(dens, dens + 0.18, n * band + 0.2 * band);
            let lit = fbm(x * sc + 3.0 + layer as f32 * 31.0, (y + 6.0) * sc * 2.6 + layer as f32 * 7.0, 5);
            let under = mix3([0.30, 0.22, 0.34], [1.0, 0.68, 0.48], (lit - n + 0.5).clamp(0.0, 1.0) * (1.0 - v * 0.4) + glow * 0.4);
            c = mix3(c, under, a * 0.92);
        }
        // Ridges, far to near: each is a 1-D noise skyline; distance mixes it toward the haze.
        for (i, (base, amp, haze)) in [(0.62f32, 0.10f32, 0.55f32), (0.70, 0.12, 0.28), (0.80, 0.14, 0.0)].into_iter().enumerate() {
            let line = base - amp * fbm(x * 0.004 * (1.0 + i as f32 * 0.6) + i as f32 * 13.0, i as f32 * 5.0, 5);
            if v > line {
                let rock = mix3([0.05, 0.04, 0.08], [0.32, 0.22, 0.30], 1.0 - haze);
                let tex = fbm(x * 0.03, y * 0.03 + i as f32 * 9.0, 3) * 0.06;
                c = mix3(add3(rock, [tex, tex, tex]), mix3(low, mid, 0.5), haze * 0.7);
                c = scale3(c, 1.0 - (v - line).min(0.2) * 0.8);
            }
        }
        c
    })
}

/// Terrain seen from above: ridged noise as height, hill-shaded, coloured by height from deep water
/// to snow.
pub fn terrain_pixels(w: u32, h: u32) -> Vec<u8> {
    let height = |x: f32, y: f32| {
        let base = fbm(x * 0.006, y * 0.006, 6);
        let ridge = 1.0 - (fbm(x * 0.012 + 40.0, y * 0.012, 5) * 2.0 - 1.0).abs();
        base * 0.75 + ridge * ridge * 0.35 - 0.12
    };
    let light = norm([-0.6, -0.5, 0.62]);
    rgba(w, h, |x, y| {
        let z = height(x, y);
        let (zx, zy) = (height(x + 1.0, y) - height(x - 1.0, y), height(x, y + 1.0) - height(x, y - 1.0));
        let n = norm([-zx * 90.0, -zy * 90.0, 1.0]);
        let lam = (n[0] * light[0] + n[1] * light[1] + n[2] * light[2]).max(0.0);
        let stops: [(f32, [f32; 3]); 8] = [
            (0.0, [0.02, 0.08, 0.20]), (0.28, [0.05, 0.25, 0.42]), (0.33, [0.78, 0.72, 0.52]), (0.36, [0.30, 0.50, 0.20]),
            (0.48, [0.14, 0.32, 0.12]), (0.60, [0.40, 0.34, 0.26]), (0.70, [0.55, 0.52, 0.50]), (0.78, [0.95, 0.96, 0.98]),
        ];
        let mut c = stops[0].1;
        for k in 1..stops.len() { if z > stops[k - 1].0 { c = mix3(stops[k - 1].1, stops[k].1, ((z - stops[k - 1].0) / (stops[k].0 - stops[k - 1].0)).clamp(0.0, 1.0)); } }
        if z < 0.3 { c } else { scale3(c, 0.25 + 0.95 * lam) }
    })
}

/// A plaster bust in the dark, lit from the upper left: a height field made from soft ellipsoids
/// (head, brow, nose, cheeks, chin, neck, shoulders) with carved eyes and noisy hair, then shaded.
pub fn bust_pixels(w: u32, h: u32) -> Vec<u8> { bust(w, h, false) }
/// The bust with its ground transparent (alpha 0 outside the figure, edges anti-aliased).
pub fn cutout_pixels(w: u32, h: u32) -> Vec<u8> { bust(w, h, true) }
fn bust(w: u32, h: u32, cut: bool) -> Vec<u8> {
    let (sw, sh) = (w as f32, h as f32);
    let k = sh / 400.0;
    let cx = sw * 0.5;
    let ell = |x: f32, y: f32, c: V3, r: V3| {
        let (dx, dy) = ((x - c.0) / r.0, (y - c.1) / r.1);
        let q = 1.0 - dx * dx - dy * dy;
        if q > 0.0 { c.2 + r.2 * q.sqrt() } else { -400.0 }
    };
    let smax = |a: f32, b: f32, s: f32| { let m = a.max(b); m + s * ((-(a - m).abs() / s).exp() + (-(b - m).abs() / s).exp()).ln() };
    let z = move |x: f32, y: f32| {
        let parts: [(V3, V3); 9] = [
            ((cx, 150.0 * k, 0.0), (82.0 * k, 108.0 * k, 85.0 * k)),
            ((cx, 118.0 * k, 50.0 * k), (60.0 * k, 16.0 * k, 26.0 * k)),
            ((cx, 160.0 * k, 66.0 * k), (12.0 * k, 34.0 * k, 26.0 * k)),
            ((cx - 34.0 * k, 178.0 * k, 50.0 * k), (26.0 * k, 22.0 * k, 16.0 * k)),
            ((cx + 34.0 * k, 178.0 * k, 50.0 * k), (26.0 * k, 22.0 * k, 16.0 * k)),
            ((cx, 228.0 * k, 38.0 * k), (34.0 * k, 26.0 * k, 32.0 * k)),
            ((cx, 300.0 * k, -10.0 * k), (46.0 * k, 90.0 * k, 50.0 * k)),
            ((cx, 430.0 * k, -30.0 * k), (240.0 * k, 120.0 * k, 80.0 * k)),
            ((cx, 205.0 * k, 52.0 * k), (22.0 * k, 7.0 * k, 10.0 * k)),
        ];
        let mut h = -400.0;
        for (c, r) in parts { h = smax(h, ell(x, y, c, r), 6.0 * k); }
        // Eye sockets and the mouth line are carved; hair is noise on the crown.
        for ex in [-30.0f32, 30.0] {
            let d = ((x - cx - ex * k) / (17.0 * k)).powi(2) + ((y - 140.0 * k) / (10.0 * k)).powi(2);
            h -= 14.0 * k * (-d).exp();
        }
        let dm = ((x - cx) / (18.0 * k)).powi(2) + ((y - 212.0 * k) / (2.5 * k)).powi(2);
        h -= 5.0 * k * (-dm).exp();
        let crown = smooth(120.0 * k, 70.0 * k, y) * smooth(-400.0, 0.0, h);
        h += crown * (fbm(x * 0.05, y * 0.012, 4) * 14.0 * k);
        h
    };
    let light = norm([-0.55, -0.6, 0.58]);
    let mut px = rgba(w, h, |x, y| {
        let hz = z(x, y);
        let bg = { let d = ((x - sw * 0.35).powi(2) + (y - sh * 0.3).powi(2)).sqrt() / sh; scale3([0.10, 0.10, 0.13], (1.0 - d).max(0.0).powi(3)) };
        if hz < -300.0 { return bg; }
        let (zx, zy) = (z(x + 1.0, y) - z(x - 1.0, y), z(x, y + 1.0) - z(x, y - 1.0));
        let n = norm([-zx * 0.5, -zy * 0.5, 1.0]);
        let lam = (n[0] * light[0] + n[1] * light[1] + n[2] * light[2]).max(0.0);
        let half = norm([light[0], light[1], light[2] + 1.0]);
        let spec = (n[0] * half[0] + n[1] * half[1] + n[2] * half[2]).max(0.0).powf(30.0) * 0.25;
        let rim = (n[0] * 0.8).max(0.0).powi(3) * 0.35;
        let albedo = [0.86, 0.79, 0.70];
        let l = lam.powf(1.4);
        add3(scale3(albedo, 0.03 + 0.95 * l), [spec + rim * 0.5, spec + rim * 0.6, spec + rim * 0.8])
    });
    if cut {
        // Coverage from four samples a texel: the figure is where the height field is.
        for y in 0..h { for x in 0..w {
            let n = [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)].iter().filter(|(dx, dy)| z(x as f32 + dx, y as f32 + dy) >= -300.0).count();
            px[((y * w + x) * 4 + 3) as usize] = (n as f32 / 4.0 * 255.0).round() as u8;
        } }
    }
    px
}
/// The cut-out's alpha, for its outline (`wrap::Silhouette::from_alpha`).
pub fn cutout_alpha() -> (u32, u32, Vec<u8>) {
    let (w, h) = PICTURE_SIZES[CUTOUT as usize];
    (w, h, cutout_pixels(w, h).chunks(4).map(|p| p[3]).collect())
}

/// Centre or radii of an ellipsoid, px.
type V3 = (f32, f32, f32);
fn norm(v: [f32; 3]) -> [f32; 3] { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt(); [v[0] / l, v[1] / l, v[2] / l] }
fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] { [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t] }
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn scale3(a: [f32; 3], k: f32) -> [f32; 3] { [a[0] * k, a[1] * k, a[2] * k] }
fn smooth(e0: f32, e1: f32, x: f32) -> f32 { let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) }
fn hash(x: i32, y: i32) -> f32 {
    let mut n = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    ((n ^ (n >> 16)) & 0xffff) as f32 / 65535.0
}
fn noise(x: f32, y: f32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (ux, uy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let a = hash(xi, yi) + (hash(xi + 1, yi) - hash(xi, yi)) * ux;
    let b = hash(xi, yi + 1) + (hash(xi + 1, yi + 1) - hash(xi, yi + 1)) * ux;
    a + (b - a) * uy
}
/// Sum of octaves of value noise, about 0..1.
fn fbm(x: f32, y: f32, octaves: u32) -> f32 {
    let (mut s, mut a, mut f, mut norm) = (0.0, 0.5, 1.0, 0.0);
    for _ in 0..octaves { s += a * noise(x * f, y * f); norm += a; a *= 0.5; f *= 2.03; }
    s / norm
}

pub type DemoUi = crate::ui::Ui<Demo, Msg>;
pub fn new_ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> DemoUi { crate::ui::Ui::new(Demo::default(), view, update, faces, w, h, scale) }
