//! The 风格 page (DESIGN.md §4.1): choose a preset to start from and one to go to, play the transition
//! (always the target's), read a preset as JSON and paste one back, and change any number of its
//! interaction and transition, with number controls made from the preset's own data.
use super::pages::{header, section};
use super::{Gallery, Msg, Target};
use crate::el::{El, Paint, col, row, text};
use crate::preset::Preset;
use crate::widgets::{Edit, Spec, TextEdit, Variant, button_as, choice, number, textarea};
use taffy::AlignItems;

#[derive(Clone, Debug, PartialEq)]
pub enum PreMsg { From(usize), To(usize), Play, Export, Recipe, Import, Json(TextEdit) }

/// The numbers of the target preset that get controls: its interaction, its transition, and its
/// looseness and lattice. Paths as `Preset::numbers` names them.
pub fn editable(p: &Preset) -> Vec<(String, f32)> {
    p.numbers().into_iter().filter(|(k, _)| k.starts_with("interaction.") || k.starts_with("transition.") || k == "loose_scale" || k == "pitch")
        // Only the steps in use.
        .filter(|(k, _)| !k.starts_with("transition.steps.") || k.split('.').nth(2).and_then(|i| i.parse::<usize>().ok()).is_some_and(|i| p.transition.steps[i].kind != crate::preset::StepKind::None))
        .collect()
}
/// A label for a number's path: the step's own names for a–e.
fn label(p: &Preset, path: &str) -> String {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.len() == 4 && parts[1] == "steps" && let Ok(i) = parts[2].parse::<usize>() {
        let k = p.transition.steps[i].kind;
        let names = k.params();
        let idx = ["a", "b", "c", "d", "e"].iter().position(|x| *x == parts[3]);
        let n = idx.map(|j| names[j]).filter(|n| !n.is_empty()).unwrap_or(parts[3]);
        let n = match n { "from" => "开始", "to" => "结束", x => x };
        return format!("{} {} · {n}", i + 1, serde_json::to_value(k).ok().and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_default());
    }
    let last = parts.last().copied().unwrap_or(path);
    let name = match last {
        "press_ms" => "按下反馈", "front_speed" => "波速", "front_width" => "波宽", "front_life" => "波前寿命", "duration_ms" => "时长",
        "stagger" => "错开", "count_max" => "粒子上限", "slide_ms" => "滑动", "loose_scale" => "松开比例", "pitch" => "点距", x => x,
    };
    if parts.contains(&"recompose") { format!("重组 {name}") } else if parts.first() == Some(&"transition") { format!("过渡 {name}") } else { name.to_owned() }
}
fn spec_for(path: &str) -> Spec {
    if path.ends_with("_ms") { return Spec { min: 0.0, max: 5000.0, step: 10.0, decimals: 0, unit: "ms" }; }
    if path.ends_with(".from") || path.ends_with(".to") { return Spec { min: 0.0, max: 1.0, step: 0.05, decimals: 2, unit: "" }; }
    Spec { min: -100_000.0, max: 100_000.0, step: 0.5, decimals: 2, unit: "" }
}

pub fn update(s: &mut Gallery, m: PreMsg) {
    let n = s.presets.len().max(1);
    match m {
        PreMsg::From(i) => { s.pre_from = i.min(n - 1); s.apply_style(s.pre_from); }
        PreMsg::To(i) => s.pre_to = i.min(n - 1),
        // The button is a commit: the frame as it was stays, and the target's transition plays over it.
        PreMsg::Play => s.apply_style(s.pre_to),
        PreMsg::Export => { s.pre_json = TextEdit::new(&s.presets[s.pre_to].to_json()); s.pre_note = format!("{} 的数据（可以复制，改了以后按“导入”）", s.presets[s.pre_to].name); }
        PreMsg::Recipe => { s.pre_json = TextEdit::new(&s.presets[s.pre_to].recipe()); s.pre_note = format!("{} 的配方：和内置的差别，可以放进网址的 # 后面，或粘贴回来导入", s.presets[s.pre_to].name); }
        PreMsg::Import if s.pre_json.text.trim().trim_start_matches('#').starts_with(crate::preset::RECIPE_PREFIX) => match Preset::from_recipe(&s.pre_json.text) {
            Ok(p) => { s.pre_note = format!("已导入配方：{}", p.name); let i = s.presets.iter().position(|q| q.hash == p.hash).unwrap_or(s.pre_to); s.presets[i] = p; s.pre_to = i; }
            Err(e) => s.pre_note = e.to_string(),
        },
        PreMsg::Import => match Preset::from_json(&s.pre_json.text) {
            Ok(p) => { s.pre_note = format!("已导入：{}", p.name); s.presets[s.pre_to] = p; }
            Err(e) => s.pre_note = e.to_string(),
        },
        PreMsg::Json(t) => s.pre_json = t,
    }
}
/// A number of the target preset was set (`Target::Preset(i)`, `i` into `editable`).
pub fn set(s: &mut Gallery, i: usize, v: f32) {
    let p = &mut s.presets[s.pre_to];
    if let Some((path, _)) = editable(p).get(i).cloned() && let Err(e) = p.set_number(&path, v) { s.pre_note = e.to_string(); }
    if s.style == Some(s.pre_to) { s.apply_style(s.pre_to); }
}

pub fn page(s: &Gallery) -> El<Msg> {
    let h = "presets";
    let names: Vec<&str> = s.presets.iter().map(|p| p.name.as_str()).collect();
    let to = &s.presets[s.pre_to.min(s.presets.len() - 1)];
    let play = row().key("presets/controls").wrap().gap(14.0).align(AlignItems::CENTER)
        .child(button_as("presets/play", "播放", Variant::Accent, Msg::Preset(PreMsg::Play)).commit())
        .child(text(format!("{} → {}，{} ms", s.presets[s.pre_from].name, to.name, to.transition.duration_ms), 14.0, Paint::DIM).key("presets/what"));
    let mut nums = row().key("presets/params").fill_width().wrap().gap(10.0);
    for (i, (path, v)) in editable(to).into_iter().enumerate() {
        let t = Target::Preset(i);
        nums = nums.child(number(&format!("presets/n{i}"), &label(to, &path), v, spec_for(&path), s.draft(t), move |e: Edit| Msg::Num(t, e)).width(330.0));
    }
    let json = col().key("presets/json-col").fill_width().gap(10.0)
        .child(row().gap(10.0)
            .child(button_as("presets/recipe", "复制配方", Variant::Default, Msg::Preset(PreMsg::Recipe)))
            .child(button_as("presets/export", "导出 JSON", Variant::Default, Msg::Preset(PreMsg::Export)))
            .child(button_as("presets/import", "导入", Variant::Default, Msg::Preset(PreMsg::Import))))
        .child(textarea("presets/json", &s.pre_json, "配方（look:v1:…）或预设的 JSON：复制，或粘贴一份再导入", 880.0, 6, 20000, |t| Msg::Preset(PreMsg::Json(t))))
        .child(text(s.pre_note.clone(), 13.0, Paint::DIM).key("presets/note"));
    col().key("presets/page").fill_width().max_width(1000.0).gap(30.0)
        .child(header(s, h, "风格切换", "preset::Preset · Transition", "每个风格是一份数据：主题、材质和它的参数、字体、标注、交互的性格，和一段过渡。过渡是几个步骤拼成的小程序（按形状揭开、错位的带、淡入、光条、色块、松开、分层、融化、检测框、刻、先画轮廓、撕开），每一步几个数。切到一个风格、进入这个风格的页面，都用目标风格的过渡；过渡中途再切，从此刻的画面接着过渡；输入一直可用。"))
        .child(section(h, "从 → 到", col().fill_width().gap(12.0)
            .child(choice("presets/from", "从", &names, s.pre_from, |i| Msg::Preset(PreMsg::From(i))))
            .child(choice("presets/to", "到", &names, s.pre_to, |i| Msg::Preset(PreMsg::To(i))))
            .child(play)))
        .child(section(h, "目标风格的数", nums))
        .child(section(h, "数据", json))
        .child(crate::el::space(1.0, 24.0))
}
