# 1.35 The stage

`El::stage()` marks the live piece a page is about (a Lab cell's picture, a demo's canvas), so that controls beside
it answer the hand even when the piece is slow to draw. A stage that draws within the frame is drawn every frame
like anything else: the thumb, the value and the piece all show a change in the same frame. The runtime times each
draw of a stage's subtree; when its draws average more than 0.6 of a frame (the frame being the time between
ticks) over the last 8, the stage is **slow**, and it is no longer slow once its draws have stayed under 0.4 of a
frame for 1 s.

A slow stage holds the image it last drew on a frame that answers input (a pointer move, a press, a key, text, the
wheel): its subtree is not painted, and the instances it drew last are drawn in its place, so the renderer has
nothing new to draw there and the controls are drawn in the frame. It draws again on the first frame without new
input, on the first frame after a release, and during a long stream of input at least every
`max(100 ms, 6 × its average draw)`, at most 500 ms (`stage::HOLD_MIN_S`, `HOLD_DRAWS`, `HOLD_MAX_S`). A draw
replaces the image whole, at once (a cut), and nothing marks a stage that holds. A stage whose box, clip or
number of elements changed draws whatever the input. Holding costs nothing beyond the stage's own draws: a
frame in which a held stage would still hold paints nothing for it, a step of it (`Ui::on_step`, below) that
comes while it would hold is not viewed but waits for its next draw, and when it may draw it is viewed and
drawn alone, as a step is, the rest of the page copied (`Ui::needs_frame` asks for that frame, `next_wake`
gives the end of its hold). A stage that held is drawn whole when it may, never by its parts alone (below):
its held image is behind all of it. When a stage's own device scale steps down under load
(the governor, below), that acts first; a stage is held only when the lowest scale is still slow. Not for controls: what is inside a stage that holds does not change while it
holds.

A stage that changes by itself (a Lab cell that runs, a demo that moves) need not rebuild the page it is on.
`Ui::on_step(f)` steps the stages' pieces on every tick, after `on_tick`, and returns the keys of the stages that
changed; `Ui::stage_view(f)` gives one stage's subtree by its key, as the page's view would make it. When nothing
else changed the view on that tick, only the stages that stepped are viewed again: each new subtree takes the
place of the old one in the boxes the last layout gave it, if it is laid out the same (the same elements with the
same keys, styles and text to measure); its places are worked out again (scroll, transforms, clips), and the paint
draws only those subtrees, copying every other element's instances, glyphs and colours from the last paint. The
frame is the one a whole view, layout and paint of the page would give, instance for instance. A step whose
subtree is laid out otherwise (it grew, an element came or went, a text changed what is measured), a stage that
holds a 3D scene, an animation or a transition in flight, or anything else that changed the view on that tick,
and the whole page is viewed, laid out and painted again. Text recomposing glyph by glyph (§1.16), in a
stage or anywhere else on the page, does not: what is copied is copied as it was painted, not as the last
frame's motion left it (a glyph hidden under its tiles, one sliding), and the tiles are drawn over it as they
are on any frame. `perf().counters.restages` counts the frames drawn so (a frame that had to paint the page
whole is not counted).
What changes with a piece outside its stage (an experiment's formula, a label with the value) is marked
`El::steps`: `on_step` returns its key with the stage's, `stage_view` gives its subtree, and it is put in
place as a stage is, but drawn every frame like anything else (never held, never at a stage's scale). An
element marked `El::steps` inside a stage is a **part** of it that changes alone (the Lab's Loupe: a lens
moving over a page that stays): a step that names the stage views only its outermost parts, by their keys
through `stage_view`, and puts them in place as a stage is, and the paint draws only them, the rest of the
stage copied from the last paint as the rest of the page is. A part that is not viewed (`stage_view` gives
`None` for its key) or is laid out otherwise, and the stage is viewed whole as above. A slow stage a part of
which was drawn alone draws on its next paint whatever the input, since its held image is no longer what it
shows. So the page that names a stage's step need not know what in it moves. Text
counts as laid out the same when its words changed but it measures as before at its widest, at its
narrowest and at its width (figures of one width: the mono face, or a fixed box). An absolutely placed
element whose offsets alone changed (each side a length before and after, the opposite side auto; a chip
dragged about a stage) counts as laid out the same too: it is moved by the change on whole pixels, with its
subtree, and the tree takes its new offsets for the next layout.

**Messages that are a subtree's own** (`El::local_messages`): what the handlers in such a subtree send (a press,
a drag, a key, the pointer over it, the wheel) is taken to change only it and the elements marked `El::steps`.
After such a message the subtree (by its key, through `stage_view`) and those elements are put in place as a
stage that stepped is, and drawn again; laid out otherwise, the page is viewed and laid out whole. A stage the
hand drags its chips about (the Lab's Weld) is marked so: a drag frame then costs no view of the page and no
layout.

The hold reads the input of the frame that lays it out: with input coalesced (`Ui::coalesce`, as the hosts
run), a tick leaves the frame's input to the paint that `Ui::settle` makes, and that paint spends it.

**A stage's own scale.** When the machine cannot draw a page in time, the stages are what give: their
device scale steps down 2 → 1.5 → 1 → 0.75 (from the device's own; on a 1× display the only step is 0.75),
while text, panels and the chrome stay at the device's. The governor (`perf::StageScale`, fed by each drawn
frame while stages are on the page) goes by the intervals between frames, since a GPU's time is not
measured on the web: the display frame is the 10th percentile of the intervals of the last second that had
no step, never under 4 ms. It steps down when the 95th percentile interval over 3 s is above 1.5 display
frames or 20 ms (50 fps), whichever is less (intervals come in whole frames, so 1.5 is the first value that
means frames were missed; the 20 ms catches a piece that misses from its first frame, which would otherwise
measure a display frame of two). It steps up after 5 s with no missed frame and every frame's cost under
60 % of the display frame, and only when the 95th percentile cost over those 5 s, times the pixels the level
up has more (the new scale over the old, squared: × 1.78 from 1.5 to 2 or from 0.75 to 1, × 2.25 from 1 to
1.5), is under 85 % of the display frame; never while a pointer is down. A step down within 5 s of a step up
holds the lower level, and so does a second step up (`perf::STAGE_UPS`, all levels together), until
`Ui::reset_stage_scale` (a new route) or a new window size: a route's visit has at most two steps up. Every step is a cut. The look's quality below 1 pins the scale at that
share of the device's (the nearest step), and the governor rests. `perf().stage_scale` and
`Perf::stage_label` (`stage 1.5x`, shown dim by a status line; nothing at the device's scale) say where it
is; the readout's look line shows it too.

The renderer draws a stage's ink and nearness at the device's scale as everything else, and shades its
pixels in the reduced material pass at the stage's scale, filtered up into the frame; a pixel whose reduced
neighbours are not all the stage's is shaded at the device's scale, so the stage's edge is sharp. What is
drawn after a stage within its box (the modal layer, menus, popovers, tooltips, the readout, a caption laid
over it) is shaded at the device's scale: the renderer keeps up to 16 boxes over the stages for it, each
what it touches merged. Up to 16 stages are shaded at their scale; more are at the device's. A stage held
under input (above) is not shaded again, since it does not change. The scale steps first: a stage is held
only when its scale is as low as it goes (or pinned, or no renderer measures it).

`Ui::slow_stage(ms)` adds `ms` to every stage's draw, for checks (by the stage's pixels: at 1.5 on a 2×
device, 0.5625 of it); `Ui::stage_state(key)` says whether a stage is
slow and how many times it has drawn.

Acceptance: `tests/stage.rs` (with 40 ms added to a stage's draw, a 2 s drag of a slider beside it at 60 Hz, with input coalesced or not, moves
the knob and changes the value in every one of the 120 frames, every frame that does not draw the stage takes
under 16.7 ms, the stage draws at most once per hold during the drag (the count is printed) and on the first
frame after the release; a stage that draws in time draws every frame of a drag; a held stage asks for a frame and
draws on the first frame without input); `tests/stage_step.rs` (a stage that steps alone for 150 frames is
viewed alone with no layout, and every frame's instances, owners, held text and boxes are those of a page viewed,
laid out and painted whole, with a count beside the stage marked `El::steps` viewed with it; a step that makes
the stage taller lays the page out; a stage whose messages are its own, dragged for 30 frames, is restaged in
each with at most one view and one layout in all, its chip where a whole layout puts it). `tests/stage_scale.rs` (a stage
costing 24 ms at 2 and by its pixels below, on a 60 Hz display, settles at 1.5 within 4 s and changes at most
3 more times in 60 s; one whose level up would fit by its pixels but misses there comes down within 5 s and
holds at 1.5; one at 59 % of the frame at 1 never steps up; one heavy for 5 s of every 20 steps up twice in
120 s and then stays down; no step up while a pointer is down; at 1.5 on a 2× device, every pixel outside the stage and
under a tip drawn over it is the one at the device's scale, and the stage's are not; the governor stepping down
twice and back up shades only the stages again, each frame the frame drawn whole at that scale, and the frame
after shades nothing). A step of the stages' scale is not a whole frame: the stages are shaded again where they
are (their boxes among the shade boxes, in the reduced pass and the filter up), the rest is kept. On the website's Lab index at
1440×900 2× with one cell running, a frame takes 2.9 ms median and 3.1 ms p95 on the CPU, from 6.9 ms.
