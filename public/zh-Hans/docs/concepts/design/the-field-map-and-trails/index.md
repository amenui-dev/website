# 1.29 场图和轨迹

Where something has moved, a short trail follows it: dragging, throwing and settling the floating box, the bottom
sheet rising, the card stack fanning; the decoration it passes loosens a little and drifts a few pixels along the
direction of motion, then returns; ground dots (`dottr`) lean that way and grow a little. A second or so after
release, the page is identical pixel for pixel to how it was before and wants no more frames. This is part of rule
2 (one way to move: the field), not a new animation.

- **One map** (`src/fieldmap.rs`, no GPU): one coarse grid over the whole window with spacing `cell` (16 logical
  px by default; 1280 × 800 is 82 × 52 nodes, one extra per side), node (i, j) at (i, j) × cell. Each node holds
  three numbers: energy `e` (0..1, which the instances that read it take as looseness) and flow `u`, `v`
  (length ≤ 1, times `drift` in px).
- **Stepping**: a fixed step of 1/60 s; `tick` accumulates elapsed time and steps whenever there is enough (ten
  small steps and one step of ten are equal node by node, so the `render` tool stays byte-deterministic). Each
  step: (1) all three numbers `x ← mix(x, 3 × 3 average, bleed)`; at nodes with ink (`creep`), inked neighbours
  weigh more and empty ones less, and `bleed` grows, so energy creeps along text and shapes (the ink is the
  coverage of what reads the map, read again on a new scene only for instances that changed, never for blades or
  for what lies inside a box with lives of its own, where nothing creeps); (2)
  `e ← max(0, e·exp(−dt/energy_life) − 0.15·dt)`, and the flow's length decays the same way by `flow_life`;
  `meander` turns the flow direction by a slow field (the product of two travelling waves, patches of about 160 px)
  by at most 40° per second, keeping its length; (3) nodes inside an `El::calm` box are cleared. The linear term
  guarantees reaching exactly 0 in bounded time: at most `τ ln(1 + e0/(0.15 τ))`, 1.24 s by default; averaging
  never raises the maximum, so this holds with `bleed` too. What becomes invisible once quantized to a byte is
  dropped, so the map returns exactly to 0, `moving()` is false and no frames are wanted (rule 3). Only nodes that
  still hold something, and one ring around them, take part in the step.
- **Breath** (`breath`, `breath_s`): above 0, energy has a floor that rises and falls periodically (with phase by
  position), so the map always moves and always wants frames; 0 by default, only for presets that want it. With
  reduced motion there is no breath.
- **Putting things in** is one action, a stir: energy and flow added within radius `R` by `(1 − (d/R)²)²`, written
  only to nodes in the scope box and not in a calm box. Two sources: an element with
  `El::stirr(Stirr { energy 1, radius None, scope Container(None) })` that moved (its rect center travelled `d`
  between two ticks, ignored under 0.5 px; a stir every half cell along that segment, in the direction of `d`, by
  `min(|d|/24, 1)`, the energy also multiplied by the look's `stirr`, the radius by default `stirr_radius` plus
  half the element's short side; the frame the window resizes and the frame the element first appears don't
  count); and `Cmd::Stir { at, v, energy, radius, scope }`, where `at` is a point, an element's center, or a stretch
  of a path (`StirAt::Path { points, from, to }`, with `v` the amounts along and across the path and the flow
  following the path's tangent).
- **Reading**: the G-buffer stage's `own_loose` takes `w = max(w, fmap_e(this point))` and then the existing
  `k + clamp(w, 0, max(resp.x − k, 0))`, so responses and the legibility cap need nothing new: 0 for `Ignore` and
  `Nudge`, up to 0.15 for legible text, up to 1 for decoration. The vertex stage translates by the flow at the
  center, `flow × drift × scale`, only for instances that fronts push (so `Response::Ignore` keeps an element, a
  title in a field a gust crosses, exactly where it is; `El::calm` does too and clears the map over its box);
  the map is sampled between its nodes, so a scope's edge is soft over one cell on both sides. Pictures and 3D
  elements are not translated
  (they loosen in place, so the page ground never shows inside a picture's box as a trail passes, `check` 19.5).
  The shading stage: dottr's ground dots add `fmap_e` to their swell and the lattice leans along the flow by at most
  half a pitch; glitchr's row slips take `fmap_e` as looseness on the ground (not on ink). Other materials' grounds
  hold nothing at rest and add nothing. The CPU model `Ui::looseness` adds the same bilinear value (reading the
  quantized bytes) into the same rule.
- **What doesn't read it**: the subtree of the element leaving the trail, recomposition cells in flight, morph
  particles, shed marks and decode blocks (instance record `look.z` is 0). An element that trails and has a ground
  fill (the floating box, the sheet) is drawn on a layer above, like a dialog: where it has ink it replaces the SDF
  beneath, so loosened decoration it passes doesn't show through it as loose marks.
- **Texture**: RGBA8, r = round(e·255), g, b = 128 + round(127·u), 128 + round(127·v), with 0 restoring exactly to
  0; uploaded only on frames where bytes changed (about 16 KB). When the whole map is 0 a uniform turns reading
  off, so a still frame is byte-identical to having no map.
- **Drawing only what changed** (§1.19): the bounding box of nodes whose bytes changed, grown by one cell; the
  G-buffer box adds pushable instances that read the map and intersect it (grown by `drift`), and the final box
  grows again by the material's read range.
- **With reduced motion**: energy is added, flow isn't, lifetimes are halved and there is no breath
  (`Ui::set_reduced_motion`). What the framework brings in and takes away (new instances condensing, the ghosts of
  what left) fades in or out over 150 ms instead of loosening and moving (`Ui::reduced_motion`).
- **What leaves takes its field with it**: an element that leaves with its page (a page change, a closed dialog)
  (its nearest keyed ancestor goes too) leaves nothing: where it stirred the map, drew a flow or a decode, or was a
  ripple's scope, the map is cleared over its box, and when none of the old keyed elements is left the whole map is
  (`Ui::forget_gone`, `FieldMap::clear_in`), so no field or colour of its stays on the next page. A press's front still
  running when the page changes belongs to the press: it goes on over the new page as a whole ring at half its
  strength, its old scope (which went with the old page) dropped. `audit::off_palette` counts
  pixels in no colour the theme makes (the ground mixed toward the ink or an accent), and `tests/fieldmap.rs` checks it
  is 0 on the page after.
- **Gusts** (`Cmd::Gust { gust, scope }`, `fieldmap::Gust`): a band `width` px wide (330) that travels along
  `dir` at `speed` px a second (240) from the line through `from`, across the scope (`Scope` as for fronts, at most
  the window). At a share `u` of the band its flow along `dir` is `strength · 0.5 (1 − cos 2πu) · MapFx::flow`
  (`strength` 1) and its energy `energy` (0.2) times the same; for `wake_s` (0.5 s) behind its trailing edge the
  flow goes the other way, `strength · wake · sin²(πu)` (`wake` 0.105: 6° on a blade the band leans 57°). Across
  `dir` the band lags `slant · n^curve` px (`slant` 0, `curve` 1; `n` is the distance across `dir` over the scope's
  extent that way, 0..1), so it can sweep a field in perspective (W-01: 500 and 0.53). A gust is never put into the
  map to decay: on each step it is worked out from the map's clock at every node and laid over what the map holds,
  the larger energy and, of each flow component, the larger in size with its sign (`FieldMap::encode`). So a trail
  under it keeps its own life, and the frame the band and its wake have left the scope (`Gust::gone`) the map's
  bytes are what they would have been without it and no frame is wanted. With reduced motion it puts no flow.
- **Lives by area** (`El::map_life(energy_life, flow_life)`, 0.1–30 s): a life is the time after which what was put
  in inside the element's visible box is back at rest, byte for byte, however much and however wide it was (a node
  takes the first such box it is in). The map there is the look's slowed down, with nothing spread: no bleed and no
  creep, so a thin path neither soaks away nor closes early; each node goes down from what was last put in it along
  the look's curve from 1 (`(1 + a) e^(−t/τ) − a`, `a = 0.15 τ`, the exponential less the linear floor) stretched so
  that it is under half a byte step exactly at the life, energy and flow each by their own; the meander turns the
  flow as many times slower. All the nodes of one stroke keep their shares of each other, so a path keeps its width
  and closes all at once, the part walked first first. Stirs narrower than a cell are put in a cell wide (the nodes
  are a cell apart; the GPU reads them bilinearly, so the edge is smooth, not in cells). With W-01's 6 s, a 40 px
  path's stalk leans over 3° 5.0 s after the pointer passed and is upright at 6.0 s.
- **Pacing what moves by itself** (`Ui::ambient_fps(n)`, 60 by default, which is no pacing): a frame wanted only by
  what moves by itself (the map, gusts, breath, flows, ticks) waits until `1 / n` s after the last drawn one,
  within half the gap between the last two ticks (at most `0.5 / n` s), so at 30 on a 60 Hz display with
  timestamps a little off it is every second refresh; `needs_frame` is false until then and `next_wake` gives
  that time (`web/amenui.js` sets its timer 20 ms early, so the wait ends on the refresh it is due on). Input is never paced, and neither is what it
  starts until 1 s after the last input; after that it is what moves by itself (a trail that takes 6 s to close
  does not run at 60 frames a second). Everything goes by the clock, not by frames, so a frame at a given moment
  is the same bytes at 30 and at 60.
- **Numbers** (`MapFx`, in `Look::map` and a preset's `interaction.map`; each a number control): `cell` 8–32 (16),
  `stirr` 0–1 (0.6), `stirr_radius` 8–160 (48), `flow` 0–2 (1), `drift` 0–24 (6), `energy_life` 0.05–3 (0.45 s),
  `flow_life` 0.05–3 (0.5 s), `bleed` 0–1 (0.2), `meander` 0–1 (0.35), `creep` 0–1 (0.3), `breath` 0–0.3 (0),
  `breath_s` 2–12 (6 s). Presets: Normcore `stirr` 0; Cyberminimalism 0.25 with radius 32; Tech-Manual 0.2; Gaze
  0.3; Bauhaus 0.4; Lo-Fi Print and Linocut 0.5; Glitch 0.7 with `energy_life` 0.25; Dotmatrix 0.8 with
  `energy_life` 0.7.
- **Components**: the `floating` box, the `bottom_sheet` panel and the `card_stack` cards (an invisible small
  element at each card's far corner carries the trail, since 3D elements don't read the map themselves) carry
  `.stirr(Stirr::default())` by default, with the preset setting the strength.
- **The page** `#stirr` (in the side rail's concepts group): an area with decoration, a picture and a passage of
  text, with a draggable floating box over it; below, a formula caption whose twelve numbers are number controls; a
  "Stir" button sends `Cmd::Stir`. The F2 readout gains a line: the number of non-zero nodes in the map and its
  mean energy (`level`).

Acceptance: unit tests in `fieldmap.rs` (after a stir energy falls monotonically and reaches exactly 0 within the
bound; ten steps equal one accumulated step of ten, node by node; nothing is written outside the scope or in calm
boxes; flow length never exceeds 1; meander keeps length and creep moves energy along ink; with reduced motion
there is no flow, and at 0 it is exactly 0); `tests/fieldmap.rs` (dragging the floating box 185 px at 60 Hz:
decoration beside the path above 0.3, `Nudge` text 0, legible `Loosen` text within the cap, `Ignore` 0, far away 0,
the box itself 0; after release and settling into a corner, the map is all 0 within 1.5 s and no frames are wanted;
under Normcore the map is never written and two seconds draw 0 frames; a press without a drag doesn't stir; with
reduced motion the flow stays 0; `Cmd::Stir` in its element and path forms; over the drag each new scene reads
the ink of no more instances than the box has, and the ink is on the shapes and 0 on the bare page); `tests/render.rs` (the same arguments
with a drag rendered three times are byte-identical); `check` item 19; `bench … map` (one step plus upload at
1280 × 800 within 0.2 ms); `craft <dir> stirr` (eight frames each in `plain`, `dottr`, `smudgr` and `glitchr`). Blades and
gusts: `tests/blades.rs` (an 80 px blade at 0, 19, 38, 57 and 72° and `stiff` 0, 0.5 and 1: the tip's tangent
within 1° of the lean, its lowest 4 px within 2° of 0.35 of it, with `stiff` 1 the lower 48 px within 0.5 px of a
straight line, neighbouring segments at most 12° apart, the length within 2 %; the GPU draws it on its shape,
its head along the tip's tangent and in its own paint upright, the lean paint at 72° of 72°; with `lean_from` 8°
and `lean_full` 20° stem and head in their paint at 5°, half way at 14°, in the lean paint at ±25°, and the default
mix linear; a title with `Response::Ignore` under a crossing gust draws the same pixels as at rest while text
beside it that reads the map moves; a gust of 0.79 over one blade with `max_lean` 72° peaks at 57° ± 0.5°, sways
back once to 6° ± 0.5° within 0.5 s of the trailing edge and is upright from 1.25 s after the peak, after which
the map is byte-identical to before and no frame is wanted; a pointer's walk with `map_life(6, 6)`, 40 px wide:
its last stalk over 3° at 5.0 s and upright at 6.0 ± 0.1 s, the end walked first upright first, then the map
byte-identical to before with no frame wanted, and a box with lives of its own away from it changing nothing;
paths 20, 40 and 80 px wide on and between the nodes' rows close within 0.1 s of each other at 6 s and keep their
width (where the stalks lean half their most) within a cell, fresh and at 5 s; a gust over a path with 6 s lives leaves the path's
bytes and its closing time as they were alone; the same schedule three times gives the same map bytes and frame;
frames at 30 and 60 a second are the same at the same moments, input is not paced and what it starts is a second
later; 3,000 blades are one draw, and while a gust crosses them the runtime's time a frame is within 0.1 ms of the
same page without them, with nothing laid out or shaped).
