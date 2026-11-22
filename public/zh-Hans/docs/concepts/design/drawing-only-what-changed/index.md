# 1.19 只绘制变化的部分

The shading stage used to redraw the whole screen whenever anything moved. Now the renderer compares this
frame's inputs with the last frame's before drawing (`src/dirty.rs`, no GPU) and gets two boxes (device
pixels): where the G-buffer (coverage and SDF) may change, and where the final frame may change. Both stages are
scissored to their boxes (the G-buffer stage clears inside its box first, with `LoadOp::Load` and a clearing
full-screen triangle), and outside is the last frame. The frame is kept in an image of the renderer's own and
copied whole to the target each frame (the web swap chain gives a new image every frame).

- The G-buffer boxes: instances whose records differ, those condensing, those touched by a live front (the circle the front
  swept, plus its width and push, intersected with its scope box), pictures and 3D elements the lens may touch
  (only their intersection with the lens's reach), 3D elements under a burst and 3D elements whose mesh
  fingerprint changed (a 3D element's instance is compared by its meshes' fingerprint, not their number on the
  page, so one that comes or goes draws again only where it is or was, never the frame whole, and the others,
  renumbered, not at all: `tests/dirty_parts.rs`); each instance's rect grown by the SDF range and the push; and where a picture given
  since the last frame (`Renderer::set_picture`: one made as it comes into view) is drawn, with the cells
  that read it, so a picture that arrives is not a frame drawn whole (`tests/dirty_parts.rs`: the next
  frame is the frame of a renderer that had it from the start, byte for byte, drawn in its box).
- The final box: the G-buffer box grown by the material's read range (`dirty::reach`: each material by its own
  parameters, plus picture mark cells), plus the areas fronts act on, the lens (3.5 radii around both old and
  new pointer), bursts, the wipe line's rect and changed instances of the annotation layer.
- Instances are matched in order, not by place: the common start and end of the two lists, and between them
  runs of 4 equal instances found again further on, so a drawing whose strokes change in number leaves what
  is drawn after it matched; each run that did not match, in either list, is a box for each of its instances
  (gathered by `merge`: content scrolled into view at the bottom and a cell that changed what it draws at the
  top are not one box across the window between them), and a run changed in place (as many instances in both)
  a box for each instance, before and after, so marks that move all over the window (drifting motes, a turning
  body's glyphs) are each where they are; one whose clip alone changed (a clipping box, such as a cached
  element, whose edge crossed the viewport's) is the strips between the two clips' edges, grown by the pad.
  Each box ends at its instance's clip, grown by the pad (what is scrolled out of a viewport costs nothing). They are matched only
  between the same cuts of the list (the modal layer, each plate), and when the cuts differ, in place. A
  pixel outside every box is covered by the same instances in the same order, so it is the same.
- **Several boxes**: over 64, boxes are first gathered by where their centres fall on an 8 × 8 grid over them
  all; then merged where they overlap, then two at a time (the pair whose union adds least) down to
  `dirty::MAX_SHADE_BOXES` (32) for the final frame, grown from what changed as it is, and from those down to
  `dirty::MAX_BOXES` (8) for the G-buffer, which draws every instance again in each box (the final boxes are
  one scissored triangle each). Each stage is drawn once per box (boxes may overlap: a pixel drawn twice is
  drawn the same), the G-buffer's boxes in one render pass for each image, each box scissored, cleared and
  drawn in turn (a tile-based GPU loads and stores the whole image for every pass it opens), unless the frame
  needs passes of its own (a morph's tiles, an opacity group, a plate that adds this frame's ink), which
  open a pair of passes per box (`Cost::passes` counts them). `dirty::parts` gives them; `dirty::region` and the readout their union. Acceptance:
  `tests/dirty_parts.rs` (a drawing that gains strokes redraws only its own cell, two cells far apart are two
  boxes under a quarter of the window, seven frames of cells changing under three materials, drawn in their
  boxes, are byte for byte the frames drawn whole, and 16 motes drifting over a window-wide drawing are
  shaded under 30 % of the window in plain, byte for byte as drawn whole).
- **Scrolling moves the last frame**: when one scroll container's offset moves by whole device pixels (or, with
  its trace showing, stays) and the ground is not in transition, the renderer copies the G-buffer and
  the kept frame inside the viewport by the offset (`dirty::Move`, `Renderer::carry`: copies into a second set of
  images, then the views swap) and the boxes are taken against the last frame moved: the band scrolled into
  view, the strips along the viewport's edges that the material reads across (its reach, beyond the band), the
  scroll bar and what changed. Instances carried with the content are compared at their moved place; what was
  clipped at the viewport's edge keeps that edge. What is not the content's but lies over the viewport is drawn
  again where it is and where the copy took it, but for a plain shape whose edges (radius and border with them)
  and clip are all past the viewport by the pad, or past the window: a page's ground drawn as a shape behind its
  scroll area (`fill(Paint::GROUND)`, the components pages) is one colour over the whole viewport, so it moves
  as the ground does (it was shaded again over the whole viewport, about 85 % of the window each frame of a
  slow scroll of /components/; `tests/scroll_move.rs`). The field map is stirred by an element's motion within the
  content it scrolls with, so scrolling past it is not a change. `Renderer::cost` counts what a frame shades
  (material and G-buffer pixels), the pixels moved and the bytes uploaded.
- **With the trace showing** (§1.8.3, past `Scroll::calm_speed`) the frame is moved too, when the trace points
  the same way in both frames and no material in use is glitchr (its rows slip on their own clock) or one from
  source. The trace changes from frame to frame only near the ink it trails: after the move, a pass over the
  viewport (`fs_trace`, `Cost::trace`) shades again every pixel the trace can reach in this frame or the last
  (`Move::trace`, the stronger of the two: from where the material reads, any ink within as far round the
  trace's samples as they spread, along its longest reach, found by the nearness at probes as far apart as it reads:
  12 px of nearness less that reach, every 4 px at the nearest) and keeps the moved
  pixel elsewhere; the boxes are drawn after it, and the marks over the viewport with them. The strips along the
  viewport's edges grow by the trace's length (what it read there has moved out of sight, or in). Dottr and charr
  hold the rows to their lattice while the trace shows (`latch`): the frame shows the content at its offset
  rounded to the pitch, so it moves by how far that rounding stepped (`Move::frame`, maybe 0) while the
  G-buffer moves with the content. In the frame the hold comes on or goes off in (the trace passing the strength
  that holds the rows), one frame shows the content held and the other at its offset, and the frame moves by the
  difference. The frame moves as the viewport's ground does (held to its lattice, or with the content when it
  holds no rows): the page's material, or the override of an opaque shape that lies under the whole viewport in both
  frames (the components pages draw in plain over the dotted look, so their ground holds no rows; their plain content
  was shaded again over the whole viewport every frame as another lattice); what lies in the viewport in another material (another lattice, or none: a row in charr or
  plain on a dottr page) is shaded again where it is and where the frame took it, as far as its nearness
  reaches (`scroll_move`'s third value; the components pages, with plain chrome round specimens in their own
  materials, moved only one frame in two before). This needs the scroll area alone in its viewport (a scroll area
  inside it that does not scroll itself, such as a strip of chips in a card, is content it carries, shaded again
  where it was and where it is; it had drawn every such frame whole), a step of
  whole device px and no held offset half way between two rows, and the boxes grow by the largest pitch. With a reduced pass for what is loose the frame is drawn whole. With one for the stages alone (§1.35)
  the frame moves and the trace is shaded at full resolution as without it; the reduced grid is fixed to the
  content (§1.35), so a stage the frame carried stands, and while the trace shows every stage in the viewport is
  shaded again where it is (its box among the shade boxes, in the reduced pass and the filter up) (a list with stages at 0.75 under plain, dottr and
  charr, scrolled fast down and up, is byte for byte the frame drawn whole). Acceptance: `tests/scroll_move.rs` (a slow scroll down
  and back up under four materials at two scales is byte for byte the frame drawn whole at every step, with the
  material shading under half of a small window; a fast scroll down, up and settling under six materials at two
  scales is byte for byte the frame drawn whole, moved with its trace in at least 12 of 40 frames; under
  glitchr it is drawn whole).
- The whole screen: the look changed (theme, material, parameters…), the window changed, a commit is showing,
  particles are flying, the low-resolution pass is being drawn, or a material that reads the material clock
  (`dottr`, `glitchr`) is in use and the clock moved.

**The material clock** now follows only the field (fronts, commits, condensing, morphs, flows, recomposition,
flings and overscroll, something loose under a time-varying material); hover bursts, sliding corner marks and
annotations drawing in no longer advance it, since otherwise one hover would set dottr's ground drifting across the
whole window and force a full redraw.

**Why a frame cost what it did**: the renderer leaves in `Perf::path` how the last frame was drawn
(`perf::FramePath`), and the full overlay shows it as one line (`FramePath::line`): `move yes`, or `move no` with the
first reason the frame did not move the last one or was drawn whole (`global` broken down as commit, morph,
loose under a reduced pass, a stage rescaled, an opacity group, plates ageing or emptied, own colours; the look,
the size, the materials or the dottr ground clock changing; soft scrolling; the trace with a reduced pass; a
refusal of `scroll_move`, `dirty::Parts::refused`: the viewport changed, a jump past it, the trace turned, glitchr
or a material from source, the latch switching, not alone, half a row); `spec yes` with the
material, or `no` with why (an override, a material from source, still counting, off); the share of the frame
shaded at full resolution (the shade boxes), with the three reasons whose boxes cover the most of it (`FramePath::shade_why`,
each a share of the window summed before the boxes merge, so they may overlap: what changed, the band a move scrolled in,
the scroll areas, the lens, the field map, the marks, a lattice other than the frame's, a picture given, a plate's
view, a stage moved or rescaled); the soft, stage and reduced scales; and, when the view was laid out since the
frame before, how many times and where the first was asked for (`layout 2 (src/ui.rs:2081)`: the `Ui::relayout`
call, or the call that left it for the frame when input is coalesced; `FramePath::layouts`, `layout_from`). A
scroll lays nothing out. `tests/scroll_move.rs` checks it
against what the renderer counted.

**Hashing**: the runtime's own maps and fingerprints (node keys, instance and glyph ids, signatures) use
`fasthash::FastHasher`, a folded-multiply word hash (as foldhash and ahash mix), not the standard library's
SipHash, which cost about 0.4 ms a frame in wasm on the M1 when the components page laid out; none of those
keys is chosen by an attacker.
A paint that follows no new view (a scroll step) allocates little: the lists a text element's glyphs go through are
kept between paints (`TextScratch`), the layout cache is looked up by the text's hash with the text kept in the
entry (`Text::layout`), the field map is compared with the window's grid before one is built (`FieldMap::fit`), and
a recording text that is the same text only takes its new place and glyphs (`Ui::recompose_jobs`, one index of the
scene for all of them). On the /components/ index a scroll step went from about 2,240 allocations (1.06 MB) to
1,480 (0.49 MB).

**What pointer movement costs**: moving the pointer neither lays out nor shapes text. Parallax uses
`El::shift(dx, dy)` (the whole subtree moves after layout, outside taffy's styles); the lens is counted in the
boxes only where it can reach: beyond `dirty::LENS_REACH` (2.2) radii from the pointer its bell is below 0.01,
picture pitch and dot ground dots don't change, and only on pictures, 3D elements and under `dottr` (other
materials' grounds don't read the lens); picture marks' read range is added only when a picture changed; the
annotation layer is compared as a set (those that appeared and disappeared), so one more character in a label
doesn't make everything after it count as changed. `Ui::dirty_inputs` gives, without a GPU, the inputs the
renderer would compare. Acceptance: `tests/pointer_moves.rs`: on every website page, of 200 pointer moves, those
where what is hovered (a control, a 3D instance, the element the annotation looks at) didn't change do no layout
and no shaping (only annotation labels that write coordinates following the pointer may shape), and a move
starting from rest redraws a box that is not the whole screen and at most 60% of the window (the lens on a dot
ground, a picture or 3D alone covers a third); the annotation crosshair and rulers follow the pointer across the
whole scope and don't fit in one box, so such pages aren't measured for area.

**Acceptance**: `check` item 15: on the button page, hover, press (with a front and the count recomposing) and
move away, 48 frames per material; the picture page and the 3D page (the lens on a picture and a mesh) in
several materials; the floating box dragged, thrown and settling into a corner, 90 frames per material, with
`plain` and `dottr` again at 2× scale; every frame drawn by boxes is byte-identical to the full frame, with a
control: most frames' boxes are less than half the window. The dirty section of `bench` (§9).
