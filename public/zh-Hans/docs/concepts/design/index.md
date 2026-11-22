# 设计



A self-drawn interface framework. Anything on screen (text, pictures, shapes, controls) can loosen along
its own true outline, morph, and come back together; the look is a swappable material. Rust and wgpu,
one codebase for native and web (WebGPU, falling back to WebGL2 when there is no WebGPU adapter or its device
fails, the canvas left untouched until a WebGPU device is there; WebGPU only without the `webgl` feature, where
a browser without it fails to start with `app::web::StartError::Adapter`, which the module's `start_failure()`
export and `web/amenui.js`'s `opts.onFail(kind, message)` tell the page).

This document is what the implementation follows. A design change starts here.

## Terms

| Term | Meaning | In the code |
| --- | --- | --- |
| Coverage buffer | RGBA8 render target: tone, accent and a type code plus anti-aliased coverage; pictures write premultiplied colour | `ink` |
| Signed distance field (SDF) buffer | RGBA8 target, max-blended: R signed distance (0.5 on the outline, ±`SPREAD` px), G picture slot, B looseness, A material override or print pitch | `near` |
| G-buffer stage | The first pass: every instance drawn into the coverage and SDF buffers | stage 1 |
| Screen-space shading stage | The second pass: the material reads both buffers and the wave field and writes the frame | stage 2 |
| Looseness | How far an instance has come apart, 0 (intact) to 1 | `loose`, `loosen` |
| Wave field | Ripple fronts, the pointer lens and other sources, evaluated analytically | `Field`, `Fx` |
| Front | One expanding ripple ring from a press or a commit | `Front` |
| Mark | A small pattern a loosened picture is redrawn with (a character, a dither cell, a stroke…) | `marks` |
| Field map | A coarse energy and flow grid over the window | `fieldmap` |

## 0. Three rules

1. **No boxes.** Every effect follows true outlines (the edges of glyphs, shapes and pictures), never a
   text box or a grid cell.
2. **One way to make things move.** Events put things into the wave field; the field and each
   instance's own clock decide the picture. Components never animate directly.
3. **Calm at rest, generous in change.** The pointer passing over text leaves the text alone. An
   interface that has stopped is a clean, readable page.


## 2. Parameters must be exact

Sliders are not the main means. The number control (`number`):

- Shows the current value and unit, with a step button on each side.
- Once focused, type a number directly; Enter confirms, Esc cancels.
- Up and Down change it by a step; with Shift, by ten steps.
- Dragging sideways across the value scrubs it continuously (a secondary means).

Every parameter of `Fx` is exposed through it: the text pitch, irregularity, misregistration, wave speed, wave
lifetime, wave width, trail length, push, lens radius and condensing time. The picture pitch and the other picture
parameters are in `Markr`, each with a number control as well.
