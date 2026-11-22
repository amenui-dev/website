# 1.26 场、绘图和字形姿态

The experiments of fields, the Lab and the decode transition were first broken down into primitives from first
principles, to see what the foundation lacked:

| Experiment | Comes down to | What was missing |
|---|---|---|
| Lit text (Lumen) | Each glyph's tone by its distance to the light | No access to each laid-out glyph |
| Contours | Scalar field → grid sampling → marching squares → polylines → strokes | No fields, and nothing could draw a slanted line (2D instances couldn't turn) |
| Silk | Many points advected by a vector field (curl of noise + flow stirred by the pointer) | `motes::FlowField` was special-purpose code with field and drawing welded together |
| Grown | Reaction–diffusion masked by an element's true coverage | The state couldn't get an element's true outline |
| Mercury | Metaball scalar field → contours + dots by value | As for contours |
| Text on a path / sphere (Flow, Loupe) | Each glyph's translation, rotation, scale and opacity | Instances had no transforms of their own |
| Orbits, spring, card stack | Points and strokes; springs | Only element boxes could be drawn |

So the foundation changed in five places, with no special cases in the effect pages:

- **Per-instance transforms** (`Instance::xf`, the eighth vec4): a 2×2 linear transform about the instance's rect
  center, stored as "minus the identity", so zero means no transform and no existing constructor changed. The
  vertex shader turns the four corners by it, and `exact_i` maps pixels back into local box coordinates, so atlas
  coverage and distance and shapes' analytic distance are computed on the turned outline and materials work as
  usual. `Instance::bounds` gives the bounding box after turning, used by partial redraw. Sampling follows the
  transform too: `morph::cover_at` first maps the point back into the box through `xf` (`morph::to_box`, like the
  shader's `exact_i`), and `cells` and `ink_bounds` use the turned bounding box, so a posed or turned glyph
  loosens into particles from where it is drawn, and `on_outline` masks and decode blocks are there as well
  (`a_posed_glyph_loosens_from_where_it_is_drawn` in `tests/transform.rs`).
- **Glyph poses** (`El::pose(|GlyphAt| Pose)`): asked once per laid-out glyph when the scene is built (index,
  count, center, content box, the range of all glyph centers, and the pen as laid out: `pen`, the x of its origin
  before the glyph's image is put on a quarter pixel, `base`, its baseline as drawn, and `advance`, kerning in
  it; a pose that moves a glyph by `target − pen` puts its pen at `target`), giving a translation, an angle, two scales,
  multipliers for tone and opacity, and a minimum accent; glyphs with opacity 0 aren't drawn. No layout or
  shaping: pointer movement only rebuilds the scene. Posed text is culled by where its glyphs go, not by its box: a
  box outside its clip still draws the glyphs posed into it, and the glyphs posed outside the clip are not made.
- **Drawings** (`el::canvas(Drawing)`, `src/draw.rs`): an element whose content is marks in its own coordinate
  space (`Mark::Dot`, `Mark::Line`), scaled to the element's width with the height in proportion. Each mark is a
  shape instance: a dot is a rounded square and a stroke a turned bar with round ends, so they have true
  outlines, every material can draw them and fronts can loosen them. Marks change every frame, so they don't
  condense (their birth time is long past). Pointer handlers on a canvas (`on_move`, `on_drag`, and
  `El::on_point`, which combines move, press-drag and leave in one handler, fingers included) report coordinates
  in the canvas's space.
- **A stroke with a ramp** (`Drawing::ramp(a, b, w, paint, mid, far)`, `Mark::Line`'s `far` and `mid`): its
  colour and opacity go from `paint` at `a` to `far` at `b`, through `mid` (a paint and where it is, 0–1 of the way)
  if given, still one instance (the far and middle paints packed into its `uv.zw`, their hues and the middle's place
  into `extra.x`, all negative; a line without a ramp is the instance it was). The instance's opacity is its strongest
  stop's and each stop's is a share of it (the start's in `extra.y`), so whatever scales an element's opacity later (a
  fade, a flow's wait, a see-through group) scales every stop: a ramp waiting at opacity 0 draws nothing. The ramp's parameter is the pixel's
  projection on a→b, `clamp(dot(p − a, b − a) / |b − a|², 0, 1)`, so each round end is its end's colour, flat. Over a
  picture (where ink is written as its own colour) the stops' colours are mixed in OKLab, each piece linear with no
  smoothing at the middle stop, and the opacity, linear along the length, is applied after the mix. Off a picture ink
  is a theme mix with one accent hue a pixel, so there the paint's numbers (tone, accent, opacity) go in a line and it
  keeps `paint`'s hue all the way: a ramp within one hue, or to transparent, comes out as drawn there; one from one
  hue to another is reported (`ViewProblem::RampHue`, a P1 in the audit). 540 ramps are one draw.
- **A drawing's own colours** (`Drawing::palette([...])`, sRGB 0–1): a mark's `Paint::hue(3 + k)` is the drawing's
  colour `k` in place of a theme accent, its tone and accent mixing towards it as towards one, so a piece's marks are
  the same wherever they are drawn (its page, a window onto it, a band) whatever the theme there. The runtime gathers
  the drawings' colours each frame into one table of at most 32 (`draw::OWN_COLOURS`; the same colour in two drawings
  counts once, in the order first drawn), renumbers each mark's hue into it (3 + its place) and hands it to the
  shader with the theme's (`Scene::own`; a colour that changes under the same hue redraws everything). Over a picture,
  where ink is written as its own colour, they are exact and a ramp between two is mixed in OKLab as between theme
  hues (a ramp packs its stops' hues in 6 bits each). Off a picture, where ink is one of the theme's three accents a
  pixel, a mark in one of them is drawn in hue 0 and reported (`ViewProblem::OwnColourOffPicture`, a P1 in the
  audit; what is drawn of a mark, cut by its clip, is what must be on a picture), as a ramp across hues is there; so is a hue past the drawing's palette or the table, and outside a drawing
  a hue of 3 or more is hue 0. `tests/own_colours.rs` (W-04's ray in its own three colours over its sky, by hand in
  OKLab within 2 / 255 under a dark and a light theme and the same pixels under both; off a picture hue 0 and a P1;
  two drawings sharing a colour count it once).
- **Blades** (`Mark::Blade`, `Drawing::blade`): a rooted stroke that the field map (§1.29) bends, for a field of
  grass. `root`, `len`, `width` (at the root, 1.5 by default; the tip is half of it, 0.5 px at least), its own
  `lean` (0), `max_lean` (57°), `stiff` (0.5), `paint`, `lean_paint` and an optional `head` glyph. Its lean is
  `lean + max_lean · clamp(flow.x / drift, −1, 1)` with the flow read at the root, so vertical flow never lays
  it down: its own lean and the flow's add, all of it within ±`max_lean` (and ±85°); its colour is
  `mix(paint, lean_paint, m)` and its head's `mix(head.paint, lean_paint, m)`, where `m` (`Blade::mix`) is
  `|lean| / max_lean` by default (`lean_from` 0, `lean_full` 0), and with `lean_full` above `lean_from` 0 up to
  `lean_from`, 1 from `lean_full` and a smoothstep between (a field that greens only once it really leans: 8° and
  20° on the website). The lean is the
  angle of the tip's tangent from upright. Along the blade, a share `s` of its length from the root, the tangent
  is `lean · (0.35 + 0.65 · g(s))` (`Blade::tangent`): the root leans 0.35 of the tip. With `stiff` 0, `g(s) = s`,
  one even bend from root to tip; with `stiff` above 0, `g` is 0 over the lower `0.6 · stiff` of the blade (that
  part is straight) and then a smoothstep to 1 at the tip. What falls over is the head, not the whole stalk laid
  flat: the chord's angle is about two thirds of the lean with `stiff` 0 (38° at 57°, 49° at 72°) and less when
  it is stiffer (33° and 41° at 0.5). The shape
  (`Blade::at`, `Blade::shape`, the same in the shader) is walked by arc length in 16 steps, so the length is kept.
  It is drawn as 10 segments when it is longer than 48 logical px, 6 when longer than 24 and 2 otherwise, their
  ends spaced evenly in `(s + 3 · g(s)) / 4` so the segments are shorter where it bends and no two neighbours
  are more than 12° apart: strokes with round ends cut along the bisector where two meet, tapering, the first and
  last pulled in by half their width so the blade runs exactly from root to tip. The head's ink box has its bottom middle on
  the tip and its up along the tip's tangent; the stem ends there. All of it is worked out in the vertex stage
  from the map at the root (`blade_seg`, `blade_head`): while only the map moves, nothing is laid out, painted
  or built on the CPU, and the instances stay as they are. A blade is never pushed by fronts or the flow (it
  leans instead) and redraws wherever the map changes under its reach. Inside a turned element (`El::turn`) a
  blade is not turned with it.
- **Fields** (`src/field.rs`): `Scalar` (any `Fn(x, y) -> f32`), `Noise` (value noise with time), `Bump`,
  `Balls` (metaballs) and `Sum`; `Grid` (sampled at nodes, read back bilinearly, `contours` producing polylines
  with marching squares, saddles decided by the cell average so lines never cross); `curl` (divergence-free
  flow); `Flow` (a vector grid; `stir` adds velocity, `fade` returns it to rest); `Swarm` (points advected by a
  velocity field); `GrayScott` (masked reaction–diffusion: u = 1, v = 0 outside the mask, no flux across the
  outline). The hash and noise of `motes` were replaced by these, and the old copies deleted. `Drawing::dots`
  and `Drawing::contours` are two ways to draw a field.
- **Two things an element tells the state**: `El::on_visible(|bool| ..)` (entering or leaving view: its box
  intersects its clip; experiments out of view don't step) and `El::on_outline(cols, rows, |Grid| ..)` (the
  subtree's true outline, hidden instances included, as coverage on a grid by `morph::cover_at`; sent only when
  the instances changed). Both are messages the runtime queues after layout or scene building and hands to
  `update` together (`Ui::flush`); the state never changes in the middle of drawing.

Acceptance: unit tests in `field.rs` (a circle's contour is closed and on the radius, open at the edges, noise is
continuous and deterministic, curl has no divergence, reaction–diffusion grows only inside the mask and
deterministically, a stirred flow comes to rest); `tests/lab.rs` (each experiment runs deterministically in the
headless runtime, Grown receives its outline and grows only inside it, lit text has light and dark).

**Depth and facing: text on a form.** A pose also has a depth and a facing: `Pose::z` (logical px, positive toward
the viewer) and `Pose::facing` (1 facing the viewer, 0 edge on, negative facing away). The defaults, 0 and 1, draw
exactly as before. The glyphs of one text element are drawn far to near by `z` (a stable sort; there is no depth
buffer). `El::form(FormFx)` keeps such text legible, with four numbers:

| name | what | range | step | default |
|---|---|---|---|---|
| `back` | opacity kept by glyphs facing away | 0–1 | 0.05 | 0 |
| `edge` | glyphs facing less than this fade out; at 60 % of it they are not drawn | 0–0.6 | 0.05 | 0.25 |
| `overlap` | a glyph whose ink is covered by nearer glyphs more than this is not drawn | 0.1–1 | 0.05 | 0.35 |
| `fog` | tone of the farthest glyphs (the nearest keep theirs) | 0–1 | 0.05 | 0.6 |

`overlap` is how dense a form may be, and it is measured on the true ink, not guessed from spacing: going from near
to far, the turned bounding boxes find the candidates, then `morph::cover_at` on a 7 × 7 grid over the glyph's ink
measures how much of it the nearer drawn glyphs cover; the samples come out of the frame budget (`budget::grant`),
and past it the boxes alone decide.

`El::reveal_by(Reveal::Word | Line | Glyph)` says how clips, poses and the window take text away. With `Word` (or
`Line`), a word (a line) is drawn whole or not at all: it takes the faintest of its glyphs' opacities, it is left
out when any of its ink would be cut by its clip or the window, and on a form it is thinned as one (left out if any
of its glyphs is covered past `overlap`). Words are Unicode word boundaries with the punctuation after a word
belonging to it (`Text::reveal_groups`). `Glyph` is the default and is what happened before.

The `amenui::form` module wraps a block of text onto a shape seen by a `mesh::Camera`. `Form { shape, spin, camera,
cover }`: the shape is `Shape::Sphere(r)`, `Shape::Surface(f)` (how far the surface is along each direction) or
`Shape::Mesh`; `spin` turns it (yaw and pitch, degrees); `cover` is how much of it the block covers (a fraction of
the longitudes and of the latitudes). `Form::pose_of(GlyphAt)` (or `form.pose()` for `El::pose`) gives each glyph
its place, turn and scale on the surface, with `z` and `facing` filled in; across and down use the same angular
step, so glyphs keep their proportions where the shape faces you. `Form::hit(x, y, w, h)` casts the pointer's ray
and returns `FormHit { point, normal, t, uv }`, the place on the surface in the block's own terms (analytic for a
sphere, `Mesh::ray` for a mesh, a march and bisection for a surface function), so what the pointer lights follows
the form as it turns, and a pointer off the form finds nothing.

**Hits follow what is drawn.** `Ui::glyph_at(x, y) -> Option<GlyphHit { key, i, z, local }>` answers with the
nearest glyph drawn at that point, after poses, turns and thinning (`hit::glyph_under` over the last paint's
glyphs, `Ui::glyphs`); `El::on_point` gets the same in `Point::glyph` and `Point::z` while hovering (during a drag
they are `None` and 0). An element turned with `El::turn(radians)` is drawn turned about its centre, and so is its
subtree; hits go through the same transform (`hit::Affine`, `hit::contains`), so a turned button is pressed where
it is drawn, not in the empty corners of its box. A 3D hit says where it is: `MeshHit` has `point` (world),
`normal`, `t` (along the ray) and `uv` (the place in the view); `Pick` is unchanged.

A press goes to the nearest element under the pointer that takes a press or a drag: a button or link inside an
element that drags (a scene hearing the pointer with `on_point`) gets its own press, and the outer element's drag
starts on its own area or on something inside it that takes no press. Fingers follow the same rule.

The recipe `form3d` (`docs/recipes/form3d.md`) is a passage on a sphere with `edge` 0.5, words revealed whole, the
words around the pointer's place on the surface lit, and a drag that turns it.

Acceptance of depth and forms: `tests/form.rs` (a passage on a sphere turned a whole way round in twelve steps
draws no glyph facing away, no two drawn glyphs cover more of each other's ink than `overlap` (+0.15 for the grid),
depths never fall along the drawing order, the same turn gives the same instances, words come and go whole, a pose
with the default depth and facing draws exactly as before, `Form::hit` finds the glyph drawn under the pointer);
`tests/pointer.rs` (after a quarter turn of the sphere the glyph `glyph_at` gives at the same point is the one now
drawn there; a square turned 30° presses at its drawn corner and not at its box's empty corner; a button inside a
pointer scene takes its own press, and the scene still drags from its own area and from text inside it); unit tests
in `mesh.rs` (a ray hits the unit sphere at length 1 with the normal along the point, within 1e-4) and `form.rs`.
