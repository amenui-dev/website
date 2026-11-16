//! Interaction craft (DESIGN.md §1.8.4, §5.2): the live previews of the gesture, menu, loading, stack,
//! chart and field components on their documentation pages. Each preview is driven by state kept here
//! and stepped on the gallery's tick only while something moves; each carries a formula caption printed
//! from the same fields it reads.
use super::{Gallery, Msg, Target};
use crate::el::{El, Font, Paint, col, layers, row, space, text};
use crate::widgets::{Loading, Show, progress, CaptionParam, Edit, FloatMsg, Floating, Menu, MenuItem, MenuMsg, Sheet, SheetMsg, Spec, Variant, bottom_sheet, button_as, floating, nested_menu, switch};
use taffy::AlignItems;

/// The previews' state.
#[derive(Clone, Debug, PartialEq)]
pub struct Craft { pub sheet: Sheet, pub pip: Floating, pub menu: Menu, pub picked: Vec<usize>, pub load: Loading, pub latency_ms: f32, pub now: f32 }
/// The sheet preview's phone-shaped frame, and the floating preview's area (logical px).
pub const PHONE: (f32, f32) = (340.0, 560.0);
pub const AREA: (f32, f32) = (520.0, 320.0);
impl Default for Craft {
    fn default() -> Self {
        let mut sheet = Sheet::default();
        sheet.height = PHONE.1;
        let mut pip = Floating::default();
        (pip.area, pip.size) = (AREA, (168.0, 100.0));
        pip.place(3);
        Self { sheet, pip, menu: Menu::new(menu_items()), picked: vec![], load: Loading::default(), latency_ms: 900.0, now: 0.0 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CMsg { Reload, SheetOpen, Sheet(SheetMsg), Pip(FloatMsg), Menu(MenuMsg), SafeZone }

/// The menu preview's rows (generic names; the labels are in the catalogs).
fn menu_items() -> Vec<MenuItem> {
    let t = |k: &str| crate::i18n::tr(k);
    let sub = |k: &str, n: usize| MenuItem::sub(t(k), (1..=n).map(|i| MenuItem::leaf(format!("{} {i}", t("craft.menu.item")))).collect());
    vec![sub("craft.menu.new", 5), sub("craft.menu.recent", 6), sub("craft.menu.share", 4), sub("craft.menu.arrange", 5), MenuItem::leaf(t("craft.menu.close"))]
}

pub fn update(s: &mut Gallery, m: CMsg) {
    let now = s.clock;
    let c = &mut s.craft;
    match m {
        CMsg::Reload => c.load.start(now),
        CMsg::SheetOpen => { c.sheet.present(now); s.cmds.push(crate::morph::Cmd::Focus { key: "craft/sheet/handle".into() }); }
        CMsg::Sheet(m) => c.sheet.update(m, now),
        CMsg::Pip(m) => c.pip.update(m, now),
        CMsg::Menu(m) => if let Some(p) = c.menu.update(m, now) { c.picked = p; },
        CMsg::SafeZone => c.menu.show_safe_zone = !c.menu.show_safe_zone,
    }
}
/// Step what springs; whether the view changed.
pub fn tick(s: &mut Gallery, t: f32) -> bool {
    let c = &mut s.craft;
    let a = c.sheet.step(t);
    let b = c.pip.step(t);
    let m = c.menu.step(t);
    // The loading comparison: the data comes `latency_ms` after the request.
    let loading = c.load.started().is_some_and(|st| c.load.arrived().is_none() || t < st + c.latency_ms / 1000.0 + 1.0);
    if let Some(st) = c.load.started() && c.load.arrived().is_none() && t >= st + c.latency_ms / 1000.0 { c.load.arrive(t); }
    c.now = t;
    a || b || m || loading
}
/// Every frame while something springs; nothing at rest.
pub fn wake(s: &Gallery, t: f32) -> Option<f32> {
    if s.craft.sheet.moving() || s.craft.pip.moving() { return Some(t); }
    // The loading comparison runs its spinner and bar while it waits.
    let c = &s.craft;
    if c.load.started().is_some_and(|st| c.load.arrived().is_none() || t < st + c.latency_ms / 1000.0 + 1.0) { return Some(t); }
    // A menu switch waits for its time, not for every frame.
    c.menu.due()
}

const fn spec(min: f32, max: f32, step: f32, decimals: usize, unit: &'static str) -> Spec { Spec { min, max, step, decimals, unit } }
/// The numbers of preview `p` (0 sheet, 1 floating): (name in the formula, range).
fn defs(p: usize) -> &'static [(&'static str, Spec)] {
    const SHEET: [(&str, Spec); 8] = [
        ("detents[0]", spec(0.1, 1.0, 0.05, 2, "")), ("detents[1]", spec(0.1, 1.0, 0.05, 2, "")), ("dismiss_velocity", spec(0.0, 5000.0, 50.0, 0, "px/s")),
        ("projection_decel", spec(0.9, 0.9995, 0.0005, 4, "")), ("backdrop", spec(0.0, 1.0, 0.05, 2, "")), ("stiffness", spec(10.0, 2000.0, 10.0, 0, "")),
        ("damping", spec(0.0, 200.0, 1.0, 0, "")), ("rubber.coeff", spec(0.0, 2.0, 0.05, 2, "")),
    ];
    const PIP: [(&str, Spec); 5] = [
        ("corner_inset", spec(0.0, 80.0, 1.0, 0, "px")), ("projection_decel", spec(0.9, 0.9995, 0.0005, 4, "")), ("stiffness", spec(10.0, 2000.0, 10.0, 0, "")),
        ("damping", spec(0.0, 200.0, 1.0, 0, "")), ("rubber.coeff", spec(0.0, 2.0, 0.05, 2, "")),
    ];
    const MENU: [(&str, Spec); 4] = [
        ("submenu_delay_ms", spec(0.0, 1000.0, 10.0, 0, "ms")), ("close_delay_ms", spec(0.0, 2000.0, 10.0, 0, "ms")), ("row_h", spec(20.0, 60.0, 1.0, 0, "px")), ("width", spec(120.0, 320.0, 4.0, 0, "px")),
    ];
    const LOAD: [(&str, Spec); 3] = [("skeleton_after_ms", spec(0.0, 2000.0, 10.0, 0, "ms")), ("swap_stagger_ms", spec(0.0, 400.0, 5.0, 0, "ms")), ("latency_ms", spec(0.0, 5000.0, 50.0, 0, "ms"))];
    match p { 0 => &SHEET, 1 => &PIP, 2 => &MENU, _ => &LOAD }
}
/// Number `i` of preview `p`.
fn get(c: &Craft, p: usize, i: usize) -> f32 {
    match (p, i) {
        (0, 0) => c.sheet.detents[0], (0, 1) => c.sheet.detents[1], (0, 2) => c.sheet.dismiss_velocity, (0, 3) => c.sheet.projection_decel,
        (0, 4) => c.sheet.backdrop, (0, 5) => c.sheet.spring.stiffness, (0, 6) => c.sheet.spring.damping, (0, _) => c.sheet.rubber.coeff,
        (3, 0) => c.load.skeleton_after_ms, (3, 1) => c.load.swap_stagger_ms, (3, _) => c.latency_ms,
        (2, 0) => c.menu.submenu_delay_ms, (2, 1) => c.menu.close_delay_ms, (2, 2) => c.menu.row_h, (2, _) => c.menu.width,
        (_, 0) => c.pip.corner_inset, (_, 1) => c.pip.projection_decel, (_, 2) => c.pip.spring.stiffness, (_, 3) => c.pip.spring.damping, _ => c.pip.rubber.coeff,
    }
}
pub fn set(s: &mut Gallery, p: usize, i: usize, v: f32) {
    let c = &mut s.craft;
    match (p, i) {
        (0, 0) => c.sheet.detents[0] = v.min(c.sheet.detents[1]), (0, 1) => c.sheet.detents[1] = v.max(c.sheet.detents[0]), (0, 2) => c.sheet.dismiss_velocity = v,
        (0, 3) => c.sheet.projection_decel = v, (0, 4) => c.sheet.backdrop = v, (0, 5) => c.sheet.spring.stiffness = v, (0, 6) => c.sheet.spring.damping = v,
        (0, _) => c.sheet.rubber.coeff = v,
        (3, 0) => c.load.skeleton_after_ms = v, (3, 1) => c.load.swap_stagger_ms = v, (3, _) => c.latency_ms = v,
        (2, 0) => c.menu.submenu_delay_ms = v, (2, 1) => c.menu.close_delay_ms = v, (2, 2) => c.menu.row_h = v, (2, _) => c.menu.width = v,
        (_, 0) => { c.pip.corner_inset = v; let k = c.pip.corner(); c.pip.place(k); }
        (_, 1) => c.pip.projection_decel = v, (_, 2) => c.pip.spring.stiffness = v, (_, 3) => c.pip.spring.damping = v, _ => c.pip.rubber.coeff = v,
    }
}
/// The caption parameters of preview `p`, read from the fields.
fn params(s: &Gallery, p: usize) -> Vec<CaptionParam> {
    defs(p).iter().enumerate().map(|(i, (n, sp))| CaptionParam::new(*n, get(&s.craft, p, i), *sp).draft(s.draft(Target::Craft(p, i)))).collect()
}
fn cap(s: &Gallery, p: usize, key: &str, formula: String, demo: El<Msg>) -> El<Msg> {
    demo.caption_params(key, formula, &params(s, p), move |i, e: Edit| Msg::Num(Target::Craft(p, i), e))
}

/// The live preview of component `name`, when this module has one.
pub fn live(s: &Gallery, name: &str) -> Option<El<Msg>> {
    match name {
        "bottom_sheet" => Some(sheet_page(s)),
        "floating" => Some(pip_page(s)),
        "nested_menu" => Some(menu_page(s)),
        "skeleton" => Some(load_page(s)),
        _ => None,
    }
}

fn sheet_page(s: &Gallery) -> El<Msg> {
    let c = &s.craft;
    let zh = crate::i18n::current() == crate::i18n::Locale::ZhHans;
    let mut page = col().key("craft/sheet/page").fill_width().fill_height().pad(20.0, 20.0).gap(12.0)
        .child(text(if zh { "相册" } else { "Album" }, 22.0, Paint::INK).font(Font::Display));
    for i in 0..5 {
        page = page.child(row().key(format!("craft/sheet/row{i}")).gap(12.0).align(AlignItems::CENTER)
            .child(space(44.0, 44.0).radius(6.0).fill(Paint::hue((i % 3) as u8).alpha(0.7)))
            .child(text(if zh { format!("第 {} 卷", i + 1) } else { format!("Roll {}", i + 1) }, 15.0, Paint::TEXT)));
    }
    page = page.child(button_as("craft/sheet/open", if zh { "分享…" } else { "Share…" }, Variant::Accent, Msg::Craft(CMsg::SheetOpen)));
    let mut body = col().key("craft/sheet/list").fill_width().gap(2.0);
    for i in 0..16 {
        body = body.child(row().key(format!("craft/sheet/item{i}")).fill_width().height(40.0).gap(12.0).align(AlignItems::CENTER)
            .child(space(28.0, 28.0).radius(14.0).fill(Paint::hue((i % 3) as u8)))
            .child(text(if zh { format!("联系人 {}", i + 1) } else { format!("Contact {}", i + 1) }, 14.0, Paint::TEXT)));
    }
    let title = if zh { "分享到" } else { "Share to" };
    let frame = layers([page, bottom_sheet("craft/sheet", &c.sheet, title, body, |m| Msg::Craft(CMsg::Sheet(m)))])
        .key("craft/sheet/frame").width(PHONE.0).height(PHONE.1).radius(22.0).border(1.0, Paint::DIM).clip();
    let sh = &c.sheet;
    let formula = format!("rest = nearest(detents·H, x + v/1000·{d:.4}/(1−{d:.4}))  ·  dismiss if end < {lo:.0}/2 or (end < {lo:.0} and −v ≥ {dv:.0})  ·  past top: x' = top + (1 − 1/(d·{rc:.2}/{h:.0} + 1))·{h:.0}  ·  spring k={k:.0} c={c:.0} m={m:.0} (ζ={z:.2})",
        d = sh.projection_decel, lo = sh.detent(0), dv = sh.dismiss_velocity, rc = sh.rubber.coeff, h = sh.height, k = sh.spring.stiffness, c = sh.spring.damping, m = sh.spring.mass, z = sh.spring.ratio());
    cap(s, 0, "craft/sheet/cap", formula, frame)
}

fn pip_page(s: &Gallery) -> El<Msg> {
    let f = &s.craft.pip;
    let player = col().key("craft/pip/player").fill_width().fill_height().pad(10.0, 8.0).gap(6.0).fill(Paint::hue(1).alpha(0.22)).respond(crate::el::Response::Nudge)
        .child(row().gap(6.0).align(AlignItems::CENTER).child(space(10.0, 10.0).radius(5.0).fill(Paint::ACCENT)).child(text("00:42", 11.0, Paint::TEXT).font(Font::Mono)))
        .child(space(0.0, 0.0).grow(1.0))
        .child(row().fill_width().height(3.0).child(space(0.0, 3.0).width_frac(0.4).fill(Paint::ACCENT)));
    // The four corners it can rest in, faint, then the box.
    let mut parts: Vec<El<Msg>> = (0..4).map(|i| {
        let (x, y) = f.corner_at(i);
        space(f.size.0, f.size.1).key(format!("craft/pip/corner{i}")).shift(x, y).radius(8.0).border(1.0, if i == f.corner() { Paint::DIM } else { Paint::FAINT })
    }).collect();
    parts.push(floating("craft/pip/box", f, player, |m| Msg::Craft(CMsg::Pip(m))));
    let area = layers(parts).key("craft/pip/area").width(AREA.0).height(AREA.1).radius(8.0).border(1.0, Paint::FAINT).clip();
    let formula = format!("end = at + v/1000·{d:.4}/(1−{d:.4})  ·  corner = nearest(end), inset {i:.0}  ·  spring k={k:.0} c={c:.0} (ζ={z:.2})  ·  past an edge: (1 − 1/(d·{rc:.2}/dim + 1))·dim",
        d = f.projection_decel, i = f.corner_inset, k = f.spring.stiffness, c = f.spring.damping, z = f.spring.ratio(), rc = f.rubber.coeff);
    cap(s, 1, "craft/pip/cap", formula, area)
}

fn menu_page(s: &Gallery) -> El<Msg> {
    let m = &s.craft.menu;
    let tr = crate::i18n::tr;
    let picked = if s.craft.picked.is_empty() { tr("craft.menu.none") } else {
        let mut it: &[MenuItem] = &m.items;
        let mut names = vec![];
        for &i in &s.craft.picked { if let Some(x) = it.get(i) { names.push(x.label.clone()); it = &x.items; } }
        names.join(" › ")
    };
    let menu = nested_menu("craft/menu", m, |x| Msg::Craft(CMsg::Menu(x)));
    let area = col().key("craft/menu/area").width(560.0).height(330.0).pad(16.0, 16.0).radius(8.0).border(1.0, Paint::FAINT).child(menu);
    let controls = row().key("craft/menu/controls").gap(16.0).align(AlignItems::CENTER)
        .child(switch("craft/menu/zone", &tr("craft.menu.show-zone"), m.show_safe_zone, None, Msg::Craft(CMsg::SafeZone)))
        .child(text(format!("{}: {picked}", tr("craft.menu.picked")), 13.0, Paint::DIM).key("craft/menu/picked"));
    let formula = format!("switch row after {sd:.0} ms; inside △(prev pointer, submenu near corners) wait {cd:.0} ms, re-armed by each move  ·  rows {rh:.0} px, menus {w:.0} px",
        sd = m.submenu_delay_ms, cd = m.close_delay_ms, rh = m.row_h, w = m.width);
    cap(s, 2, "craft/menu/cap", formula, col().key("craft/menu/demo").gap(12.0).child(area).child(controls))
}

/// One row of the loading comparison, as it is when loaded.
fn load_row(k: &str, i: usize) -> El<Msg> {
    let tr = crate::i18n::tr;
    let titles = ["craft.load.r0", "craft.load.r1", "craft.load.r2", "craft.load.r3"];
    row().key(format!("{k}/row{i}")).gap(10.0).height(44.0).align(AlignItems::CENTER)
        .child(space(34.0, 34.0).radius(6.0).fill(Paint::hue((i % 3) as u8)).key(format!("{k}/row{i}/pic")))
        .child(col().gap(3.0).child(text(tr(titles[i % 4]), 13.5, Paint::TEXT).key(format!("{k}/row{i}/t"))).child(text(tr("craft.load.meta"), 11.0, Paint::DIM).key(format!("{k}/row{i}/m"))))
}
fn load_page(s: &Gallery) -> El<Msg> {
    let c = &s.craft;
    let tr = crate::i18n::tr;
    let now = c.now;
    let (st, arrived) = (c.load.started(), c.load.arrived());
    let waiting = st.is_some() && arrived.is_none();
    let panel = |k: &str, name: String, body: El<Msg>| col().key(format!("craft/load/{k}")).width(250.0).height(300.0).pad(14.0, 12.0).gap(8.0).radius(8.0).border(1.0, Paint::FAINT).clip()
        .child(text(name, 13.0, Paint::DIM).key(format!("craft/load/{k}/name"))).child(body);
    let rows = |k: &str, show: &dyn Fn(usize) -> Show| (0..4).fold(col().key(format!("craft/load/{k}/rows")).gap(6.0), |c, i| c.child(match show(i) {
        Show::Content => load_row(k, i), Show::Skeleton => load_row(k, i).skeleton(), Show::Nothing => load_row(k, i).hidden_until_morphed(true),
    }));
    // 1. A spinner until everything is there: eight dots, one lit, a step every 100 ms.
    let spin = if waiting {
        let lit = ((now - st.unwrap_or(0.0)) * 10.0).floor() as usize % 8;
        let dots: Vec<El<Msg>> = (0..8).map(|d| {
            let a = d as f32 / 8.0 * std::f32::consts::TAU;
            space(6.0, 6.0).radius(3.0).fill(if d == lit { Paint::ACCENT } else { Paint::FAINT }).shift(15.0 + 14.0 * a.cos(), 15.0 + 14.0 * a.sin()).key(format!("craft/load/spin/d{d}"))
        }).collect();
        let ring = layers(dots).key("craft/load/spin/ring").width(36.0).height(36.0);
        col().key("craft/load/spin/wait").fill_width().height(220.0).center().child(ring)
    } else { rows("spin", &|_| Show::Content) };
    // 2. The skeleton rules.
    let skel = rows("skel", &|i| c.load.show(i, now));
    // 3. A bar that fills as the expected time passes, then the rows.
    let prog = if waiting {
        let f = ((now - st.unwrap_or(0.0)) / (c.latency_ms / 1000.0).max(0.05)).clamp(0.0, 0.95);
        col().key("craft/load/prog/wait").gap(10.0).child(progress("craft/load/prog/bar", Some(f), 222.0)).child(rows("prog", &|_| Show::Nothing))
    } else { rows("prog", &|_| Show::Content) };
    // 4. Optimistic: the rows are there at once; the one just added shows faint until it is confirmed.
    // (Drawn faint the way a subtree that takes no input is, until the answer comes.)
    let opt = (0..4).fold(col().key("craft/load/opt/rows").gap(6.0), |c, i| c.child(load_row("opt", i).disabled(i == 3 && waiting)));
    let opt = col().key("craft/load/opt/col").gap(4.0).child(opt)
        .child(text(if waiting { tr("craft.load.sending") } else { tr("craft.load.sent") }, 11.0, if waiting { Paint::DIM } else { Paint::ACCENT }).key("craft/load/opt/state"));
    let grid = row().key("craft/load/grid").wrap().gap(12.0)
        .child(panel("spin", tr("craft.load.spinner"), spin)).child(panel("skel", tr("craft.load.skeleton"), skel))
        .child(panel("prog", tr("craft.load.progress"), prog)).child(panel("opt", tr("craft.load.optimistic"), opt));
    let demo = col().key("craft/load/demo").gap(12.0).child(button_as("craft/load/reload", &tr("craft.load.reload"), Variant::Accent, Msg::Craft(CMsg::Reload))).child(grid);
    let l = &c.load;
    let formula = format!("t < {a:.0} ms: static parts only  ·  t ≥ {a:.0} ms and no data: row.skeleton()  ·  data at {lat:.0} ms: row i at data + i·{st:.0} ms, same boxes",
        a = l.skeleton_after_ms, st = l.swap_stagger_ms, lat = c.latency_ms);
    cap(s, 3, "craft/load/cap", formula, demo)
}
