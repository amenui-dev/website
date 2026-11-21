# 1.10 Transforms

Two things: **particles** (anything breaks into particles and flies over to become anything else) and
**flows** (on a page change, the parts both pages share move to their new places). Both go through the
two-stage drawing: particles and instances in flight are drawn into the coverage and SDF buffers as usual and
the material treats them as usual; there is no separate look. The code is in `src/morph.rs` (data and pure
functions), `src/ui.rs` (running it) and `src/particles.wgsl` (the particle pass).

**Commands.** `update` never touches the runtime; it puts commands in its own state, and the runtime takes
them with `Ui::on_commands(fn(&mut S) -> Vec<Cmd>)` before each layout (a flow needs the picture from before
the layout). `Cmd::Morph { from, to, spec }` (two keys) and `Cmd::Flow(FlowSpec)`. `Ui::morph(from, to, spec)`
can also be called directly. `Ui::morph_from_last(to, spec)` (or a `MorphSpec` with `from_last: Some(key)`) takes the
source from the frame drawn last rather than from an element on the page, so a title flies from where it was across a
route change even though its old element is gone: the glyphs drawn for `key` last frame are kept, and after the new
layout each flies, whole, to the glyph of `to` in the same place in order. In every material, plain too: the unit is
the glyph, never a copy at each end fading over (plain, its edges are clear all the way; a material draws its own marks of it as
it goes, loosened by `0.6 × sin(π × p)` at its own progress `p`, so it comes apart into the dots or characters on the
way and is whole at both ends). Each goes in a straight line, eased in and out on a
sine (`p = 0.5 − 0.5 × cos(π × t)`), its size and colour going from the old glyph's to the new one's (drawn as the old
glyph for the first half, as the new for the second); in the word's order from the end that faces the way it goes
(the glyph in front leaves first, so the word opens out a little rather than running into itself): the way, from the
old glyphs' centre to the new ones', projected onto the text's direction (the old word's first glyph to its last); going
on along it the last glyph leaves first, going back the first does. Where that projection is under 10 % of the way's
length (near square to the text, as a title going straight down the page), no end is in front and there is no stagger:
all glyphs go together, each for the whole `duration_ms`. Otherwise each starts 12 ms
after the one before, the whole stagger at
most 15 % of `duration_ms`, each flying for `duration_ms` less the whole stagger, so the last lands at `duration_ms`
and the word moves as a word. The frame at `duration_ms` is the new page as drawn directly, byte for byte; nothing tightens after.
Glyphs only one side has fade where they are, the old over the first 30 %, the new over the last 30 %; the rest of the
new page is in place from the first frame, and the flying glyphs stir nothing. With less motion nothing flies: the old
title fades out where it was and the new one in where it is, 150 ms each. `MorphSpec` is `Clone`, not `Copy`, for
this field; of its other fields this morph reads only `duration_ms`. A subtree with `El::hidden_until_morphed(true)` is laid out as usual but not
drawn, takes no input and can't be hit until a morph reveals it; after the morph, the view decides again.

#### Particles

**A particle is a cell.** A particle is one cell of the material lattice (device pitch
`P = max(1.5, lattice × scale)`, the same grid as `dottr`'s), and has no colour of its own: it
shows what the source element drew in its cell, and on landing what the target draws in its cell. Both are
read pixel by pixel from images: when a morph starts, the renderer runs the G-buffer stage once on its own
for the source's and the target's instances (looseness 1, already born), giving four images (source
coverage, target coverage, source SDF, target SDF), once per morph. So a particle at progress 0 is exactly
the cell it replaces and at progress 1 exactly the target's cell, whatever the material and the content
(text, shapes, pictures, whole subtrees). What a stage (§1.35) drew of the source is taken as the frame before
the morph drew it, element by element, even when the view that asks for the morph (`Cmd::Morph` from a
changed state) poses its glyphs elsewhere or hides them: a stage's glyphs may be posed anew by every step,
and the tiles start where they were seen.

**Sampling (CPU, once per morph).** An element's instances, within their ink grown by `SPREAD` (the SDF's
range), are cut into cells on the screen grid; each cell's coverage is the average of four points at the
cell center ±P/4 (the way `dottr` reads coverage): glyphs read the CPU copy of the atlas, shapes
compute distance on the CPU, pictures are 1; ink in the page background colour (tone and accent both 0) does
not count as visible. Cells with coverage ≥ `threshold` fly; the rest are "dust" and stay in place, the
source's dust fading out over the first 20% of its progress and the target's fading in over the last 20%.
When the total passes `count_max`, cells become 2, 3 … times the lattice wide until it no longer does;
nothing is dropped at random.

**Pairing.** Each side's flying cells are sorted by Morton code normalized to their element's bounds and
paired by rank (`morph::pair`, a pure function): pairs = max(ns, nt), every source and target used at least
once, deterministic. With more targets, the extra targets start from the source of nearest rank (one
starting point, `SPLIT`, fading in over the first 20%); with more sources, the extras land on shared targets
(`MERGE`, fading out over the last 20%).

**The record** (`Particle`, 48 bytes, uploaded once per morph): `ab` the top-left corners of the source and
target cells (device pixels); `run` start and end progress, delay, duration (seconds, on this flight's
clock); `misc` kind, cell size, seed, and the progress at which the drawing style switches.

**The shader** (`particles.wgsl`, vertex and fragment only, works on WebGL2, no per-frame CPU work):
`l = clamp((clock − delay) / dur)`, progress `u = mix(u0, u1, ease(l))`; position
`mix(A, B, u) + curl · sin(πu) · (normal · (0.8 + 0.5n₁) + forward · 0.35n₂)`, where `n` is smooth noise at
the source position (neighbours drift together, so the group travels in arcs) plus jitter from the seed;
dust doesn't drift. Delay = `stagger × flight × 0.95 × (0.6 × the side facing the target first + 0.4 ×
noise)` and each cell's own duration = `flight × (1 − stagger)`, so the last cell lands before the flight
ends and the front ones peel away first. Fragment: content cross-fades between the two images by `u`; where
the two sides are drawn differently (a picture on one side, text on the other, different `near.g`) each cell
switches at its own progress point. In the air: ink in the page background colour (the ground of a default
button) counts tone and accent as coverage, so it doesn't punch background-coloured holes in what it passes;
in a morph involving a picture, all text cells take the theme colours they show and are drawn as picture
during the flight (after 30 ms and before the last 30 ms), because a pixel can only be drawn one way and a
text cell over a picture cell would be taken for the picture's colour. `near.b = 1` (fully loose).

**Three phases.** 1. The source's looseness goes from `k0` (usually 0) to 1 over `loosen_ms × (1 − k0)`.
2. Hand-off: from this moment the source isn't drawn and particles take over at exactly the same positions,
flying for `duration_ms`; the target is laid out but not drawn. 3. Particles give way to the target at
looseness 1, which closes to 0 within `tighten_ms`. Looseness uses the existing mechanism (an instance's
static looseness, ramped on the CPU each frame), so under `plain` it is plain's own loose look (fading).
What a morph shows does not condense a second time.

**Reversing.** When the morph requested is exactly the reverse of the one in progress: during the flight,
the CPU computes each cell's current progress with the same formula, rewrites it as `u0 = now, u1 = the
other end` with delay 0 and uploads again, so particles return from where they are along the same path;
while still loosening, the source closes back from its current looseness; while closing, a new morph starts
from the target's current looseness. A new morph during a different one ends the old one at once.

**Parameters** (`MorphSpec`, all numbers; the website's particle page sets them with number controls and the
easing with a segmented choice):

| Name | Default | Meaning |
|---|---|---|
| `loosen_ms` | 220 | How long the source takes to loosen |
| `duration_ms` | 900 | The flight, from the first cell leaving to the last landing |
| `tighten_ms` | 260 | How long the target takes to close |
| `count_max` | 40000 | Particle limit |
| `threshold` | 0.5 | How much coverage a cell needs to fly |
| `curl` | 48 px | The farthest a flight strays from a straight line |
| `stagger` | 0.35 | The share of the flight that departures are spread over |
| `ease` | ease in-out | linear / ease in-out (cubic) / exponential deceleration |
| `lattice` | 5 | The material lattice pitch (`Fx::pitch`), filled in by the application |

#### Particles on the GPU

When the cell count passes `Particles::gpu_above` (20000 by default) and the renderer can do it, a morph does
not cut and pair cells on the CPU but simulates a grid on the GPU (`morph::Grid`): the ink bounds of the two
elements (grown by the SDF range) are each covered by a grid with the same rows and columns, cells a whole
number of device pixels and no smaller than the lattice (`MorphSpec::lattice`), at most `count_max` cells;
cell i pairs with cell i (pairing sorted by position, non-crossing like the CPU's Morton ranks). Departure
times, durations, arcs and the style switch point are computed from the index hash and position as on the
CPU. State lives in a texture, one texel per cell: the offset from the eased path and its velocity, stepped
at 1/60 s (a fixed step: the same clock gives the same state): curl noise (`curl`, `curl_scale`), pointer
attraction or repulsion (`pointer`, `pointer_radius`), a push from passing fronts (`front`), damping, and a
spring back to the path. The drawn position is path + offset × sin(πu), so at progress 0 and 1 each particle
is exactly on its own cell and the hand-off and landing frames are unchanged (`check` item 4 tests both
paths, and that the GPU path gives the same result twice on the same clock).

Three tiers (`render::ParticleTier`, which the host tells the renderer from the adapter): WebGPU and native
use a compute shader (`sim_cs.wgsl` writes an rgba32f storage texture); WebGL2 ping-pongs with a fragment
shader between float targets (rgba32f, or rgba16f if that fails); with neither, the CPU path stays, and the
readout says "cpu (no float targets)". Integration is one WGSL function (`integrate` in `particles.wgsl`).
The readout (F2) shows the particle count and where it is simulated (`gpu compute`, `gpu rgba32f`…). The
particle page (`#particles`): mesh → text → picture → another mesh, the particle count is a number control
(up to 1,000,000), the pointer pushes particles away, and a press on the stage sends a front. Samples:
`samples <dir> particles`.

#### Flows

`El::flow(id)` gives a subtree an identity across pages. In the layout after `Cmd::Flow`, within the subtrees
with the same flow id on both pages, instances are paired by "flow id + index in the subtree" (glyphs counted
apart from other instances, and glyphs must be the same glyph); only instances that disappear from an old
position and appear at a new one are paired. Paired instances keep their birth time (they don't condense
again), and their rect (size included) moves from old to new over `flow_ms` with ease in-out, their looseness
raised by a bump of `flow_loose × sin(π e)`, loosest in mid-flight and sharp on arrival. In flight they are
drawn on a layer above the page (pictures and shapes first, then text), so they never pass beneath other
things and are never taken for a picture below. Interpolation is on the CPU: the runtime keeps each
instance as laid out and, each frame (`tick`), edits a copy by the flow and morph ramps; nothing is done
without animation. A flier's clip goes from its old clip to its new one in step with its rect, so what was
inside its clip at both ends is inside it all the way. Unpaired instances of the old page stay in the scene,
loosening to 1 and fading out within `out_ms`; new instances of the new page fade in over `fade_ms`.

A subtree with a picture on both pages grows instead (an index's window into its piece, and back): its first
picture on each page are paired whatever their places in the subtrees (a picture, not cells that read one:
`el::cells` is a picture instance too), and the picture is drawn as one, its
box going from the old one to the new one and shown only inside the part of it that shows, which goes from
the old one's (the window) to the new one's (the full view); both on whole pixels every frame. It is drawn
in the page pass as the page's last picture, after everything that dissolves, so it is above every other
window, and what rides on it is ink over a picture (its own colour). Nothing else of the subtree is
paired, since the two pages lay out different things over the picture: the rest of the old subtree rides with
the picture. Type keeps its size: an element's glyphs go together (a word, a label), whole, their centre (an
own-size label's own point, `El::own_size`) where the picture takes it; shapes that touch go together the same
way, each part whole, so no box of panels comes apart. A drawing's marks are geometry, not notes: each goes with the picture's transform, its pen kept (a line's width across), with no setting (as design's U rules: geometry scales, strokes stay as drawn, only type keeps its size), so a drawn box of four lines stays closed and a drawing over a window's picture (a curtain of rays) shrinks into the window with it and is never drawn at its page place over the index; under a view scale (`El::view_scale`) the whole old subtree is the layout's geometry and goes so too (a border keeps its width), and only own-size labels ride at their own point (`tests/ride_scaled.rs`, with no view scale). It stays inside
the part that shows, without loosening, whole until the picture lands; what of it lies along the edge
of the part that showed (its centre within 3 px of it: a frame round the window) is the window's, and fades
over `out_ms` where it was, under the picture. The rest of the new subtree is there at once when the picture
lands, as the old goes, so each note over the picture is drawn once every frame and never missing; everything
new outside the flows comes in over `fade_ms` once the picture has landed. What the new page lays out only
after the flow began (a view while it is on its way: a note shown once what it follows is in view, a
drawing's new marks) comes in as what was new when it began does, from the same start (`Ui::flow_late`):
never drawn at once where it is, outside the part of the picture that shows. A picture that grows keeps the look's ground the old
page's until then (`Ui::look`), so around it is the page it came from, and the new page's ground comes in
under a picture that covers it. A picture that shrinks takes its page with it: the look's ground is the new
page's at once, what else of the old page lies in the part that showed rides with the picture too, and what
lies outside it goes at once, so around the picture is only the new page's ground. Text that flows with it
(a title) is paired as above and drawn over the picture; neighbouring windows are old instances that dissolve
under it.

An element keyed the same (`El::key`) in the old and new subtree of a picture that grows is not two layouts: each of
its instances (matched by which of the element's instances it is: glyph or not, how many of that kind before it, and
the glyph) goes from where it was to where it is with the picture. Its point (its centre, or an own-size label's own
point) is carried by the picture's transform, and what is left between that and its new place closes on the
picture's cubic, as does its size (a line's width does not change); its colour goes linearly. It is drawn once every
frame, over the picture after what rides on it, inside the part that shows, and lands on its new layout within half a
pixel. Own-size type keyed the same at another type size on the new page cannot change size on the way: it is
reported (`ViewProblem::FlowOwnSize`) and the two cross-fade once the picture lands. One keyed the same that rode on
the growing picture on the old page and is outside every flow on the new (a layer over the page's frame: the site's
pointer box and its label) is carried by that picture in the same way (its glyphs matched by their place in the
text), never drawn at once at its new place outside the part that shows (`tests/flow.rs`). A picture with the same
flow id on both pages whose box changed grows too. `tests/keyed.rs` (into the piece and back).

On the website: on a page change, the page title (`page-title`) and the side rail's selection background
(`rail-selection`) flow, the selection sliding from the old item to the new one; on the page-change page,
the cards' pictures and titles flow into the detail view. Parameters (`FlowSpec`): `flow_ms` 420,
`flow_loose` 0.25, `out_ms` 180. The works index into a piece: `flow_ms` 420, `flow_loose` 0, `out_ms` 180,
`fade_ms` 180.

**Acceptance.** `cargo test`: pairing (every cell used, the pair count, deterministic, non-crossing), the
easing's ends, exact landing, reversing starts from current progress and returns to the start, and the
count limit enlarges cells by whole multiples; the runtime's phase times (half loosened, before and after
the hand-off, the target appears at looseness 1 without condensing, the end), reversing in flight and while
loosening; flow rects at t = 0 / ½ / 1, kept birth times, unpaired instances dissolving, a straight swap
without a flow command; the commands of two website pages; `tests/flow.rs` (pages built as the site's: a
picture with abs-placed text notes, a `Drawing` over it, shades at a real alpha, a window with a frame drawn
round it and a live line under it, a neighbouring window; an index window growing into its piece at 420 ms
and back, both ways at 40, 80, 200 and 350 ms: one picture in one rect on whole pixels where the ease puts
it, its birth time kept, the index's ground around it, the notes over it in their own colour and no green
over it, every other instance drawn at a size and colour one of the pages draws and not loosened over the
picture, the frame and live line gone by their opacity within `out_ms` and never drawn over the picture,
no other window over it, the title's glyphs once each, over the picture, opaque and inside their clips;
landed, the new page as at rest on its own ground; back to an index whose other window is in a layer, that
window's ray, a ramp in a colour of its own through a middle stop, not drawn at 0, 40, 80, 200 and 350 ms and never
in the accent while it fades in, in its own colour at rest). `check`: the hand-off frame (source at looseness 1
against particles at t = 0) and the landing frame (particles at t = 1 against the target at looseness 1),
under `plain` and `dottr`, differ by ≤ 2/255 per pixel within the elements' bounds (in fact 0); in the light
theme mid-flight there are particles outside both elements (with a control). `film` gives samples for every
pair × every material × seven moments, and the page change there and back at five moments each. The web
version under WebGL2 captures three mid-flight frames on a virtual clock, all different, with no console
errors.

With less motion a flow is a fade in place over `FlowSpec::fade_ms` (150 ms by default; the website uses 120 for
its route fade), and what comes and goes fades over the same time (`tests/transform.rs`).

With or without less motion, what is new on the page (not shared with the old one) fades in over `fade_ms` where it
is laid out while the old page fades out, so a route change never draws the new page whole over the old from its first
frame. With no flight (`flow_ms` 0, an opacity change) nothing is shared: the whole old page fades out and the whole
new one in, shared parts too. A new window size during the flow (a bar that goes with the old page) moves what fades in with the page, and
shared parts fly on to where they now are. `tests/transform.rs`
(`a_flow_fades_the_new_page_in_and_goes_on_through_a_resize`: with 120 ms the new page is under half its opacity at
40 ms and the frames at 40 and 80 ms differ from both ends; with the 420 ms default the shared tile grows at 60, 200
and 350 ms through a resize and ends exactly on its box).
