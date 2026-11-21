# AGENTS.md — writing interfaces with amenui

amenui is a self-drawn interface framework in Rust (wgpu; the same code runs natively and on the web).
Everything on screen — glyphs, shapes, pictures — is an instance with a true outline that can loosen,
morph and condense again; how it looks is a swappable material. You write a view (state → tree) and an
update (state, message → state); the runtime does layout, input, animation and drawing.

## The mental model

```text
view(&state) → El tree → layout (taffy + parley) → instances (glyph / shape / picture)
  → stage 1: ink + near images (true outlines) → stage 2: material → stage 3: annotation layer
input → hit → message → update(&mut state) → view again        input → field (fronts, lens)
```

There is one way to make things move: events put things in the field (a front from a press, the lens),
and commands ask the runtime (`Cmd::Morph`, `Cmd::Flow`). Widgets never animate themselves.

## The ten things you will actually write

Every block below is compiled by `cargo test` (this file is a doc-test of `amenui::agents`).

1. A view and an update (Elm style; messages are plain values):
```rust
use amenui::el::{El, Paint, col, text};
use amenui::widgets::button;
#[derive(Clone)] enum Msg { Inc }
struct S { n: u32 }
fn view(s: &S) -> El<Msg> {
    col().pad(24.0, 24.0).gap(12.0)
        .child(text(format!("{}", s.n), 32.0, Paint::INK).key("count"))
        .child(button("inc", "Add one", Msg::Inc))
}
fn update(s: &mut S, m: Msg) { match m { Msg::Inc => s.n += 1 } }
let _ = (view(&S { n: 0 }), update as fn(&mut S, Msg));
```

2. A runtime, headless (tests, tools); press by position, read the tree:
```rust
# use amenui::el::{El, Paint, col, text}; use amenui::widgets::button;
# #[derive(Clone)] enum Msg { Inc } struct S { n: u32 }
# fn view(s: &S) -> El<Msg> { col().child(text(s.n.to_string(), 20.0, Paint::INK).key("count")).child(button("inc", "+", Msg::Inc)) }
# fn update(s: &mut S, _: Msg) { s.n += 1 }
let faces = amenui::text::embedded();
let mut ui = amenui::ui::Ui::new(S { n: 0 }, view, update, faces, 400, 300, 1.0);
let (x, y) = ui.node("inc").unwrap().rect.center();
ui.press(x, y);
ui.release(x, y);
assert_eq!(ui.export_tree().find("count").unwrap().text.as_deref(), Some("1"));
```

3. Layout: rows, columns, sizes, gaps, padding (logical px; nothing shrinks unless asked):
```rust
use amenui::el::{El, Paint, row, col, space, text};
let bar: El<()> = row().fill_width().pad(16.0, 8.0).gap(12.0)
    .child(text("Title", 18.0, Paint::INK).grow(1.0))
    .child(space(80.0, 30.0).radius(4.0).fill(Paint::ACCENT));
let page: El<()> = col().fill_width().fill_height().child(bar).child(col().grow(1.0).scroll());
let _ = page;
```

4. Keys: give every element you will look up, animate, or that holds state a stable key:
```rust
use amenui::el::{El, Paint, col, text};
let rows: El<()> = (0..3).fold(col(), |c, i| c.child(text(format!("Row {i}"), 14.0, Paint::TEXT).key(format!("row/{i}"))));
let _ = rows;
```

5. Numbers are number controls, never sliders as the main way (Draft while typing, Set on commit):
```rust
use amenui::el::El;
use amenui::widgets::{Edit, Spec, number};
#[derive(Clone)] enum Msg { Width(Edit) }
const W: Spec = Spec { min: 0.0, max: 64.0, step: 1.0, decimals: 0, unit: "px" };
let draft: Option<String> = None;
let n: El<Msg> = number("width", "Width", 12.0, W, draft.as_deref(), Msg::Width).width(320.0);
let _ = n;
```

6. Text fields keep a `TextEdit` in the state:
```rust
use amenui::el::El;
use amenui::widgets::{TextEdit, input};
#[derive(Clone)] enum Msg { Name(TextEdit) }
let name = TextEdit::new("Lin");
let field: El<Msg> = input("name", &name, "Name", 280.0, 40, Msg::Name);
let _ = field;
```

7. Ripples: a press reaches only its own control unless you say more; decide what loosens:
```rust
use amenui::el::{El, Paint, Response, Scope, row, space};
use amenui::widgets::button;
let area: El<()> = row().key("area").ripplr_scope().gap(16.0)
    .child(button("go", "Press", ()).ripplr(Scope::Container(None)))
    .child(space(60.0, 40.0).fill(Paint::ACCENT).respond(Response::Loosen))
    .child(space(60.0, 40.0).fill(Paint::hue(1)).respond(Response::Ignore));
let _ = area;
```

8. Commands for what moves: keep them in the state, hand them over with `on_commands`:
```rust
use amenui::morph::{Cmd, FlowSpec, MorphSpec};
let mut cmds: Vec<Cmd> = vec![];
cmds.push(Cmd::Morph { from: "a".into(), to: "b".into(), spec: MorphSpec::default() });
cmds.push(Cmd::Flow(FlowSpec::default()));     // before a page change: shared `.flow(id)` parts move
cmds.push(Cmd::Press { key: "go".into() });    // press an element as a person would (not counted as input)
cmds.push(Cmd::Commit { x: 400.0, y: 300.0, scope: None });  // keep the frame, play the new look's transition from here
cmds.push(Cmd::Focus { key: "go".into() });     // focus an element after this layout
cmds.push(Cmd::Scrub(Some(0.5)));               // hold the morph in progress half way (None: let it go on)
assert_eq!(cmds.len(), 6);
```

9. Materials and the look: one global material, overrides per subtree:
```rust
use amenui::el::{El, Paint, text};
use amenui::style::{Look, Material, LIGHT};
let look = Look { theme: LIGHT, material: Material::Dottr, legible_cap: 0.15, ..Look::default() };
let card: El<()> = text("Halftone", 20.0, Paint::INK).material(Material::Halftonr.id());
let _ = (look, card);
```

10. Content that changes recomposes when it is keyed; turn it off where it must be immediate:
```rust
use amenui::el::{El, Paint, text};
let count: El<()> = text("12", 28.0, Paint::INK).key("count");            // recomposes 12 → 13
let clock: El<()> = text("09:41", 14.0, Paint::DIM).key("clock").recompose(false);
let _ = (count, clock);
```

## Fitting, clamping, faces and plates

```rust
use amenui::el::{El, Fit, Paint, Plate, col, text};
// A stage drawn at its own size and zoomed to fit the window (only shrinks, to a quarter at least).
let stage: El<()> = col().key("frame").fill_width().fill_height().fit(Fit::default())
    .child(col().key("stage").width(1440.0).height(900.0));
// At most three lines, ending in the face's own ellipsis; `Ui::clamped("blurb")` says whether it was cut.
let blurb: El<()> = text("A long paragraph to read.", 14.0, Paint::TEXT).key("blurb").clamp(3);
// Ink that stays: what the board's subtree draws is kept, each pixel gone 4 s after it was last drawn.
let board: El<()> = col().key("board").fill_width().fill_height().plate(Plate { fade_s: 4.0 });
let _ = (stage, blurb, board);
```

- `absolute(left, top, right, bottom)` pins against the parent's padding box, never the window.
- A face that comes later: declare it (`text::with_declared`, `Ui::declare_face`), set text in it as usual, and add
  its bytes when they come (`Ui::add_face`); fetch what `Ui::faces_wanted()` lists. Text stands in meanwhile.
- `Ui::clear_focus()` / `Cmd::ClearFocus` after a route change; `El::preview(true)` to show a sheet or menu in place.
- Plates are for music videos, the lab and drawing boards, not ordinary pages; `Cmd::ClearPlate { key }` empties one.
  Put text to read and controls beside a plate (its siblings), never inside it: `Ui::problems` reports `OnPlate`.
- A clamped text copies what it shows, without the ellipsis; the accessibility tree has the whole text.
- `Look::off.ground` turns the field's dots (the ground's own marks) off.

## Meshes, particles, presets

A 3D object is one more element: `el::mesh(MeshId::TORUS)` or `el::scene3d(Scene3d { items, camera, light, spec, picture })`,
sized like any element. Meshes are generated in code (`MeshId::CUBE`, `ICO[0..4]`, `TORUS`…; `Ui::add_mesh`,
`mesh::load_gltf`); `MeshInstances { mesh, transforms, paint, id }` draws many copies in one call. The global
material draws it (its outline and creases are edges in the nearness). Hear the pointer with
`.on_pick(|Pick| ..)` (`Pick::Hover` / `Pick::Press` carry `MeshHit { object, instance }`); turn it with
`widgets::orbit(key, scene, &state.orbit, Msg::Orbit)` and call `Orbit::update` / `Orbit::step` (see
`docs/recipes/mesh.md`). Morphs with many tiles run on the GPU by themselves (`Look::particles.gpu_above`).
A scene that changes into another with shared actors is a commit plus a flow (`docs/recipes/chapter.md`).
`Ui::on_pointer` tells the state where the pointer is.
A style preset is data: `preset::presets()`, change a number (`Preset::set_number("transition.steps.0.a", 3.0)`)
or read JSON (`Preset::from_json`), then `preset.apply(&look)`; a commit (`El::commit`) that applies it
plays its transition (`docs/recipes/preset.md`).
A preset's `post` is the finishing pass (bloom, chromatic offset, scanlines, grain, vignette, curvature, then a
dither and a palette — `amenui::post`, all off by default, DESIGN.md §1.21) and its `marks` make marks on 3D elements
follow depth. `El::shed(Shed)` makes a moving element drop marks, `El::flow_field(FlowField)` carries marks along a
noise field over a box (speed 0: still), and `El::shift(dx, dy)` moves a subtree after layout (pointer parallax
without a layout).
Gestures are state (`amenui::gesture`: `Spring`, `Animated`, `RubberBand`, `project`): `widgets::bottom_sheet` and `widgets::floating`
report drags, `Sheet::update` / `Floating::update` take them with the time, `step` runs on the tick while `moving()`; put them
after the page in `el::layers([...])` (docs/recipes/gesture.md). Under a live demo, `El::caption_params` prints the numbers that
drive it and lets them be edited (DESIGN.md §5.2). `widgets::nested_menu` steers submenus by a safe-zone triangle
(`El::on_move` hears the pointer over an element; `El::guide` draws an outline in the annotation layer). While data
loads, draw the real rows with `.skeleton()` as `widgets::Loading::show` says (docs/recipes/loading.md).
Fields and drawings (DESIGN.md §1.26): `el::canvas(Drawing)` draws dots and strokes in its own space (each a shape with
a true outline; its pointer handlers get points in that space; `El::on_point` hears move, press-drag and leave in one handler);
`amenui::field` samples noise, bumps and metaballs on a `Grid`, traces iso-contours, advects a `Swarm` along `curl` or a stirred `Flow`,
and grows `GrayScott` inside a mask. A text element's glyphs can be posed one by one (`El::pose(|GlyphAt| Pose)`: move, turn,
scale, tone, alpha; no layout). `El::on_visible` says when an element can be seen (step a demo only then), `El::on_outline` sends a
subtree's true outline as a coverage grid (a mask). Every small mark draws from one frame budget (`budget::grant(n)`). `El::decode(Decode)`
makes an element appear by decoding and disappear by the same backwards. The Lab's experiments (`examples/lab/` in the website repository, https://github.com/amenui-dev/website) show each:
`experiment::frame(key, name, hint, piece, formula, &params, msg, label)` puts a piece on a 4:3 stage it fills, the formula on one
line and the numbers behind `[ params ]`; seen cells step every frame (with reduced motion they rest). Sampling (`morph::cover_at`,
`cells`, `on_outline`, decode) follows `Instance::xf` and poses, so a posed glyph loosens from where it is drawn; `cover_at` counts any
fill (a ground-toned panel too), `ink_at` only ink that shows. The lens touches only pictures and 3D elements that may loosen and that
no handler operates. The landing page and first scene, Body (the website's `examples/body.rs`, `#body`), is written the same way: one posed text element (the
skin, its glyph count conserved), drawings for its volume, reflection, dust and the instrument markings, springs for `gather`
and the opening, a signed-distance form sampled to a spiral of surface points (DESIGN.md §6.2). `Pose::hue` picks a glyph's accent.
The field map (DESIGN.md §1.29, `amenui::fieldmap`) is a coarse grid of energy and flow over the window that settles to exactly zero:
an element with `.stirr(Stirr::default())` stirs it behind itself as it moves (floating, bottom sheet and card stack do by default;
`Look::map` / a preset's `interaction.map` set how strong), `Cmd::Stir { at: StirAt::Point | Key | Path, .. }` stirs it from the state,
and it loosens and drifts what may loosen as fronts do (caps and responses unchanged; `Ui::looseness` reads it too). The `scribblr`
material (DESIGN.md §1.30, `amenui::strand`) is plain at rest; loosened, an element's ink becomes one thread of constant width that knots
where the map has energy and trails off along its flow, then writes itself back (`Ui::strands()` lists the threads drawn; `Look::hold_ms`
holds its pose for a beat). `El::trace(Trace { points, progress, weight, dash, gap, pull, stir, paint })` draws a long path by progress
under any material, solid or dashed by arc length, pushed by the map and stirring it at its head.
Materials are named for what they do (`dottr`, `halftonr`, `charr`, `smudgr`, `carvr`, `glitchr`, `scribblr`, plus
`plain` and `near`); an old name such as `halftone` is refused and the error names the new one. Each is one WGSL file
(`src/materials/<name>.wgsl`, its numbers in a header, its own numbers read as `mp(SELF, i)`): `amenui add material
<name>` copies one, and `Look::material_source(name, wgsl, params)` adds a changed copy under a name of its own before
the first window or `Offscreen` is made. `page.effect(Effect { material, response, scope })` puts a material, a response
and a scope on any subtree in one line, and `el::wrap_text(src)` makes a page to read from plain text or Markdown.
Small type: `.tracking(em)` spaces letters (laid out by parley); `.whisper()` is lowercase, 0.6 em, in the bundled Latin serif
(`Font::Whisper`, `text::SERIF`). The annotation layer's scope-wide readouts (DESIGN.md §4.2–§4.3): `gaze::EDGE` edge notes (true values
only: scope size, pointer, material and theme, loosest element, non-zero map nodes), `gaze::OSC` a waveform of the map's energy along one
row, `gaze::LEVEL` a bar of its mean; `Ui::readout()` returns what they drew and `export_tree()`'s root has `field: { level, nonzero }`.
The tenth preset, Loose Thread (`loose-thread`), puts these together; `post.binary` (black and white only) and `post.invert` are
finishing-pass switches.
Text on a form (DESIGN.md §1.26): a pose has a depth and a facing (`Pose::z`, `Pose::facing`) and glyphs draw far to near;
`El::form(FormFx { back, edge, overlap, fog })` leaves out what faces away, fades the rim and thins glyphs nearer ones cover;
`form::Form { shape, spin, camera, cover }` wraps text onto a sphere, a surface or a mesh (`.pose(move |g| form.pose_of(g))`) and
`Form::hit(x, y, w, h)` finds the pointer on it (`docs/recipes/form3d.md`). `El::reveal_by(Reveal::Word)` shows words whole.
`Ui::glyph_at(x, y)` is the glyph drawn there (`Point::glyph` in `on_point`); `El::turn(radians)` turns an element, and it is
pressed where it is drawn. Slow one-off work goes through `Ui::prepare(key, |s| progress)` and `Ui::ready(key)`; the renderer
warms up with `Renderer::begin` and `Warmup::step` (DESIGN.md §1.23).
Text flows around true outlines with `.around(Around::new(vec![Silhouette::circle(..)], gap))`; a piece of a line
beside an obstacle is justified so both its edges are flush, adding at most `Around::justify_max` ems per gap
(default 0.25; inter-character next to CJK, inter-word at spaces, between Latin letters only with
`Around::justify_latin_letters`); Latin pieces then take up to `Around::track_max` (0.08 em) between letters and, if still short,
hyphenate the next word (`Around::hyphenate`, on by default); a piece that would need more stays start-aligned (DESIGN.md §1.2, "around").

## The rules

- One way to do each thing: state in the state, changes in `update`, motion through the field and `Cmd`.
- Key everything you will find again (tests, `--events` targets, flows, morphs, recomposition). Keys must
  be unique on a page; `Ui::problems()` lists duplicates and says what to write.
- Every numeric parameter is a named field with a default and is shown as a number control.
- Text to read and controls stay legible: fronts nudge them and loosen them at most `Look::legible_cap`.
  Mark decoration `respond(Response::Loosen)`; mark a showcase `legible(false)`.
- No third-party or character images in the repository.
- Verify with pictures *and* data: render a frame and read its tree.
- A `Ui` you drive by hand lays out at each input. To see what a host does with several moves a frame, turn
  on `ui.coalesce(true)` and call `ui.settle()` (or draw) before reading the tree.

## How to verify

```sh
cargo run --release --bin render -- --page recipe-form --size 1280x800 --at 1000 --out out.png --tree out.json
cargo run --release --bin render -- --page recipe-counter --events inc.json --at 300 --out c.png --tree c.json
cargo run --release --bin render -- --page recipe-form --audit --strict   # what is wrong with the frame, most serious first
cargo run --release --bin render -- --page component-button --audit --strict   # a component's examples (examples/components/), one page
```

In Claude Code this file is also the `amenui` skill (`.claude/skills/amenui/SKILL.md`, made from it by
`tools/skill.sh`; `cargo test` fails when they differ). The repository's `.claude/settings.json` runs
`tools/audit-hook.sh` after every Edit or Write: a change to a recipe (`examples/<name>.rs`), a component's
examples (`examples/components/<name>.rs`) or its widget (`src/widgets/<name>.rs`) renders that one page with
`--audit --strict`, and its P0 and P1 findings and the view's problems come back to you as the hook's output.
Other files are left alone.

`inc.json`: `[{"at_ms": 100, "kind": "press", "target": "inc"}, {"at_ms": 130, "kind": "release", "target": "inc"}]`
(kinds: press, release, move — `x`, `y` or `target`; key — `key` as a DOM name, `shift`, `cmd`, `alt`;
wheel — `dx`, `dy`; text — `text`). The clock is fixed at 60 Hz from 0, so the same arguments give the
same bytes. The tree (`--tree`, or `Ui::export_tree()`) has each element's key, kind, role and label,
rect, text, material, response, legibility, focus / hover / press, scroll offset and looseness.
`--material` takes `plain`, `dottr`, `halftonr`… (or a number), `--theme` takes `dark`, `light`, `paper`…
`--audit` measures the frame (`amenui::audit`, DESIGN.md §1.31): text cut off, contrast by WCAG, glyphs over each other, text
under 9 device px, a page that never settles; each finding names the element, the number and what to do.

## Commands

```sh
cargo test                                   # everything that needs no GPU (and a few that use one if present)
cargo clippy --all-targets -- -D warnings    # no warnings allowed
cargo run --release --bin render -- --help   # one frame + tree
cargo run --release --bin sheet -- sheets    # contact sheets of the picture marks
cargo run --release --bin amenui -- add dialog --to ../my-app   # copy a component (and what it needs) into a project
cargo run --release --bin amenui -- add material smudgr        # copy a material's WGSL file (src/materials/smudgr.wgsl)
cargo run --release --bin sheet -- --material scribblr --loose 1,0.5,0 --arrival --out s.png   # a material's sample sheet, checked
```

The native window (`window`), the GPU pixel acceptance (`check`, lavapipe works), filmstrips (`film`) and
the benchmarks (`bench`) are binaries of the website repository, https://github.com/amenui-dev/website.

## Where things live

- `src/el.rs` the tree (`El`, builders), `src/ui.rs` the runtime, `src/scene.rs` instances and the field,
  `src/morph.rs` morphs, flows, recomposition, `src/style.rs` themes, materials, `Look`,
  `src/shader.wgsl` the three passes, `src/text.rs` type, `src/tree.rs` the exported tree,
  `src/drive.rs` scripted input and `render`, `src/error.rs` errors.
- `src/widgets/` one file per widget (header: purpose, props, states, messages, usage); `registry.json`.
- `examples/` the recipes, `docs/recipes/` their pages; the site (the gallery, the Lab, the scenes, the docs pages) is the separate repository https://github.com/amenui-dev/website.
- `docs/DESIGN.md` the contract (read §1.15–§1.17 for ripples, recomposition and this surface).
