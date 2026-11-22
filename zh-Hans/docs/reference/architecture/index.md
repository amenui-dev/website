# 架构



amenui is one Cargo package: a library, the `amenui` command line and two tools. Everything except
`render.rs` and the shaders runs without a GPU, so the runtime can be driven and tested headless.
[DESIGN.md](../DESIGN.md) gives the reasons and the numbers; this page says where each part lives.

## The pipeline

```text
view(&state) → El tree → layout (taffy, parley) → instances (glyph, shape, picture, mesh)
  → G-buffer stage: coverage buffer + signed distance field, from true outlines
  → screen-space shading stage: the material
  → annotation layer → finishing pass (post)
input → hit test → message → update(&mut state) → view again
```

A press or the pointer lens puts energy into the wave field; commands in the state (`Cmd::Morph`,
`Cmd::Flow` and the rest in `morph.rs`) ask the runtime for transforms. Widgets never animate themselves.

## Modules

### Describing and running an interface

| Module | Responsibility |
| --- | --- |
| [`el`](../../src/el.rs) | The element tree (`El`), its builders, roles and accessibility data |
| [`ui`](../../src/ui.rs) | The runtime: state, view, incremental layout, hit testing, focus, scrolling, text editing, the accessibility mirror, when to draw |
| [`text`](../../src/text.rs) | Shaping and line breaking with parley, the glyph atlas, the bundled faces |
| [`wrap`](../../src/wrap.rs) | Text that flows around the true outline of pictures and shapes |
| [`i18n`](../../src/i18n.rs) | Message catalogs, plural rules, number and date formats, locale direction |
| [`gesture`](../../src/gesture.rs) | Springs, rubber banding and throw projection for drags |
| [`markdown`](../../src/markdown.rs) | A small Markdown subset rendered as elements |
| [`error`](../../src/error.rs) | Errors on the public path, each saying what to write instead |
| [`widgets`](../../src/widgets/mod.rs) | One file per widget, distributed as source through `registry.json` |

### What is drawn

| Module | Responsibility |
| --- | --- |
| [`scene`](../../src/scene.rs) | Instances and the wave field: what the renderer draws |
| [`render`](../../src/render.rs), [`shader.wgsl`](../../src/shader.wgsl), [`materials`](../../src/materials.rs) | wgpu: the G-buffer stage, the material stage (each material a file in `src/materials/`), the annotation layer |
| [`style`](../../src/style.rs) | Themes, materials, their parameters, `Look` |
| [`preset`](../../src/preset.rs) | Style presets as serializable data, with their transitions |
| [`post`](../../src/post.rs) | The finishing pass: bloom, chromatic offset, scanlines, grain, dither, palette |
| [`marks`](../../src/marks.rs) | The mark atlas and palettes pictures loosen into, made on the CPU at start-up |
| [`gaze`](../../src/gaze.rs) | The annotation layer: frames, numbers, sizes, leaders, rulers and readouts |
| [`dirty`](../../src/dirty.rs) | Drawing only the boxes that changed |
| [`perf`](../../src/perf.rs) | Frame times, counters and the automatic quality level |

### Motion

| Module | Responsibility |
| --- | --- |
| [`morph`](../../src/morph.rs), [`particles.wgsl`](../../src/particles.wgsl), [`sim_cs.wgsl`](../../src/sim_cs.wgsl) | Morphs, flows between pages, recomposition of changed text; the GPU particle simulation |
| [`fieldmap`](../../src/fieldmap.rs) | The energy grid over the window that trails stir and that settles to zero |
| [`strand`](../../src/strand.rs) | The `scribblr` thread and long paths drawn by progress |
| [`motes`](../../src/motes.rs) | Small marks the runtime moves by itself: shed marks and flow fields |
| [`decode`](../../src/decode.rs) | The decode transition |
| [`budget`](../../src/budget.rs) | The frame budget every small mark draws from |

### Drawings, fields and 3D

| Module | Responsibility |
| --- | --- |
| [`draw`](../../src/draw.rs) | `el::canvas`: dots and strokes with true outlines in their own coordinate space |
| [`field`](../../src/field.rs) | Noise, bumps, metaballs, curl, contours, swarms and Gray–Scott on a grid |
| [`mesh`](../../src/mesh.rs), [`mesh.wgsl`](../../src/mesh.wgsl) | Generated meshes and a glTF subset, drawn off screen and resolved into coverage and distance |
| [`experiment`](../../src/experiment.rs) | The frame the website's Lab experiments sit in |

### Tools and data for tools

| Module | Responsibility |
| --- | --- |
| [`tree`](../../src/tree.rs) | The laid-out tree as data (`Ui::export_tree`) |
| [`drive`](../../src/drive.rs) | Scripted input on a fixed 60 Hz clock, behind the `render` tool and the tests |
| [`offscreen`](../../src/offscreen.rs) | Rendering without a window, for the native tools |
| [`app`](../../src/app/mod.rs) | Running an application: `app::native::run` (winit, feature `window`), `app::web::Host` and `web_app!` (wasm), the `App` hooks |
| [`recipes`](../../src/recipes.rs) | The recipes in `examples/`, compiled into the library |
| [`sources`](../../src/sources.rs) | `AGENTS.md`, `llms.txt`, `registry.json`, the changelog and widget sources, compiled in |
| [`pictures`](../../src/pictures.rs) | Sample pictures computed in code |
| [`add`](../../src/add.rs) | Resolving a component and its dependencies from the registry |

## Binaries

| Binary | Purpose |
| --- | --- |
| [`amenui`](../../src/bin/amenui.rs) | `amenui add <component>… [--to <dir>] [--force]` |
| [`render`](../../src/bin/render.rs) | One deterministic frame of a recipe, and its tree |
| [`sheet`](../../src/bin/sheet.rs) | Contact sheets of the picture mark sets and their options |

The native window, the web build, the GPU acceptance check and the benchmarks are in the
[website repository](https://github.com/amenui-dev/website), which depends on this one.
