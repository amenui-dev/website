//! The material pages (one per material: a fixed specimen held at a looseness, the material's numbers,
//! a button that sends a front over it) and the annotation page (modes and kinds over a sample).
use super::pages::{header, ratio, section};
use super::{Gallery, IMAGE_SLOT, Msg, Page, Target};
use crate::demo::{SKY, TERRAIN};
use crate::el::{El, Font, Paint, Response, Scope, col, picture, row, space, text, text_lh};
use crate::style::{GAZE_MODES, MAT_PARAMS, Material, gaze};
use crate::widgets::{SELECTED, Spec, Variant, button_as, card, number, segmented, switch};
use taffy::{AlignItems, JustifyContent};

pub fn page(s: &Gallery) -> El<Msg> {
    match s.page {
        Page::Material(m) => material_page(s, m),
        _ => gaze_page(s),
    }
}

pub const MATERIAL_DESC: [&str; 9] = [
    "文字和形状不松开：按住的和新出现的东西变淡。底色是平的。",
    "全局点阵：松开的文字和形状变成点，每个点的大小是它底下墨的多少；底色上有一层随波前起伏的小点。",
    "松开时变成旋转的半色调网点，网点的大小由墨的多少决定；可以分成三个网屏，各自错开一点。",
    "松开时每一格显示一个字符，按格里墨的多少从字符记号表里挑选；越松开，格子越粗。",
    "静止时纸面有细微的纹理，强调色的墨略微错位；松开时出现复印的斑点，墨越过轮廓洇开。",
    "只有墨和底两种颜色。松开时轮廓像用刀刻出来，边缘出现沿着轮廓的刀痕；越松开，刻痕带越宽。",
    "静止时没有变化。松开时一行行错开、颜色通道分离、色块重复；指针移到控件上或按下控件时，控件上短暂地出现同样的效果。",
    "把中间图画出来：离轮廓的远近画成一圈圈等距线，墨里是暖色，墨外是冷色；轮廓本身用墨色，松开的地方带一层颜色。",
    "静止时和 plain 相同。松开时填充变淡，墨变成一根连续的细线：先贴着轮廓，再用直线把字连成一笔，松到头时游走、打结，线头顺着流向甩出去；收回时倒着来。",
];
pub const LOOSE_SPEC: Spec = Spec { min: 0.0, max: 1.0, step: 0.05, decimals: 2, unit: "" };

/// The fixed specimen: headline, paragraph, picture, a row of controls, a card.
pub fn specimen(s: &Gallery, m: u8, h: &str) -> El<Msg> {
    let sz = s.theme().sizes;
    // On a narrow window both columns take what is left inside the specimen's margins.
    let fit = |w: f32| w.min((s.width - if s.width < super::NARROW { 36.0 + 72.0 } else { super::RAIL + 180.0 }).max(160.0));
    let left = col().key(format!("{h}/left")).width(fit(430.0)).gap(18.0)
        .child(text_lh("松开与合拢", 52.0, 64.0, Paint::INK).font(Font::Display).key(format!("{h}/headline")))
        .child(text_lh("材质决定松开时画什么。静止时，页面是一张干净、可读的页面；波前经过时，文字、形状和图片按各自的轮廓松开，再合拢回来。",
            sz.body, (sz.body * 1.7).round(), Paint::TEXT).key(format!("{h}/body")))
        .child(row().key(format!("{h}/controls")).gap(10.0).wrap().align(AlignItems::CENTER)
            .child(button_as(format!("{h}/cancel"), "取消", Variant::Default, Msg::Press))
            .child(button_as(format!("{h}/save"), "保存", Variant::Accent, Msg::Press))
            .child(segmented(&format!("{h}/seg"), &["日", "周", "月"], s.pick.min(2), 30.0, Msg::Pick)))
        .child(switch(format!("{h}/switch"), "显示网格", s.on, None, Msg::Switch).width(260.0));
    let right = col().key(format!("{h}/right")).width(fit(360.0)).gap(16.0)
        .child(col().key(format!("{h}/picture-box")).child(picture(SKY, ratio(SKY)).key(format!("{h}/picture")).pic_fx(IMAGE_SLOT)))
        .child(card(format!("{h}/card"), Some("卡片"), text_lh("一张卡片：边框、标题和一段文字。", 14.0, 24.0, Paint::TEXT)).width(fit(360.0)))
        .child(row().key(format!("{h}/shapes")).gap(14.0).align(AlignItems::CENTER)
            .child(space(84.0, 84.0).radius(42.0).fill(Paint::ACCENT).key(format!("{h}/disc")))
            .child(space(84.0, 84.0).radius(42.0).fill(Paint::hue(1)).key(format!("{h}/disc2")))
            .child(space(110.0, 64.0).radius(8.0).fill(Paint::INK).key(format!("{h}/slab"))));
    // The specimen shows how the material loosens everything, text included: it answers fronts fully.
    row().key(format!("{h}/specimen")).respond(Response::Loosen).legible(false).material(m).loosen(s.loose).fill_width().wrap().gap(40.0).pad(36.0, 32.0).radius(6.0).border(1.0, Paint::FAINT)
        .align(AlignItems::START).child(left).child(right)
}

fn material_page(s: &Gallery, m: u8) -> El<Msg> {
    let mat = Material::from_id(m);
    let h = format!("material-{}", mat.name());
    let h = h.as_str();
    let mut nums = row().key(format!("{h}/params")).fill_width().wrap().gap(10.0);
    for (i, p) in MAT_PARAMS[m as usize].iter().enumerate() {
        let t = Target::Mat(m, i);
        let spec = Spec { min: p.min, max: p.max, step: p.step, decimals: p.decimals, unit: p.unit };
        nums = nums.child(number(&format!("{h}/p{i}"), p.name, s.mats[m as usize].0[i], spec, s.draft(t), move |e| Msg::Num(t, e)).width(330.0));
    }
    if MAT_PARAMS[m as usize].is_empty() { nums = nums.child(text("这个材质没有自己的参数；点阵的点距在骨架演示页里设置。", 13.0, Paint::DIM)); }
    let live = row().key(format!("{h}/live")).gap(12.0).wrap().align(AlignItems::CENTER)
        .child(number(&format!("{h}/loose"), "松开度", s.loose, LOOSE_SPEC, s.draft(Target::Loose), |e| Msg::Num(Target::Loose, e)).width(330.0))
        .child(button_as(format!("{h}/ripple"), "涟漪", Variant::Default, Msg::Nothing).ripple(Scope::Container(Some(format!("{h}/specimen")))))
        .child(text("按“涟漪”从按钮处放出一道波前；松开度把整个样张固定在一个松开的程度。", 13.0, Paint::DIM));
    col().key(format!("{h}/page")).fill_width().max_width(1000.0).gap(34.0)
        .child(header(s, h, mat.name(), &format!("Material::{:?}", mat), MATERIAL_DESC[m as usize]))
        .child(section(h, "样张", col().fill_width().gap(14.0).child(specimen(s, m, h)).child(live)))
        .child(section(h, "参数", nums))
        .child(space(1.0, 24.0))
}

/// A small layout for the annotation layer to look at: a header, three figures, a picture and a list.
fn sample(_s: &Gallery) -> El<Msg> {
    let figure = |k: &str, name: &str, v: &str, label: &str| col().key(format!("gaze/fig-{k}")).width(170.0).gap(6.0).pad(14.0, 12.0).radius(6.0).border(1.0, Paint::FAINT).annotate(label)
        .child(text(name, 12.0, Paint::DIM))
        .child(text(v, 26.0, Paint::INK).font(Font::Mono));
    let rows = ["上传 1,204 张图片", "整理 38 个相册", "导出 6 份报告"];
    let mut list = col().key("gaze/list").width(250.0).gap(2.0).annotate("最近");
    for (i, r) in rows.iter().enumerate() {
        list = list.child(row().key(format!("gaze/row{i}")).height(30.0).pad(10.0, 0.0).align(AlignItems::CENTER).radius(4.0).on_press(Msg::Row(i))
            .child(text(*r, 13.0, Paint::TEXT)));
    }
    col().key("gaze/sample").gaze_scope().fill_width().gap(18.0).pad(40.0, 36.0).radius(6.0).border(1.0, Paint::FAINT)
        .child(row().key("gaze/head").fill_width().align(AlignItems::CENTER).justify(JustifyContent::SPACE_BETWEEN).annotate("标题栏")
            .child(text("本周", 24.0, Paint::INK).font(Font::Display))
            .child(button_as("gaze/export", "导出", Variant::Accent, Msg::Press).annotate("导出按钮")))
        .child(row().key("gaze/figs").gap(14.0).wrap()
            .child(figure("a", "图片", "1,204", "图片数"))
            .child(figure("b", "相册", "38", "相册数"))
            .child(figure("c", "报告", "6", "报告数")))
        .child(row().key("gaze/lower").gap(18.0).align(AlignItems::START)
            .child(col().key("gaze/pic").width(300.0).annotate("地形图").child(picture(TERRAIN, ratio(TERRAIN)).key("gaze/picture").pic_fx(IMAGE_SLOT)))
            .child(list))
}

fn gaze_page(s: &Gallery) -> El<Msg> {
    let h = "gaze";
    let g = s.gaze_page;
    let mode = GAZE_MODES.iter().position(|m| m.0 == g.mode).unwrap_or(0);
    let names: Vec<&str> = GAZE_MODES.iter().map(|m| m.1).collect();
    let mut chips = row().key("gaze/kinds").gap(6.0).wrap();
    for (i, name) in gaze::NAMES.iter().enumerate() {
        let bit = 1u16 << i;
        let on = g.has(bit);
        chips = chips.child(row().key(format!("gaze/kind:{i}")).height(26.0).pad(10.0, 0.0).radius(4.0).align(AlignItems::CENTER)
            .fill(if on { SELECTED } else { Paint::GROUND }).border(1.0, if on { Paint::ACCENT } else { Paint::FAINT }).on_press(Msg::GazeKind(bit))
            .child(text(*name, 13.0, if on { Paint::ACCENT } else { Paint::TEXT })));
    }
    let controls = col().key("gaze/controls").fill_width().gap(14.0)
        .child(row().gap(12.0).align(AlignItems::CENTER).child(text("模式", 13.0, Paint::DIM)).child(segmented("gaze/mode", &names, mode, 30.0, Msg::GazeMode)))
        .child(row().gap(12.0).align(AlignItems::START).child(row().height(26.0).align(AlignItems::CENTER).child(text("画什么", 13.0, Paint::DIM))).child(chips))
        .child(row().gap(10.0).wrap()
            .child(number("gaze/density", "最多几个", g.density, Spec { min: 0.0, max: 60.0, step: 1.0, decimals: 0, unit: "" }, s.draft(Target::Density), |e| Msg::Num(Target::Density, e)).width(330.0))
            .child(number("gaze/draw", "画出时长", g.draw_ms, Spec { min: 0.0, max: 3000.0, step: 20.0, decimals: 0, unit: "ms" }, s.draft(Target::DrawMs), |e| Msg::Num(Target::DrawMs, e)).width(330.0))
            .child(number("gaze/osc-amp", "波形高", g.osc_amp, Spec { min: 4.0, max: 120.0, step: 2.0, decimals: 0, unit: "px" }, s.draft(Target::OscAmp), |e| Msg::Num(Target::OscAmp, e)).width(330.0))
            .child(number("gaze/osc-y", "波形基线", g.osc_y, Spec { min: 0.0, max: 1.0, step: 0.05, decimals: 2, unit: "" }, s.draft(Target::OscY), |e| Msg::Num(Target::OscY, e)).width(330.0))
            .child(number("gaze/level-full", "电平满格", g.level_full, Spec { min: 0.01, max: 0.5, step: 0.01, decimals: 2, unit: "" }, s.draft(Target::LevelFull), |e| Msg::Num(Target::LevelFull, e)).width(330.0)));
    col().key("gaze/page").fill_width().max_width(1000.0).gap(34.0)
        .child(header(s, h, "标注层", "Gaze / El::annotate", "在材质之上按排版结果画出框、编号、尺寸、类型和引线，数字都是真的。“悬停”只标出指针下的元素和它的上级，“焦点路径”标出有焦点的元素和它的上级。元素出现时框线画出来，编号从零数上去。"))
        .child(section(h, "样例", sample(s)))
        .child(section(h, "设置", controls))
        .child(space(1.0, 24.0))
}
