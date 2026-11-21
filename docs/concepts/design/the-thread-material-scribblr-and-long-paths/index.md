# 1.30 The thread material `scribblr` and long paths

A still page is byte-identical to `plain`. When something not meant to be read (a heading, a shape, decoration)
loosens, its fill fades and its ink becomes one continuous thin line. Up to looseness 0.3 the line lies exactly on
the glyph outlines, with straight bridges between glyphs and between a glyph's rings, so the whole element is drawn
in one stroke as clean outlined letters. Looser, vertices one by one slide onto the chord between the vertices kept
on either side and are removed, the two sides of the stroke merging into one line and then into long chords across
several glyphs, while the remaining vertices wander off and the line bends into one soft curve through them. The
one to three places with the most energy knot into tangles of the same thin line looping through itself dozens of
times, and two long tails, one coming in from
the scope's edge to the line's start and one going out from its end to another edge. Coming back runs in reverse:
knots and tails shorten, vertices return in the opposite order, the line settles back onto the outline and the
glyphs fill in. Text to read and controls never become thread and always stay as they are, except a control's
own fill held loose (below).

- **Two parts.** The shading stage's `m_scribblr` is just `exact` (a flat ground, equal to `plain` at `k = 0`). The
  thread is geometry: in `finish`, after decode (`Ui::paint_strands`), the runtime computes the thread for each
  loosened element and draws it as turned round-ended bars (`SHAPE` instances, the same kind as `Mark::Line`),
  after the page's instances, into the G-buffer stage as usual, with true outlines. The element's own instances fade
  by `fill(k) = 1 − smoothstep(0.02, 0.25, k)` (instance opacity changed on the CPU, the original kept;
  `Ui::looseness` uses the original), so the fill has faded before the line leaves the outline, and there is never
  an offset line over a half-transparent fill.
- **How the thread is computed** (`src/strand.rs`, no GPU): (1) glyphs are sampled for coverage at the `grain` pitch
  (`morph::cover_at`), `Grid::contours(0.5)` traces the iso-line, simplified to half a pixel and cached by glyph,
  size and transform; shapes use their rounded box. (2) They are joined in order into one line: each ring walks back
  to its start, then a bridge goes to the nearest vertex of the next ring; one Ramer–Douglas–Peucker pass over the
  whole line records each vertex's distance from its chord (`Rest::chord`), no greater than its parent's
  (`Rest::keep`), so a larger tolerance keeps fewer vertices, always a subset of those kept at a smaller one. (3) The
  tolerance is `tolerance(k) = 0.5 + smoothstep(0.3, 0.5, k) × 0.3 × line height` (a text's line height, a shape's
  short side): a vertex whose value is below it slides onto its chord by smoothstep over half a looseness from the
  looseness at which the tolerance passes it (between 0.3 and 0.5), and once there produces no segment, so at
  looseness 1 every vertex with a value below `0.5 + 0.3 × line height` is gone. While sliding, a vertex follows its
  chord's ends and only its distance from the chord shrinks in proportion, so nothing jumps. Remaining vertices move
  by `smoothstep(0.3, 1, k) × (reach × (0.4 W + J) + pull × drift × (u, v))`: `W` is the curl of smooth noise
  (patches of about 60 px, differentiated analytically, length at most 1, so a whole word drifts together), and `J`
  is a hashed direction per half line height along the path (length 0.3–1), eased between cells and seconds, so
  vertices a chord apart go their own ways and those between them follow. (4) Knots: `clamp(round(path length / 200),
  1, 3)` per element, at the vertices kept to the end where the map's energy is highest, at least 80 px apart
  (`strand::knot_plan`). They are a primary and its seconds: the most energetic is the largest (`knot_r`), each next
  one smaller by `knot_fall` (1, `knot_fall`, `knot_fall^1.75`); each centre sits off the line by 0.5–0.9 of the
  largest radius, above and below in turn, then moves clear of text to read (below). The largest has
  `strand::knots = round(60·knot·(0.5 + 0.5·e)·smoothstep(0.2, 0.6, k))` control points (36 at the defaults, fully
  loose), the others 64–100 % of that; a knot of fewer than `strand::MIN_KNOT` (12) is not drawn, since so few
  points read as a star. The knot is `strand::tangle`: control points that each turn about the centre from the
  last, 55 % of the steps 1.9–2.8 radians (across the knot), 30 % 0.5–1.3 (round it, making loops), 15 % 2.9–3.4
  (back the way it came, a narrow loop); most at `r × (0.25–1)`, `escape` of them set out so the curve gets to
  `r × (1.5–2.2)` and back (a narrow stroke out of the knot); the centre itself wanders within `0.25 r`, and the
  whole is stretched along a hashed way (1.3 against 0.75), so it is lopsided. The line goes from its vertex
  through the curve (`strand::curve`) of those points and back, and goes on. **Curves** (`strand::curve(ctrl,
  max_turn)`): a uniform cubic B-spline with each end point tripled, so it starts and ends on them; sampled
  densely, then drawn as the fewest straight pieces that turn at most `max_turn` (15°) from one to the next, stay
  within a quarter pixel of the curve and are 1.5–48 px long (shorter only at the tip of a tight loop). A B-spline
  does not pass through its points, so where the points fold back the curve makes a narrow loop. (5) Two tails: the one at the end follows the flow there (or, when the
  flow's way reaches less than six tenths as far as the farthest way, the farthest way); the one at the start takes
  whichever of 16 directions reaches farthest without running the same way as the end tail, joined backwards to the
  line's start; a way that has a clear run to the scope's edge (past no text to read) wins over one that stops
  short, so a tail never cuts across the page behind a picture or the controls. Their length is at least
  `max(tail, 0.6 × the scope's long side)`, and when the way meets the edge, exactly enough to reach it; each is one long slow curve: `tail_turns` gentle bends (each 12–25°, smaller with more turns, at about even
  places, one way then the other, so a tail never strays more than 25° from its way) are the control points of a
  curve sampled at 24–32 even places, with a waver of the hand across it (0.6–1.5 % of its length, two or three
  times along it); a way that would pass within 24 px of text to read scores 0.3 of its reach, and a tail whose tip would
  land within 24 px of text to read is drawn back until it does not; the line goes round pictures 12 px clear (a
  picture is drawn over what lies under it); a `Trace`'s points keep 12 px off pictures and 24 px off text to read
  (`strand::clear_path`). The clearances move the control points and the knot centres, before the curve is made
  through them, rather than the sampled curve, so the curve stays one smooth line; tails are cut where they meet
  the line and go round picture corners; every point draws
  back in proportion to `k`. Once the line bends, where it turns into a tail tighter than a 10 px radius it turns
  on an arc of `12 px × smoothstep(0.3, 0.5, k)` instead (from 4 px before the join, or farther back while the
  line there is tighter), then on a curve tangent to the tail three radii on; on the side the tail goes, or the
  long way round on the other when that side is outside the scope or in a box to keep clear of. A line end or a
  tail stub that would run straight for 20 px or more between two control points (a glyph's upright stroke, a
  tail drawn back) gets one more control point halfway, `15 % × smoothstep(0.3, 0.5, k)` of its length to the side
  away from the next turn, so it bends to its end. Tails, and the long line once drawn, are at `tail_tone` (0.5) of the line's opacity, and knots are whole:
  bright knots on quiet lines, so the density reads. Leaving the scope box, the last segment is cut at the edge, and a tail stops before the
  box of text to read or a control (and before calm boxes). (6) Past `k = 0.3` the line bends toward the curve
  through its vertices: the vertices left are simplified (to `2 px × smoothstep(0.3, 0.7, k)`), the polyline
  through them is blended with their curve by `(1 − angular) × smoothstep(0.3, 0.7, k)`, its corners rounded as
  far as the blend goes; below 0.3 the line is the outline. (7) No piece of thread enters text to read: a knot
  whose circle (`2.2 r`) meets a calm box grown by 6 px moves away from it by up to `3 r`, then shrinks, and is
  not drawn below 12 px; a pose vertex inside a calm box goes to its nearest edge; any piece still crossing a box
  grown by 5 px goes round it. The line's opacity is `smoothstep(0, 0.12, k)`, bridges, knots and tails
  `smoothstep(0.1, 0.3, k)`, and its width is `weight` throughout. The looseness the thread shows is `strand::shown(k)`:
  the same at the ends, but it hurries through 0.15–0.4 (crossed while `k` goes from 0.22 to 0.30), where a
  half-drawn letter and a few loose lines look neither like text nor like a thread, and a held pose (a step of the beat) is
  never worked out inside that band (`strand::held`): drawing back from 1 to 0 spends at most 8 frames there. At `k = 0` there are no thread instances (rule 3
  holds by construction).
- **Text to read and controls** (`legible`) don't become thread, their fill doesn't fade, and the map's flow
  doesn't push them (under `scribblr` their instances loosen in place in the G-buffer stage, `look.w` = 1). Rule 3 in
  its own words: the pointer passing over text leaves the text alone. Thread is only for headings
  (`legible(false)`), shapes and decoration, and for a control's own fill that the page holds loose
  (`El::loose` above 0 on a control whose own instances are all shapes): as in every material (§1.15) the fill
  comes apart, here into a thread round its outline that keeps out of its label's box, while within the island
  round the label it stays whole (the faded fill is restored there, `core_at`); fronts passing a control still
  leave it alone.
- **The thread starts from the outline as drawn**: the G-buffer stage pushes instances that aren't for reading
  along the map's flow by `flow × drift` (§1.29), and the thread's rest path adds the same offset, so as the fill
  fades the line lies exactly on it, not as an offset shadow beside it. The thread's bars don't read the map
  themselves (`look.z` = 0); fronts push them as usual (`resp.y` the same as the element's).
- **The pose follows the material clock**: the time for wandering, knots and tails is `mclock`, which advances only
  while the map or something else moves, so an element held loose at rest keeps its pose and wants no frames; seeds
  depend only on the element, not the beat, so poses change gradually rather than being redrawn every beat. With
  a beat (§1.32) the pose is rounded to its step (the thread's vertices are equal one by one within a step), while
  opacity and fill stay continuous every frame.
- **Elements of shapes only** (a rule, a box, a ground) have no knots and no tails, and wander half as far: they
  come apart into one line that wavers gently. Knots and tails belong to text.
- **Budget** (§1.27): an element asks for `(rest vertices + 6 × knot points + 64) × 3` segments, at most 2400; when
  it gets fewer, curves first turn up to 30° a piece instead of 15°, then knots lose points (a sixth at a time), a
  knot left with fewer than 12 is dropped (never a straight star), then every n-th vertex is taken, and finally the
  end is cut short. The thread never breaks.
- **What never becomes thread**: canvases, morphs, recomposing text, decoding elements, and elements in a role
  hand-off or a flow (they loosen as `plain`). Thread appears only when the global material is `scribblr` (not under
  a subtree override).
- **Long paths** `El::trace(Trace { points, progress, weight, dash, gap, pull, stir, paint })`: a very long polyline
  (in the element's own coordinates) drawn by `progress`, bent by the flow by `pull × drift`; with `dash` above 0
  it is cut into segments by arc length from the start (with `gap` between), so segments already drawn don't move as
  it draws; with `stir` above 0 its head stirs the map where it passes each frame. It draws under every material.
  The `#stirr` page's "Draw a line" button draws a looping dashed line through the area.
- **Numbers** (row 9 of `MAT_PARAMS`): `weight` 0.5–3 (1), `grain` 1–6 (2), `reach` 0–64 (40), `knot` 0–1 (0.6),
  `knot_r` 4–120 (48), `tail` 0–1200 (480), `pull` 0–2 (1), `angular` 0–1 (0.35); `Look::beat` (§1.32; a preset's
  `interaction.beat`) sets how long a pose is held. The thread's shape (`strand::ThreadShape`, in `StrandFx`): `knot_fall` 0.3–1
  (0.55), `escape` 0–4 (2), `tail_turns` 1–6 (3), `tail_tone` 0.2–1 (0.5). `strand::figure` uses the same.
- **A drawn tangle** `strand::figure(seed, r, above, below)`: the same knots and tails generated once from a fixed
  seed (one knot of radius `r`, 36 control points, and one line passing through it top to bottom, `above` long above the knot and
  `below` long below), not reading the map, for use as page content (`El::trace`, §4.4).
- **While a dialog is open**: the page's thread and drawn tangles are not drawn (they would land on the dialog's
  layer, above the card); elements inside the dialog behave as usual.
- **Tails**: every segment stops before the box of text to read (not only testing whether a point falls inside, so a
  step never jumps over a line of text), and never passes back through the inner half of the element's own box.
- **Pictures over the thread**: the thread (an element's and `El::trace`) passes behind pictures, and no thread is
  drawn inside a picture's box (`strand::outside` cuts each segment at the picture's box). In the G-buffer stage,
  thread ink over a picture would be read as the picture's own colour (turning red), so it can't be drawn on top.
- **The page** `#material-scribblr`.

Acceptance: unit tests in `strand.rs` (no segments at `k = 0`; at `k` up to 0.3 every vertex is within half a pixel
of the outline, even with the map moving; the ends join; the vertex ranking is nested, with a chord's ends ranked
before the vertices that slide onto it; fully loose, segments are at most 15% of the vertex count with a median
segment length of at least 24 px, and over 30 frames from 1 back to 0 no vertex moves more than `reach / 6` per
frame; `curve` over a hundred random sets of control points: under 3 % of turns over 20°, none over 60°, the
ends on the first and last point, the same input the same output; a knot at `r = 48` with 36 points, for ten
seeds each: under 3 % of turns over 20°, at least 40 self-crossings, at least 6 small loops (two passes through a
crossing less than `3 r` of line apart), at least 35 % of its length within `0.5 r` of its centroid and at least
two strokes out past `1.4 r` and back; 1–3 knots, the largest at least 1.6 times the radius of the next, the
centres not all the same distance from the line, with a knot radius of at least `0.5 × knot_r` at looseness 0.5;
every piece of a tail turning at most 8° from the last and at most 25° from its way; two tails: the end tail's ends at
least 300 px apart, stopping within 8 px of the scope's edge, the start tail at least half as long,
neither entering the box of text to read, and the end tail at least 100 px long at looseness 0.33; nothing outside
the scope box; within budget; the same input gives the same bytes; `figure` twice is the same, its height is
`above + below`, under 3 % of its turns over 20°, and its knot crosses itself at least 40 times; a segment cut at several boxes keeps the parts outside them;
dashes are `dash` long each, earlier segments don't change as more is drawn, and `dash = 0` is solid);
`tests/strand.rs` (no thread and no frames at rest; dragging the box past a large heading makes thread, one line,
with tails running with the flow; text to read has no thread, its instances' opacity and position unchanged and not
pushed by the flow; drawing a title back from 1 to 0 at 60 Hz, at most 8 frames show a looseness in 0.15–0.4;
once stopped the thread is gone and no frames are wanted; at budget scale 0.25 it is still one
line; recomposing text has no thread; with a beat of 83 ms steps the same step is identical and different steps differ, with
0 every frame differs; at the default `angular` fully loose, over nine in ten turns between the line's pieces are
at most 25°, and at looseness 0.2 the line is within half a pixel of the one at `angular` 1; on the sample page
(`amenui::sample`) at looseness 1, 0.7 and 0.4 no piece of any kind crosses the ink box of text to read (grown by
4 px), and the rule has no knot or tail; "回" is four rings (two boxes with inner and outer edges each) with three bridges; neither
end of a morph has thread; "Loose thread" at 64 px fully loose without knots or tails: segments at most 15% of the
rest vertex count, median segment length at least 24 px, at most a quarter of the thread's ink within 2 px of the
rest outline, and the bounding box at least `0.6 × reach` larger than the still text on each side; for the Loose
Thread style page see §4.4); `check` item 20 (1. at looseness 0 byte-identical to `plain` on dark and light
grounds, control: different at 1; 2. no page background in a fully loosened picture on a light ground; 3. at
looseness 1, at 20 places well clear of other segments, the thread's ink integrated along the normal equals
`weight × scale` within half a pixel, measured for 1 and 2 px widths at 1× and 1.5 px at 2×; 4. over 30 frames from
1 to 0, the farthest distance of the thread's ink (bridges and tails only while visible) from the rest outline never
grows; 5. under `scribblr`, dragging the floating box with its trail for 90 frames, partial redraw equals full
redraw); `bench … strand` (200 glyphs fully loose within 1 ms); `craft <dir> scribblr` (eight frames of a heading
from 1 back to 0, each frame stirring the map by its looseness and filling the whole heading up to that frame's
looseness before drawing, so the energy at the heading is at least 0.8 in frame 1; the last frame waits for the map
to settle; dark and light grounds), and `craft <dir> thread-ref` (the style page with the heading held fully loose,
one frame each on black and paper, to compare side by side with the reference).
