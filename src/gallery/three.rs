//! The 3D page (三维, DESIGN.md §1.18): one low-poly object to turn, and a field of copies drawn in one
//! call. Every number is a number control; the material is the global one (the top bar's menu).
use super::pages::{ParamDef, header, section};
use super::{Gallery, Msg, Target};
use crate::el::{El, Paint, Response, col, row, scene3d, text};
use crate::mesh::{BUILTIN, Camera, Light, Mat4, Mesh3d, MeshHit, MeshId, MeshInstances, Pick, Projection, Scene3d};
use crate::widgets::{Orbit, OrbitMsg, choice, number, orbit, switch};
use taffy::AlignItems;

#[derive(Clone, Debug, PartialEq)]
pub enum TMsg {
    Orbit(OrbitMsg),
    /// The pointer over the field: the copy it is on, or a press.
    Pick(Pick),
    /// Which built-in mesh the object is.
    Mesh(usize),
    /// Draw the object as a picture (its marks) instead of through the material.
    Picture,
}

/// The page's state.
#[derive(Clone, Debug, PartialEq)]
pub struct Three {
    pub orbit: Orbit,
    pub mesh: usize,
    pub spec: Mesh3d,
    /// The light's direction (degrees around and above) and the ambient share.
    pub light_yaw: f32,
    pub light_pitch: f32,
    pub ambient: f32,
    pub fov: f32,
    /// The field: how many copies, how high the wave, its phase, and the field camera's pitch.
    pub count: f32,
    pub wave: f32,
    pub phase: f32,
    pub field_pitch: f32,
    pub as_picture: bool,
    pub hover: Option<MeshHit>,
    pub picked: Option<MeshHit>,
}
impl Default for Three {
    fn default() -> Self {
        Self { orbit: Orbit::default(), mesh: MeshId::ICO[1].0 as usize, spec: Mesh3d::default(), light_yaw: -40.0, light_pitch: 52.0, ambient: 0.28, fov: 35.0,
               count: 2000.0, wave: 0.35, phase: 0.0, field_pitch: 38.0, as_picture: false, hover: None, picked: None }
    }
}

const fn p(name: &'static str, min: f32, max: f32, step: f32, decimals: usize, unit: &'static str, default: f32) -> ParamDef {
    ParamDef { name, spec: crate::widgets::Spec { min, max, step, decimals, unit }, default }
}
/// Every number of the page, in the order of `slot`.
pub const PARAMS: [ParamDef; 18] = [
    p("分辨率", 0.25, 1.0, 0.05, 2, "", 0.5),
    p("深度边", 0.0, 0.5, 0.005, 3, "", 0.04),
    p("法线边", 0.0, 90.0, 1.0, 0, "°", 15.0),
    p("环境光", 0.0, 1.0, 0.02, 2, "", 0.28),
    p("光 方位", -180.0, 180.0, 5.0, 0, "°", -40.0),
    p("光 高度", -90.0, 90.0, 5.0, 0, "°", 52.0),
    p("视角", 10.0, 100.0, 1.0, 0, "°", 35.0),
    p("距离", 0.8, 30.0, 0.1, 1, "", 3.2),
    p("方位", -360.0, 360.0, 1.0, 0, "°", 30.0),
    p("俯仰", -85.0, 85.0, 1.0, 0, "°", 22.0),
    p("拖动", 0.05, 2.0, 0.05, 2, "°/px", 0.4),
    p("惯性摩擦", 0.5, 12.0, 0.1, 1, "/s", 3.5),
    p("方向键", 1.0, 45.0, 1.0, 0, "°", 5.0),
    p("滚轮", 0.0002, 0.01, 0.0001, 4, "/px", 0.0015),
    p("数量", 1.0, 2000.0, 1.0, 0, "", 2000.0),
    p("波高", 0.0, 1.5, 0.05, 2, "", 0.35),
    p("相位", 0.0, 6.3, 0.1, 1, "", 0.0),
    p("场 俯仰", 5.0, 85.0, 1.0, 0, "°", 38.0),
];
pub fn slot(t: &mut Three, i: usize) -> &mut f32 {
    match i {
        0 => &mut t.spec.resolution_scale, 1 => &mut t.spec.edge_depth, 2 => &mut t.spec.edge_normal, 3 => &mut t.ambient, 4 => &mut t.light_yaw,
        5 => &mut t.light_pitch, 6 => &mut t.fov, 7 => &mut t.orbit.distance, 8 => &mut t.orbit.yaw, 9 => &mut t.orbit.pitch, 10 => &mut t.orbit.per_px,
        11 => &mut t.orbit.friction, 12 => &mut t.orbit.step_deg, 13 => &mut t.orbit.dolly, 14 => &mut t.count, 15 => &mut t.wave, 16 => &mut t.phase, _ => &mut t.field_pitch,
    }
}

pub fn update(s: &mut Gallery, m: TMsg) {
    let t = &mut s.three;
    match m {
        TMsg::Orbit(o) => t.orbit.update(o, s.clock),
        TMsg::Pick(Pick::Hover(h)) => t.hover = h,
        TMsg::Pick(Pick::Press(h)) => t.picked = h,
        TMsg::Mesh(i) => t.mesh = i.min(BUILTIN.len() - 1),
        TMsg::Picture => t.as_picture = !t.as_picture,
    }
}

fn light(t: &Three) -> Light { Light::from_angles(t.light_yaw, t.light_pitch, t.ambient) }

/// The low-poly object, as the page shows it (also used by the acceptance sheet).
pub fn object(t: &Three) -> Scene3d {
    let mut sc = Scene3d::single(MeshId(t.mesh as u32));
    sc.items[0].transforms = vec![Mat4::uniform(1.6)];
    sc.camera = Camera { projection: Projection::Perspective { fov: t.fov }, ..Camera::default() };
    sc.light = light(t);
    sc.spec = t.spec;
    sc.picture = t.as_picture.then_some(super::STYLE_SLOT);
    sc
}

/// The field: up to 2,000 icosahedra on a grid riding a wave, one call; the copy under the pointer
/// in the accent colour (a second group of one).
pub fn field(t: &Three) -> Scene3d {
    let n = (t.count.round() as usize).clamp(1, 2000);
    let (cols, rows) = (50usize, 40usize);
    let mut all = Vec::with_capacity(n);
    for k in 0..n.min(cols * rows) {
        let (i, j) = ((k % cols) as f32, (k / cols) as f32);
        let (x, z) = ((i - (cols as f32 - 1.0) * 0.5) * 0.24, (j - (rows as f32 - 1.0) * 0.5) * 0.24);
        let y = t.wave * (x * 0.9 + t.phase).sin() * (z * 0.7 + t.phase * 0.5).cos();
        all.push(Mat4::translate(x, y, z).mul(&Mat4::rotate_y(k as f32 * 37.0)).mul(&Mat4::uniform(0.2)));
    }
    let hot = t.hover.filter(|h| h.object == 1 && (h.instance as usize) < all.len()).map(|h| h.instance as usize);
    let mut items = vec![MeshInstances { mesh: MeshId::ICO[0], transforms: all.clone(), paint: Paint::INK, id: 1 }];
    if let Some(h) = hot { items[0].transforms[h] = Mat4::scale(0.0, 0.0, 0.0); items.push(MeshInstances::one(MeshId::ICO[0], all[h], Paint::ACCENT, 2)); }
    let camera = Camera { projection: Projection::Perspective { fov: 30.0 }, yaw: 20.0, pitch: t.field_pitch, distance: 17.0, target: [0.0; 3], near: 0.5, far: 60.0 };
    Scene3d { items, camera, light: light(t), spec: t.spec, picture: None }
}

pub fn page(s: &Gallery) -> El<Msg> {
    let h = "three";
    let t = &s.three;
    let view = orbit("three/object", object(t), &t.orbit, |m| Msg::Three(TMsg::Orbit(m))).width(520.0).height(380.0);
    let picked = match t.picked { Some(m) => format!("按下：第 {} 个", m.instance + 1), None => "按下：—".into() };
    let hovered = match t.hover { Some(m) => format!("指针：第 {} 个", m.instance + 1), None => "指针：—".into() };
    let field = scene3d(field(t)).key("three/field").fill_width().height(340.0).respond(Response::Loosen)
        .on_pick(|p| Some(Msg::Three(TMsg::Pick(p))));
    let mut nums = row().key("three/params").fill_width().wrap().gap(10.0);
    for (i, d) in PARAMS.iter().enumerate() {
        let tg = Target::Three(i);
        let v = *slot(&mut t.clone(), i);
        // The camera's numbers change on every frame while it turns: faster than `Recompose::min_interval_ms`, so they are replaced at once.
        nums = nums.child(number(&format!("three/p{i}"), d.name, v, d.spec, s.draft(tg), move |e| Msg::Num(tg, e)).width(300.0));
    }
    // The ascii material's normal weight lives with the material; it is here too.
    let ascii = crate::style::Material::Ascii as u8;
    let aw = crate::style::MAT_PARAMS[ascii as usize][3];
    let tg = Target::Mat(ascii, 3);
    nums = nums.child(number("three/ascii-normal", "ASCII 法线权重", s.mats[ascii as usize].0[3], crate::widgets::Spec { min: aw.min, max: aw.max, step: aw.step, decimals: aw.decimals, unit: aw.unit }, s.draft(tg), move |e| Msg::Num(tg, e)).width(300.0));
    let controls = col().key("three/controls").gap(12.0)
        .child(choice("three/mesh", "网格", &BUILTIN, t.mesh, |i| Msg::Three(TMsg::Mesh(i))).max_width(420.0))
        .child(switch("three/picture", "按图片画（记号）", t.as_picture, None, Msg::Three(TMsg::Picture)))
        .child(text("拖动转动，滚轮推近拉远；获得焦点后方向键转动，Shift 每次四倍。松手时还在转，会慢慢停下。", 13.0, Paint::DIM).max_width(380.0));
    col().key("three/page").fill_width().max_width(1100.0).gap(34.0)
        .child(header(s, h, "三维", "El::mesh / scene3d / MeshInstances", "三维物体是页面上的又一种东西：在布局给它的矩形里画，用当前的材质，会松开、变形，可以按到。网格在一张离屏图里画出明暗、法线、深度和编号，再变成和文字一样的墨与近度：轮廓和棱都是“边”。"))
        .child(section(h, "一个物体", row().key("three/object-row").fill_width().wrap().gap(24.0).align(AlignItems::START).child(view).child(controls)))
        .child(section(h, "两千个，一次画出", col().key("three/field-col").fill_width().gap(10.0).child(field)
            .child(row().gap(24.0).child(text(hovered, 13.0, Paint::DIM).key("three/hovered")).child(text(picked, 13.0, Paint::DIM).key("three/picked")))))
        .child(section(h, "参数", nums))
        .child(crate::el::space(1.0, 24.0))
}
