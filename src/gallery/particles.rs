//! The particle page (粒子, DESIGN.md §1.10, GPU particles): one stage that goes mesh → text → picture →
//! mesh, with the count as a number up to 1,000,000. Above `Particles::gpu_above` tiles the morph runs on
//! the GPU (a simulated grid); the pointer pushes the swarm, a press sends a front through it.
use super::pages::{ParamDef, header, section};
use super::{Gallery, IMAGE_SLOT, Msg, Target};
use crate::demo::SKY;
use crate::el::{El, Font, Paint, Scope, col, picture, row, scene3d, stack, text, text_lh};
use crate::mesh::{Mat4, MeshId, Scene3d};
use crate::morph::{Cmd, MorphSpec, Particles};
use crate::widgets::{Variant, button_as, number, segmented};
use taffy::AlignItems;

pub const FORMS: [&str; 4] = ["网格", "文字", "图片", "另一个网格"];

#[derive(Clone, Debug, PartialEq)]
pub enum PMsg { Next, To(usize) }

#[derive(Clone, Debug, PartialEq)]
pub struct Parts { pub shape: usize, pub count: f32, pub duration_ms: f32, pub stagger: f32, pub curl: f32 }
impl Default for Parts { fn default() -> Self { Self { shape: 0, count: 200_000.0, duration_ms: 1400.0, stagger: 0.35, curl: 60.0 } } }

const fn p(name: &'static str, min: f32, max: f32, step: f32, decimals: usize, unit: &'static str, default: f32) -> ParamDef {
    ParamDef { name, spec: crate::widgets::Spec { min, max, step, decimals, unit }, default }
}
/// The page's numbers, in the order of `slot`: the morph's, then `Particles`.
pub const PARAMS: [ParamDef; 12] = [
    p("粒子数", 100.0, 1_000_000.0, 10_000.0, 0, "", 200_000.0),
    p("飞行", 100.0, 6000.0, 50.0, 0, "ms", 1400.0),
    p("错开", 0.0, 0.9, 0.05, 2, "", 0.35),
    p("弧度", 0.0, 400.0, 2.0, 0, "px", 60.0),
    p("GPU 起点", 0.0, 1_000_000.0, 1000.0, 0, "", 20000.0),
    p("旋度", 0.0, 20000.0, 100.0, 0, "px/s²", 1600.0),
    p("旋度尺度", 4.0, 600.0, 2.0, 0, "px", 110.0),
    p("阻尼", 0.0, 20.0, 0.1, 1, "/s", 2.5),
    p("回拉", 0.0, 200.0, 0.5, 1, "/s²", 6.0),
    p("指针", -60000.0, 60000.0, 500.0, 0, "px/s²", -9000.0),
    p("指针范围", 0.0, 600.0, 5.0, 0, "px", 140.0),
    p("波前推力", 0.0, 200000.0, 1000.0, 0, "px/s²", 30000.0),
];
pub fn slot<'a>(pp: &'a mut Parts, pa: &'a mut Particles, i: usize) -> &'a mut f32 {
    match i {
        0 => &mut pp.count, 1 => &mut pp.duration_ms, 2 => &mut pp.stagger, 3 => &mut pp.curl, 4 => &mut pa.gpu_above, 5 => &mut pa.curl,
        6 => &mut pa.curl_scale, 7 => &mut pa.damping, 8 => &mut pa.spring, 9 => &mut pa.pointer, 10 => &mut pa.pointer_radius, _ => &mut pa.front,
    }
}

/// Stage size (logical px): the tiles are this area over the count.
const STAGE: (f32, f32) = (900.0, 440.0);
pub fn key(i: usize) -> String { format!("parts/c{i}") }

pub fn update(s: &mut Gallery, m: PMsg) {
    let to = match m { PMsg::Next => (s.parts.shape + 1) % FORMS.len(), PMsg::To(i) => i.min(FORMS.len() - 1) };
    if to == s.parts.shape { return; }
    let pp = &s.parts;
    // The tiles: as many as asked over the stage, never under a device pixel.
    let lattice = ((STAGE.0 * STAGE.1) / pp.count.max(1.0)).sqrt().max(1.0);
    let spec = MorphSpec { duration_ms: pp.duration_ms, stagger: pp.stagger, curl: pp.curl, count_max: pp.count, lattice, loosen_ms: 160.0, tighten_ms: 200.0, ..MorphSpec::default() };
    s.cmds.push(Cmd::Morph { from: key(pp.shape), to: key(to), spec });
    s.parts.shape = to;
}

fn mesh(id: MeshId, yaw: f32) -> Scene3d {
    let mut sc = Scene3d::single(id);
    sc.items[0].transforms = vec![Mat4::rotate_y(yaw).mul(&Mat4::uniform(1.9))];
    sc
}

fn form(s: &Gallery, i: usize) -> El<Msg> {
    let k = key(i);
    let el = match i {
        0 => scene3d(mesh(MeshId::ICO[1], 20.0)).key(k).width(420.0).height(420.0),
        1 => text_lh("粒子", 220.0, 260.0, Paint::INK).font(Font::Display).weight(4.0).key(k),
        2 => col().key(k).width(760.0).child(picture(SKY, super::pages::ratio(SKY)).key("parts/picture").pic_fx(IMAGE_SLOT)),
        _ => scene3d(mesh(MeshId::TORUS, -30.0)).key(k).width(560.0).height(420.0),
    };
    el.hidden_until_morphed(i != s.parts.shape)
}

pub fn page(s: &Gallery) -> El<Msg> {
    let h = "parts";
    let stage = stack((0..FORMS.len()).map(|i| form(s, i))).key("parts/stage").width(STAGE.0).height(STAGE.1).radius(6.0).border(1.0, Paint::FAINT)
        .ripple_scope().ripple(Scope::Container(None));
    let controls = row().key("parts/controls").wrap().gap(12.0).align(AlignItems::CENTER)
        .child(button_as("parts/next", "下一步", Variant::Accent, Msg::Parts(PMsg::Next)))
        .child(segmented("parts/forms", &FORMS, s.parts.shape, 30.0, |i| Msg::Parts(PMsg::To(i))));
    let mut nums = row().key("parts/params").fill_width().wrap().gap(10.0);
    for (i, d) in PARAMS.iter().enumerate() {
        let t = Target::Parts(i);
        let v = *slot(&mut s.parts.clone(), &mut s.particles.clone(), i);
        nums = nums.child(number(&format!("parts/p{i}"), d.name, v, d.spec, s.draft(t), move |e| Msg::Num(t, e)).width(330.0));
    }
    col().key("parts/page").fill_width().max_width(1000.0).gap(30.0)
        .child(header(s, h, "粒子", "MorphSpec · Particles", "网格、文字、图片之间来回变：每一格是源在那一格画的东西，落下时是目标在那一格画的东西。格子多于“GPU 起点”时，粒子在显卡上模拟（旋度噪声、指针、波前），只有起点和终点由路径决定，所以交接那一帧和落地那一帧不变。指针推开粒子，按舞台放出一道波前。"))
        .child(section(h, "舞台", col().fill_width().gap(14.0).child(stage).child(controls)
            .child(text("读数（F2）里写着粒子数和它们在哪里模拟。", 13.0, Paint::DIM))))
        .child(section(h, "参数", nums))
        .child(crate::el::space(1.0, 24.0))
}
