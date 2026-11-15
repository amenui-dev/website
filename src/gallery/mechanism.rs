//! The mechanism (DESIGN.md §6, chapter 13, and the 三维 page): a small machine built from primitives
//! in code — a housing (a back plate and a base), three meshing spur gears that turn at their true
//! ratios, their shafts, a cam on the big gear's shaft with a follower riding on it, and the nuts and
//! bolts that hold it. Perspective camera, a strong key light with two flat tones, drawn outlines on
//! the silhouettes and creases, and a disc on the ground under it.
//!
//! Drag the background to orbit (it coasts), the wheel to dolly, drag a gear to turn the whole train;
//! hover a part for its name (and tooth count) on a leader; press a part and it loosens into particles
//! taken from its own surface and forms again; `explode` separates the parts along their axes; the
//! material switch runs the whole machine as ASCII, dots, halftone, lino or chrome. Every one of these
//! has a key (see `key`).
use crate::el::{Drag, El, Key, Paint, Phase, Response, scene3d, space, stack, text};
use crate::mesh::{self, Camera, Library, Light, Mat4, Mesh3d, MeshHit, MeshId, MeshInstances, Pick, Projection, Scene3d};
use crate::morph::{Cmd, MorphSpec};
use crate::style::Material;
use crate::widgets::{Orbit, OrbitMsg};
use std::sync::OnceLock;

/// The parts, by the id their group carries in picks (index + 1).
pub const PARTS: [Part; 8] = [
    Part { name: "后板", en: "back plate", teeth: 0 },
    Part { name: "底板", en: "base", teeth: 0 },
    Part { name: "大齿轮", en: "gear A", teeth: 24 },
    Part { name: "小齿轮", en: "pinion B", teeth: 12 },
    Part { name: "惰轮", en: "idler C", teeth: 18 },
    Part { name: "轴", en: "shafts", teeth: 0 },
    Part { name: "凸轮与从动杆", en: "cam & follower", teeth: 0 },
    Part { name: "螺母与螺栓", en: "nuts & bolts", teeth: 0 },
];
#[derive(Clone, Copy, Debug)]
pub struct Part { pub name: &'static str, pub en: &'static str, pub teeth: u32 }
/// The ground disc's group id (not a part: no hover, no press).
const GROUND: u32 = 99;

/// The materials the switch offers: (name, material). The first is the two-tone drawing.
pub const MATS: [(&str, Material); 6] = [("双色", Material::Plain), ("ASCII", Material::Ascii), ("点阵", Material::Dot), ("半色调", Material::Halftone), ("凸版", Material::Lino), ("金属", Material::Chrome)];

/// The gears: teeth, pitch radius, centre (x, y). They mesh: centre distances are the sums of the radii.
const MODULE: f32 = 0.05;
fn gears() -> [(u32, f32, [f32; 2]); 3] {
    let r = |t: u32| t as f32 * MODULE * 0.5;
    let a = (24, r(24), [-0.62, 0.05]);
    let b = (12, r(12), [a.2[0] + r(24) + r(12), a.2[1]]);
    let ang = 52f32.to_radians();
    let c = (18, r(18), [b.2[0] + (r(12) + r(18)) * ang.cos(), b.2[1] + (r(12) + r(18)) * ang.sin()]);
    [a, b, c]
}

/// The meshes, in the order the runtime is given them (`Ui::add_mesh`): their ids follow the built-ins.
pub fn meshes() -> Vec<mesh::Mesh> {
    let g = gears();
    let cam: Vec<[f32; 2]> = (0..48).map(|i| { let a = i as f32 / 48.0 * std::f32::consts::TAU; let r = 0.16 + 0.07 * (1.0 + a.sin()) * 0.5 + 0.02 * (2.0 * a).cos(); [r * a.cos(), r * a.sin()] }).collect();
    let hex: Vec<[f32; 2]> = (0..6).map(|i| { let a = (i as f32 / 6.0 + 1.0 / 12.0) * std::f32::consts::TAU; [0.055 * a.cos(), 0.055 * a.sin()] }).collect();
    vec![
        mesh::gear(g[0].0, g[0].1, 0.14, 0.045),
        mesh::gear(g[1].0, g[1].1, 0.14, 0.045),
        mesh::gear(g[2].0, g[2].1, 0.14, 0.045),
        mesh::block(1.0, 1.0, 1.0),
        mesh::rod(0.5, 1.0, 24),
        mesh::extrude_bored(&cam, 0.045, 0.1, 30.0),
        mesh::extrude(&hex, [0.0, 0.0], 1.0, 20.0),
    ]
}
/// The library with them, for picking on the CPU (the same ids as the runtime's).
pub fn library() -> &'static Library {
    static LIB: OnceLock<Library> = OnceLock::new();
    LIB.get_or_init(|| { let mut l = Library::default(); for m in meshes() { l.add(m); } l })
}
const BASE_ID: u32 = 10;
fn id(k: u32) -> MeshId { MeshId(BASE_ID + k) }

/// The mechanism's state.
#[derive(Clone, Debug, PartialEq)]
pub struct Mech {
    pub orbit: Orbit,
    /// The big gear's angle (degrees) and its speed while it coasts (degrees a second).
    pub turn: f32,
    pub vturn: f32,
    /// How far apart the parts stand (0 together, 1 apart), and where it is going.
    pub explode: f32,
    pub explode_to: f32,
    /// The part under the pointer, the part chosen with the keyboard, the material, and a part that is
    /// loosening (and since when).
    pub hover: Option<usize>,
    pub chosen: Option<usize>,
    pub mat: usize,
    pub loose: Option<(usize, f32)>,
    /// A gear being turned by the pointer: its angle and the train's at the press.
    pub grab: Option<(f32, f32)>,
    pub last: f32,
}
impl Default for Mech {
    fn default() -> Self {
        let mut orbit = Orbit::default();
        (orbit.yaw, orbit.pitch, orbit.distance) = (-24.0, 16.0, 4.6);
        orbit.min_distance = 1.6;
        orbit.max_distance = 9.0;
        Self { orbit, turn: 0.0, vturn: 0.0, explode: 0.0, explode_to: 0.0, hover: None, chosen: None, mat: 0, loose: None, grab: None, last: 0.0 }
    }
}

/// What the mechanism is told.
#[derive(Clone, Debug, PartialEq)]
pub enum MMsg {
    Drag(Drag),
    Wheel(f32),
    Key(Key, bool),
    Pick(Pick),
    Explode(f32),
    Material(usize),
    /// Turn the train by itself at this speed (degrees a second of the big gear; the ghost).
    Spin(f32),
    /// Orbit by itself at this speed (degrees a second; the ghost).
    Swing(f32),
    Loosen(usize),
}

/// How long a pressed part takes to loosen, fly and form again (s).
const LOOSE_S: f32 = 1.5;

impl Mech {
    /// Each gear's angle: the big one's `turn`, the pinion against it at 24 / 12, the idler with it
    /// at 12 / 18 of the pinion's (meshing gears turn opposite ways; a half-tooth offset so they mesh).
    pub fn angles(&self) -> [f32; 3] {
        let g = gears();
        let a = self.turn;
        let b = -a * g[0].0 as f32 / g[1].0 as f32 + 180.0 / g[1].0 as f32;
        let c = -b * g[1].0 as f32 / g[2].0 as f32 + 180.0 / g[2].0 as f32 + 52.0 * (1.0 + g[1].0 as f32 / g[2].0 as f32);
        [a, b, c]
    }
    /// Whether anything moves by itself (ask for frames while it does).
    pub fn moving(&self, now: f32) -> bool {
        self.orbit.moving() || self.vturn != 0.0 || (self.explode - self.explode_to).abs() > 1e-3 || self.loose.is_some_and(|(_, t)| now - t < LOOSE_S + 0.1)
    }
    /// On every tick: coast, ease the explode, end a loosening.
    pub fn step(&mut self, now: f32) -> bool {
        let dt = (now - self.last).clamp(0.0, 0.1);
        self.last = now;
        let mut moved = self.orbit.step(now);
        if self.vturn != 0.0 && self.grab.is_none() {
            self.turn += self.vturn * dt;
            moved = true;
        }
        if (self.explode - self.explode_to).abs() > 1e-3 {
            let k = 1.0 - (-dt * 7.0).exp();
            self.explode += (self.explode_to - self.explode) * k;
            if (self.explode - self.explode_to).abs() < 2e-3 { self.explode = self.explode_to; }
            moved = true;
        }
        if let Some((_, t)) = self.loose && now - t >= LOOSE_S { self.loose = None; moved = true; }
        moved
    }
    pub fn update(&mut self, m: MMsg, now: f32, cmds: &mut Vec<Cmd>, key: &str) {
        match m {
            MMsg::Drag(d) => {
                if d.phase == Phase::Start {
                    let sc = self.scene();
                    let hit = sc.pick(library(), d.x / d.w.max(1.0), d.y / d.h.max(1.0), d.w / d.h.max(1.0));
                    self.grab = hit.and_then(part_of).filter(|p| (2..=4).contains(p)).map(|_| (d.x, self.turn));
                    if self.grab.is_some() { self.vturn = 0.0; }
                }
                match self.grab {
                    // A gear follows the pointer across it: half a degree of the big gear a pixel.
                    Some((_, t0)) => {
                        let prev = self.turn;
                        self.turn = t0 + d.dx * 0.6;
                        if d.phase == Phase::End { self.grab = None; self.vturn = ((self.turn - prev) * 60.0).clamp(-400.0, 400.0); }
                    }
                    None => self.orbit.update(OrbitMsg::Drag(d), now),
                }
            }
            MMsg::Wheel(dy) => self.orbit.update(OrbitMsg::Wheel(dy), now),
            MMsg::Key(k, shift) => match k {
                Key::Left | Key::Right | Key::Up | Key::Down => self.orbit.update(OrbitMsg::Key(k, shift), now),
                Key::Char('[') | Key::Char(']') => { let s = if shift { 60.0 } else { 15.0 }; self.turn += if k == Key::Char(']') { s } else { -s }; self.vturn = 0.0; }
                Key::Char('+') | Key::Char('=') => self.orbit.update(OrbitMsg::Wheel(-120.0), now),
                Key::Char('-') => self.orbit.update(OrbitMsg::Wheel(120.0), now),
                Key::Char('e') | Key::Char('E') => self.explode_to = if self.explode_to > 0.5 { 0.0 } else { 1.0 },
                Key::Char('m') | Key::Char('M') => self.mat = (self.mat + 1) % MATS.len(),
                Key::Char(',') => self.chosen = Some(self.chosen.map_or(PARTS.len() - 1, |c| (c + PARTS.len() - 1) % PARTS.len())),
                Key::Char('.') => self.chosen = Some(self.chosen.map_or(0, |c| (c + 1) % PARTS.len())),
                Key::Enter => if let Some(c) = self.chosen { self.loosen(c, now, cmds, key) },
                Key::Char(' ') => self.vturn = if self.vturn == 0.0 { 90.0 } else { 0.0 },
                _ => {}
            },
            MMsg::Pick(Pick::Hover(h)) => self.hover = h.and_then(part_of),
            MMsg::Pick(Pick::Press(h)) => if let Some(p) = h.and_then(part_of) && !(2..=4).contains(&p) { self.loosen(p, now, cmds, key) },
            MMsg::Explode(v) => { self.explode_to = v.clamp(0.0, 1.0); }
            MMsg::Material(i) => self.mat = i.min(MATS.len() - 1),
            MMsg::Spin(v) => self.vturn = v,
            MMsg::Swing(v) => { self.orbit.vyaw = v; self.orbit.step(now); }
            MMsg::Loosen(p) => self.loosen(p, now, cmds, key),
        }
    }
    /// Part `p` loosens into particles from its own surface and forms again where it is: a morph from
    /// a copy of it to another copy, over the machine, while the machine leaves it out.
    fn loosen(&mut self, p: usize, now: f32, cmds: &mut Vec<Cmd>, key: &str) {
        if self.loose.is_some() { return; }
        self.loose = Some((p, now));
        let spec = MorphSpec { loosen_ms: 260.0, duration_ms: 820.0, tighten_ms: 360.0, curl: 60.0, stagger: 0.45, count_max: 9000.0, lattice: 2.0, ..MorphSpec::default() };
        cmds.push(Cmd::Morph { from: format!("{key}/lone-a"), to: format!("{key}/lone-b"), spec });
    }

    /// The machine as a scene: every part placed (exploded by `explode`), the hovered or chosen one in
    /// the accent colour, `leave` left out.
    pub fn scene_with(&self, only: Option<usize>, leave: Option<usize>) -> Scene3d {
        let g = gears();
        let ang = self.angles();
        let e = self.explode;
        let hot = self.hover.or(self.chosen);
        let mut items: Vec<MeshInstances> = vec![];
        let paint_of = |p: usize, base: Paint| if hot == Some(p) && only.is_none() { Paint::ACCENT } else { base };
        let mut push = |p: usize, mesh: MeshId, t: Vec<Mat4>, paint: Paint| {
            if only.is_some_and(|o| o != p) || leave == Some(p) { return; }
            items.push(MeshInstances { mesh, transforms: t, paint: paint_of(p, paint), id: p as u32 + 1 });
        };
        // Housing: the back plate behind the gears, the base under everything.
        push(0, id(3), vec![Mat4::translate(0.0, 0.18, -0.24 - 0.5 * e).mul(&Mat4::scale(2.3, 1.7, 0.08))], Paint::tone(0.42));
        push(1, id(3), vec![Mat4::translate(0.0, -0.82 - 0.35 * e, 0.0).mul(&Mat4::scale(2.5, 0.1, 0.9))], Paint::tone(0.3));
        // The gears, each on its axis (z), pushed forward one after another when exploded.
        let gz = |k: usize| 0.0 + e * 0.32 * (k as f32 + 1.0);
        for (k, (_, _, c)) in g.iter().enumerate() {
            let paint = match k { 0 => Paint::INK, 1 => Paint::hue(0), _ => Paint::tone(0.7) };
            push(2 + k, id(k as u32), vec![Mat4::translate(c[0], c[1], gz(k)).mul(&Mat4::rotate_z(ang[k]))], paint);
        }
        // Shafts through the gears, from the back plate forward.
        let shafts: Vec<Mat4> = g.iter().enumerate().map(|(k, (_, _, c))| Mat4::translate(c[0], c[1], -0.06 + gz(k) * 0.5).mul(&Mat4::rotate_x(90.0)).mul(&Mat4::scale(0.06, 0.42 + gz(k), 0.06))).collect();
        push(5, id(4), shafts, Paint::tone(0.55));
        // The cam on the big gear's shaft, in front of it; the follower rides on its top.
        let c0 = g[0].2;
        let cz = 0.16 + gz(0) + e * 0.25;
        let camr = |a: f32| { let a = a.to_radians(); 0.16 + 0.07 * (1.0 + a.sin()) * 0.5 + 0.02 * (2.0 * a).cos() };
        let lift = camr(90.0 - ang[0]);
        let fy = c0[1] + lift + 0.28 + e * 0.2;
        push(6, id(5), vec![Mat4::translate(c0[0], c0[1], cz).mul(&Mat4::rotate_z(ang[0]))], Paint::hue(1));
        push(6, id(3), vec![Mat4::translate(c0[0], fy, cz).mul(&Mat4::scale(0.07, 0.56, 0.07)), Mat4::translate(c0[0], c0[1] + 0.78 + e * 0.2, cz).mul(&Mat4::scale(0.22, 0.05, 0.12))], Paint::INK);
        // Nuts on the shafts' front ends, bolts in the base's corners.
        let mut nuts: Vec<Mat4> = g.iter().enumerate().map(|(k, (_, _, c))| Mat4::translate(c[0], c[1], 0.1 + gz(k) * 1.15 + if k == 0 { 0.2 + e * 0.3 } else { 0.0 }).mul(&Mat4::scale(1.0, 1.0, 0.06))).collect();
        for (x, z) in [(-1.1, -0.3), (1.1, -0.3), (-1.1, 0.3), (1.1, 0.3)] { nuts.push(Mat4::translate(x, -0.75 - 0.35 * e + e * 0.25, z).mul(&Mat4::rotate_x(90.0)).mul(&Mat4::scale(0.8, 0.8, 0.08))); }
        push(7, id(6), nuts, Paint::tone(0.8));
        // The ground: a thin disc under the base (seen as an ellipse), not a part.
        if only.is_none() {
            items.push(MeshInstances::one(id(4), Mat4::translate(0.0, -0.9 - 0.35 * e, 0.0).mul(&Mat4::scale(3.6, 0.005, 2.4)), Paint::tone(0.12), GROUND));
        }
        let spec = Mesh3d { resolution_scale: 1.0, edge_normal: 28.0, tones: if self.mat == 0 { 2.0 } else { 0.0 }, terminator: 0.32, outline: if self.mat == 0 { 1.4 } else { 0.0 }, ..Mesh3d::default() };
        let camera = self.orbit.camera(Camera { projection: Projection::Perspective { fov: 32.0 }, target: [0.05, -0.12, 0.0], near: 0.3, far: 30.0, ..Camera::default() });
        Scene3d { items, camera, light: Light::from_angles(-35.0, 48.0, 0.26), spec, picture: None }
    }
    pub fn scene(&self) -> Scene3d { self.scene_with(None, self.loose.map(|l| l.0)) }
}
fn part_of(h: MeshHit) -> Option<usize> { (h.object >= 1 && h.object <= PARTS.len() as u32).then(|| h.object as usize - 1) }

/// Where a part's centre falls in the view (0..1 each way), for its leader.
pub fn anchor(m: &Mech, p: usize, aspect: f32) -> Option<(f32, f32)> {
    let sc = m.scene_with(Some(p), None);
    let lib = library();
    let mut acc = [0.0f32; 3];
    let mut n = 0.0;
    for it in &sc.items {
        let Some(mesh) = lib.get(it.mesh) else { continue };
        let c = mesh.tris.iter().flatten().fold([0.0; 3], |a, q| mesh::add(a, *q));
        let k = mesh.tris.len() as f32 * 3.0;
        for t in &it.transforms { acc = mesh::add(acc, t.point(mesh::mulf(c, 1.0 / k.max(1.0)))); n += 1.0; }
    }
    if n == 0.0 { return None; }
    let w = mesh::mulf(acc, 1.0 / n);
    let q = sc.camera.proj(aspect).mul(&sc.camera.view()).0;
    let cl = [q[0] * w[0] + q[4] * w[1] + q[8] * w[2] + q[12], q[1] * w[0] + q[5] * w[1] + q[9] * w[2] + q[13], q[3] * w[0] + q[7] * w[1] + q[11] * w[2] + q[15]];
    (cl[2] > 0.0).then(|| ((cl[0] / cl[2] + 1.0) * 0.5, (1.0 - cl[1] / cl[2]) * 0.5))
}

/// The machine in a `w` × `h` box (logical px): the view (drag, wheel, keys, picks), the two copies of a
/// loosening part over it, and the leader with the hovered part's name.
pub fn view<M: Clone + 'static>(m: &Mech, key: &str, w: f32, h: f32, size: f32, msg: impl Fn(MMsg) -> M + Clone + 'static) -> El<M> {
    let mat = MATS[m.mat].1;
    let (a, b, c, d, e) = (msg.clone(), msg.clone(), msg.clone(), msg.clone(), msg.clone());
    let mut v = scene3d(m.scene()).key(format!("{key}/view")).width(w).height(h).focusable(true).label("机件").cursor(crate::el::Cursor::Grab)
        .on_drag_with(move |dr| Some(a(MMsg::Drag(dr))))
        .on_wheel(move |_, dy| Some(b(MMsg::Wheel(dy))))
        .on_key(move |k, sh| matches!(k, Key::Left | Key::Right | Key::Up | Key::Down | Key::Enter | Key::Char('[' | ']' | '+' | '=' | '-' | 'e' | 'E' | 'm' | 'M' | ',' | '.' | ' ')).then(|| c(MMsg::Key(k, sh))))
        .on_pick(move |p| Some(d(MMsg::Pick(p))));
    if mat != Material::Plain {
        v = v.material(mat.id());
        if mat != Material::Chrome { v = v.loose(1.0).legible(false); }
    }
    let mut layers = vec![v];
    // A loosening part: two copies of it alone, over the machine; the morph goes from one to the other.
    if let Some((p, _)) = m.loose {
        let lone = |k: &str, hidden: bool| { let mut l = scene3d(m.scene_with(Some(p), None)).key(format!("{key}/{k}")).width(w).height(h).respond(Response::Loosen).hidden_until_morphed(hidden); if mat != Material::Plain { l = l.material(mat.id()); } l };
        layers.push(lone("lone-a", false).absolute(Some(0.0), Some(0.0), None, None));
        layers.push(lone("lone-b", true).absolute(Some(0.0), Some(0.0), None, None));
    }
    // The leader: from the part's centre up to a label above the machine.
    if let Some(p) = m.hover.or(m.chosen) && let Some((fx, fy)) = anchor(m, p, w / h.max(1.0)) {
        let (px, py) = (fx * w, fy * h);
        let ly = (h * 0.08).max(size);
        let part = &PARTS[p];
        let label = if part.teeth > 0 { format!("{:02} {} · {} · z = {}", p + 1, part.name, part.en, part.teeth) } else { format!("{:02} {} · {}", p + 1, part.name, part.en) };
        let lx = if px > w * 0.5 { (px - size * 16.0).max(0.0) } else { px };
        layers.push(space(1.0, (py - ly - size * 0.4).max(1.0)).fill(Paint::ACCENT).key(format!("{key}/leader-v")).absolute(Some(px), Some(ly + size * 0.4), None, None).respond(Response::Ignore).material(Material::Plain.id()));
        layers.push(space(size * 0.5, size * 0.5).radius(size * 0.25).fill(Paint::ACCENT).key(format!("{key}/leader-dot")).absolute(Some(px - size * 0.25), Some(py - size * 0.25), None, None).respond(Response::Ignore).material(Material::Plain.id()));
        layers.push(text(label, size, Paint::INK).font(crate::el::Font::Mono).key(format!("{key}/leader-label")).nowrap().absolute(Some(lx), Some(ly - size), None, None).respond(Response::Ignore).material(Material::Plain.id()));
    }
    let _ = e;
    stack(layers).key(key.to_owned()).width(w).height(h)
}

/// The keys, for the hint line and the docs.
pub const KEYS: &str = "拖动转动视角 · 拖齿轮转整个轮系 · 滚轮推拉 · 方向键 / [ ] / E 拆开 / M 材质 / , . 选零件 / Enter 散开";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gears_mesh_at_their_ratios() {
        let mut m = Mech::default();
        let a0 = m.angles();
        m.turn += 30.0;
        let a1 = m.angles();
        // The pinion has half the teeth: twice the angle, the other way; the idler turns with the big gear.
        assert!(((a1[1] - a0[1]) + 60.0).abs() < 1e-3, "{:?}", (a0, a1));
        assert!(((a1[2] - a0[2]) - 40.0).abs() < 1e-3, "{:?}", (a0, a1));
        let g = gears();
        let d = |p: [f32; 2], q: [f32; 2]| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt();
        assert!((d(g[0].2, g[1].2) - (g[0].1 + g[1].1)).abs() < 1e-4);
        assert!((d(g[1].2, g[2].2) - (g[1].1 + g[2].1)).abs() < 1e-4);
    }
    #[test]
    fn every_part_is_in_the_scene_and_can_be_picked_or_left_out() {
        let m = Mech::default();
        let sc = m.scene();
        for p in 0..PARTS.len() { assert!(sc.items.iter().any(|i| i.id == p as u32 + 1), "part {p}"); }
        assert!(m.scene_with(None, Some(2)).items.iter().all(|i| i.id != 3));
        assert_eq!(m.scene_with(Some(4), None).items.len(), 1);
        assert!(library().get(id(6)).is_some());
        // The middle of the view is on the machine.
        assert!(sc.pick(library(), 0.45, 0.5, 1.6).is_some());
    }
}
