# 8. Layout

```
src/el.rs        the description
src/text.rs      shaping, multiple fonts, pixel fonts, the glyph atlas
src/scene.rs     instance records, the field, Fx
src/morph.rs     morphs and flows: parameters, sampling, pairing, particle records (no GPU)
src/style.rs     themes, materials, Markr, Look
src/marks.rs     the mark atlas and palettes (generated on the CPU)
src/ui.rs        the runtime: layout, hit testing, focus, scrolling, birth times (no GPU)
src/render.rs    wgpu: the intermediate buffers, the two passes
src/shader.wgsl  the shaders
src/particles.wgsl the particle pass
src/widgets/     components
src/gaze.rs      the annotation layer: which elements, numbering, label placement (no GPU)
src/wrap.rs      setting text around outlines: silhouettes, the free spans of each band (no GPU)
src/perf.rs      frame times, counters, automatic quality (no GPU)
src/offscreen.rs offscreen rendering and readback (for native tools)
src/pictures.rs  sample pictures (generated in code)
src/sources.rs   text and component sources compiled into the library (for the docs site and tools)
src/bin/sheet.rs the mark sheet sample
src/bin/render.rs one frame and its tree (for models, §1.17)
src/mesh.rs      3D: matrices, meshes, a glTF subset, scenes, picking (no GPU)
src/mesh.wgsl    the mesh pass and resolve
src/sim_cs.wgsl  the compute shader for particle simulation (follows particles.wgsl)
src/dirty.rs     drawing only what changed: the two boxes (no GPU)
src/preset.rs    presets and transitions (data)
src/drive.rs     scripted input, opening a page or recipe, the core of render
src/tree.rs      the exported tree
src/error.rs     errors on public paths
src/recipes.rs   recipes (the files in examples/)
examples/        recipe code; docs/recipes/ holds the recipe pages
AGENTS.md, llms.txt  entry points for models
registry.json    the component list (§3)
tests/           the framework's tests
```

The website (the gallery shell and its pages, the skeleton demo, the code for the body and the Lab, the site docs, the web
shell and its packaging script, the native window, and the sample, acceptance and timing tools pages, scenes, film,
samples, shot, check and bench) is in a separate repository, [amenui-dev/website](https://github.com/amenui-dev/website),
with the same file paths as written here.
