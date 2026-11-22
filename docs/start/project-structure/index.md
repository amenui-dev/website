# Project structure


Where things are in the amenui repository. The crate is one library with a few command-line tools; this site, its gallery and the Lab live in a separate repository, `amenui-dev/website`, which depends on amenui like any application would.

## The runtime

| Where | What |
|---|---|
| `src/el.rs` | the element tree and its builders |
| `src/ui.rs` | the runtime: layout, input, focus, scrolling, animation |
| `src/app/` | opening a window: `native.rs` (winit, AccessKit in `access.rs`) and `web.rs` (a canvas) |
| `src/style.rs`, `src/preset.rs` | looks, themes and presets as data |
| `src/i18n.rs`, `assets/i18n/` | languages and message catalogs |
| `src/text.rs`, `assets/` | text shaping and the bundled faces |
| `src/tree.rs`, `src/drive.rs`, `src/audit.rs` | the laid-out tree as data, driving a page from outside, and the audit |

## Drawing

| Where | What |
|---|---|
| `src/scene.rs`, `src/render.rs` | what is drawn, and the wgpu renderer that draws it |
| `src/shader.wgsl` | the drawing passes: instances, material, annotation |
| `src/materials/*.wgsl` | one material each (`plain`, `dottr`, `halftonr`…) |
| `src/mesh.wgsl`, `src/particles.wgsl`, `src/sim_cs.wgsl` | 3D objects, particles and the compute pass that moves them |
| `src/offscreen.rs` | rendering without a window, for the tools and tests |

## Components and tools

| Where | What |
|---|---|
| `src/widgets/` | the components, one file each |
| `registry.json` | the component registry that `amenui add` reads |
| `examples/` | the recipes, small complete applications |
| `src/bin/` | `amenui` (add components to a project), `render` and `sheet` |
| `web/` | the JavaScript side of the web build |
| `docs/DESIGN.md` | the design contract |
| `tests/` | the test suite |
