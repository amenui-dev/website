//! One page per style preset (DESIGN.md §4): each a small composition of its own, made of the existing
//! components, with at least one real interaction. Opening a page applies its style.
use super::pages::ratio;
use super::{Gallery, Msg, STYLE_SLOT, Target};
use crate::demo::{BUST, SKY, SPHERE, TERRAIN};
use crate::el::{El, Font, Paint, Scope, col, picture, row, space, text, text_lh};
use crate::style::{Material, styles};
use crate::widgets::{Spec, Variant, button_as, card, choice, input, number, segmented, slider, switch, tabs};
use taffy::{AlignItems, JustifyContent};

pub const STYLE_DESC: [&str; 12] = [
    "深色底，等宽字。标注层框出页面上的每个元素，标出编号、类型和尺寸，十字线跟着指针。",
    "黑底，琥珀色的灯。像素字，粗点距；松开时文字散成发光的点。",
    "米白的纸和一种专色。纸纹、套印错位，松开时像复印件一样起斑、洇墨。",
    "黑底和两种信号色。静止时干净；指针移到控件上，控件就短暂地错位。",
    "米色纸上的黑和红。粗重的标题像刻出来的，松开时边缘出现刀痕。",
    "冷色、半透明。层叠的面板只画轮廓，互相透出；引线标出每个部件。",
    "白纸、黑线和一种信号色。部件带编号和引线，零件表的一行对应图上的一个部件。",
    "系统灰，默认字号。松开的幅度只有三成，几乎看不出材质。",
    "近黑的底，一条细的强调色。大片留白；只在指针下标出元素。",
    "三原色加黑，几何形和粗网格。松开时变成半色调网点。",
    "银色的渐变底。大字是金属，圆斑是液体，靠近时融在一起。",
    "牛皮纸上的拼贴：每一块用自己的材质，形状的边缘像撕开的纸。",
];

pub fn page(s: &Gallery, i: usize) -> El<Msg> {
    let st = styles()[i];
    let h = format!("style-{}", st.hash);
    let body = match i {
        0 => gaze_feed(s, &h),
        1 => dotmatrix(s, &h),
        2 => print_zine(s, &h),
        3 => glitch(s, &h),
        4 => linocut(s, &h),
        5 => electronics(s, &h),
        6 => manual(s, &h),
        7 => normcore(s, &h),
        8 => cyber(s, &h),
        9 => bauhaus(s, &h),
        10 => y2k(s, &h),
        _ => mixed(s, &h),
    };
    // A press anywhere on the composition sends its front over the whole composition (DESIGN.md §1.15):
    // pictures and shapes loosen, text and controls only as far as the legibility cap.
    col().key(format!("{h}/page")).fill_width().min_width(0.0).gap(26.0).pad(if s.width < super::NARROW { 18.0 } else { 48.0 }, 36.0).ripple_scope().ripple(Scope::Container(None))
        .child(row().key(format!("{h}/caption")).fill_width().gap(14.0).align(AlignItems::BASELINE).wrap()
            .child(text(st.name, 13.0, Paint::DIM).font(Font::Mono).key(format!("{h}/name")))
            .child(text(STYLE_DESC[i], 13.0, Paint::DIM).key(format!("{h}/desc")).grow(1.0)))
        // A composition is drawn at its own size: a narrow window scrolls it sideways.
        .child(if s.width < super::NARROW { col().key(format!("{h}/pan")).fill_width().min_width(0.0).scroll_x().child(body) } else { body })
}

fn pic(h: &str, k: &str, id: u32, w: f32) -> El<Msg> { pic_at(h, k, id, w, 0.0) }
/// A picture in the style's marks, held at a looseness (1: always shown as marks).
fn pic_at(h: &str, k: &str, id: u32, w: f32, loose: f32) -> El<Msg> {
    col().key(format!("{h}/{k}-box")).width(w).child(picture(id, ratio(id)).key(format!("{h}/{k}")).pic_fx(STYLE_SLOT).loose(loose))
}

// ── 1 Algorithmic / Digital Gaze: a feed, framed and measured ───────────────────────────────────────
fn gaze_feed(s: &Gallery, h: &str) -> El<Msg> {
    const ITEMS: [(u32, &str, &str, f32); 6] = [
        (BUST, "人物", "IMG_0412", 0.93), (TERRAIN, "地形", "IMG_0418", 0.88), (SKY, "天空", "IMG_0421", 0.97),
        (SPHERE, "物体", "IMG_0433", 0.71), (BUST, "人物", "IMG_0440", 0.84), (TERRAIN, "地形", "IMG_0452", 0.66),
    ];
    let filter = s.choices[0].min(3);
    let names = ["全部", "人物", "地形", "天空"];
    let mut feed = row().key(format!("{h}/feed")).gap(22.0).wrap().fill_width();
    for (i, (id, tag, file, conf)) in ITEMS.iter().enumerate() {
        if filter != 0 && *tag != names[filter] { continue; }
        feed = feed.child(col().key(format!("{h}/item{i}")).width(250.0).gap(8.0).on_press(Msg::Row(i)).focusable(false)
            .child(pic(h, &format!("p{i}"), *id, 250.0))
            .child(row().justify(JustifyContent::SPACE_BETWEEN).fill_width()
                .child(text(*file, 12.0, Paint::TEXT).font(Font::Mono))
                .child(text(format!("{tag} {conf:.2}"), 12.0, Paint::ACCENT).font(Font::Mono))));
    }
    col().key(format!("{h}/comp")).fill_width().gap(24.0).gaze_scope()
        .child(row().key(format!("{h}/head")).fill_width().align(AlignItems::END).justify(JustifyContent::SPACE_BETWEEN).wrap().gap(16.0)
            .child(col().gap(4.0)
                .child(text("FIELD 07 / 观察记录", 12.0, Paint::DIM).font(Font::Mono))
                .child(text_lh("视野里的一切都被看见", 34.0, 44.0, Paint::INK).font(Font::Display)))
            .child(segmented(&format!("{h}/filter"), &names, filter, 30.0, |i| Msg::Choose(0, i))))
        .child(feed)
}

// ── 2 Neo-Dotmatrix: a departure board ──────────────────────────────────────────────────────────────
fn dotmatrix(s: &Gallery, h: &str) -> El<Msg> {
    let t = 8.0 * 3600.0 + 42.0 * 60.0 + s.clock.floor();
    let clock = format!("{:02}:{:02}:{:02}", (t / 3600.0) as u32 % 24, (t / 60.0) as u32 % 60, t as u32 % 60);
    let arrive = s.choices[1] == 1;
    let rows: [(&str, &str, &str); 5] = if arrive {
        [("08:31", "上海 虹桥", "3"), ("08:40", "杭州 东", "5"), ("08:52", "南京 南", "1"), ("09:05", "苏州", "4"), ("09:18", "宁波", "2")]
    } else {
        [("08:47", "北京 南", "2"), ("08:55", "广州 南", "6"), ("09:03", "成都 东", "4"), ("09:12", "西安 北", "1"), ("09:26", "武汉", "7")]
    };
    let tick = s.clock.floor() as usize;
    let mut board = col().key(format!("{h}/board")).width(760.0).gap(2.0).pad(22.0, 18.0).radius(4.0).border(2.0, Paint::FAINT);
    board = board.child(row().key(format!("{h}/cols")).fill_width().gap(18.0).pad(0.0, 6.0)
        .child(text("时间", 12.0, Paint::DIM).width(90.0)).child(text("目的地", 12.0, Paint::DIM).grow(1.0))
        .child(text("站台", 12.0, Paint::DIM).width(60.0)).child(text("状态", 12.0, Paint::DIM).width(120.0)));
    for (i, (time, dest, plat)) in rows.iter().enumerate() {
        // The status of one row changes every few seconds, as boards do.
        let late = (tick / 4 + i).is_multiple_of(5);
        let status = if late { format!("晚点 {} 分", 3 + (tick / 4 + i) % 7) } else { "正点".to_owned() };
        board = board.child(row().key(format!("{h}/row{i}")).fill_width().gap(18.0).height(36.0).align(AlignItems::CENTER)
            .child(text(*time, 22.0, Paint::ACCENT).font(Font::Mono).width(90.0))
            .child(text(*dest, 24.0, Paint::ACCENT).grow(1.0))
            .child(text(*plat, 22.0, Paint::ACCENT).font(Font::Mono).width(60.0))
            .child(text(status, 24.0, if late { Paint::hue(1) } else { Paint::ACCENT }).width(120.0).key(format!("{h}/status{i}-{late}"))));
    }
    col().key(format!("{h}/comp")).gap(26.0)
        .child(row().key(format!("{h}/sign")).gap(30.0).align(AlignItems::END).wrap()
            .child(text_lh(if arrive { "到达" } else { "出发" }, 72.0, 80.0, Paint::ACCENT).font(Font::Display).key(format!("{h}/title-{arrive}")))
            .child(text(clock, 48.0, Paint::ACCENT).font(Font::Mono).print(4.0).key(format!("{h}/clock"))))
        .child(board)
        .child(led_panel(s, h, t))
        .child(row().key(format!("{h}/controls")).gap(12.0).align(AlignItems::CENTER).wrap()
            .child(segmented(&format!("{h}/mode"), &["出发", "到达"], s.choices[1].min(1), 30.0, |i| Msg::Choose(1, i)))
            .child(button_as(format!("{h}/refresh"), "刷新", Variant::Default, Msg::Press))
            .child(text("站台", 12.0, Paint::DIM))
            .child(number(&format!("{h}/platform"), "", s.slides[5], Spec { min: 1.0, max: 9.0, step: 1.0, decimals: 0, unit: "" }, s.draft(Target::StyleNum(5)), |e| Msg::Num(Target::StyleNum(5), e)).width(130.0))
            .child(text("时差", 12.0, Paint::DIM))
            .child(number(&format!("{h}/offset"), "", s.slides[6], Spec { min: -720.0, max: 720.0, step: 1.0, decimals: 0, unit: "分" }, s.draft(Target::StyleNum(6)), |e| Msg::Num(Target::StyleNum(6), e)).width(150.0)))
}

/// The second panel: a big clock printed in dots, and a ticker that runs along under it.
fn led_panel(s: &Gallery, h: &str, t: f32) -> El<Msg> {
    let t = t + s.slides[6] * 60.0;
    let big = format!("{:02}:{:02}", (t / 3600.0).rem_euclid(24.0) as u32, (t / 60.0).rem_euclid(60.0) as u32);
    let secs = format!("{:02}", t.rem_euclid(60.0) as u32);
    let plat = s.slides[5].round().clamp(1.0, 9.0) as u32;
    let msg = format!("{plat} 站台　·　请站在黄线以内候车　·　列车即将进站，请注意安全　·　下一班 08:47 北京 南　·　");
    // The ticker moves a character's width every second: the text is the view, the clock moves it.
    let step = (s.clock.floor() as usize) % 40;
    let shown: String = msg.chars().chain(msg.chars()).skip(step).take(60).collect();
    col().key(format!("{h}/panel2")).width(760.0).gap(10.0).pad(22.0, 18.0).radius(4.0).border(2.0, Paint::FAINT)
        .child(row().fill_width().justify(JustifyContent::SPACE_BETWEEN)
            .child(text(format!("{plat} 站台　·　本站时间"), 12.0, Paint::DIM))
            .child(text("NEXT 08:47", 12.0, Paint::DIM).font(Font::Mono)))
        .child(row().gap(10.0).align(AlignItems::END)
            .child(text_lh(big, 150.0, 156.0, Paint::ACCENT).font(Font::Mono).print(7.0).key(format!("{h}/bigclock")))
            .child(text_lh(secs, 56.0, 64.0, Paint::hue(1)).font(Font::Mono).print(4.0).key(format!("{h}/secs"))))
        .child(space(1.0, 2.0).fill_width().fill(Paint::FAINT))
        // The ticker: one long line, clipped to the panel, moved one character a second.
        .child(col().key(format!("{h}/ticker")).fill_width().height(40.0).clip()
            .child(text_lh(shown, 24.0, 36.0, Paint::ACCENT).width(3000.0).key(format!("{h}/ticker-text")).absolute(Some(0.0), Some(2.0), None, None)))
}

// ── 3 Lo-Fi Print: a zine page ───────────────────────────────────────────────────────────────────────
fn print_zine(s: &Gallery, h: &str) -> El<Msg> {
    let issue = s.choices[2].min(2);
    let issues = ["第 5 期", "第 6 期", "第 7 期"];
    let para = |k: &str, t: &str| text_lh(t, 15.0, 27.0, Paint::TEXT).key(format!("{h}/{k}"));
    col().key(format!("{h}/comp")).width(900.0).gap(0.0)
        .child(row().key(format!("{h}/mast")).fill_width().height(44.0).pad(18.0, 0.0).fill(Paint::INK).align(AlignItems::CENTER).justify(JustifyContent::SPACE_BETWEEN)
            .child(text("慢 读 · LO-FI ZINE", 15.0, Paint::GROUND).font(Font::Mono))
            .child(text(issues[issue], 15.0, Paint::GROUND)))
        .child(row().key(format!("{h}/spread")).fill_width().gap(36.0).pad(0.0, 30.0).align(AlignItems::START)
            .child(col().key(format!("{h}/left")).width(470.0).gap(18.0)
                .child(text_lh("复印机里的春天", 64.0, 72.0, Paint::ACCENT).font(Font::Display).weight(1.5).key(format!("{h}/headline")))
                .child(text("文 / 编辑部　图 / 读者来稿", 13.0, Paint::DIM))
                .child(para("p1", "这一期收的是读者寄来的照片和短文。我们把它们放大、复印、再复印，直到颗粒和墨点都成了画面的一部分。"))
                .child(para("p2", "纸张偏黄，粉色的专色和黑色分两次印，所以总有一点对不准。那一点错位，是手工的证明。"))
                .child(row().gap(12.0).align(AlignItems::CENTER).wrap()
                    .child(button_as(format!("{h}/subscribe"), "订阅", Variant::Accent, Msg::Press))
                    .child(segmented(&format!("{h}/issue"), &issues, issue, 30.0, |i| Msg::Choose(2, i)))))
            .child(col().key(format!("{h}/right")).width(390.0).gap(12.0)
                .child(pic_at(h, "photo", if issue == 1 { SKY } else { BUST }, 390.0, 1.0))
                .child(row().gap(10.0).align(AlignItems::CENTER)
                    .child(space(28.0, 28.0).radius(14.0).fill(Paint::ACCENT))
                    .child(text_lh("图注：石膏像，复印三次。", 13.0, 20.0, Paint::DIM)))))
        .child(space(1.0, 3.0).fill_width().fill(Paint::INK))
}

// ── 4 Glitch: a title card that bursts ───────────────────────────────────────────────────────────────
fn glitch(s: &Gallery, h: &str) -> El<Msg> {
    col().key(format!("{h}/comp")).fill_width().gap(30.0).pad(0.0, 30.0)
        .child(col().key(format!("{h}/card")).gap(10.0)
            .child(text("ERR 0x3F　连接已中断", 14.0, Paint::hue(1)).font(Font::Mono))
            // Pressable, so the pointer coming onto it fires a burst over it.
            .child(text_lh("信号中断", 120.0, 132.0, Paint::INK).font(Font::Display).weight(3.0).key(format!("{h}/title")).on_press(Msg::Press).focusable(false))
            .child(text_lh("画面会在你经过时短暂地错位。静止时，它就是一张干净的页面。", 16.0, 28.0, Paint::DIM)))
        .child(row().key(format!("{h}/bar")).fill_width().height(2.0).fill(Paint::ACCENT))
        .child(row().key(format!("{h}/controls")).gap(12.0).align(AlignItems::CENTER).wrap()
            .child(button_as(format!("{h}/retry"), "重新连接", Variant::Accent, Msg::Press))
            .child(button_as(format!("{h}/skip"), "跳过", Variant::Default, Msg::Press))
            .child(button_as(format!("{h}/report"), "发送报告", Variant::Quiet, Msg::Press))
            .child(switch(format!("{h}/auto"), "自动重试", s.toggles[3], None, Msg::Toggle(3)).width(200.0))
            .child(text(format!("已重试 {} 次", s.presses), 13.0, Paint::DIM).font(Font::Mono).key(format!("{h}/count"))))
        .child(glitch_monitor(s, h))
}

/// Below the card: three channels, each a picture and a signal level, that tear when the pointer
/// comes onto them.
fn glitch_monitor(s: &Gallery, h: &str) -> El<Msg> {
    let chans = [(SKY, "CH 01", 0.82), (TERRAIN, "CH 02", 0.41), (BUST, "CH 03", 0.0)];
    let mut r = row().key(format!("{h}/monitor")).gap(20.0).wrap();
    for (i, (id, name, level)) in chans.iter().enumerate() {
        let lost = *level == 0.0 && s.presses == 0;
        let bars = (0..12).fold(row().gap(3.0).align(AlignItems::END).height(22.0), |r, b| {
            let on = (b as f32) < level * 12.0 || (!lost && *level == 0.0 && b < 9);
            r.child(space(6.0, 6.0 + b as f32 * 1.4).fill(if on { Paint::ACCENT } else { Paint::FAINT }))
        });
        r = r.child(col().key(format!("{h}/ch{i}")).width(260.0).gap(8.0).on_press(Msg::Press).focusable(false)
            .child(pic(h, &format!("ch{i}-pic"), *id, 260.0))
            .child(row().fill_width().justify(JustifyContent::SPACE_BETWEEN).align(AlignItems::CENTER)
                .child(text(*name, 12.0, Paint::TEXT).font(Font::Mono))
                .child(text(if lost { "无信号" } else { "已连接" }, 12.0, if lost { Paint::hue(0) } else { Paint::hue(1) }))
                .child(bars)));
    }
    r
}

// ── 5 Metamorphosis Linocut: a poster ────────────────────────────────────────────────────────────────
fn linocut(s: &Gallery, h: &str) -> El<Msg> {
    let shown = [BUST, TERRAIN, SKY][s.choices[3].min(2)];
    row().key(format!("{h}/comp")).gap(44.0).align(AlignItems::START).wrap()
        .child(col().key(format!("{h}/left")).width(520.0).gap(20.0)
            .child(col().key(format!("{h}/hero")).width(520.0).height(300.0)
                .child(space(190.0, 190.0).radius(95.0).fill(Paint::ACCENT).loose(0.6).key(format!("{h}/sun")).absolute(Some(320.0), Some(0.0), None, None))
                .child(text_lh("刀与木", 140.0, 150.0, Paint::INK).font(Font::Display).weight(3.0).loose(0.45).key(format!("{h}/headline")).absolute(Some(0.0), Some(130.0), None, None)))
            .child(text_lh("每一道线都是从板上挖掉的。留下的才是墨。", 18.0, 30.0, Paint::INK))
            .child(text_lh("版画展　十一月二十日至十二月十日　每天九点至五点", 14.0, 24.0, Paint::DIM))
            .child(row().gap(12.0).align(AlignItems::CENTER)
                .child(button_as(format!("{h}/next"), "换一张", Variant::Accent, Msg::Choose(3, (s.choices[3] + 1) % 3)))
                .child(text(format!("第 {} 张，共 3 张", s.choices[3] % 3 + 1), 13.0, Paint::DIM))))
        .child(col().key(format!("{h}/right")).width(380.0).gap(10.0)
            // The print itself is held loose: cut in strokes that follow the form.
            .child(col().key(format!("{h}/print-box")).width(380.0).child(picture(shown, ratio(shown)).key(format!("{h}/print")).pic_fx(STYLE_SLOT).loose(1.0)))
            .child(text("木版　34 × 28 厘米", 12.0, Paint::DIM)))
}

// ── 6 Transparent Electronics: see-through layers of a device ───────────────────────────────────────
fn electronics(s: &Gallery, h: &str) -> El<Msg> {
    let open = s.toggles[0];
    let k = |d: f32| if open { d } else { d * 0.25 };
    let slide = s.slides[2] * 120.0;
    let layer = |key: &str, l: f32, t: f32, w: f32, hh: f32, r: f32, p: Paint, name: &str| space(w, hh).radius(r).fill(p).key(format!("{h}/{key}")).annotate(name).absolute(Some(l), Some(t), None, None);
    let device = col().key(format!("{h}/device")).width(820.0).height(470.0).gaze_scope()
        .child(layer("case", 240.0, 40.0 + k(0.0), 340.0, 380.0, 34.0, Paint::INK, "外壳"))
        .child(layer("board", 270.0 - k(40.0), 90.0 + k(30.0), 280.0, 210.0, 10.0, Paint::hue(2), "主板"))
        .child(layer("chip", 330.0 - k(60.0), 140.0 + k(50.0), 90.0, 90.0, 6.0, Paint::hue(1), "处理器"))
        .child(layer("battery", 300.0 + k(70.0), 250.0 + k(60.0), 220.0, 110.0, 14.0, Paint::ACCENT, "电池"))
        .child(layer("screen", 255.0 + k(90.0) - slide * 0.3, 60.0 - k(20.0) + slide * 0.5, 310.0, 330.0, 26.0, Paint::INK, "屏幕"))
        .child(layer("button", 585.0 + k(20.0), 140.0, 14.0, 60.0, 7.0, Paint::ACCENT, "电源键"));
    col().key(format!("{h}/comp")).gap(18.0)
        .child(device)
        .child(row().key(format!("{h}/controls")).gap(18.0).align(AlignItems::CENTER).wrap()
            .child(switch(format!("{h}/explode"), "展开", open, None, Msg::Toggle(0)).width(160.0))
            .child(text("屏幕位置", 13.0, Paint::DIM))
            .child(slider(&format!("{h}/slide"), s.slides[2] * 100.0, Spec { min: 0.0, max: 100.0, step: 1.0, decimals: 0, unit: "" }, 260.0, |v| Msg::Slide(2, v / 100.0))))
        .child(readouts(s, h))
}

/// A row of readouts under the device, each a see-through gauge.
fn readouts(s: &Gallery, h: &str) -> El<Msg> {
    let vals = [("电压", 3.7 + s.slides[2] * 0.5, "V", 0.62 + s.slides[2] * 0.3), ("温度", 31.0 + s.slides[2] * 9.0, "°C", 0.38 + s.slides[2] * 0.2), ("频率", 2.4, "GHz", 0.8)];
    let mut r = row().key(format!("{h}/readouts")).gap(18.0).wrap();
    for (i, (name, v, unit, f)) in vals.iter().enumerate() {
        r = r.child(col().key(format!("{h}/gauge{i}")).width(250.0).height(120.0).pad(18.0, 16.0).radius(14.0).fill(Paint::INK).gap(10.0)
            .child(row().fill_width().justify(JustifyContent::SPACE_BETWEEN)
                .child(text(*name, 13.0, Paint::DIM))
                .child(text(format!("{v:.1} {unit}"), 18.0, Paint::INK).font(Font::Mono)))
            .child(col().width(214.0).height(18.0).radius(9.0).fill(Paint::hue(2))
                .child(space(214.0 * f.min(1.0), 18.0).radius(9.0).fill(Paint::ACCENT))));
    }
    r
}

// ── 7 Tech-Manual: an exploded diagram and its parts list ───────────────────────────────────────────
const PARTS: [(&str, &str); 6] = [("上盖", "1"), ("面板", "1"), ("旋钮", "2"), ("电路板", "1"), ("电池仓", "1"), ("底座", "1")];
fn manual(s: &Gallery, h: &str) -> El<Msg> {
    let hot = |i: usize| s.hovered.as_deref() == Some(format!("{h}/part{i}").as_str()) || s.hovered.as_deref() == Some(format!("{h}/row{i}").as_str());
    let part = |i: usize, l: f32, t: f32, w: f32, hh: f32, r: f32| {
        let on = hot(i);
        space(w, hh).radius(r).fill(if on { Paint::ACCENT.alpha(0.85) } else { Paint::tone(0.06) }).border(1.0, if on { Paint::ACCENT } else { Paint::INK })
            .key(format!("{h}/part{i}")).annotate(PARTS[i].0).on_press(Msg::Row(i)).focusable(false).absolute(Some(l), Some(t), None, None)
    };
    // The parts stacked along a diagonal, pulled apart.
    let diagram = col().key(format!("{h}/diagram")).width(600.0).height(520.0).gaze_scope()
        .child(part(0, 210.0, 40.0, 240.0, 46.0, 8.0))
        .child(part(1, 230.0, 120.0, 200.0, 70.0, 4.0))
        .child(part(2, 270.0, 215.0, 36.0, 36.0, 18.0))
        .child(space(1.0, 300.0).fill(Paint::DIM).absolute(Some(330.0), Some(70.0), None, None))
        .child(part(3, 220.0, 280.0, 220.0, 54.0, 2.0))
        .child(part(4, 250.0, 360.0, 160.0, 44.0, 6.0))
        .child(part(5, 190.0, 440.0, 280.0, 40.0, 10.0));
    let mut list = col().key(format!("{h}/parts")).width(300.0).gap(0.0)
        .child(row().fill_width().pad(10.0, 8.0).child(text("零件表", 15.0, Paint::INK).font(Font::Display).grow(1.0)).child(text("数量", 12.0, Paint::DIM)))
        .child(space(1.0, 2.0).fill_width().fill(Paint::INK));
    for (i, (name, n)) in PARTS.iter().enumerate() {
        let on = hot(i);
        let mut r = row().key(format!("{h}/row{i}")).fill_width().height(34.0).pad(10.0, 0.0).gap(12.0).align(AlignItems::CENTER).on_press(Msg::Row(i)).focusable(false)
            .child(text(format!("{:02}", i + 1), 12.0, if on { Paint::GROUND } else { Paint::DIM }).font(Font::Mono))
            .child(text(*name, 14.0, if on { Paint::GROUND } else { Paint::TEXT }).grow(1.0))
            .child(text(format!("×{n}"), 12.0, if on { Paint::GROUND } else { Paint::DIM }).font(Font::Mono));
        if on { r = r.fill(Paint::ACCENT); }
        list = list.child(r).child(space(1.0, 1.0).fill_width().fill(Paint::FAINT));
    }
    list = list.child(text_lh("把指针移到一行上，图上对应的部件会标出来；反过来也一样。", 12.0, 20.0, Paint::DIM).key(format!("{h}/hint")));
    row().key(format!("{h}/comp")).gap(40.0).align(AlignItems::START).wrap()
        .child(col().gap(8.0).child(text("图 3　分解图（单位：毫米）", 12.0, Paint::DIM).font(Font::Mono)).child(diagram))
        .child(list)
}

// ── 8 Normcore: a plain settings form ────────────────────────────────────────────────────────────────
fn normcore(s: &Gallery, h: &str) -> El<Msg> {
    let field = |label: &str, el: El<Msg>| row().fill_width().gap(16.0).align(AlignItems::CENTER).child(row().width(120.0).child(text(label, 14.0, Paint::TEXT))).child(el);
    col().key(format!("{h}/comp")).width(560.0).gap(16.0)
        .child(text("设置", 26.0, Paint::INK).font(Font::Display))
        .child(text("通用", 12.0, Paint::DIM))
        .child(card(format!("{h}/general"), None, col().gap(12.0).fill_width()
            .child(field("名称", input(&format!("{h}/name"), &s.text, "我的电脑", 300.0, 40, Msg::Text)))
            .child(field("外观", segmented(&format!("{h}/look"), &["自动", "浅色", "深色"], s.choices[4].min(2), 28.0, |i| Msg::Choose(4, i))))
            .child(field("字号", number(&format!("{h}/size"), "", s.slides[3], Spec { min: 10.0, max: 22.0, step: 1.0, decimals: 0, unit: "pt" }, s.draft(Target::StyleNum(3)), |e| Msg::Num(Target::StyleNum(3), e)).width(300.0)))).width(560.0))
        .child(text("通知", 12.0, Paint::DIM))
        .child(card(format!("{h}/notes"), None, col().gap(8.0).fill_width()
            .child(switch(format!("{h}/updates"), "自动检查更新", s.toggles[4], None, Msg::Toggle(4)).fill_width())
            .child(switch(format!("{h}/alerts"), "显示通知", s.toggles[5], None, Msg::Toggle(5)).fill_width())).width(560.0))
        .child(row().fill_width().gap(10.0).justify(JustifyContent::END)
            .child(button_as(format!("{h}/cancel"), "取消", Variant::Default, Msg::Press))
            .child(button_as(format!("{h}/save"), "存储", Variant::Accent, Msg::Press)))
}

// ── 9 Cyberminimalism: one command line ──────────────────────────────────────────────────────────────
const COMMANDS: [(&str, &str); 8] = [
    ("layout.reset", "恢复默认布局"), ("layout.grid", "显示网格"), ("theme.dark", "切换到深色"), ("files.open", "打开文件"),
    ("files.recent", "最近的文件"), ("view.zoom", "缩放"), ("lab.ripple", "放出一道波前"), ("lab.gaze", "打开标注层"),
];
fn cyber(s: &Gallery, h: &str) -> El<Msg> {
    let q = s.query.text.to_lowercase();
    let hits: Vec<&(&str, &str)> = COMMANDS.iter().filter(|(c, d)| q.is_empty() || c.contains(&q) || d.contains(&q)).collect();
    let w = (s.width - if s.width < super::NARROW { 40.0 } else { super::RAIL + 100.0 }).clamp(280.0, 560.0);
    let mut results = col().key(format!("{h}/results")).width(w).gap(0.0);
    let sel = s.row.unwrap_or(0).min(hits.len().saturating_sub(1));
    for (i, (c, d)) in hits.iter().enumerate() {
        let on = sel == i;
        results = results.child(row().key(format!("{h}/hit{i}")).fill_width().height(34.0).pad(12.0, 0.0).gap(16.0).align(AlignItems::CENTER).on_press(Msg::Row(i)).focusable(false)
            .child(text(if on { "›" } else { " " }, 13.0, Paint::ACCENT).font(Font::Mono).width(10.0))
            .child(text(*c, 13.0, if on { Paint::ACCENT } else { Paint::INK }).font(Font::Mono).width(150.0))
            .child(text(*d, 13.0, Paint::DIM).grow(1.0)));
    }
    if hits.is_empty() { results = results.child(row().fill_width().height(34.0).pad(34.0, 0.0).align(AlignItems::CENTER).child(text("没有匹配的命令", 13.0, Paint::DIM))); }
    // Fixed height for the list, so the line above stays put while results come and go.
    let list = col().key(format!("{h}/list")).width(w).height(8.0 * 34.0).child(results);
    let picked = hits.get(sel).map_or("—".to_owned(), |(c, _)| (*c).to_owned());
    let height = (s.height - 210.0).max(560.0);
    col().key(format!("{h}/comp")).fill_width().height(height).align(AlignItems::CENTER).justify(JustifyContent::CENTER).gap(0.0).gaze_scope()
        .child(text("SESSION 2A · 本地", 11.0, Paint::DIM).font(Font::Mono).absolute(Some(0.0), Some(0.0), None, None))
        .child(text("23:58", 11.0, Paint::DIM).font(Font::Mono).absolute(None, Some(0.0), Some(0.0), None))
        .child(row().key(format!("{h}/line")).width(w).gap(12.0).align(AlignItems::CENTER)
            .child(text("›", 18.0, Paint::ACCENT).font(Font::Mono))
            .child(input(&format!("{h}/query"), &s.query, "输入命令", w - 30.0, 40, Msg::Query).border(0.0, Paint::GROUND)))
        .child(space(w, 1.0).fill(Paint::ACCENT).key(format!("{h}/rule")))
        .child(space(1.0, 10.0))
        .child(list)
        .child(row().key(format!("{h}/status")).fill_width().justify(JustifyContent::SPACE_BETWEEN).absolute(Some(0.0), None, Some(0.0), Some(0.0))
            .child(text(format!("{} 条结果　·　↑↓ 选择　·　回车执行", hits.len()), 11.0, Paint::DIM).font(Font::Mono))
            .child(text(format!("已选 {picked}"), 11.0, Paint::ACCENT).font(Font::Mono).key(format!("{h}/picked"))))
}

// ── 10 Bauhaus: a geometric poster with a control group ─────────────────────────────────────────────
fn bauhaus(s: &Gallery, h: &str) -> El<Msg> {
    let pick = s.choices[5].min(2);
    let big = |i: usize| if pick == i { 1.0 } else { 0.62 };
    let poster = col().key(format!("{h}/poster")).width(620.0).height(620.0)
        .child(space(620.0, 8.0).fill(Paint::INK).absolute(Some(0.0), Some(140.0), None, None))
        .child(space(8.0, 620.0).fill(Paint::INK).absolute(Some(400.0), Some(0.0), None, None))
        .child(space(300.0 * big(0), 300.0 * big(0)).radius(150.0 * big(0)).fill(Paint::ACCENT).key(format!("{h}/circle")).absolute(Some(60.0), Some(190.0), None, None))
        .child(space(200.0 * big(1), 200.0 * big(1)).fill(Paint::hue(1)).key(format!("{h}/square")).absolute(Some(410.0), Some(150.0), None, None))
        .child(space(210.0, 110.0 * big(2)).radius(55.0 * big(2)).fill(Paint::hue(2)).key(format!("{h}/bar")).absolute(Some(408.0), Some(470.0), None, None))
        .child(text_lh("形·色·线", 96.0, 110.0, Paint::INK).font(Font::Display).weight(2.5).key(format!("{h}/headline")).absolute(Some(20.0), Some(16.0), None, None))
        .child(text_lh("包豪斯 1919 — 1933", 15.0, 22.0, Paint::INK).font(Font::Mono).absolute(Some(20.0), Some(580.0), None, None));
    row().key(format!("{h}/comp")).gap(32.0).align(AlignItems::START).wrap()
        .child(poster)
        .child(col().key(format!("{h}/side")).width(260.0).gap(18.0)
            .child(text_lh("形式追随功能。", 24.0, 34.0, Paint::INK).font(Font::Display))
            .child(text_lh("选一个形状，它会变大。三种颜色、三种形状和一道粗网格，就是这张海报的全部。", 15.0, 26.0, Paint::TEXT))
            .child(segmented(&format!("{h}/shape"), &["圆", "方", "条"], pick, 32.0, |i| Msg::Choose(5, i)))
            .child(choice(&format!("{h}/weight"), "网格", &["细", "粗"], s.choices[6].min(1), |i| Msg::Choose(6, i)))
            .child(button_as(format!("{h}/print"), "印一张", Variant::Accent, Msg::Press)))
}

// ── 11 Neo-Y2K: chrome type, liquid blobs, a player ─────────────────────────────────────────────────
fn y2k(s: &Gallery, h: &str) -> El<Msg> {
    let gap = s.slides[0];
    let liquid = Material::Liquid.id();
    let pct = Spec { min: 0.0, max: 100.0, step: 1.0, decimals: 0, unit: "" };
    let near = 1.0 - gap;
    // Blobs whose distance follows the slider; close enough, the liquid joins them.
    let blob = |k: &str, d: f32, x: f32, y: f32, p: Paint| space(d, d).radius(d * 0.5).fill(p).material(liquid).key(format!("{h}/{k}")).absolute(Some(x), Some(y), None, None);
    let blobs = col().key(format!("{h}/blobs")).width(500.0).height(300.0)
        .child(blob("blob-a", 170.0, 10.0 + near * 110.0, 50.0, Paint::ACCENT))
        .child(blob("blob-b", 130.0, 330.0 - near * 110.0, 70.0, Paint::hue(1)))
        .child(blob("blob-c", 70.0, 220.0, 210.0 - near * 50.0, Paint::hue(2)))
        .child(blob("blob-d", 54.0, 420.0 - near * 60.0, 200.0 - near * 40.0, Paint::ACCENT));
    // A slider whose thumb is a drop of the liquid on a chrome-edged track.
    let v = s.slides[4];
    let liquid_slider = col().key(format!("{h}/drop")).width(300.0).height(60.0).on_drag(|x, _| Msg::Slide(4, x))
        .on_key(move |k, _| match k { crate::el::Key::Left => Some(Msg::Slide(4, v - 0.05)), crate::el::Key::Right => Some(Msg::Slide(4, v + 0.05)), _ => None })
        .child(space(300.0, 8.0).radius(4.0).fill(Paint::DIM).absolute(Some(0.0), Some(26.0), None, None))
        .child(space(52.0, 52.0).radius(26.0).fill(Paint::hue(2)).material(liquid).key(format!("{h}/drop-thumb")).absolute(Some(v * 248.0), Some(4.0), None, None));
    let pill = |k: &str, label: &str, m: Msg| row().key(format!("{h}/{k}")).height(52.0).pad(26.0, 0.0).radius(26.0).fill(Paint::INK).align(AlignItems::CENTER).on_press(m)
        .child(text(label, 16.0, Paint::INK).weight(0.6));
    let player = col().key(format!("{h}/player")).width(400.0).gap(16.0).pad(22.0, 22.0).radius(24.0).border(1.0, Paint::DIM)
        .child(row().gap(16.0).align(AlignItems::CENTER)
            // A chrome ring: a band wide enough to carry a bevel.
            .child(space(84.0, 84.0).radius(42.0).border(20.0, Paint::INK).key(format!("{h}/ring")))
            .child(col().gap(2.0).child(text("液态时间", 17.0, Paint::INK)).child(text("Y2K MIX · 04:12", 12.0, Paint::DIM).font(Font::Mono))))
        .child(slider(&format!("{h}/progress"), s.slides[1] * 100.0, pct, 356.0, |v| Msg::Slide(1, v / 100.0)))
        .child(row().gap(10.0).justify(JustifyContent::CENTER).fill_width()
            .child(pill("prev", "上一首", Msg::Press))
            .child(pill("play", if s.toggles[1] { "暂停" } else { "播放" }, Msg::Toggle(1)))
            .child(pill("next", "下一首", Msg::Press)))
        .child(row().gap(12.0).align(AlignItems::CENTER).child(text("音量", 13.0, Paint::DIM)).child(liquid_slider));
    col().key(format!("{h}/comp")).gap(14.0)
        .child(text_lh("液态金属", 150.0, 170.0, Paint::INK).font(Font::Display).weight(3.0).key(format!("{h}/headline")))
        .child(row().key(format!("{h}/lower")).gap(40.0).align(AlignItems::START).wrap()
            .child(col().gap(10.0).child(blobs)
                .child(row().gap(12.0).align(AlignItems::CENTER).child(text("距离", 13.0, Paint::DIM))
                    .child(slider(&format!("{h}/gap"), gap * 100.0, pct, 320.0, |v| Msg::Slide(0, v / 100.0)))))
            .child(player))
}

// ── 12 Mixed media: a collage ────────────────────────────────────────────────────────────────────────
fn mixed(s: &Gallery, h: &str) -> El<Msg> {
    let tab = s.tab.min(2);
    let panels = ["剪下来的东西都留着：车票、照片的一角、写坏的字。", "每一块用自己的材质：网点的照片，刻出来的标题，透明的标签。", "拼贴没有规矩，只要每一块都还认得出来。"];
    let note = |k: &str, t: &str, p: Paint, w: f32| col().key(format!("{h}/{k}")).width(w).pad(18.0, 16.0).fill(p).child(text_lh(t, 14.0, 23.0, Paint::INK));
    row().key(format!("{h}/comp")).gap(30.0).align(AlignItems::START).wrap()
        .child(col().key(format!("{h}/left")).width(460.0).gap(22.0)
            .child(col().key(format!("{h}/title-box")).pad(22.0, 14.0).fill(Paint::hue(2)).child(text_lh("剪贴簿", 88.0, 100.0, Paint::ACCENT).font(Font::Display).weight(2.0).key(format!("{h}/headline")).material(Material::Lino.id())))
            .child(note("note1", "十一月　旧报纸、照片和一张车票。", Paint::hue(2), 300.0))
            .child(tabs(&format!("{h}/tabs"), &["收集", "材质", "规矩"], tab, 18.0, 2.0, Msg::Tab, text_lh(panels[tab], 14.0, 24.0, Paint::TEXT).key(format!("{h}/panel{tab}"))).width(420.0)))
        .child(col().key(format!("{h}/right")).width(420.0).gap(22.0)
            .child(col().key(format!("{h}/photo-card")).pad(14.0, 14.0).fill(Paint::hue(2)).child(pic(h, "photo", BUST, 360.0)))
            .child(row().gap(18.0).align(AlignItems::START)
                .child(col().key(format!("{h}/label")).width(200.0).gap(6.0).pad(16.0, 14.0).radius(8.0).fill(Paint::hue(1)).material(Material::Xray.id())
                    .child(text("标签　NO. 12", 13.0, Paint::INK).font(Font::Mono)).child(text_lh("透明的贴纸，下面的东西透出来。", 13.0, 21.0, Paint::TEXT)))
                .child(space(120.0, 120.0).radius(60.0).fill(Paint::ACCENT).key(format!("{h}/stamp")).material(Material::Halftone.id()).loose(0.7))))
}

