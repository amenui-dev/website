# 1.12 The frame-time readout and the resolution budget

**The readout** (the first item of the top bar's Performance menu, or F2; `Look::overlay`): the annotation
layer draws a block above the material in Departure Mono 11 px (on the page background at 0.92 opacity), in the
corner `Look::overlay_at` names (`Corner::BottomRight` by default), `Look::overlay_inset` ([12, 12] px) in from its
edges. When the block there would cover text to read, it moves on clockwise to the first corner where it covers none
(`Ui::legible_ink`, `Ui::overlay_box`); when every corner covers some, it stays in the asked one:

```
F2 frames       58/s  drawn 1234 skipped 56
ms               last    avg    max
update+layout    0.42   0.50   2.10
scene build      0.80   0.91   3.02
gpu submit       1.20   1.13   4.00
total            ...
instances 1234  particles 0  glyph atlas 12%
layouts 12  shaped 345  views 40
quality 0.5 auto  WebGL2  dpr 2
```

The data (`src/perf.rs`, no GPU): `Perf.pending` is the time spent since the last frame, added to by the
runtime. `update+layout` is `update`, `view` and layout (timing the outermost `send` / `relayout`, minus the
scene building inside it); `scene build` is `paint` and `finish` (building instances, animation, corner marks,
annotations); the renderer's CPU time in `draw` from the start to the last `queue.submit` is `gpu submit`;
then `Perf::frame` records the three as one frame (keeping only the last two seconds).
`Perf::summary(now)` gives the frames drawn in the last second and each item's last / mean / max in ms.
Time comes from `perf::now_ms()`: `performance.now()` on the web, `Instant` natively. There are also work
counters (`Counters`: layouts, parley layouts, `view` calls, scenes), used by tests too. The backend name and
device pixel ratio are filled in by the host (`Perf::backend`: WebGPU, WebGL2, Vulkan, Metal). While the
readout is open it wakes every 0.5 s to refresh, so the frame count falls to 0 at rest; when closed it
doesn't wake. On the web the readout's on / detailed / off state is kept in `localStorage`
(`amenui.readout`, every read and write in try / catch so it runs as usual when storage is unavailable;
`readout()`, `set_readout(v)`); natively it isn't kept.

**The resolution budget** (`Look::quality`: 1, 0.75, 0.5, or 0 = automatic). The screen-space shading stage
can first draw into an image at `q` times the resolution (`lo`, `ceil(width × q) × ceil(height × q)`, in the
frame's format): `fs_material` uses `p = frag × (1 / q)` (`qual.x`), so pixel centers of the small image land
on full-resolution coordinates and lengths, pitches and sampling in the material functions don't change.
Then the full-resolution `fs_material_up` decides per pixel:

- **Pixels at rest** (`rest_at(p)`: on a picture `pic_k(p) = 0`; elsewhere `kt(p) < 0.02`, no burst, lens
  < 0.02) are recomputed by the material at full resolution (`shade(p)`). For materials that equal `plain` at
  rest, that just reads the coverage buffer; for materials with a look of their own at rest (dottr's ground,
  smudgr's grain, `near`), their resting look is at full resolution too. So a still page is identical pixel
  for pixel to `q = 1`, and text is always sharp.
- **Loose pixels** take `lo`'s bilinearly filtered colour. Loose things (marks, scattered dots, carved cuts)
  are changing anyway, and half resolution doesn't show, so the cost drops to `q²`.

Pixels at rest are not shaded in `lo`; they only write 1 into alpha (loose pixels are shaded and write 0).
The full-resolution pass takes one bilinear sample of `lo`: with alpha > 0 (a resting cell nearby) it
recomputes with `shade(p)` here, otherwise it uses the sampled colour. So the edges of loose areas stay
sharp, and the rest test runs once over `q²` of the pixels.

**A material pass for one material** (`Renderer::specialise`): `shade` dispatches on each pixel's material
(an override's, else the global one), so its pipeline holds every material's branch, which a GPU runs with
fewer pixels at once. A frame with one built-in global material and no instance overriding it (in the
scene or drawn into a plate since it was cleared) draws the material pass (`fs_material`, both the full and
the reduced one) with a pipeline of its own made with the WGSL override constant `MAT` set to that
material, and `fs_material_up` and `fs_trace` too; `eff_mat` then returns `MAT`. The passes are made ahead,
from the renderer's `render::SPECIALISE_AFTER` (2nd) frame drawn, one a frame after the frame is sent (WebGPU's
`createRenderPipelineAsync` is not reached through wgpu), and in idle time once the warm-up is done
(`Renderer::warm_idle` goes on while `Renderer::preparing`, within its budget, for what the last frame drew): the
global material's pass, upsample and trace pass, then the passes for each material of the page (below), the
page's materials being every override its elements have, drawn or hidden (`Scene::mats`: a word waiting for a
morph in a material of its own). So a page at rest has made, before its first scroll or hover, every pass that
scroll or hover draws with (`tests/warmup.rs`). Each is kept by material, so a page that changes its global material
and back, or shows an override and hides it, makes nothing again. A frame whose pass is not made yet, and any
with a material from source, draws with the pass for any material (the readout's `spec no (warming)`). While
a morph's tiles fly, the frame's materials include both its ends' (the tiles show the word's material). The frames
are the same pixel for pixel (`tests/specialise.rs`). Offscreen, on the CPU rasteriser, `specialise` is
`None`: compiling another pass costs more there than it saves. An override in the global material is not
one. A page of several built-in materials (up to `render::GATED_MAX`, 4: the gallery's plain chrome round
cells in their own) gets a pass for each, made ahead as above (each material's pass, upsample and trace
pass), with `MAT` set and the
override constant `ONLY`: every pixel finds its material as the pass for any finds it (where a scroll holds
the rows to a lattice, at the row it reads), and the pass for `MAT` draws only the pixels of `MAT`
(`shade_new` discards the rest), so each box is drawn once per material and each pixel by one pass, and the
frame is the frame of the pass for any, byte for byte (`tests/specialise.rs`: at rest, loose and scrolled
fast). The readout says `spec each dottr+plain+charr`. For measuring what the pass for any
material costs against one material's (an A/B on one page, on the device), `Renderer::spec_force`:
`Some(true)` makes the global material's pass from the next frame and uses it even over overrides (their
pixels are then drawn in the global material), `Some(false)` never makes it nor uses one already made, nor
the passes for each material; the readout says which
(`spec yes …`, `spec no (forced off)`).

`lo` is drawn only when the loose area is large enough: `Ui::loose_estimate(now, lens_radius)` estimates the
share of the window that may be loose (1 with fronts, condensing, morphs or flows; otherwise the sum of the
areas of instances with static looseness, bursts, the lens on pictures and compare wipes). Below
`LO_MIN_AREA = 0.15` there is a single full-resolution pass: for one lens or one held control, the extra pass
costs more than it saves. Particles in flight always count as large. `Renderer::always_lo` lets acceptance
force `lo`.

**Automatic** (`perf::AutoQuality`): the level goes by the intervals between frames, since what is slow is
often the GPU, whose time is not measured on the web (on an M1 the CPU's share of a 67 ms scroll frame was
about 10 %, so a CPU-time rule never stepped). The display frame is the 10th percentile of the last second's
intervals, never under 4 ms; an interval over 250 ms is a pause, not a frame. Only frames drawn while something
moves count (scrolling, or a loose area of at least `LO_MIN_AREA`): one level down (1 → 0.75 → 0.5) when the
95th percentile of the last 30 of them is above 1.5 display frames or 20 ms, whichever is less; one level up
after 2 s of them with no missed frame and every frame's CPU cost under 60 % of the display frame. The level is
kept for the session. Offscreen it does not move (`Renderer::govern`). The readout says "quality 0.75 auto".

**Scrolling under the automatic level** (Fable's ruling; `render::SOFT_SCROLL`): only with `Look::quality` at
0 (automatic) and the level below 1, the scroll areas (`Field::scrolls`) are shaded in `lo` while a scroll
offset has changed within `ui::SOFT_STOP = 0.1` s, from the first frame that scrolls; no fixed level ever
softens scrolling. The level is no lower than the material's floor (`render::soft_floor`): 0.75 for charr,
halftonr, carvr, glitchr, scribblr, smudgr (its grain is shaded with the rest, not in the full-resolution pass)
and a material from source; 0.5 for plain, near and dottr. Marks keep their pitch and place in screen pixels
(the reduced pass samples the full frame's coordinates): only edges soften, and the upscale is the bilinear
read, with no sharpening. The ink stays at full resolution: a reduced texel whose ink around it (±0.75 / q +
0.5 px, centre and four diagonals) is not all the same, in coverage or a glyph's tone over a shape, is marked
at rest, so glyphs, strokes of 2 px or less, focus corners and the edges of panels and the chrome are shaded
in the full-resolution pass; the annotation layer and overlays are drawn after the material and never
reduced. Once it stops, `Ui::needs_frame` asks for one more frame (`Ui::soft_drawn` says the last one was
softened) and that frame is drawn sharp everywhere. Softening stands down on a frame that can move the last
one (`dirty::Move`, §1.19): that frame shades only the strip that scrolled in and what changed, sharp, which
costs less than any reduced pass; a softened frame, and the sharp one after it, are drawn whole. The real costs
of scrolling are there (the move, the per-pixel cost of `shade`); 0.5 is the fallback.

**Acceptance**: `tests/perf.rs` (the level steps down on slow intervals while something moves and never at rest,
a missed frame restarts the wait up, costly frames do not raise it, last-second statistics, F2 and the menu
open the readout and its text is in the annotation layer, it wakes every 0.5 s while open);
`tests/stage_scale.rs` (a scroll area softens while it scrolls and nothing outside it does, a fixed level never
softens, the frame due after it stops is sharp, a frame that moves the last one is not softened); `check` item 12: under every material at rest, `q = 0.5` (with `lo` forced) and `q = 1` are
identical pixel for pixel and body text equals `plain` (with item 5's tolerance), with a control: once
loosened, the two resolutions can be told apart. Frame times on llvmpipe (relative comparison only) are in
§9.

`Look::overlay_keep` keeps the readout in `overlay_at`'s corner even where it covers text to read (for a page whose
own text fills every corner); by default it moves on clockwise to the next clear corner. In a narrow window the
readout wraps at its two-space field breaks to fit between `overlay_inset` and 12 px from the window's other side, so at 390 px wide every number stays in the
window (`tests/render.rs`).
