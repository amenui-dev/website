//! One page per style preset (DESIGN.md §4): each a small composition of its own, made of the existing
//! components, with at least one real interaction. Opening a page applies its style.
use super::pages::ratio;
use super::{Gallery, Msg, STYLE_SLOT, Target};
use crate::demo::{SKY, SPHERE, TERRAIN, TREE};
use amenui::el::{El, Font, Paint, Scope, Stirr, col, picture, row, space, text, text_lh};
use amenui::style::styles;
use amenui::widgets::{Spec, Variant, button_as, card, choice, input, number, switch, segmented};
use taffy::{AlignItems, JustifyContent};

pub const STYLE_DESC: [&str; 10] = [
    "深色底，等宽字。标注层框出页面上的每个元素，标出编号、类型和尺寸，十字线跟着指针。",
    "黑底，琥珀色的灯。像素字，粗点距；松开时文字散成发光的点。",
    "米白的纸和一种专色。纸纹、套印错位，松开时像复印件一样起斑、洇墨。",
    "黑底和两种信号色。静止时干净；指针移到控件上，控件就短暂地错位。",
    "米色纸上的黑和红。粗重的标题像刻出来的，松开时边缘出现刀痕。",
    "白纸、黑线和一种信号色。部件带编号和引线，零件表的一行对应图上的一个部件。",
    "系统灰，默认字号。松开的幅度只有三成，几乎看不出材质。",
    "近黑的底，一条细的强调色。大片留白；只在指针下标出元素。",
    "三原色加黑，几何形和粗网格。松开时变成半色调网点。",
    "黑底、白线和一种强调色。静止时是干净的字；指针拨过的地方散成线，沿边写着小字的读数。",
];

pub fn page(s: &Gallery, i: usize) -> El<Msg> {
    let st = styles()[i];
    let h = format!("style-{}", st.hash);
    let (margin, measure) = preset_rules(i);
    let inset = if s.width < super::NARROW { margin.max(18.0) } else { 48.0 };
    let r = Room { narrow: s.width < super::NARROW, w: s.width - 2.0 * inset };
    let body = match i {
        0 => gaze_feed(s, &h, r),
        1 => dotmatrix(s, &h, r),
        2 => print_zine(s, &h, r),
        3 => glitch(s, &h, r),
        4 => linocut(s, &h, r),
        5 => manual(s, &h, r),
        6 => normcore(s, &h, r),
        7 => cyber(s, &h, r),
        8 => bauhaus(s, &h, r),
        _ => loose_thread(s, &h, r),
    };
    // A press anywhere on the composition sends its front over the whole composition (DESIGN.md §1.15):
    // pictures and shapes loosen, text and controls only as far as the legibility cap.
    // The caption is two lines: the style's name, a label, then its description, a sentence that is
    // read, no wider than the preset's measure: 6 px a character at 13 px (the body faces set a line of
    // these sentences at 6.1 to 6.3 px a character, so a line keeps inside the measure).
    // The page is held to its preset's rules (`preset::Rule`); the strips above it are the site's
    // chrome and keep the audit's own.
    col().key(format!("{h}/page")).fill_width().min_width(0.0).gap(26.0).pad(inset, 36.0).ripplr_scope().ripplr(Scope::Container(None)).audit_under(&presets()[i])
        .child(col().key(format!("{h}/caption")).fill_width().gap(4.0)
            .child(text(st.name, 13.0, Paint::DIM).font(Font::Mono).key(format!("{h}/name")))
            .child(text_lh(STYLE_DESC[i], 13.0, 20.0, Paint::TEXT).key(format!("{h}/desc")).fill_width().max_width(measure * 6.0)))
        .child(body)
}

/// The style presets (`preset::presets`), made once.
fn presets() -> &'static [amenui::preset::Preset] {
    static ALL: std::sync::OnceLock<Vec<amenui::preset::Preset>> = std::sync::OnceLock::new();
    ALL.get_or_init(amenui::preset::presets)
}

/// Each preset's side margin and measure (`preset::Rule::Margin`, `Rule::Measure`), by style.
fn preset_rules(i: usize) -> (f32, f32) {
    use amenui::preset::Rule;
    static RULES: std::sync::OnceLock<Vec<(f32, f32)>> = std::sync::OnceLock::new();
    let all = RULES.get_or_init(|| presets().iter().map(|p| p.rules.iter().fold((0.0, 80.0), |(m, w), r| match *r {
        Rule::Margin { min } => (min, w),
        Rule::Measure { max } => (m, max),
        _ => (m, w),
    })).collect());
    all.get(i).copied().unwrap_or((0.0, 80.0))
}

/// The width a style page is set in. Below `NARROW` it is the window less the page's side insets, and
/// the composition is set again in one column of that width: a row of columns becomes one column in
/// source order, a fixed width no wider than the room, pictures as wide as the room at their own ratio,
/// and nothing read by scrolling sideways.
#[derive(Clone, Copy)]
pub struct Room { pub narrow: bool, pub w: f32 }
impl Room {
    /// A fixed width `w`: below `NARROW` no wider than the room.
    fn of(self, w: f32) -> f32 { if self.narrow { w.min(self.w) } else { w } }
    /// A picture's or a column's width: the room below `NARROW`, else `w`.
    fn full(self, w: f32) -> f32 { if self.narrow { self.w } else { w } }
    /// A display line's size (`DISPLAY`): below `NARROW`, `fit` in the room.
    fn size(self, line: usize) -> f32 { let (_, _, size, wide, _) = DISPLAY[line]; if self.narrow { fit(size, wide, self.w) } else { size } }
}

/// The largest size, in steps of 2 px and no more than the designed `size`, at which a line whose widest
/// unbreakable piece is `wide` px at `size` fits `room`; 40 px at least.
pub fn fit(size: f32, wide: f32, room: f32) -> f32 {
    if wide <= room { return size; }
    ((size * room / wide / 2.0).floor() * 2.0).clamp(40.0_f32.min(size), size)
}

/// The pieces of a text that do not break: its words, a word without letters or digits (`&`, `·`)
/// kept with the one before it.
pub fn units(line: &str) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for w in line.split(char::is_whitespace).filter(|w| !w.is_empty()) {
        match out.last_mut() {
            Some(last) if !w.chars().any(char::is_alphanumeric) => { last.push(' '); last.push_str(w); }
            _ => out.push(w.to_owned()),
        }
    }
    out
}

/// The style pages' display lines: the style, the line as written, its designed size, the width at that
/// size in the style's display face, as shown (in English: `english.txt`), of what must not break, and
/// that run when it is longer than one unbreakable piece (`units`; empty: the widest piece).
/// `tests/styles.rs` measures them again.
pub const DISPLAY: [(usize, &str, f32, f32, &str); 10] = [
    (0, "视野里的一切都被看见", 34.0, 210.0, ""),
    (1, "出发", 72.0, 210.0, ""),
    (1, "到达", 72.0, 204.0, ""),
    // Two lines, `Spring in the` / `photocopier`: never `the` alone on a line.
    (2, "复印机里的春天", 64.0, 380.0, "Spring in the"),
    (3, "信号中断", 120.0, 348.0, ""),
    (4, "刀与木", 140.0, 463.0, ""),
    (6, "设置", 26.0, 99.0, ""),
    (8, "形·色·线", 96.0, 400.0, ""),
    (8, "形式追随功能。", 24.0, 101.0, ""),
    // Bauhaus below `NARROW`: one word a line.
    (8, "形\n色\n线", 96.0, 283.0, ""),
];
/// The departure board's LED clock in the style's mono face: `00:00` at 150 px and the seconds `00`
/// at 56 px, px wide (`tests/styles.rs` measures them again).
pub const LED: [(&str, f32, f32); 2] = [("00:00", 150.0, 490.0), ("00", 56.0, 70.0)];
/// The departure board's row in the style's faces, as shown: a time (mono, 22 px), the longest
/// destination (24 px) and a platform (mono, 22 px), with whether the face is mono, the size and the
/// width in px (`tests/styles.rs` measures them again).
pub const BOARD: [(&str, bool, f32, f32); 3] = [("08:55", true, 22.0, 70.0), ("广州 南", false, 24.0, 182.0), ("6", true, 22.0, 14.0)];

fn pic(h: &str, k: &str, id: u32, w: f32) -> El<Msg> { pic_at(h, k, id, w, 0.0) }
/// A picture in the style's marks, held at a looseness (1: always shown as marks).
fn pic_at(h: &str, k: &str, id: u32, w: f32, loose: f32) -> El<Msg> {
    col().key(format!("{h}/{k}-box")).width(w).child(picture(id, ratio(id)).key(format!("{h}/{k}")).markr(STYLE_SLOT).loose(loose))
}

// ── 1 Algorithmic / Digital Gaze: a feed, framed and measured ───────────────────────────────────────
fn gaze_feed(s: &Gallery, h: &str, r: Room) -> El<Msg> {
    const ITEMS: [(u32, &str, &str, f32); 6] = [
        (TREE, "风景", "IMG_0412", 0.93), (TERRAIN, "地形", "IMG_0418", 0.88), (SKY, "天空", "IMG_0421", 0.97),
        (SPHERE, "物体", "IMG_0433", 0.71), (TREE, "风景", "IMG_0440", 0.84), (TERRAIN, "地形", "IMG_0452", 0.66),
    ];
    let filter = s.choices[0].min(3);
    let names = ["全部", "风景", "地形", "天空"];
    let mut feed = row().key(format!("{h}/feed")).gap(22.0).wrap().fill_width();
    for (i, (id, tag, file, conf)) in ITEMS.iter().enumerate() {
        if filter != 0 && *tag != names[filter] { continue; }
        feed = feed.child(col().key(format!("{h}/item{i}")).width(r.full(250.0)).gap(8.0).on_press(Msg::Row(i)).focusable(false)
            .child(pic(h, &format!("p{i}"), *id, r.full(250.0)))
            .child(row().justify(JustifyContent::SPACE_BETWEEN).fill_width()
                .child(text(*file, 12.0, Paint::TEXT).font(Font::Mono))
                .child(text(format!("{tag} {conf:.2}"), 12.0, Paint::ACCENT).font(Font::Mono))));
    }
    // Below `NARROW` the heading is the column's width and wraps.
    let heading = col().gap(4.0).min_width(0.0).max_width(r.w)
        .child(text("FIELD 07 / 观察记录", 12.0, Paint::DIM).font(Font::Mono))
        .child(text_lh("视野里的一切都被看见", r.size(0), 44.0 * r.size(0) / 34.0, Paint::INK).font(Font::Display));
    let chips = segmented(&format!("{h}/filter"), &names, filter, 30.0, |i| Msg::Choose(0, i));
    // Below `NARROW` the filter chips go under the heading, where the annotation layer's labels would
    // cover their words: they are left out of the layer, and the heading and the feed are its scope.
    let (heading, feed) = if r.narrow { (heading.gaze_scope(), feed.gaze_scope()) } else { (heading, feed) };
    let comp = col().key(format!("{h}/comp")).fill_width().gap(24.0);
    if r.narrow { comp } else { comp.gaze_scope() }
        .child(row().key(format!("{h}/head")).fill_width().align(AlignItems::END).justify(JustifyContent::SPACE_BETWEEN).wrap().gap(16.0)
            .child(heading).child(chips))
        .child(feed)
}

// ── 2 Neo-Dotmatrix: a departure board ──────────────────────────────────────────────────────────────
// A live board: the clock, the LED panel's seconds and the ticker move every second on purpose
// (`.ongoing()`), so the audit does not count the frames they ask for as restless.
fn dotmatrix(s: &Gallery, h: &str, r: Room) -> El<Msg> {
    let t = 8.0 * 3600.0 + 42.0 * 60.0 + s.clock.floor();
    let clock = format!("{:02}:{:02}:{:02}", (t / 3600.0) as u32 % 24, (t / 60.0) as u32 % 60, t as u32 % 60);
    let arrive = s.choices[1] == 1;
    let rows: [(&str, &str, &str); 5] = if arrive {
        [("08:31", "上海 虹桥", "3"), ("08:40", "杭州 东", "5"), ("08:52", "南京 南", "1"), ("09:05", "苏州", "4"), ("09:18", "宁波", "2")]
    } else {
        [("08:47", "北京 南", "2"), ("08:55", "广州 南", "6"), ("09:03", "成都 东", "4"), ("09:12", "西安 北", "1"), ("09:26", "武汉", "7")]
    };
    let tick = s.clock.floor() as usize;
    let status = |i: usize| {
        // The status of one row changes every few seconds, as boards do.
        let late = (tick / 4 + i).is_multiple_of(5);
        (late, if late { format!("晚点 {} 分", 3 + (tick / 4 + i) % 7) } else { "正点".to_owned() })
    };
    let w = r.full(760.0);
    let mut board = col().key(format!("{h}/board")).width(w).gap(if r.narrow { 10.0 } else { 2.0 }).pad(22.0, 18.0).radius(4.0).border(2.0, Paint::HAIRLINE);
    if r.narrow {
        // Below `NARROW` an entry is two lines: the time, the destination and the platform at the right
        // edge, then the status under the destination; a hairline between entries. The type goes down
        // (2 px steps, 12 px at least) until the longest destination sets on one line.
        let ((_, _, time_at, time_w), (_, _, _, dest_w), (_, _, _, plat_w)) = (BOARD[0], BOARD[1], BOARD[2]);
        let (gap, inside) = (12.0, w - 44.0 - 4.0);
        let line = time_w + dest_w + plat_w;
        let f = if line + 2.0 * gap <= inside { 1.0 } else { ((24.0 * (inside - 2.0 * gap) / line / 2.0).floor() * 2.0).max(12.0) / 24.0 };
        let tw = (time_w * f).ceil();
        board = board.child(row().key(format!("{h}/cols")).fill_width().gap(gap)
            .child(text("时间", 12.0, Paint::DIM).width(tw)).child(text("目的地", 12.0, Paint::DIM).grow(1.0)).child(text("站台", 12.0, Paint::DIM)));
        for (i, (time, dest, plat)) in rows.iter().enumerate() {
            let (late, st) = status(i);
            if i > 0 { board = board.child(space(1.0, 1.0).fill_width().fill(Paint::HAIRLINE)); }
            board = board.child(col().key(format!("{h}/row{i}")).fill_width().gap(2.0)
                .child(row().fill_width().gap(gap).align(AlignItems::CENTER)
                    .child(text(*time, time_at * f, Paint::ACCENT).font(Font::Mono).width(tw))
                    .child(text(*dest, 24.0 * f, Paint::ACCENT).grow(1.0))
                    .child(text(*plat, time_at * f, Paint::ACCENT).font(Font::Mono)))
                .child(row().child(space(tw + gap, 1.0))
                    .child(text(st, 24.0 * f, if late { Paint::hue(1) } else { Paint::ACCENT }).key(format!("{h}/status{i}-{late}")))));
        }
    } else {
        board = board.child(row().key(format!("{h}/cols")).fill_width().gap(18.0).pad(0.0, 6.0)
            .child(text("时间", 12.0, Paint::DIM).width(90.0)).child(text("目的地", 12.0, Paint::DIM).grow(1.0))
            .child(text("站台", 12.0, Paint::DIM).width(60.0)).child(text("状态", 12.0, Paint::DIM).width(120.0)));
        for (i, (time, dest, plat)) in rows.iter().enumerate() {
            let (late, st) = status(i);
            board = board.child(row().key(format!("{h}/row{i}")).fill_width().height(36.0).gap(18.0).align(AlignItems::CENTER)
                .child(text(*time, 22.0, Paint::ACCENT).font(Font::Mono).width(90.0))
                .child(text(*dest, 24.0, Paint::ACCENT).grow(1.0))
                .child(text(*plat, 22.0, Paint::ACCENT).font(Font::Mono).width(60.0))
                .child(text(st, 24.0, if late { Paint::hue(1) } else { Paint::ACCENT }).width(120.0).key(format!("{h}/status{i}-{late}"))));
        }
    }
    col().key(format!("{h}/comp")).gap(26.0)
        .child(row().key(format!("{h}/sign")).gap(30.0).align(AlignItems::END).wrap()
            .child(text_lh(if arrive { "到达" } else { "出发" }, r.size(1 + usize::from(arrive)), 80.0 * r.size(1 + usize::from(arrive)) / 72.0, Paint::ACCENT).font(Font::Display).key(format!("{h}/title-{arrive}")))
            .child(text(clock, 48.0, Paint::ACCENT).font(Font::Mono).print(4.0).key(format!("{h}/clock")).ongoing()))
        .child(board)
        .child(led_panel(s, h, t, r))
        .child(row().key(format!("{h}/controls")).gap(12.0).align(AlignItems::CENTER).wrap()
            .child(segmented(&format!("{h}/mode"), &["出发", "到达"], s.choices[1].min(1), 30.0, |i| Msg::Choose(1, i)))
            .child(button_as(format!("{h}/refresh"), "刷新", Variant::Default, Msg::Press))
            .child(text("站台", 12.0, Paint::DIM))
            .child(number(&format!("{h}/platform"), "", s.slides[5], Spec { min: 1.0, max: 9.0, step: 1.0, decimals: 0, unit: "" }, s.draft(Target::StyleNum(5)), |e| Msg::Num(Target::StyleNum(5), e)).width(130.0))
            .child(text("时差", 12.0, Paint::DIM))
            .child(number(&format!("{h}/offset"), "", s.slides[6], Spec { min: -720.0, max: 720.0, step: 1.0, decimals: 0, unit: "分" }, s.draft(Target::StyleNum(6)), |e| Msg::Num(Target::StyleNum(6), e)).width(150.0)))
}

/// The second panel: a big clock printed in dots, and a ticker that runs along under it.
fn led_panel(s: &Gallery, h: &str, t: f32, r: Room) -> El<Msg> {
    let t = t + s.slides[6] * 60.0;
    let big = format!("{:02}:{:02}", (t / 3600.0).rem_euclid(24.0) as u32, (t / 60.0).rem_euclid(60.0) as u32);
    let secs = format!("{:02}", t.rem_euclid(60.0) as u32);
    let plat = s.slides[5].round().clamp(1.0, 9.0) as u32;
    let msg = format!("Platform {plat}  ·  please wait behind the yellow line  ·  the train is arriving, please take care  ·  next 08:47 Beijing South  ·  ");
    // The ticker moves a character's width every second: the text is the view, the clock moves it.
    let step = (s.clock.floor() as usize) % 40;
    let shown: String = msg.chars().chain(msg.chars()).skip(step).take(60).collect();
    // Below `NARROW` the panel is the room's width, and the clock and its seconds are set smaller
    // together (2 px steps) until both fit inside it, their dot pitch with them, 4 px at least.
    let w = r.full(760.0);
    let ((_, big_at, big_w), (_, secs_at, secs_w)) = (LED[0], LED[1]);
    // At clock size `z` the two are (big_w + secs_w) × z / big_at wide and 10 px apart, inside the
    // panel's 22 px padding.
    let big_size = if r.narrow { fit(big_at, big_w + secs_w, w - 44.0 - 10.0) } else { big_at };
    let secs_size = if r.narrow { ((secs_at * big_size / big_at / 2.0).floor() * 2.0).max(4.0) } else { secs_at };
    col().key(format!("{h}/panel2")).width(w).gap(10.0).pad(22.0, 18.0).radius(4.0).border(2.0, Paint::HAIRLINE)
        .child(row().fill_width().justify(JustifyContent::SPACE_BETWEEN)
            .child(text(format!("{plat} 站台　·　本站时间"), 12.0, Paint::DIM))
            .child(text("NEXT 08:47", 12.0, Paint::DIM).font(Font::Mono)))
        .child(row().gap(10.0).align(AlignItems::END)
            .child(text_lh(big, big_size, 156.0 * big_size / big_at, Paint::ACCENT).font(Font::Mono).print((7.0 * big_size / big_at).max(4.0)).key(format!("{h}/bigclock")))
            .child(text_lh(secs, secs_size, 64.0 * secs_size / secs_at, Paint::hue(1)).font(Font::Mono).print((4.0 * secs_size / secs_at).max(4.0)).key(format!("{h}/secs")).ongoing()))
        .child(space(1.0, 2.0).fill_width().fill(Paint::HAIRLINE))
        // The ticker: one long line, clipped to the panel, moved one character a second.
        .child(col().key(format!("{h}/ticker")).fill_width().height(40.0).clip()
            .child(text_lh(shown, 24.0, 36.0, Paint::ACCENT).width(3000.0).key(format!("{h}/ticker-text")).ongoing().absolute(Some(0.0), Some(2.0), None, None)))
}

// ── 3 Lo-Fi Print: a zine page ───────────────────────────────────────────────────────────────────────
fn print_zine(s: &Gallery, h: &str, r: Room) -> El<Msg> {
    let issue = s.choices[2].min(2);
    let issues = ["第 5 期", "第 6 期", "第 7 期"];
    let para = |k: &str, t: &str| text_lh(t, 15.0, 27.0, Paint::TEXT).key(format!("{h}/{k}"));
    // Below `NARROW` the spread is one column: the headline, the byline, the paragraphs and the
    // controls, then the photo and its caption.
    let spread = if r.narrow { col().gap(30.0) } else { row().gap(36.0) };
    col().key(format!("{h}/comp")).width(r.of(900.0)).gap(0.0)
        .child(row().key(format!("{h}/mast")).fill_width().height(44.0).pad(18.0, 0.0).fill(Paint::INK).align(AlignItems::CENTER).justify(JustifyContent::SPACE_BETWEEN)
            .child(text("慢 读 · LO-FI ZINE", 15.0, Paint::GROUND).font(Font::Mono))
            .child(text(issues[issue], 15.0, Paint::GROUND)))
        .child(spread.key(format!("{h}/spread")).fill_width().pad(0.0, 30.0).align(AlignItems::START)
            .child(col().key(format!("{h}/left")).width(r.of(470.0)).gap(18.0)
                .child(text_lh("复印机里的春天", r.size(3), 72.0 * r.size(3) / 64.0, Paint::ACCENT).font(Font::Display).weight(1.5).key(format!("{h}/headline")))
                .child(text("文 / 编辑部　图 / 读者来稿", 13.0, Paint::DIM))
                .child(para("p1", "这一期收的是读者寄来的照片和短文。我们把它们放大、复印、再复印，直到颗粒和墨点都成了画面的一部分。"))
                .child(para("p2", "纸张偏黄，粉色的专色和黑色分两次印，所以总有一点对不准。那一点错位，是手工的证明。"))
                .child(row().gap(12.0).align(AlignItems::CENTER).wrap()
                    .child(button_as(format!("{h}/subscribe"), "订阅", Variant::Primary, Msg::Press))
                    .child(segmented(&format!("{h}/issue"), &issues, issue, 30.0, |i| Msg::Choose(2, i)))))
            .child(col().key(format!("{h}/right")).width(r.full(390.0)).gap(12.0)
                .child(pic_at(h, "photo", if issue == 1 { SKY } else { TREE }, r.full(390.0), 1.0))
                .child(row().fill_width().gap(10.0).align(AlignItems::CENTER)
                    .child(space(28.0, 28.0).radius(14.0).fill(Paint::ACCENT))
                    .child(text_lh("图注：孤树，复印三次。", 13.0, 20.0, Paint::DIM)))))
        .child(space(1.0, 3.0).fill_width().fill(Paint::INK))
}

// ── 4 Glitch: a title card that bursts ───────────────────────────────────────────────────────────────
fn glitch(s: &Gallery, h: &str, r: Room) -> El<Msg> {
    col().key(format!("{h}/comp")).fill_width().gap(30.0).pad(0.0, 30.0)
        .child(col().key(format!("{h}/card")).gap(10.0)
            .child(text("ERR 0x3F　连接已中断", 14.0, Paint::hue(1)).font(Font::Mono))
            // Pressable, so the pointer coming onto it fires a burst over it.
            .child(text_lh("信号中断", r.size(4), 132.0 * r.size(4) / 120.0, Paint::INK).font(Font::Display).weight(3.0).key(format!("{h}/title")).on_press(Msg::Press).focusable(false))
            .child(text_lh("画面会在你经过时短暂地错位。静止时，它就是一张干净的页面。", 16.0, 28.0, Paint::DIM)))
        .child(row().key(format!("{h}/bar")).fill_width().height(2.0).fill(Paint::ACCENT))
        .child(row().key(format!("{h}/controls")).gap(12.0).align(AlignItems::CENTER).wrap()
            .child(button_as(format!("{h}/retry"), "重新连接", Variant::Primary, Msg::Press))
            .child(button_as(format!("{h}/skip"), "跳过", Variant::Default, Msg::Press))
            .child(button_as(format!("{h}/report"), "发送报告", Variant::Quiet, Msg::Press))
            .child(switch(format!("{h}/auto"), "自动重试", s.toggles[3], None, Msg::Toggle(3)).width(200.0))
            .child(text(format!("已重试 {} 次", s.presses), 13.0, Paint::DIM).font(Font::Mono).key(format!("{h}/count"))))
        .child(glitch_monitor(s, h, r))
}

/// Below the card: three channels, each a picture and a signal level, that tear when the pointer
/// comes onto them.
fn glitch_monitor(s: &Gallery, h: &str, r: Room) -> El<Msg> {
    let chans = [(SKY, "CH 01", 0.82), (TERRAIN, "CH 02", 0.41), (TREE, "CH 03", 0.0)];
    let mut mon = row().key(format!("{h}/monitor")).gap(20.0).wrap();
    for (i, (id, name, level)) in chans.iter().enumerate() {
        let lost = *level == 0.0 && s.presses == 0;
        let bars = (0..12).fold(row().gap(3.0).align(AlignItems::END).height(22.0), |r, b| {
            let on = (b as f32) < level * 12.0 || (!lost && *level == 0.0 && b < 9);
            r.child(space(6.0, 6.0 + b as f32 * 1.4).fill(if on { Paint::ACCENT } else { Paint::HAIRLINE }))
        });
        mon = mon.child(col().key(format!("{h}/ch{i}")).width(r.full(260.0)).gap(8.0).on_press(Msg::Press).focusable(false)
            .child(pic(h, &format!("ch{i}-pic"), *id, r.full(260.0)))
            .child(row().fill_width().justify(JustifyContent::SPACE_BETWEEN).align(AlignItems::CENTER)
                .child(text(*name, 12.0, Paint::TEXT).font(Font::Mono))
                .child(text(if lost { "无信号" } else { "已连接" }, 12.0, if lost { Paint::hue(0) } else { Paint::hue(1) }))
                .child(bars)));
    }
    mon
}

// ── 5 Metamorphosis Linocut: a poster ────────────────────────────────────────────────────────────────
fn linocut(s: &Gallery, h: &str, r: Room) -> El<Msg> {
    let shown = [TREE, TERRAIN, SKY][s.choices[3].min(2)];
    // Below `NARROW` the headline is set by `fit` and the space above it scales with it; the sun scales
    // with the column, so it stays inside it.
    let z = r.size(5);
    let k = z / 140.0;
    let sun = r.full(520.0) / 520.0;
    row().key(format!("{h}/comp")).gap(44.0).align(AlignItems::START).wrap()
        .child(col().key(format!("{h}/left")).width(r.full(520.0)).gap(20.0)
            // The headline in the flow, 130 px under the top with the sun behind it: the hero is as tall
            // as its lines: two in English, "Knife &" / "wood" (`assets/english.txt`).
            .child(col().key(format!("{h}/hero")).width(r.full(520.0))
                .child(space(190.0 * sun, 190.0 * sun).radius(95.0 * sun).fill(Paint::ACCENT).loose(0.6).key(format!("{h}/sun")).absolute(Some(320.0 * sun), Some(0.0), None, None))
                .child(space(1.0, 130.0 * k))
                .child(text_lh("刀与木", z, 150.0 * k, Paint::INK).font(Font::Display).weight(3.0).loose(0.45).key(format!("{h}/headline"))))
            .child(text_lh("每一道线都是从板上挖掉的。留下的才是墨。", 18.0, 30.0, Paint::INK))
            .child(text_lh("版画展　十一月二十日至十二月十日　每天九点至五点", 14.0, 24.0, Paint::TEXT))
            .child(row().gap(12.0).align(AlignItems::CENTER)
                .child(button_as(format!("{h}/next"), "换一张", Variant::Primary, Msg::Choose(3, (s.choices[3] + 1) % 3)))
                .child(text(format!("第 {} 张，共 3 张", s.choices[3] % 3 + 1), 13.0, Paint::DIM))))
        .child(col().key(format!("{h}/right")).width(r.full(380.0)).gap(10.0)
            // The print itself is held loose: cut in strokes that follow the form.
            .child(col().key(format!("{h}/print-box")).width(r.full(380.0)).child(picture(shown, ratio(shown)).key(format!("{h}/print")).markr(STYLE_SLOT).loose(1.0)))
            .child(text("木版　34 × 28 厘米", 12.0, Paint::DIM)))
}

// ── 6 Tech-Manual: an exploded diagram and its parts list ───────────────────────────────────────────
const PARTS: [(&str, &str); 6] = [("上盖", "1"), ("面板", "1"), ("旋钮", "2"), ("电路板", "1"), ("电池仓", "1"), ("底座", "1")];
fn manual(s: &Gallery, h: &str, r: Room) -> El<Msg> {
    let hot = |i: usize| s.hovered.as_deref() == Some(format!("{h}/part{i}").as_str()) || s.hovered.as_deref() == Some(format!("{h}/row{i}").as_str());
    // Below `NARROW` the drawing is the room's width at its own ratio: every part scales with it.
    let k = r.of(600.0) / 600.0;
    // The annotation layer's labels are the parts list's names as shown, in English (`english.txt`).
    let names: Vec<String> = PARTS.iter().map(|(n, _)| super::english::english(n)).collect();
    let part = |i: usize, l: f32, t: f32, w: f32, hh: f32, rad: f32| {
        let on = hot(i);
        let (l, t, w, hh, rad) = (l * k, t * k, w * k, hh * k, rad * k);
        space(w, hh).radius(rad).fill(if on { Paint::ACCENT.alpha(0.85) } else { Paint::tone(0.06) }).border(1.0, if on { Paint::ACCENT } else { Paint::INK })
            .key(format!("{h}/part{i}")).annotate(names[i].clone()).on_press(Msg::Row(i)).focusable(false).absolute(Some(l), Some(t), None, None)
    };
    // The parts stacked along a diagonal, pulled apart.
    let diagram = col().key(format!("{h}/diagram")).width(600.0 * k).height(520.0 * k).gaze_scope()
        .child(part(0, 210.0, 40.0, 240.0, 46.0, 8.0))
        .child(part(1, 230.0, 120.0, 200.0, 70.0, 4.0))
        .child(part(2, 270.0, 215.0, 36.0, 36.0, 18.0))
        .child(space(1.0, 300.0 * k).fill(Paint::DIM).absolute(Some(330.0 * k), Some(70.0 * k), None, None))
        .child(part(3, 220.0, 280.0, 220.0, 54.0, 2.0))
        .child(part(4, 250.0, 360.0, 160.0, 44.0, 6.0))
        .child(part(5, 190.0, 440.0, 280.0, 40.0, 10.0));
    let mut list = col().key(format!("{h}/parts")).width(r.of(300.0)).gap(0.0)
        .child(row().fill_width().pad(10.0, 8.0).child(text("零件表", 15.0, Paint::INK).font(Font::Display).grow(1.0)).child(text("数量", 12.0, Paint::DIM)))
        .child(space(1.0, 2.0).fill_width().fill(Paint::INK));
    for (i, (name, n)) in PARTS.iter().enumerate() {
        let on = hot(i);
        let mut r = row().key(format!("{h}/row{i}")).fill_width().height(34.0).pad(10.0, 0.0).gap(12.0).align(AlignItems::CENTER).on_press(Msg::Row(i)).focusable(false)
            .child(text(format!("{:02}", i + 1), 12.0, if on { Paint::GROUND } else { Paint::DIM }).font(Font::Mono))
            .child(text(*name, 14.0, if on { Paint::GROUND } else { Paint::TEXT }).grow(1.0))
            .child(text(format!("×{n}"), 12.0, if on { Paint::GROUND } else { Paint::DIM }).font(Font::Mono));
        if on { r = r.fill(Paint::ACCENT); }
        list = list.child(r).child(space(1.0, 1.0).fill_width().fill(Paint::HAIRLINE));
    }
    list = list.child(text_lh("把指针移到一行上，图上对应的部件会标出来；反过来也一样。", 12.0, 20.0, Paint::TEXT).key(format!("{h}/hint")));
    row().key(format!("{h}/comp")).gap(40.0).align(AlignItems::START).wrap()
        .child(col().gap(8.0).child(text("图 3　分解图（单位：毫米）", 12.0, Paint::DIM).font(Font::Mono)).child(diagram))
        .child(list)
}

// ── 7 Normcore: a plain settings form ────────────────────────────────────────────────────────────────
fn normcore(s: &Gallery, h: &str, r: Room) -> El<Msg> {
    // Below `NARROW` a field's label goes above its control.
    let field = |label: &str, el: El<Msg>| if r.narrow { col().fill_width().gap(6.0).child(text(label, 14.0, Paint::TEXT)).child(el) }
        else { row().fill_width().gap(16.0).align(AlignItems::CENTER).child(row().width(120.0).child(text(label, 14.0, Paint::TEXT))).child(el) };
    let w = r.of(560.0);
    col().key(format!("{h}/comp")).width(w).gap(16.0)
        .child(text("设置", r.size(6), Paint::INK).font(Font::Display))
        .child(text("通用", 12.0, Paint::DIM))
        .child(card(format!("{h}/general"), None, col().gap(12.0).fill_width()
            .child(field("名称", input(&format!("{h}/general/name"), &s.text, "我的电脑", r.of(300.0), 40, Msg::Text)))
            .child(field("外观", segmented(&format!("{h}/look"), &["自动", "浅色", "深色"], s.choices[4].min(2), 28.0, |i| Msg::Choose(4, i))))
            .child(field("字号", {
                // The control without a label of its own: at 300 px beside the labels, below `NARROW` as
                // wide as its steps and field, from the card's left edge as the other controls are.
                let size = number(&format!("{h}/size"), "", s.slides[3], Spec { min: 10.0, max: 22.0, step: 1.0, decimals: 0, unit: "pt" }, s.draft(Target::StyleNum(3)), |e| Msg::Num(Target::StyleNum(3), e));
                if r.narrow { row().child(size) } else { size.width(300.0) }
            }))).width(w))
        .child(text("通知", 12.0, Paint::DIM))
        .child(card(format!("{h}/notes"), None, col().gap(8.0).fill_width()
            .child(switch(format!("{h}/updates"), "自动检查更新", s.toggles[4], None, Msg::Toggle(4)).fill_width())
            .child(switch(format!("{h}/alerts"), "显示通知", s.toggles[5], None, Msg::Toggle(5)).fill_width())).width(w))
        .child(row().fill_width().gap(10.0).justify(JustifyContent::END)
            .child(button_as(format!("{h}/cancel"), "取消", Variant::Default, Msg::Press))
            .child(button_as(format!("{h}/save"), "保存", Variant::Primary, Msg::Press)))
}

// ── 8 Cyberminimalism: one command line ──────────────────────────────────────────────────────────────
const COMMANDS: [(&str, &str); 8] = [
    ("layout.reset", "恢复默认布局"), ("layout.grid", "显示网格"), ("theme.dark", "切换到深色"), ("files.open", "打开文件"),
    ("files.recent", "最近的文件"), ("view.zoom", "缩放"), ("lab.ripple", "放出一道波前"), ("lab.gaze", "打开标注层"),
];
fn cyber(s: &Gallery, h: &str, r: Room) -> El<Msg> {
    let q = s.query.text.to_lowercase();
    let hits: Vec<&(&str, &str)> = COMMANDS.iter().filter(|(c, d)| q.is_empty() || c.contains(&q) || d.contains(&q)).collect();
    let w = r.of((s.width - if r.narrow { 40.0 } else { super::side(s) + 100.0 }).clamp(280.0, 560.0));
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
        // Below `NARROW` the status line wraps inside the results' width: what is picked goes under the count.
        .child(if r.narrow { row().width(w).gap(4.0).wrap().absolute(Some(0.0), None, None, Some(0.0)) } else { row().fill_width().absolute(Some(0.0), None, Some(0.0), Some(0.0)) }
            .key(format!("{h}/status")).justify(JustifyContent::SPACE_BETWEEN)
            .child(text(format!("{} 条结果　·　↑↓ 选择　·　回车执行", hits.len()), 11.0, Paint::DIM).font(Font::Mono))
            .child(text(format!("已选 {picked}"), 11.0, Paint::ACCENT).font(Font::Mono).key(format!("{h}/picked"))))
}

// ── 9 Bauhaus: a geometric poster with a control group ─────────────────────────────────────────────
/// Below `NARROW`, the Bauhaus grid's horizontal bar's drop under the headline's first line, and the
/// vertical bar's distance from its widest line, in headline sizes.
const GRID_DROP: f32 = 0.112;
const GRID_CLEAR: f32 = 0.1;
fn bauhaus(s: &Gallery, h: &str, r: Room) -> El<Msg> {
    let pick = s.choices[5].min(2);
    let big = |i: usize| if pick == i { 1.0 } else { 0.62 };
    // Below `NARROW` the poster is the room's width at its own ratio: its shapes and grid scale with it,
    // and the headline is set by `fit` in the width beside its offset, one word a line.
    let k = r.of(620.0) / 620.0;
    let (lines, at) = (DISPLAY[7], 20.0 * k);
    let z = if r.narrow { fit(lines.2, lines.3, 620.0 * k - at) } else { lines.2 };
    let lh = 110.0 * z / 96.0;
    // The grid: as designed, or below `NARROW` placed by the headline, so that the horizontal bar lies
    // between its first and second lines and the vertical bar clears its widest line. The square and the
    // bar to the right of the vertical one keep their places beside it, inside the poster.
    let (gy, gx) = if r.narrow { (16.0 * k + lh + GRID_DROP * z, at + DISPLAY[9].3 * z / 96.0 + GRID_CLEAR * z) } else { (140.0, 400.0) };
    let right = 620.0 * k - gx - 10.0 * k;
    let sq = (200.0 * big(1) * k).min(right);
    let poster = col().key(format!("{h}/poster")).width(620.0 * k).height(620.0 * k)
        .child(space(620.0 * k, 8.0 * k).fill(Paint::INK).absolute(Some(0.0), Some(gy * if r.narrow { 1.0 } else { k }), None, None))
        .child(space(8.0 * k, 620.0 * k).fill(Paint::INK).absolute(Some(gx * if r.narrow { 1.0 } else { k }), Some(0.0), None, None))
        .child(space(300.0 * big(0) * k, 300.0 * big(0) * k).radius(150.0 * big(0) * k).fill(Paint::ACCENT).key(format!("{h}/circle")).absolute(Some(60.0 * k), Some(190.0 * k), None, None))
        .child(space(sq, sq).fill(Paint::hue(1)).key(format!("{h}/square")).absolute(Some(if r.narrow { gx + 10.0 * k } else { 410.0 * k }), Some(if r.narrow { gy + 10.0 * k } else { 150.0 * k }), None, None))
        .child(space(if r.narrow { right + 2.0 * k } else { 210.0 * k }, 110.0 * big(2) * k).radius(55.0 * big(2) * k).fill(Paint::hue(2)).key(format!("{h}/bar")).absolute(Some(if r.narrow { gx + 8.0 * k } else { 408.0 * k }), Some(470.0 * k), None, None))
        .child(text_lh(if r.narrow { "形\n色\n线" } else { "形·色·线" }, z, lh, Paint::INK).font(Font::Display).weight(2.5).key(format!("{h}/headline")).absolute(Some(at), Some(16.0 * k), None, None))
        .child(text_lh("包豪斯 1919 — 1933", 15.0, 22.0, Paint::INK).font(Font::Mono).absolute(Some(at), Some(620.0 * k - 40.0), None, None));
    row().key(format!("{h}/comp")).gap(32.0).align(AlignItems::START).wrap()
        .child(poster)
        .child(col().key(format!("{h}/side")).width(r.full(260.0)).gap(18.0)
            .child(text_lh("形式追随功能。", r.size(8), 34.0 * r.size(8) / 24.0, Paint::INK).font(Font::Display))
            .child(text_lh("选一个形状，它会变大。三种颜色、三种形状和一道粗网格，就是这张海报的全部。", 15.0, 26.0, Paint::TEXT))
            .child(segmented(&format!("{h}/shape"), &["圆", "方", "条"], pick, 32.0, |i| Msg::Choose(5, i)))
            .child(choice(&format!("{h}/weight"), "网格", &["细", "粗"], s.choices[6].min(1), |i| Msg::Choose(6, i)))
            .child(button_as(format!("{h}/print"), "印一张", Variant::Primary, Msg::Press)))
}

// ── 10 Loose Thread: a quiet page that frays where the pointer passes ──────────────────────────────
/// Whether style page `i` is Loose Thread.
pub fn is_loose_thread(i: usize) -> bool { i == 9 }
/// How long the page's tangle takes to draw on when the page is entered (s; the preset's transition).
const THREAD_DRAW_S: f32 = 0.7;
/// Whether the tangle is still drawing on at `t`.
pub fn drawing(s: &Gallery, t: f32) -> bool { s.thread_drawn.is_some_and(|d| t - d < THREAD_DRAW_S + 0.05) }
/// A tangle made for a radius and the lengths above and below its knot (their bits) and its points.
type Made = Option<((u32, u32, u32), std::rc::Rc<Vec<[f32; 2]>>)>;
/// The tangle (a knot and one long strand, DESIGN.md §4.4), made once.
fn tangle(r: f32, above: f32, below: f32) -> std::rc::Rc<Vec<[f32; 2]>> {
    thread_local! { static MADE: std::cell::RefCell<Made> = const { std::cell::RefCell::new(None) }; }
    let key = (r.to_bits(), above.to_bits(), below.to_bits());
    MADE.with(|m| {
        let mut m = m.borrow_mut();
        if let Some((k, p)) = m.as_ref() && *k == key { return p.clone(); }
        let p = std::rc::Rc::new(amenui::strand::figure(17, r, above, below));
        *m = Some((key, p.clone()));
        p
    })
}
fn loose_thread(s: &Gallery, h: &str, r: Room) -> El<Msg> {
    use amenui::el::Role;
    use amenui::style::{THREAD, THREAD_PAPER, theme_index};
    // English whatever the language (the demo pages show no Chinese).
    let en = |k: &str| amenui::i18n::Locale::En.tr(k);
    let narrow = s.width < super::NARROW;
    let w = if narrow { r.w } else { 640.0 };
    let (pic_w, side_w, height) = if narrow { (w, w, 760.0) } else { (300.0, 260.0, 480.0) };
    // The pointer stirs the field map where it passes: a point under it that stirs as it moves.
    let stylus = s.thread_at.map(|(x, y)| space(1.0, 1.0).key(format!("{h}/stylus")).stirr(Stirr { energy: 1.0, radius: Some(72.0), scope: Scope::Container(Some(format!("{h}/comp"))) }).absolute(Some(x), Some(y), None, None));
    // Its own scope for ripples, the map and the threads (they stop at its edge).
    // The 72 px on the right are the strand's (drawn only at full width).
    let mut comp = col().key(format!("{h}/comp")).width(if narrow { w } else { w + 72.0 }).height(height).gap(0.0).gaze_scope().ripplr_scope().on_move(|p| Some(Msg::ThreadAt(p)));
    if !narrow { comp.style.padding.right = taffy::prelude::length(72.0); }
    // The tangle: content, drawn under any material, its knot under the picture's bottom right corner
    // (behind it) and its strand from the top of the page to just above the footer, clear of the text to read.
    if !narrow {
        let r = 70.0;
        let progress = s.thread_drawn.map_or(1.0, |d| ((s.clock - d) / THREAD_DRAW_S).clamp(0.0, 1.0));
        let (cx, cy) = (pic_w + 20.0, 0.66 * height);
        let pts: Vec<[f32; 2]> = tangle(r, cy - 4.0, height - cy - 34.0).iter().map(|p| [p[0] + 90.0, p[1] + cy]).collect();
        comp = comp.child(space(180.0, height).key(format!("{h}/tangle")).trace(amenui::strand::Trace { points: std::rc::Rc::new(pts), progress, weight: 1.0, pull: 2.5, ..Default::default() }).absolute(Some(cx - 90.0), Some(0.0), None, None));
    }
    if let Some(st) = stylus { comp = comp.child(st); }
    let paper = s.theme == theme_index(THREAD_PAPER.name);
    // The footer's controls: small quiet type, the one that is on in the accent; no boxes.
    let word = |t: String, on: bool| text(t, 12.0, if on { Paint::ACCENT } else { Paint::TEXT }).whisper();
    let grounds = [THREAD.name, THREAD_PAPER.name];
    let pick = usize::from(paper);
    let keys = move |k: amenui::el::Key, _: bool| -> Option<Msg> {
        use amenui::el::Key;
        let to = match k { Key::Left | Key::Up | Key::Home => 0, Key::Right | Key::Down | Key::End => 1, _ => return None };
        (to != pick).then(|| Msg::Theme(theme_index(grounds[to])))
    };
    let ground = row().key(format!("{h}/ground")).gap(8.0).align(AlignItems::BASELINE).on_key(keys).role(Role::RadioGroup).label(en("thread.ground"))
        .child(row().key(format!("{h}/ground:0")).on_press(Msg::Theme(theme_index(THREAD.name))).focusable(false).role(Role::Radio).selected(!paper).label(en("thread.black")).child(word(en("thread.black"), !paper)))
        .child(text("·", 12.0, Paint::TEXT).whisper())
        .child(row().key(format!("{h}/ground:1")).on_press(Msg::Theme(theme_index(THREAD_PAPER.name))).focusable(false).role(Role::Radio).selected(paper).label(en("thread.paper")).child(word(en("thread.paper"), paper)));
    let flag = |k: &str, label: String, on: bool, f: u8| row().key(format!("{h}/{k}")).on_press(Msg::PostFlag(f)).role(Role::Switch).checked(on).label(label.clone()).child(word(label, on));
    let footer = row().key(format!("{h}/footer")).width(w).gap(28.0).align(AlignItems::BASELINE).justify(JustifyContent::END).absolute(Some(0.0), Some(height - 22.0), None, None)
        .child(ground)
        .child(flag("binary", en("thread.binary"), s.post.binary, 0))
        .child(flag("invert", en("thread.invert"), s.post.invert, 1));
    comp
        .child(text(en("thread.kicker"), 11.0, Paint::TEXT).whisper().key(format!("{h}/kicker")))
        .child(space(1.0, 22.0))
        .child(text_lh(en("thread.title"), 30.0, 40.0, Paint::INK).font(Font::Whisper).tracking(0.3).key(format!("{h}/title")).respond(amenui::el::Response::Loosen).legible(false))
        .child(space(1.0, 34.0))
        .child(row().key(format!("{h}/spread")).width(w).gap(if narrow { 24.0 } else { w - pic_w - side_w }).align(AlignItems::START).wrap()
            .child(col().key(format!("{h}/fig")).gap(10.0)
                .child(pic(h, "pic", SPHERE, pic_w))
                .child(text(en("thread.caption"), 11.0, Paint::TEXT).whisper().key(format!("{h}/fig/caption"))))
            .child(col().key(format!("{h}/side")).width(side_w).pad(0.0, 6.0)
                .child(text_lh(en("thread.body"), 14.0, 24.0, Paint::TEXT).key(format!("{h}/body")))))
        .child(footer)
}
