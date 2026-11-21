# 1.6.4 Materials as source

Each material is one WGSL file, `src/materials/<name>.wgsl`, and the material pass's shader is the framework's own
(`shader.wgsl`) with every material's file put in at its place in id order (`materials::shader()`). A file begins
with its name and its numbers, then its function:

```text
// @material smudgr
// @param 纸纹 0.0 1.0 0.05 2 - 0.6        label, min, max, step, decimals, unit (- for none), default
...
fn m_smudgr(p: vec2f) -> vec3f { … mp(SELF, 0) … }
```

Every WGSL source the renderer compiles is written to the grammar, not to what naga accepts: Chrome's compiler
(Tint) throws out a module that mixes `||` and `&&` without parentheses, puts a bitwise `&`, `|` or `^` or a
shift beside another operator without them, or compares a comparison, and naga takes all of these. A token
scanner holds the material pass's shader (with every material, so its specialised, gated and override
pipelines too), the mesh pass and the particles to it (`tests/wgsl_tint.rs`).

The header matches `style::MAT_PARAMS` (a unit test holds them equal), and a material reads its own two `vec4f`s
of numbers as `mp(SELF, 0)` and `mp(SELF, 1)`. `registry.json` lists the materials under `materials`, and
`amenui add material <name>` copies a file into a project. `Look::material_source(name, wgsl, params)` adds a
material from source under a name of its own: it takes the next free id (9–15, `Material::Custom(id)`), its
functions are renamed for the name (`m_<its header name>` becomes `m_<name>`, any other function `f` becomes
`f_<name>`), `SELF` becomes its id, and the shader dispatches it on that id with `params` as its numbers. Materials
are added before the first window or `Offscreen` is made (a renderer puts its shader together once; a material
added later draws as plain there); adding a name again replaces its source and keeps its id. An added material's
scroll trace is plain's, and its redraw region is the whole window (what it reads around a pixel is not known).

Acceptance: the shader put together from the files was the shader of before, byte for byte, when the files were
split out; `tests/add.rs` copies `smudgr` out with `amenui add material`, adds it back with `material_source` and
draws the same frame as the built-in one, byte for byte; `tests/registry.rs` reads every material back by its name,
refuses each old name with the new one in the error, and refuses a `look:v1:` recipe with `v3` in the error.
