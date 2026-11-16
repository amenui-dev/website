# Project structure

| Where | What |
|---|---|
| `src/el.rs` | the element tree and its builders |
| `src/ui.rs` | the runtime: layout, input, focus, scrolling, animation |
| `src/shader.wgsl` | the three drawing passes: instances, material, annotation |
| `src/widgets/` | the components, one file each |
| `src/i18n.rs`, `assets/i18n/` | languages and message catalogs |
| `src/gallery/` | this documentation, the Lab and the scenes |
| `docs/DESIGN.md` | the design contract |
| `docs/site/` | the source of these pages (both languages) |
| `registry.json` | the component registry |
