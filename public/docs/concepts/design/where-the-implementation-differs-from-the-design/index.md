# 9. Where the implementation differs from the design

(The film and the xray material were removed later; entries below that mention them are the record of the time.)

Components and the gallery:

- §5 first had the sidebar as "components, materials, scenes". By the design lead's decision it is now three groups,
  "components, marks, typesetting"; the material is switched in the top bar, and scenes and style presets came later.
- The text field had no selection and no IME (§1.9 put IME off long before). Clicking a field didn't move the caret to the
  click; the caret stayed where it was. (Done later, §1.8.1.)
- On a page change the new page condenses from loose and the old page is simply replaced; no old page stays outside the
  front (only theme and material switches split two states along a front). Shared parts flowing came later (§1.10).
- In the defaults of the marks page, dithering uses the picture's own colours rather than the theme's ink: the light
  theme's ink is dark, and dark marks on a black ground can't be seen.
- The looseness under a dialog is a dialog parameter (default 0.35), not a fixed constant.
- In a narrow window, a page opened directly from the hash doesn't scroll its item in the tab row into view.
- The native window was only type-checked for macOS targets. On Linux it was actually run at the end under Xvfb +
  lavapipe, driven by real X input from xdotool (see README); the first run lacked `libxkbcommon-x11.so` (winit needs it on
  X11), and once installed it ran without code changes. For verification the window gained `--shot path@ms` (reading back
  its own frame) and `--exit-after`.

Transforms: particles and flows:

- Particles are not single-colour dots. Done as single-colour dots, the handover frame doesn't match: under `plain` a
  loosened element is its faded self, not dots; under `dot` cells with coverage between 0.06 and the threshold are also
  drawn as small dots, and dropping them by threshold loses them. So particles are cell-by-cell images (see §1.10), with
  colours read from the source's and target's first pass; the spec's src rgba and dst rgba are not in the record.
- Cells below the threshold don't fly; they stay and fade (dust) rather than not existing, again so the handover frame
  doesn't change.
- In morphs involving a picture, text cells in flight are drawn the way the picture is drawn (in the theme's colours).
  Otherwise text cells over picture cells are taken for the picture's colour (a patch of red dots). The cost is that for
  30 ms after handover the text turns from `dot`'s round dots into this picture's marks, and turns back 30 ms before
  landing.
- Pairing glyphs by "flow id + index" also requires the same glyph, and only pairs instances that vanish in the old place
  and appear in the new one. Otherwise on a page change the title's characters would turn into other characters without
  condensing, and the sidebar labels would gain a copy flying off.
- Flowing parts in flight are drawn on a separate layer (like the dialog's layer, replacing what's below in near). A
  layer's near used to replace only where coverage was over half; now it replaces wherever there is any ink, so the
  translucent edges of glyphs aren't taken for the picture below. While a dialog is open, flowing parts don't get their own
  layer.
- Once a morph starts, element positions are fixed: scrolling and window resizes aren't followed (a resize ends the morph).
  There is one morph at a time; a new morph immediately ends the one in progress (unless it is that one's reverse), and the
  particles jump to the old target's look before setting off.
- Web acceptance advances time with DevTools' virtual clock: under SwiftShader a frame takes about 10 s, and real time
  can't capture frames in flight.

The annotation layer, every material, twelve presets:

- `near.r` became a signed nearness (0.5 on the outline); it used to be outside the outline only. lino and print both need
  to know "how deep inside"; with only the outside distance the second pass would have to search ring by ring, too slow.
  Everything that reads near.r (dot's ground, particles) converts accordingly, and the existing `check` results don't
  change (0/255).
- The instance record gained a vec4 (`look`, 96 bytes): material override code, hue, height box. They didn't fit in
  `paint`, and both the accent number and the override code are needed in the first pass.
- The hue (three accent colours) is in `ink.b`.
- Contour lines are only within three lines of the edge: one every 7 px across a whole large shape gives moiré.
- The dot material has dots on the ground even at rest, so "body text the same as plain" compares only the glyphs' own
  pixels for dot. print's paper grain is a rest-time treatment, and comparisons allow its amplitude (default ≤ 4/255).
- It was first said that under dot a front passing a picture leaves dark dotted holes. After the rewrite, picture pixels
  take only the §1.6.1 marks under every material, and the ground's dots no longer land on pictures; `pages … materials`'
  `dot-ripple-picture.png` and `check` item 6 (with a front) verify this.
- The annotation layer's "instance count" and "thumbnail of the intermediate buffer" were not done. The SIZE label sits
  inside the element at the bottom right; placed below it overlapped the neighbouring caption.
- Themes went from three to fifteen, and the top bar's theme and material became menus (segmented controls didn't fit).
- Dotmatrix's ticking text relies on `Ui::on_tick` (each tick tells the state the current time), a small interface for
  "things that change by themselves", not animation: only the view changes.
- The style pages' pictures use a separate `PicFx` slot (4); the pictures on the Linocut and Lo-Fi Print pages stay fully
  loose at rest and are printed with their marks (strokes, two-colour halftone).
- With more pages the sidebar is longer than the window, so tests that click a sidebar item first make sure it is in view.

Showcase scenes:

- §6 only gave one sentence for each of seven scenes; what was built is in §6's table. Several intents in the task were
  folded into these seven: "a machine operating the interface, the page building itself" into the opening; "a wall of
  styles, several wipe lines" into one picture's printings (five windows on the same picture, with draggable dividers);
  "annotation gaze" into the perspective scene.
- Several wipe lines are not several half-planes in the field (the field has only one) but the same picture drawn once in
  each of several clip windows, each window with its own `PicFx` slot; the dividers sit in the 6 px gaps between windows.
  The reason: shapes and text drawn over a picture are taken for the picture's colour by the material (the entry in §9
  "Transforms: particles and flows"), so the printings scene draws nothing on the picture and the labels sit above it.
- Text over a picture is still taken for the picture (the first pass's near takes the maximum, and the picture mark can't
  be overridden). The scenes avoid that arrangement.
- "Hold a key" uses Space, and counts only when not consumed by a focused control (with focus on a button, Space presses
  the button). There is also a hold-to-take-apart button and a "keep apart" switch, for touch and screenshots.
- The perspective scene's "intermediate buffer" is drawn with the new `near` material on a copy rather than showing the
  renderer's intermediate texture directly: the copy is laid out elsewhere and has its own nearness. The copy's panel isn't
  filled, otherwise the filled panel's nearness would cover everything above it (nearness takes the maximum).
- In the opening, the "machine" marker is a small absolutely positioned element inside the current control, with a new key
  and the same flow id at each step, flying to the next control with `Cmd::Flow`; it has no animation of its own.
- The sound scene lays out again every frame (the spectrum bars' heights are view), about 24 ms a frame at 1280 × 800 on
  llvmpipe (layout included).
- Scenes are the last group in the sidebar (first would push the style page items out of the 1800 px test window), with a
  separate "Scenes" entry in the top bar.
- Font weight applies the same offset to particle sampling, but outside `plain` weight only affects coverage and nearness
  moves with it (recomputed in the first pass), so materials that read nearness, like lino, see the bolded outline.
- The scenes' "quiet at rest" is verified only from instances (the first pass's looseness comes only from the instance's
  own looseness, birth time and fronts, and the lens is 0 once the pointer leaves), not by reading back the near buffer.

Finishing (components × materials):

- The first pass's ink r became "tone × (1 − accent)", and colour is solved linearly as
  `ground × (1 − r − g) + ink × r + hue × g`. The old nested mix wasn't linear: the anti-aliased edges of text over another
  colour solved toward the ink, and accent text over a panel had edges read as the other two hues (blue dots). b's code
  also became "hue × 0.15 + shape × 0.05", so text over a shape of the same hue still mixes into that hue; `Ink.txt` means
  this pixel has text (even just an edge).
- print's misregistration doesn't apply to text: where there's text (here or at the offset sample) the accent is taken in
  place, so text knocked out of an accent doesn't ghost or shift. Shape edges still shift.
- In a dialog layer, the very faint outer fringe of glyphs doesn't replace the nearness of the panel below (lino would take
  the text's edge for the panel's outline).
- `check` items 10 and 11; `pages … components` gives samples and contact sheets of each component page × each material.

Finishing (four scenes):

- The opening's machine marker used to sit in an absolute element inside the control, positioned by its right inset; taffy
  gives such an absolute element inside a grid the window as its containing block, so the marker was placed outside the
  window (invisible; "too small" was really "not in the picture"). Now the marker is an ordinary child after the control,
  pulled to the control's top right by negative margins that sum to zero, so the page doesn't move as the marker comes and
  goes. Going from one control to the next is a morph (the source is the previous step's marker, hidden and kept for
  sampling), not a flow: the morph's stagger gives a trail of particles.
- "Any input lets a person take over" counts only presses, wheel and keys, not plain pointer motion: a pointer crossing the
  window isn't a wish to take over.
- The five panels of the printings scene each fix a suitable pitch (characters at least 9, pixel art 0.6×, bricks 1.3×), and
  the parameter strip's "pitch" is their common base; the single-mark mode uses `PicFx` slot 5.
- The morph scene's "cycle" holds each form for a constant 2.6 s (one more number wouldn't fit on the parameter strip's
  line).
- `El::calm` is a box, not a per-instance flag: the material reads the field per pixel in the second pass and instances
  have no room for another flag; pixels in the box read a wave of 0, and instances centred in the box aren't pushed.

Interaction and performance (§1.11 on):

- The time-varying parts of materials (glitch's beat, the drift of dot's ground) now read the material clock, which only
  runs while something moves (§1.11). dot's ground used to drift slowly even at rest; now a page at rest is a still image,
  and the drift only advances during fronts, condensing and hover. "The material declares itself time-varying and something
  on screen is loose" counts only for glitch (its time term acts on loose things); dot's drift doesn't count and stops at
  rest.
- The departure board's ticking used to rely on `on_tick` every frame. Now `on_tick` is only called when there are frames,
  so `Ui::on_wake` was added: the state says when it next needs a tick; the departure board wants the next whole second, and
  the scenes' machine and beat want every frame (they are drawing anyway while they move).
- The resolution budget (§1.12) changed "pixels at rest take ink directly" to "pixels at rest are recomputed by the
  material at full resolution": materials with their own look at rest (dot's ground, print's grain) would be wrong taking ink
  directly; for materials that equal plain at rest the two are the same. Rest is judged per `lo` cell (if any neighbouring
  cell is at rest, full resolution), not per pixel.
- Numbers on llvmpipe (`pages … perf` with each material's page fully loose, `scenes perf` with each scene in interaction,
  ms per frame, only comparable to each other, often 10–20% apart between two runs of the same command on the same
  machine): the second pass isn't the bottleneck. At 1280 × 800 plain takes about 33 ms a frame, of which CPU recording and
  submission is under 4 ms and the rest is llvmpipe drawing the first pass (each glyph drawn twice, with its 12 px nearness
  margin); each material adds only 2–10 ms over plain. So `q = 0.5` saves little here: scenes (before / after at q = 1 /
  q = 0.5) opening 40.4 / 39.5–42.4 / 28.7–37.0, morph 48.5 / 50.3–50.4 / 38.9–50.5, reading 26.1 / 25.5–26.5 / 27.1–30.3,
  printings 33.5 / 31.6–36.0 / 33.8–37.7, perspective 18.1 / 18.1–20.0 / 15.9–18.9, sound 23.0 / 23.2–23.7 / 20.3–25.1,
  styles 24.4 / 24.1–26.0 / 25.3–27.2; the material page at 2560 × 1600 (scale 2) is 126–166 ms at q = 1 and 125–148 ms at
  q = 0.5 (glitch 166 → 137, dot 142 → 135, the rest within noise). The gain is in frames with large loose areas (the
  opening's particles, morph flight, glitch's full-screen offset); when the loose area is small `LO_MIN_AREA` skips the
  second pass, so it is never slower. The automatic level always drops to 0.5 on llvmpipe (every frame is over 12 ms), which
  only makes the loose parts coarser and never makes the resting parts worse.
- Text input (§1.8.1): `TextEdit::caret` and `anchor` are still character indices (the old code and tests count
  characters), and positions in events are bytes of the displayed string. A selected space has no glyph, so a lone selected
  space shows no outline; between characters the outlines on either side (3 px each) cover it. The selection outline is
  drawn into the intermediate buffer like any other glyph, so materials treat it like ink (under dot it also scatters into
  dots when a front passes).
- The native clipboard (arboard) compiles, but isn't verified automatically under Xvfb (no second program reads the X
  clipboard); IME events exist only as winit's interface and weren't tried with a real IME under Xvfb. Typing through
  xdotool goes through `KeyboardInput.text` and is verified.
- Read-only selection crosses elements in node order; with the pointer in the gap between two selectable texts it takes the
  nearest one. Selection isn't kept across page changes.
- Incremental layout (§1.13): taffy's `dirty()` is unreliable for the root (the root still reports dirty after computing),
  so "recompute or not" only looks at whether anything changed in this comparison.
- Virtual list rows were meant to be absolutely positioned (`top = i × row height`), but taffy computed the width of
  absolute children of a column in a scroll area as the window's width; they are laid out normally instead, with the
  column's top padding standing in for the rows before.
- The gallery's main row (the row holding the sidebar and the page) had no `min-height: 0` and took the height of the
  sidebar's content (about 2360 px), so once the sidebar was longer than the window the page itself was as tall as the
  sidebar and never scrolled (it looked like the content below the window was clipped). With `min_height(0)` the page and
  sidebar scroll separately; tests scroll a sidebar item into view before clicking it.
- The annotation layer's labels show coordinates and sizes, which are new strings when scrolling and need shaping; "no
  shaping while scrolling" refers to the page itself.
- `bench` (1280 × 800, CPU, mean of 40, ms), before / after: one press on the buttons page 3.34 / 1.83; scrolling one step
  in a plain list of ten thousand rows (all rows in the tree) 386.9 / 3.05, rebuilding the whole view 400.9 / 100.8; one
  step in the gallery's ten-thousand-row virtual list (including relayout when crossing the built range) — / 0.78;
  rewrapping a 5000-character article around a moving circle 3.21 / 1.94.
- Touch (§1.8.2): a finger's press, when it can't yet be told from a scroll, is sent on release (a mouse still presses on
  down); the visual press (front, hold) is there on the frame of the press. With nothing scrollable under the finger it
  presses straight away, like a mouse.
- Finger velocity on the web is computed from the events' own timestamps: under CPU rasterisation (SwiftShader) a frame
  takes long and events are processed in batches, and velocity from processing time would be 0.
- Flinging and overscroll are for fingers only; the native window's touch goes through winit's `Touch` and was never tried
  without a touch screen.
- The accessibility mirror (§1.14): on the web, mirror elements are `div`s with a `role`, not native elements such as
  `<button>` (native elements have their own keyboard behaviour, which would duplicate the runtime's key handling). A piece
  of text has no role (to a screen reader it is text). Native screen readers came later, through AccessKit (§1.14).
- Each tab of a tab view and each list item isn't a focus position in the runtime (the whole tab view or list is one); when a
  screen reader puts focus on them the focus can't be handed over, but activation (click) still selects them.
- Web scene acceptance used DevTools' virtual clock (screenshots while paused). Once pages only draw when needed, a
  screenshot under a paused virtual clock waits for a frame that never comes (nothing requests one), so it switched to real
  time: after each step it waits until no new frame has been drawn for 3 s (or a time limit) and then takes the screenshot.
  Under SwiftShader a frame of the printings scene at 1280 × 800 takes minutes and the screenshot times out, so the last five
  scenes run at 960 × 600. The list and each of the seven scenes are operated once with no console errors.

Ripple rules, recomposition, the surface for models:

- The scope is named `Scope::Own`, not `Self_` as in the task (`Self_` reads awkwardly and an underscored variant name is
  unusual in Rust); `Scope::Off` was added too.
- Scopes are one box per front (judged by instance centre), not a scope number per instance: an instance can be inside
  several nested scopes at once and one number can't hold that (§1.15).
- The legibility cap also governs holding, re-condensing after a press and scroll overscroll, not only fronts and the lens;
  and these don't look at whether the response is `Nudge` (otherwise nothing would show when a button is pressed or a list
  overscrolls), only at the legibility cap. `Nudge` only means a front doesn't loosen it. The lens only ever loosened
  pictures and the ground and is unchanged.
- `Nudge` pushes as a whole: legible things measure the push from the centre of their outermost control (or their own run
  of text), and all their instances move together. Pushed by each instance's own centre, the text on a button would be split
  in two from the middle (seen in the samples). A surface is not a control here: an element that follows the pointer
  (`on_point`, `on_move`) or one larger than a quarter of the window (a stage, a page that takes keys) leaves each control
  in it its own piece. Taken as one piece, a whole page was shoved sideways when a commit's front crossed the window's
  centre.
- dot's row offset only acts where things are truly loose (`near.b` past 0.2); text held at the legibility cap isn't offset.
- Recomposed tiles switch from the old piece to the new piece at their own switch point, without a crossfade: with a
  crossfade the whole number goes grey halfway through the flight. When it changes again midway, the new round lands when
  the previous round would have landed (at least a third of a flight); otherwise after ten quick presses the number would
  stay on the old value and only jump at the end.
- A press on empty space on the page no longer releases a front, so the old acceptance "pressing the ground releases a
  front" (`tests/frames.rs`) now presses a button; the whole style page is one ripple scope (pressing any control sends the
  front across the page and text only reaches the legibility cap); the material page's sample is entirely `Loosen` and not
  legible (it exists to show how text loosens), and the "Ripple" button sends a front into the sample; sidebar items (page
  changes) are global.
- In `tests/pointer.rs`, "a held row is fully loose" and "an overscrolled row is looser than 0.3" became equal to / not over
  the legibility cap: the rows contain text and are legible by the rule.
- So that tests press the same point, the number control's value got a key (`{key}/shown`); the old path `…/value/0` no
  longer exists.
- "Ten quick presses" uses the new `Cmd::Press { key }` (pressing an element like a person: message, front, condensing),
  which the state issues once every 33 ms in `on_tick`.
- `shot`, `pages` and `film` were not merged into `render`: they are fixed scripts of their own (the demo, each page ×
  theme × material, the moments of a morph), and only the image output tool was duplicated; cropping, contact sheets and
  PNG encoding moved into `offscreen` (`crop`, `sheet`, `png_bytes`), used by `film` and `scenes`.
- Web events and `render`'s GPU tests: without an adapter, the two drawing tests in `tests/render.rs` and `tests/recipes.rs`
  are skipped (printing a line); the container here has lavapipe and ran them all.
- Interface text in strings was still written directly in Chinese: the gallery had no language files, and the task said to
  keep it as it was. (Language files were added later, §1.20.)

3D, GPU particles, drawing only what changed, presets and transitions:

- 3D is drawn by material by default, not as a picture: the task said "treat it as a picture (the ground colour doesn't
  appear in it)" and also asked that every material draw it without extra code, and both can only hold this way: its ink is
  opaque flat shading, and `ambient` keeps even the darkest face off the ground colour; only with `Scene3d.picture` on is it
  drawn with picture marks. Meshes default to `Response::Loosen` (like pictures), even when draggable.
- Picking casts rays on the CPU with the same transforms rather than reading back the id buffer: readback is asynchronous
  on WebGL2 and would make hover and press a frame late and nondeterministic in headless tests. The id buffer is still drawn,
  to find the edges of "the other copy".
- Normals aren't interpolated: meshes are flat-shaded triangle lists, and under non-uniform scale normals are transformed by
  the model matrix only (not the inverse transpose). Extruded text outlines weren't done. Edge detection only looks at the
  right and bottom neighbours, so edge lines are one pixel wide and biased to one side; at 0.5 resolution the silhouette
  steps by 2 px (nearness is bilinear, so the edge is a soft step, not jagged).
- Mesh morphs take cells over the whole element rect (the CPU doesn't know the silhouette), and empty cells fly
  transparent; (changed later: on the CPU path a mesh's particles leave from or land on the 3D surface, see "Fixes and the
  film".) A force "flowing along the mesh surface" wasn't done.
- GPU particle pairing is "two grids with the same rows and columns, cell i to cell i", not the CPU's coverage threshold
  plus Morton rank: no sorting needed; handover and landing are still pixel-identical. There is no "dust": cells empty on
  both sides fly transparent. (The GPU path first could only reverse once; later any number of times.) On WebGL2 the third
  tier, RGBA8 fixed-point packing, wasn't done: when no float target works it stays on the CPU path (stated in the readout).
  The particle count is the grid's cell count (element extent / cell area), so "200,000" is a ceiling, not exact.
- Particle bench (lavapipe, only relative; one frame = one simulation step + the first pass twice (ink, near) + material,
  1280 × 800, in flight): CPU path 22,802 cells 50 ms; GPU (compute) 48,400 cells 94 ms, 192,696 cells 294 ms, 990,912
  cells 1056 ms. On lavapipe the bottleneck is fill (each cell drawn twice), not the CPU. Under SwiftShader (WebGL2, rgba32f
  emulated in fragments) 198,468 cells fly, a frame takes several seconds, and screenshots caught frames in flight.
- Drawing only what changed: the box is one rectangle (a union), not a set of rectangles; materials' read extents are
  estimated generously from their parameters. With materials that read the material clock (dot, glitch), a running clock
  redraws the whole screen: under dot, pressing the counter button (a front, the ground drifting) is still full screen.
  Bench (lavapipe, 1280 × 800, mean GPU frame): plain hover 19.7 → 4.8 ms (box 5.5% of the window), caret blink 18.2 → 3.6 ms
  (2.2%), pressing the counter 18.4 → 6.8 ms (26.6%); dot hover 37.2 → 7.1 ms, blink 43.2 → 5.2 ms, pressing the counter
  39.7 → 36.2 ms (full screen, see above).
- The material clock no longer runs for hover bursts, badge slides or annotation drawing in (§1.19); under glitch, a hover
  burst's row offset no longer changes beat by beat, it only weakens with the burst.
- A preset's "default response of text" has only two classes (text and controls, everything else); one per role (button,
  tab…) wasn't done. A transition's "layering" uses brightness as depth (the material step only sees the picture, not the
  tree's depth); "detection boxes" draw boxes without labels; "colour blocks" only translate, no rotation; the offset band's
  channel split acts only on the old picture (the new picture is sampled once, see below). `shade_new` can have only one
  call site in the shader: each call site is compiled in full, and several made the material pipeline take tens of seconds
  to compile on a CPU rasteriser and run out of memory.
- The `render` tool only draws the last frame: at a commit with no retained picture yet, it draws the old scene into
  `before` with `Split::look` (the look at commit time), so a single frame can still show a transition midway.
- The gallery's source page is one run per line, and the layout cache per generation went from 2048 to 4096 and later to
  8192 (`text::LAYOUT_CACHE`): once the morph page's source (`morph.rs`) grew, scrolling reshaped on cache generation changes
  (`tests/incremental.rs`).
- Found and not fixed: recomposition with a keyed number that changes every frame (such as a gliding camera readout) kept
  accumulating tiles until memory ran out; the 3D page turned off recomposition on its camera numbers to avoid it. (Fixed
  later, see below.)

Fixes and the film:

- The recomposition leak: when it changed again midway, the new round's source was the fragments cut from the previous
  round's tiles, cut again by tiles; fragments multiplied each round, and "tiles per glyph" counted fragments, so the cap
  grew too. Now tiles in flight carry their fragments directly as the new round's source (fragments aren't cut again), the
  glyph count only counts whole glyphs, a change within `Recompose::min_interval_ms` (80) replaces directly, and
  `max_tiles` (24000) bounds the whole window. `tests/ripple.rs`: a keyed number changing for two thousand frames keeps
  instance and tile counts bounded; changing every 100 ms (retargeting in flight each time) three hundred times keeps tiles
  under the cap and ends at rest. The 3D page's camera numbers no longer turn off recomposition.
- Meshes emitting particles: on the CPU path, when one side is a `scene3d`, that side's cells are points on the mesh
  surface sampled by world area (`Scene3d::surface`), enough to cover the half facing the camera; each frame they are
  reprojected with the object's current transform (`Ui::reproject`; the object turns and the particles follow), and points
  occluded by the mesh (on the CPU, the nearest ray intersection, `Scene3d::project`, equivalent to a depth comparison with
  the mesh) aren't drawn before they leave the surface (kind code + 8; those landing on the back + 16, fading out before
  landing). What the particles show is still taken from the mesh picture of the frame the morph began (not redrawn each
  frame). The GPU path (cell count above `gpu_above`) still samples cells from the screen.
- Local events no longer redraw the whole screen: dot's ground drift reads the "ground clock" (`Ui::ground_clock`,
  `rip.w`), which only runs while something acting on the whole window moves (commits, window-wide fronts, morphs, flows,
  flings and overscroll); the part of glitch that reads the material clock is only in loose places, and when the clock runs
  only instances with static looseness are added to the box. `check` item 15 is still byte-identical.
- Pictures in controls: legible pictures sit 1/32 below their slot number in near.g (rounding on decode is unchanged), and
  the second pass's lens raises them at most to the legibility cap (`burst_t.w` carries `legible_cap`).
- Any number of GPU particle reversals: every reversal after the first sets all cells off at once with the same duration,
  so each cell's starting point for the leg is an affine function of "its progress at the first reversal",
  `alpha × u + beta` (`morph::Turn`), with one more `gt` in the uniform.
- Added to the runtime for the film: `Cmd::Commit` (a commit without a press, keeping the picture before layout),
  `Cmd::Focus`, `Cmd::Scrub` (holding a running morph at a point in flight, `MorphRun::scrub`; on release it continues from
  there), `Ui::on_pointer` (the pointer's position in the window), `Node::flow()`; a morph ends when both its elements are
  gone; `Cmd::Press` doesn't call `on_input`.
- A full glyph atlas (the film used many sizes across several scales) left later text undrawable; now a full atlas is
  cleared and the frame drawn again (recomposition tiles and the dissolving old page are dropped).
- The stage has no ground fill: a shape filling the stage would be treated as a shape by materials (under lino a carved
  block, and its override material code would cover everything in the stage), so the stage is marked out only by corner crop
  marks and the clapperboard.
- The clapperboard's ticks are `focusable(false)` (the keyboard uses ← / → and number keys), so Tab moves between the
  controls in the stage.
- Autoplay's "any input" includes pointer motion (not only presses, wheel and keys), since pointer parallax and hover are
  interaction in the film.
- Going from chapter 12 back to chapter 1, Normcore has no transition, so the commit's front spreads from the stage centre
  (not a hard cut).
- Objects interpolate by rect during a chapter change, so between two chapters with different aspect ratios they stretch
  briefly.
- Chapter 7's sideways flick takes the CPU path when a mesh object takes part (emitting from the surface); chapter 12's
  finale is the GPU path, with the mesh sampled from the screen.

Scrolling, the mark and the mechanism, languages, components and docs:

- It was said the native window's first frame takes 45 s. Measured: pipeline creation totals about 2.2–2.5 s (the material
  pipeline `fs_material` 2.1–2.5 s, each of the others within tens of ms), and on an idle machine the first frame appears
  about 3 s after start. The 45 s run had another lavapipe render (`reel`) running on the same machine: when llvmpipe's
  threads are taken, the first frame can drag past a minute. So pipelines were not made to compile lazily per material;
  `AMENUI_TIMING=1` prints each pipeline's compile time.
- Scrolling "landing line by line on the dot grid" (dot and ascii's `latch`) is mostly invisible at 60 Hz; what shows is
  each material's trail and stagger. Wheel ticks aren't smoothed.
- The GPU particle path still samples cells from the screen, not the mesh surface: no mesh takes part in a GPU-path morph in
  the film any more (when the mechanism throws out one part the cell count is within the CPU path's range, leaving from the
  surface).
- The logo started as "a ring crossed by a bar", which drawn out is a well-known traffic sign; it became "a circle with a
  square over its bottom right quarter".
- The mechanism doesn't loosen while held and dragged (`Response::Ignore`), otherwise under plain the whole machine fades
  while dragged.
- 3D views keep their own aspect ratio during morphs and flows (centre to centre, uniform scale); pictures still interpolate
  by rect (on a page change a picture leaves from the card's rect, which the test wants).
- Languages only covered the framework, the components' own text, the gallery shell and sidebar, the film and the
  mechanism, and the docs site; the body text of the old style pages, scenes, material pages and mark pages was still only in
  Chinese (in English the page names were English).
- Text scrolled into view for the first time on a docs page may be shaped once (`tests/incremental.rs`'s wheel test allows
  at most two).
- Fixed a long-standing problem: when a node moved from a parent being removed to a parent that stays, taffy didn't take it
  out of the old parent's list, and removing the old parent read the already removed child and crashed (seen going from the
  list page to ten thousand rows). Now the staying children are taken out of the node to be removed first, and the parent
  that receives them is set again.
- The date field wasn't done.

Wrapping, the finishing pass, depth marks, falling and flow fields, embedding in web pages:

- Wrapping: the gap is measured at the widest point of the outline over each band's whole height, left and right
  independently (it used to be measured by a circle swept along the outline, and a band that only grazed the top of the
  circle was pushed away by the wider lines below); the left run sits against the obstacle, so it is ragged on the column's
  outer side (the left margin), not on the outline's side. Chinese has no spaces between words and parley's justification
  only uses spaces, so both sides can't be even. Seen diagonally, text and the outline in the next band may be closer than
  `gap` (`tests/wrap.rs` now measures per band). Xiaozhi's logo is drawn on the same 2 px grid as the obstacle.
- The finishing pass (§1.21) isn't in the first or second pass but replaces the step that copies the retained picture to the
  target each frame, so with everything off it costs nothing, and partial redraws, commits and transitions are unaffected;
  the blue noise is the R2 low-discrepancy sequence, not a true blue-noise mask; "Atkinson" is an ordered approximation, not
  error diffusion (a fragment shader can't pass error from pixel to pixel).
- Depth marks (§1.18) change the mesh's ink (lightness and tone); the mark cell size doesn't change with depth.
- Falling and flow-field marks move on the CPU and are redrawn as instances each frame (at most 4000 on the flow field
  page); they aren't part of the particle system (§1.10).
- `El::shift` only moves the drawn position and hit testing, not layout: what's moved aside doesn't push its neighbours.
- One document has only one `<amenui-view>`: every export of the wasm module reads the same state (one runtime, one
  renderer). Supporting several would need a view number on each export and one state and device per view, which wasn't
  done; a second element doesn't draw, says so in its box, and carries `data-amenui-limit`.

Justified wrapping:

- Wrapping: runs next to an obstacle are no longer moved whole against it (which left the column's left margin ragged);
  they are justified instead: the remaining width is spread between clusters, so the left and right runs are even on both
  sides. Each gap gains at most `Around::justify_max` (default 0.25 em); Chinese justifies almost every run beside Xiaozhi
  (1 run of 83 across twenty positions can't), while English can only stretch spaces and has no hyphenation, a run beside the
  circle has only a few words, and over a third of runs exceed the cap and are set from the start, ragged on the outline's
  side. Lines to justify are shaped again with letter spacing (one more shaping, only when the obstacle changes grid
  position); clusters with added spacing turn off optional ligatures, and the next cluster after one is in another layout run,
  so kerning between the two no longer applies (no effect on Chinese).

Interaction craft:

- The bottom sheet, when presented, both slides up and condenses from loose by the framework's rule (every new instance
  does); no "already there" exception was added.
- Handover goes one way only: pulling down a list at its top becomes dragging the sheet; once the sheet is dragged to its
  highest detent it doesn't hand the gesture back to list scrolling.
- Dragging the mouse on the sheet body drags the sheet (the mouse has no drag scrolling); the list scrolls with the wheel.
- When `El::absolute` has both left and right (top and bottom) insets, taffy 0.15 sizes it from the root, so sheets and
  floating windows use the new `el::layers` instead of absolute positioning.
- When a skeleton is replaced by content, text condenses from loose (the rule for new instances) rather than "recomposing"
  from bars into text: bars and glyphs don't correspond one to one, and forcing a correspondence only sends bars flying
  around.
- The card stack's "thick" look uses 3D cubes, not rotation of 2D instances: the framework's 2D instances don't rotate, and
  3D elements already have outlines and crease edges.
- The dot-matrix chart wasn't put into film chapter 5 or 8: there were no still samples of those two chapters to compare
  against at the time, and without judging it, it wasn't swapped.
- The film's formula notes were only added to chapter 6; the other chapters got none (no adding text to the stage without
  seeing samples).
- English wrapping got below 15% mainly from letter spacing between letters (at most 0.08 em); hyphenation itself helped
  little: Xiaozhi's English is mostly short words, Liang's patterns don't break before two letters or after three, and words
  like `along`, `places` and `reaches` can't be broken. With hyphenation alone and no spacing it is 33% (43 / 129).

Fields, the Lab, decoding transitions, the narrowed scope, the frame budget:

- Instances gained one more vec4 (`xf`, 128 bytes). Particles and recomposition cut cells by the instance's rect, ignoring
  `xf`: rotated glyphs in a morph are sampled unrotated; `on_outline` also measures coverage by the unrotated rect (the
  shapes in the experiments don't rotate). WebGL2 uses one more varying slot (12, the minimum is 15).
- A canvas's marks come from the view every frame: an experiment rebuilds the view, compares and draws the scene each frame,
  without layout or shaping (`tests/pointer_moves.rs` passes as before). A partial scene that "redraws only this element"
  wasn't done: the scene is still redrawn whole, with the first and second passes clipped to the box.
- Fields are all computed on the CPU, WebGL2 included; compute shaders are only used for the existing particles, not to
  speed up fields.
- Reaction-diffusion growing into a shape uses f 0.037, k 0.060 (Pearson's maze parameters 0.029 / 0.057 die out with this
  discretisation and such small shapes); the mask's edge has no flux, so thin strokes can grow. Letters use Noto Sans
  outlines (pixel type strokes are too thin on a 120 × 75 grid).
- The budget is a number of marks, not time itself: the ratio is adjusted from the previous frame's time. The GPU morph cell
  count scales by the same ratio, and the CPU path is also capped at `MAX_MARKS` (40000 before). The flow field's old cap of
  4000 was removed; the budget governs it.
- Decoding's colour blocks are decided by the coverage of the finest level's cells; anti-aliased ink edges below 0.02 cell
  coverage don't count, so a block's outer edge is sometimes one cell wider than the box (full-screen cell alignment).
- The Lab's look is a theme (`LAB`), put on by the look when entering a Lab page, not by a commit (no transition); the top
  bar's theme and material menus have no effect on those pages.
- Narrowed scope: the chrome, liquid and collage materials, the Neo-Y2K and Mixed media presets (with their themes, style
  pages and the "melt" and "tear" transition steps), and film chapter 11 (Y2K) and chapter 12 (collage, with the finale's
  million particles) were removed with no compatibility kept; the mechanism is now chapter 11. The first pass's ink.b height
  code reserved for "height materials", liquid's soft buffer and its pass went too. The records above still mention them as
  they were at the time.
- Performance: the gallery's view used to parse `registry.json` several times each time (`docs::items` and `widget_names`
  called once per page in the sidebar and the page table), 35 ms per view, and the Lab needs at least one view per frame;
  now it is parsed once and kept, and a Lab frame (input, update, view, layout, scene, CPU) went from about 40 ms to 3–8 ms
  (`bench <count> lab`).

Field maps and trails, threads, small type, Loose Thread:

- Trails are named `stirr` as the lead session decided (`El::stirr`, `Cmd::Stir`); read the draft's `wake` as that;
  `Ui::on_wake` is unchanged. The corresponding numbers in `MapFx` are `stirr`, `stirr_radius` and `flow` (the draft's
  `wake`, `wake_radius`, `stir`).
- The character 回 has four outline loops, not the two the draft said: each of its two squares has an inner and an outer
  edge. So one thread has three bridges (`tests/strand.rs`).
- A thread's rest path adds the displacement the first pass pushes instances by along the flow (`flow × drift`). Without
  it, while the field map moves, the fill is pushed a few pixels and the thread stays put, and the body text doubles.
- A thread end's direction is read at the rest point where it starts, and on leaving the scope box the last segment is cut at
  the edge rather than dropping the whole point. In the old way, the number of points in a thread end varied with `k`, and
  the farthest point jumped outward while retracting (measured by `check` item 20.4).
- Thread segment counts and rest paths are recomputed on the CPU each frame (needed anyway while the field map moves);
  outlines are cached by glyph, size and transform. `bench … strand` (200 glyphs fully loose, 6775 segments) takes about
  0.98 ms on this machine, just within 1 ms; half of it computing threads and a smaller part building instances.
- Edge notes use a serif (Source Serif 4), not integer multiples of Departure Mono as draft §4.5 said: the lead session
  decided "the serif is for wide-tracked small type and edge notes". So `check` item 7's "rotated pixel type has only ink and
  ground colours" became "every pixel is a blend between the ground and the annotation colour" (anti-aliased grey edges are
  different coverage of one colour, with no other colour mixed in), plus more than 30% of each line's ink being solid; 1080p
  legibility is verified by the same measure. Weight 400, not 300: at 1× only just over a tenth of 300's ink is solid, which
  doesn't read.
- `Whisper` is a fixed font (`Font::Whisper` → `text::SERIF`), not a fourth font role in the theme: there is only one serif
  and no theme wants to replace it. There is no Chinese serif (as the lead session decided), so the demos only write Latin
  in small type.
- "No dithering, nearest colour only" already works (the shader's palette branch takes the nearest colour when dithering is
  off), so no new dithering kind was added; `binary` just swaps the palette to `one_bit`, turns dithering off and rounds line
  widths.
- Edge notes, waveforms and level bars only draw while the pointer is in scope (hover and focus path modes); the "all" mode
  always draws. The draft didn't say when readouts appear on hover.
- The pointer itself doesn't stir the field map (only elements with `El::stirr` stir it as they move), so the Loose Thread
  style page has a 1 px dot following the pointer to stir it; no "pointer stirs" runtime mechanism was added.
- Fixed two old misalignments: `style::THEME_IDS` still had the removed `y2k` and `kraft`, so from the 11th on names and
  themes didn't match (`--theme glitch` got the lamp); the style pages' `STYLE_DESC` still had the descriptions of three
  removed styles, so from the 6th on descriptions and pages didn't match.

Where the changes made for looks (field maps, threads, small type, Loose Thread) differ from the change list:

- Text and controls meant to be read don't get threads, don't fade and aren't pushed by the flow under `scribblr` (list
  item 1, as written); the old item "draw only threads along the outline" was removed. Fronts push them as usual (the list
  only mentioned the flow).
- Vertices are removed on the list's tolerance, but not "slide as soon as the tolerance is passed": the tolerance is
  `0.5 + smoothstep(0.3, 0.5, k) × 0.3 × line height`, and a vertex slides onto its chord over half a looseness span from
  the looseness at which the tolerance passes it, following the chord's two ends while sliding and only shortening its own
  offset from the chord. Sliding directly by the list's `smoothstep(0.3, 1, k)`, going from 1 to 0 made some vertices jump
  7–9 px in one frame (more than `reach / 6`): nested vertices were pulled onto far chords while their parent vertex was
  still sliding.
- Independent displacement isn't hashed by vertex index but by arc length along the path, one cell every half line height,
  eased between cells: the remaining vertices, a chord apart, each move their own way, and vertices between them that are
  sliding follow, so nothing jumps. The effect is the same as "by index".
- Thread ends grow at least to the scope's edge (the list's `max(tail, 0.6 × long side)` fell 50 px short of the edge on a
  diagonal); the starting one takes the direction that can go farthest and differs from the last one, not "the opposite
  direction": the style page's title is at the top left and the opposite direction leaves the scope at once. Turns aren't
  noise but alternate offsets to either side at a quarter, half and three quarters, so 2–5 clear turns. The direction is
  chosen by how far the whole thread end can actually go (stopping where it meets text meant to be read), not by one
  straight line.
- List item 2's segment counts and lengths are measured at `angular` 1: a curved line has many short pieces, and the
  median would measure those. The default `angular` is 0.35 (the line is mostly curve).
- "Black and white only" thresholds by coverage only under plain and `scribblr` (where the ink is the material drawn as
  is); other materials draw their own marks and still take the nearest colour. Very faint fills (less than a fifth of the way
  from the ground to the ink) count as ground, or panels would become solid ink.
- Badges move out 6 px (of the list's two options, this one), so quiet text also has badges.
- Threads pass behind pictures (no threads in a picture's box): a thread's ink on a picture in the first pass would be read
  as the picture's colour, which showed as red threads in the samples. The list didn't mention this. So the thread ball's
  knot no longer sits on the picture's top right corner; it moved to the bottom right, one corner behind the picture.
- The Loose Thread page's footer no longer has `amenui · loose thread · 2026`, and the caption only says `fig. 1`: the
  thread ball's long thread passes to the right of the picture and would cover those two lines. There is no thread ball in a
  narrow window.
- The field map's `energy_life` is 2.0, `bleed` 0.06, `creep` 0.15, and the dot under the pointer has radius 72: the list
  wanted looseness of at least 0.8 after a sweep, 0.5 one second after the pointer leaves and 0 at three seconds. The first
  holds; the other two can't hold together under §1.29's decay (the lifetime's exponential plus a fixed 0.15 per second):
  returning to 0 within three seconds needs `τ ln(1 + e0 / (0.15 τ)) ≤ 3`, that is `τ` at most 2, and then from 0.9 only
  about 0.35 remains after one second. "0 at three seconds, no frames" (rule 3) was chosen, and one second is verified at
  no less than 0.3. The old `bleed` 0.2 and `creep` 0.4 spread four tenths of the energy on ink each step, and a swept title
  only reached 0.7.
- `craft <dir> thread-ref` only gives the style page's two frames; the side by side with the reference picture is put
  together outside the repository (no third-party pictures in the repository).

Text on a form, the warm-up and the audit (§1.23, §1.26, §1.31):

- Showing text by whole words is `El::reveal_by(Reveal::Word)`: `El::reveal(bool)` already names something else (an
  element that asks to be scrolled into view, such as a selected row).
- A word shown whole takes its faintest glyph's opacity, and on a form it is thinned as one, by its most covered glyph.
  Averaging drew a word half turned away at half strength.
- `edge` fades from the full value down to 60 % of it, where a glyph is no longer drawn; the recipe uses 0.5, so no
  word is drawn squeezed at the rim.
- `Point::glyph` is `None` during a drag: a drag comes through `Drag`, which has no glyph, and adding one would have
  changed every literal of it.
- `Warmup::step` takes the text system (the mark atlas is built from the faces the runtime draws with), and
  `Renderer::begin` makes the core pipelines at once rather than over the first steps, since plain needs them for the
  first frame. A view reads progress from the state or through `Ui::ready` (it has no runtime to ask).
- The audit is its own entry (`drive::audit`), not a field of `RenderSpec`, so code that builds `RenderSpec` keeps
  building. A preset is given the same way: an opener wrapped by `drive::in_preset`.
- Under the Glitch preset, labels in the form, submenu and loading recipes measure under the audit's own floors
  (2.3–4.2:1; a P0 at 2.95:1 in the submenu): its finishing pass (chroma offset, scanlines) splits and darkens the
  strokes the contrast measure reads. `PostSpec::clear_text` keeps text to read clear of them (§1.21), and the
  audit reads a bent frame where the bend drew it: the rest was the curvature moving text 3–10 px off its ink boxes
  near the corners. `MeshHit` lost `Eq` and `Hash` with its float fields.
- The contrast measure is the strongest ink pixel of a line, the colour the text is drawn in; percentiles of the ink
  read antialiased edges at small sizes as low contrast.

Controls in loosened subtrees, the warm-up in the hosts, opacity, scoped commits, morphs from the last frame, leaders
and echoes (§1.1, §1.6, §1.7, §1.9, §1.10, §1.12, §1.23, §1.28, §1.29):

- The plate (a ground-coloured panel behind loosened text to read) is not done yet.
- `Progress` keeps its `stage` beside `done` and `total`; the web export `warm()` returns only the two numbers.
- `MorphSpec` lost `Copy` with `from_last`; code that reused a spec clones it.
- With the Crack's pieces leaving from 0.28, the test that every piece is whole or gone samples at 0.53 of the step
  rather than 0.6, the same moment after the pieces start.
- In the press-flash check, a switch's P1 "eroded" finding in dark `dottr` at release is not counted: it is the
  material's own look at rest on the thumb, not the flash; only P1 contrast findings are.

Layout, faces and plates (the fifth set of framework needs):

- `El::fit`: the containment check of `tests/render.rs` is skipped at or under `min`, where clipping is the specified
  behaviour (there it checks that `FitShort` is reported exactly when the subtree is cut).
- Clearing the focus: `tests/focus.rs` checks that the method and the command leave the same way and that the page
  is then as never focused; it does not compare each leaving frame with the focus moving to another element.
- `El::fit` and the readout's wrapping are measured; the comparison with the website's `fit_stage` is with
  `FitMode::Reflow` (§1.2), run on the website's stills with a site change that is not in the website yet.
- Faces that come later: the warm-up's budget with faces arriving and material shaders fetched as assets are not
  done; the module still carries the material shaders.
- Effect switches (§1.34): the field dots have no switch yet (see there).
- Plates: ageing by material (the table of how each material lets a stroke go) and the field acting on a plate's
  ink per pixel are not done; every material fades a stroke's coverage over the last fifth as plain does, and the
  pointer and fronts act on plate ink only as on any decoration drawn this frame (not on kept pixels). The check that
  an older stroke differs by more than 25 % at `a` 0.75 in dottr, charr and scribblr waits for that; until then an old
  stroke at `a` 0.9 keeps at most 55 % of a new one's ink (`tests/plate.rs`). A full-window
  plate at 1440 × 900 costs on lavapipe (a CPU rasteriser) 2.7 ms a frame with ink kept (12.3 → 15.0 ms) and
  8.5 ms over a full redraw in a frame where ink ages (49.0 → 57.5 ms); the 0.6 ms budget is for a GPU and has not
  been measured on one. A see-through group (`El::opacity`) that contains a plate is drawn as if it ended where
  the plate starts.

