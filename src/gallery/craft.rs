//! Interaction craft (DESIGN.md §1.8.4, §5.2): the live previews of the gesture, menu, loading, stack,
//! chart and field components on their documentation pages. Each preview is driven by state kept here
//! and stepped on the gallery's tick only while something moves; each carries a formula caption printed
//! from the same fields it reads.
use super::{Gallery, Msg, Target};
use crate::el::{El, Font, Paint, col, layers, row, space, text};
use crate::widgets::{CaptionParam, Edit, FloatMsg, Floating, Sheet, SheetMsg, Spec, Variant, bottom_sheet, button_as, floating};
use taffy::AlignItems;

/// The previews' state.
#[derive(Clone, Debug, PartialEq)]
pub struct Craft { pub sheet: Sheet, pub pip: Floating }
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
        Self { sheet, pip }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CMsg { SheetOpen, Sheet(SheetMsg), Pip(FloatMsg) }

pub fn update(s: &mut Gallery, m: CMsg) {
    let now = s.clock;
    let c = &mut s.craft;
    match m {
        CMsg::SheetOpen => { c.sheet.present(now); s.cmds.push(crate::morph::Cmd::Focus { key: "craft/sheet/handle".into() }); }
        CMsg::Sheet(m) => c.sheet.update(m, now),
        CMsg::Pip(m) => c.pip.update(m, now),
    }
}
/// Step what springs; whether the view changed.
pub fn tick(s: &mut Gallery, t: f32) -> bool {
    let c = &mut s.craft;
    let a = c.sheet.step(t);
    let b = c.pip.step(t);
    a || b
}
/// Every frame while something springs; nothing at rest.
pub fn wake(s: &Gallery, t: f32) -> Option<f32> { (s.craft.sheet.moving() || s.craft.pip.moving()).then_some(t) }

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
    match p { 0 => &SHEET, _ => &PIP }
}
/// Number `i` of preview `p`.
fn get(c: &Craft, p: usize, i: usize) -> f32 {
    match (p, i) {
        (0, 0) => c.sheet.detents[0], (0, 1) => c.sheet.detents[1], (0, 2) => c.sheet.dismiss_velocity, (0, 3) => c.sheet.projection_decel,
        (0, 4) => c.sheet.backdrop, (0, 5) => c.sheet.spring.stiffness, (0, 6) => c.sheet.spring.damping, (0, _) => c.sheet.rubber.coeff,
        (_, 0) => c.pip.corner_inset, (_, 1) => c.pip.projection_decel, (_, 2) => c.pip.spring.stiffness, (_, 3) => c.pip.spring.damping, _ => c.pip.rubber.coeff,
    }
}
pub fn set(s: &mut Gallery, p: usize, i: usize, v: f32) {
    let c = &mut s.craft;
    match (p, i) {
        (0, 0) => c.sheet.detents[0] = v.min(c.sheet.detents[1]), (0, 1) => c.sheet.detents[1] = v.max(c.sheet.detents[0]), (0, 2) => c.sheet.dismiss_velocity = v,
        (0, 3) => c.sheet.projection_decel = v, (0, 4) => c.sheet.backdrop = v, (0, 5) => c.sheet.spring.stiffness = v, (0, 6) => c.sheet.spring.damping = v,
        (0, _) => c.sheet.rubber.coeff = v,
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
