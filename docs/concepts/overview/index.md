# Overview
## The tree and messages


After every message the runtime calls `view(&state)` once, gets a new `El` tree, compares it with the last by key and lays out again only what changed. The state is the only source; components never move by themselves.

### Commands

What the runtime should do (a morph, a flow, focus, scroll somewhere) goes into commands kept in the state and handed over with `Ui::on_commands`: `Cmd::Morph`, `Cmd::Flow`, `Cmd::Focus`, `Cmd::ScrollTo`, `Cmd::Commit`.


## Layout


Layout is taffy's flexbox, in logical pixels; a `stack()` lays its children over one another and keeps its size; elements do not shrink unless asked. `row()`, `col()`, `stack()`, `space(w, h)`; `.gap`, `.pad`, `.grow`, `.fill_width`, `.scroll()`. `El::rtl(true)` makes a subtree run right to left. `El::zoom(k)` multiplies every length in a subtree by `k`.


## Ripple rules


A press sends a front that by default reaches only the control pressed (`Scope::Own`); `Scope::Container` and `Scope::Global` reach further. Every element answers in its own way: `Ignore`, `Nudge`, `Loosen`. Text to read and controls loosen at most to `Look::legible_cap`.


## Transforms


Two kinds: a morph (`Cmd::Morph`: one element breaks into cells that fly onto another's shape) and a flow (`Cmd::Flow`: what two pages share moves to its new place on a page change). Both go through the same two passes; the material draws what is in flight as usual.


## Presets


A style preset is data: theme, material and its numbers, annotation, interaction character, how scrolling looks, and a transition program. `Preset::apply` gives the look; `Preset::set_number("transition.steps.0.a", 3.0)` changes one number.

There are ten. The tenth, Loose Thread, is black and white with one accent on the `scribblr` material: clean type at rest, threads where the pointer stirs the field map, edge notes and corner frames on hover, a 700 ms transition that loosens into thread and then draws outlines first. The finishing pass has two switches for it: `post.binary` (black and white only) and `post.invert`.


## 3D


`el::scene3d(Scene3d)` is one more element on the page: meshes are drawn off screen into shade, normals and depth, then become ink and nearness like glyphs, so every material can draw them. Generated primitives include gears, extruded profiles, creased boxes and rods; `Mesh3d::tones = 2` gives two flat tones, `outline` draws outlines. See [the 3D page](#three).


## Particles


When a morph has many cells it is simulated on the GPU (a compute shader, or ping-ponging float targets), otherwise on the CPU, where a mesh side's cells start on its surface. `Look::particles.gpu_above` decides where the GPU takes over.
