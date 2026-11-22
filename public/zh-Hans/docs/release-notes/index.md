# 更新记录


Newest first. The acceptance checks for each part are in [DESIGN.md §7](docs/DESIGN.md).

Versions follow [Semantic Versioning](https://semver.org) as Cargo reads it: before 1.0, a change to
the second number (0.1 → 0.2) may break code, and a change to the third (0.1.0 → 0.1.1) does not. The
crate and the `amenui` npm package (`web/`) always carry the same version, because the page side must
match the module that `web_app!` generates. The minimum Rust is the one CI builds with (`rust-version`
in Cargo.toml); raising it is a breaking change before 1.0 only in the second number.

## Marks only while the cards move

- A subtree marked `El::audit_under(preset)` is audited under that preset's rules, its accent share of its own box,
  and the rest of the page under the page's (`render --preset`) or the floors alone. Breaking: `audit::audit` takes
  the theme, `audit::audit_under` takes the preset (not its name and rules), `Props` and `Node` have `audit_under`,
  and `Driven::audit` and `Driven::audit_under` have no default body: a wrapper that forwarded only `audit` (so a
  preset run silently used the base audit) now has to forward `audit_under` too.
- A preset carries rules (`Preset::rules`, `preset::Rule`: contrast for running text or all text to read, text size,
  margin, measure, accent share) that the audit holds its pages to as well (`audit::audit_under`, `render --preset`,
  `drive::in_preset`); the ten presets each have theirs. A rule can only tighten: a preset with a contrast under 4.5
  or a text size under 9 px is refused when read (`PresetError::Looser`). Breaking: `audit::Kind` has `Margin`,
  `Measure` and `Accent`; `LookError` has `UnknownPreset` and `BadPreset`, `PresetError` has `Looser`; `Preset` has `rules` (older JSON reads with
  none).
- Under Glitch, text to read is kept clear of the chroma offset and most of the scanlines (`PostSpec::clear_text`,
  off by default): on its ink and 2 px round it the offset is 0 and the scanlines 0.08. Pictures, shapes, the ground
  and `legible(false)` text keep the whole pass. The audit reads a frame bent by curvature where the bend drew it
  (`audit::unbend`; with no curvature, the frame as drawn), and reports text the bend carries past the window's
  edge as `Clipped` (`audit::bent_off`). Breaking: `PostSpec` and `Post` have `clear_text`, `Scene` has `legible`.
- The audit reports a cut as `Clipped` only where no scroll view can bring that edge's side into view: not the last
  row of a page that scrolls, cut by the window's bottom edge (the fold), nor a line in a strip that scrolls sideways
  cut by the strip's end or the window's. A box in the scroll view that does not scroll still clips what it cuts, and
  a scroll view clips across the way it scrolls.
- The audit tells large text by its size (`audit::LARGE_PX` 24, `LARGE_BOLD_PX` 19 when bold): it needs 3:1, a P0
  under it, and a preset's contrast rules do not apply to it. A 64 px headline at 3.28:1 passes.
- At rest a card stack's flat cards are on whole device pixels however large the box is drawn (a page's `El::fit`
  zoom too): each side's outline is one pixel at its grey, where the top and right ones had been two at half
  strength. `Mesh3d.snap` names a rectangle an orthographic view puts on whole device pixels (`Scene3d::proj_in`);
  the card stack's scene sets it to its top card. Breaking: `Mesh3d` has `snap`. A 3D element's edge on a pixel
  boundary reads as exactly there (the resolve's zero on a whole step, and distances within a step of a half pixel
  taken as that), so it covers the pixel inside fully and the one outside not at all.

- The pointer leaving an element whose messages are its own (`El::local_messages`) restages it as a move over it does;
  the leave had viewed and laid out the whole page. On the Lab's loupe this takes a 10 ms frame each time the pointer
  leaves the glass down to an ordinary one.

- Under `dottr` and `charr` a thin line at looseness 1 is a row of the least marks instead of falling between the
  cells: a loose cell with ink in it sampled below 0.25 (`INK_FLOOR`) is drawn as if sampled at 0.25 (`dottr` a dot,
  `charr` the lightest character, centred on the line). Cells sampled at 0.25 or more are drawn as before; `halftonr`
  is unchanged.

- A card stack under `dottr`, `charr` and `halftonr` is drawn as under `plain` at rest: its cards loosen to 1 within
  120 ms while a card's spring moves (`LOOSEN_S`) and back to 0 within 240 ms after the last one stops (`TIGHTEN_S`),
  and then ask for no frames. They had been held at looseness 1. `Stack::moving` is true until they are tight.

## Faster scrolling, no late stalls

- The reduced grid (stages at their own scale, a quality below 1) is fixed to the content of the largest scroll area,
  and the filter up weighs its texels itself: a stage a scroll carries is no longer shaded again in every moved
  frame. The reduced target is three texels larger each way.

- While the trace holds the rows, the frame moves as the viewport's ground does: an opaque shape with a material
  override under the whole viewport (a plain page over a dotted look, as the components pages are) is that ground.
  Their plain content had been shaded again over the whole viewport every frame as another lattice.

- While the trace holds the rows, a scroll area inside the moving one that does not scroll itself no longer makes
  every frame be drawn whole ("latch: not alone"): it is content the frame carries, shaded again where it was and
  where it is. On /components/ that was 81 frames in 180 of a fast back-and-forth scroll.

- With coalesced input (the web and desktop hosts), what a scroll brings into view no longer condenses as if it were new:
  the paint left for the frame is a scroll's when only scrolls asked for it, and when a scroll and something else
  (rows added, a new page) asked for it, what was in the layout before does not condense and new keys do. A fast
  scroll had condensed most of the screen every frame.

- A wheel gesture stays with what took its first step (`WHEEL_LATCH` = 0.5 s between steps, the pointer still):
  a 3D view (`El::on_wheel`, `widgets::orbit`) scrolled under the pointer no longer takes over a page's scroll
  momentum, which stopped the page and laid it out again every frame. An `on_wheel` handler that returns `None`
  leaves the step to scrolling. A layout a handler's message asks for is put down to the handler's call in
  `FramePath::layout_from` (was `Ui::send`).

- The runtime hashes its own maps and fingerprints with `fasthash::FastHasher` (a folded multiply), not SipHash.
- A scroll step's paint allocates about a third less and half the bytes: text's glyph lists are kept between paints,
  layout-cache look-ups no longer copy the text, the field map is not built to be compared each tick, and recording
  texts index the scene once.
- While a scroll goes on, `Ui::a11y_diff` sends only focus: the mirror's new places go once the scroll rests
  (`Ui::next_wake` asks for that frame). Every scroll frame on the web had rebuilt the mirror and sent every
  element in the scroll area again, serialised.

- `FramePath::layouts` and `layout_from`: the layouts since the frame before and where the first was asked for
  (the `Ui::relayout` call's file and line), shown at the end of `FramePath::line` as `layout 2 (src/ui.rs:2081)`.
  `Ui::layouts_since_drawn`.

- A partial frame draws stage 1's boxes in one render pass for each of its two images (was a pair of passes for
  each box: the M1 Lab's median frame opened 8, each loading and storing both images whole on a tile-based GPU).
  `Cost::passes` counts them.

- `FramePath::shade_why`: the three reasons whose boxes cover most of a partial frame's shaded pixels, each as a
  share of the window, shown after the share in `FramePath::line` (`shade 54% (stage moved 43%, changed 12%, band 6%)`).

- A 3D element supersampled at a whole scale (the card stack, at 2) reads each pixel's texel from the pixel's own
  position, exactly: one driver lost the outline's top row and another its bottom.

- A cached subtree (`El::cached`) in a scroll area moves with the scroll: the frame moves its plate as it moves
  the last frame and inks only the band scrolled into view and what changed in it (every scroll frame of the
  Lab's Loupe was drawn whole, "plate emptied"). Any change to a cached subtree now inks its plate only within
  the frame's boxes. `PlateSpan` has a new field, `replace`.
- An instance whose clip alone changed (a clipping box's edge crossing a viewport's) is drawn again between the two
  edges, not in its whole box: the frame a cached sheet's edge scrolls past drew most of the window.

- A frame that moves the last one redraws, for what came into view or went, each instance's box within its clip
  (was one box round each run of them, which on the Lab spanned the window's width): stage 1 shades about half
  the pixels it did on a fast scroll.
- A scroll's trace pass probes the nearness as far apart as it reads (was every 4 px): the same frame, with
  fewer reads for each pixel it tests (it shades a little more of the viewport again, which costs less than the
  reads saved: on the Lab a fast-scroll frame took about a quarter less on the CPU rasteriser).
- The warm-up makes the trace pass (it was compiled on the first fast scroll, a stall of seconds), and after the
  warm-up, idle time makes every material pass, upsample and trace pass the page may need
  (`Renderer::preparing`; was one a drawn frame, so a page at rest made none before its first scroll).
- Each material pipeline (the finishing pass, the upsample, the trace pass, a material's own passes) is drawn with
  once, into one pixel, as it is made: the first frame that needs it no longer waits for a driver to finish it
  (the first switch to a dark scheme with bloom took twice as long as the later ones on the CPU rasteriser).
- Rows of another lattice (or none) in a viewport no longer draw the frame whole while the trace holds the rows:
  the frame moves as the page's material does, and what is in the other material is shaded again.
- A plain shape behind a whole scroll area (a page's `fill(Paint::GROUND)`) moves with the scroll as the ground
  does: a slow scroll of the components pages shaded about 85 % of the window each frame, now the band and the
  edges.

## A Claude Code skill and a save hook

- `.claude/skills/amenui/SKILL.md` is AGENTS.md as the `amenui` skill (made by `tools/skill.sh`; `tests/skill.rs`
  fails when they differ).
- `.claude/settings.json` runs `tools/audit-hook.sh` after Edit and Write: a change to a recipe, a component's
  examples or its widget renders that one page with `render --audit --strict` and hands its P0 and P1 findings and
  the view's problems back to the model.
- `render --page component-<name>`: a component's examples, one under another, as a page to render and audit.

## Examples in English; `near` at rest without the accent

- The component examples and the widgets' `Usage:` lines show English only (Badge "New" and "Draft", Avatar
  "Grace Hopper", ...); Badge's "New" is Primary, not Accent, and Button's disabled example is a disabled
  Primary.
- `near` draws its bands inside the ink in the hairline grey (was the accent): at rest a material shows no accent.

## A text's pens to the state, and its glyphs as drawn

- `El::on_pens(f)`: a text sends its glyphs' pens as the shaper set them (logical px in the content box, the
  numbers `GlyphAt::pen` gives a pose) after a paint, when they changed; a state no longer collects them from
  inside a pose.
- `Ui::glyphs_drawn(key) -> Vec<GlyphDrawn>`: each glyph of a text as the last paint drew it, after its pose:
  its number, the box its raster covers (logical px, window coordinates) and its opacity.
- A message the runtime queues in a paint (`on_outline`, `on_pens`) asks for the next frame.

## near in the ink only; knowing a cached page has no kept image

- `near` at rest is the ink and its greys only: outside ink its three nearest bands are the ink at 0.5, 0.3 and
  0.15 (was the second accent; none further out), and where loose the dim grey at 0.35 (was the third, the
  warning colour). Inside ink the bands stay hairline.
- `Ui::on_kept(f)` and `Ui::kept_image(key)`: a cached element over the plates' limit (four, twice the window's
  area) has no kept image and a `cached_view` of it draws nothing; `f` is told its key and `false`, then `true`
  when it has one again or goes, and the page is viewed again on that frame.

## A card stack's turned outlines are smooth

- `Mesh3d::resolution_scale` goes up to 2 (was 1): above 1 the mesh pass is supersampled, so a silhouette and
  its drawn outline fall on a pixel's share of them. The card stack draws at 2: a turned card's hairline is a
  smooth line, its whole pixels still exactly the grey asked (was steps of two pixels, at 0.5).

## A held stage costs nothing between its draws

- A slow stage holding its image no longer paints the page on every frame it holds: a frame in which it would
  hold paints nothing for it, a step that comes while it would hold waits for its next draw (not viewed), and it
  draws alone, the rest of the page copied, on its own cadence and on the first frame without input. A stage
  that held draws whole then, not by its parts (`El::steps` inside it), so the page comes to rest after it.

## A card stack's outlines, card by card

- `Stack::outlines`: card n's outline grey (the last entry repeats; empty: the ink). The default stack is raised
  cards (`paints [RAISED]`, was the three accents, ink and dim), outlined 1 px, the top one in the ink and the ones
  under it in the hairline grey.
- `MeshInstances::outline` (and `MeshInstances::outlined`): the grey a group of copies' drawn outline
  (`Mesh3d::outline`) takes, by its tone; `None` is the ink as before. **A struct literal of `MeshInstances` needs
  `outline: None`** (`MeshInstances::one` sets it).

## The accent only under the hand, in charts and examples

- `dot_chart`: the picked range is ink (its dots up to the ink, the line under the axis ink) and the rest of the
  field dim; the accent is only on the dot under the pointer (`DotChart::under`) and on a control point and its
  tick while dragged (`DotChart::dragged`).
- `avatar`'s last argument is a shade, not a hue: 0 ink, 1 dim, 2 text (`widgets::SHADES`), initials in the ground.
- The examples: a main action is `Variant::Primary` (`counter`, `morph`, `mesh`, `chapter`), and nothing is in the
  accent that is not held or dragged or a named specimen (`counter`'s shape, `morph`'s disc, `chapter`'s numbers
  and `annotate`'s panel are ink; `annotate`'s meter and `loading`'s pictures are greys, not hues; the
  `caption` specimen is raised in a hairline frame). `badge`'s usage line shows `Variant::Primary`.

## Menus and sheets are raised

- `widgets::MENU_SURFACE` is the raised grey (it was the ground): menus, selects, context and nested menus, the
  sheet and the bottom sheet are raised in their hairline frame, and their selected and hot rows (`selected_on`)
  are the hairline grey with an ink label. A popover keeps the ground. A nested menu's chevron (a row with a
  submenu) is the text grey: dim reads 2.93:1 on the raised panel in the light scheme.

## Recomposing text keeps the cheap redraw

- A stage that steps (`Ui::on_step`) or a subtree whose messages are its own is still drawn alone, the rest of
  the page copied, while a label or a slot's text recomposes glyph by glyph; before, the whole page was painted
  until the tiles landed. `perf().counters.restages` now counts only the frames that were drawn so.

## Parts of a stage that change alone

- `El::steps()` inside a stage marks a part of it that changes alone (a lens over a page that stays): a step
  (`Ui::on_step`) that names the stage views and draws only those parts, by their keys through `Ui::stage_view`,
  and the rest of the stage is the last frame's. A part `stage_view` does not give, or laid out otherwise, and
  the stage is viewed whole as before. The page naming the step no longer needs to name what in the stage moves.

## Bloom in a region

- `El::bloom()` marks what the look's bloom glows over: once any element is marked, the bloom glows only inside
  the box round what marked elements draw, and only from what is drawn there (`Post::bloom_region`). With none
  marked it covers the whole frame as before.

## The default theme's colours

- The default theme (`DARK`, `LIGHT`) takes the AA.9 table: new grounds, ink and accents (the accent is `#f47a3c`
  dark and `#b23f0d` light; `#ff6b2b` is retired), and four greys of its own (`Theme::greys`: raised, hairline,
  dim, text). `Paint::DIM`, `TEXT` and `FAINT` (now `HAIRLINE`, and `RAISED` new) are drawn as those colours
  exactly, not as the ink mixed toward the ground; a tone between two greys is drawn between their colours. Their
  tones moved to whole 255ths (dim 0.66 → 168/255, text 0.86 → 219/255). Other themes take their greys mixed as
  before (`Greys::mixed`); **a theme built from `DARK` or `LIGHT` with its own ground and ink keeps their greys
  unless it sets `greys: Greys::mixed(ground, ink)`** (or `Greys::mixed_dim(ground, ink, 4.5)`, which takes dim
  toward the ink until it reads 4.5:1 on the ground).
- `Paint::EXTREME`: white on a dark scheme, black on a light one (`Theme::extreme`). It is the ink image's fourth
  hue code (0.45, which the unused glyph-height code held).
- `El::fill_hover` and `El::fill_held`: a fill under the pointer and while held or dragged; `El::border_hover`: a
  border under the pointer or with focus.
- `Variant::Primary`: an ink plate with a ground label, the extreme under the pointer, for the one main action of
  a group. `Variant::Accent` stays for an explicitly coloured action. Badges take `Primary` too.
- The default controls rest in ink: a switch that is on (ink track, ground knob; off a hairline track and a dim
  knob), a slider (ink run and thumb, a hairline rest; the thumb is the accent while dragged), a radio's dot, a
  checkbox's outline and tick, a toggle that is on and a choice's current option (ink plates with ground labels).
  So are a pagination's current page (an ink plate), a current tab's rule, a progress bar's fill and the label of a
  selected list row or menu option; `widgets::SELECTED`, the plate of a selected or keyed row, is the raised grey (it
  was the accent at 0.12), the hairline grey on a raised surface (`widgets::selected_on`). Hot rows of the command,
  context and nested menus are that plate with an ink label, a skip link an ink plate. Outlines of interactive
  controls stay dim at rest and turn ink under the pointer or with focus, and while a select or menu is open or a
  number is typed into (they were the accent then); panel frames (menus, dialogs, sheets, popovers, toasts,
  tooltips) are hairline. A switch's knob is the accent while pressed. The hover and focus marks are ink
  (`Fx::marks_accent` is 0 by default).
- The recipes' and samples' main actions are `Primary`; the components' button page shows it.

## Display type in its own material

- `near` has ink bands (`墨带`, 1 to 3, and `带宽`, px; 0 by default, the bands as before): only that many bands
  outside the ink, in the ink at 0.5, 0.3 and 0.15 (two: 0.5 and 0.25), each the width with as much ground before
  it, square round the ink's pixel corners, for pixel type on whole pixels and on the lattice.
- `carvr` never cuts deeper than its band (`刻带`), the knife's wander included.
- A glyph 32 px or more each way set in `near` or `carvr` takes all the ground its quad covers, padded by the
  spread once more, so what the material draws round the letter is drawn; text at reading sizes keeps the
  override just round its ink.

## Cached subtrees

- A cached subtree (`El::cached`) in which nothing it draws from changed is passed by when the page is painted:
  it is neither drawn nor hashed, and its plate is laid on as it was (`Counters::kept` counts them). It was drawn
  and hashed instance by instance on every paint (about 9 ms for 11,000 glyphs natively).

## Shaders in Chrome

- `fs_ink`'s code test no longer mixes `||` and `&&` without parentheses, which Chrome's WGSL compiler rejects
  (the ink, nearness and clearing passes were invalid there). A test now holds every WGSL source the renderer
  compiles to the expression rules Chrome enforces and naga does not.

## Fast scrolls under dottr and charr

- The frame in which a fast scroll's trace starts or stops holding the rows to the lattice moves the last frame by
  what it shows, instead of shading the whole viewport (the readout's `latch on/off`).
- A lattice material used only outside the scrolled viewport (a bar in dottr over a list in charr) no longer stops
  the last frame from moving (the readout's `latch: mixed lattices`).

## 3D elements that come and go

- A 3D element that comes on the page or goes (scrolled into view, a section opened) draws the frame again only where
  it is or was, not the frame whole (the readout's `meshes`); the 3D elements after it are not drawn again.

## Pictures that arrive

- A picture given after its page is drawn (`Renderer::set_picture`) is drawn on the next frame where it is, with the
  cells that read it, not by a frame drawn whole: a page need not turn `Renderer::partial` off for it.

## Flows

- What a new page lays out only after a flow began (a label shown once what it follows is in view, a drawing's
  new marks) comes in as what was new when the flow began: over a growing picture once it has landed. It was drawn
  at once where it is, outside the part of the picture that shows.
- An element keyed the same on both pages that rode on the growing picture on the old page and is outside every flow
  on the new one (a layer over the page's frame: a pointer's box and its label) is carried by the picture to its new
  place. It was drawn at once there, outside the part of the picture that shows.

## Stage scale

- The stages' scale steps up only when the frames' cost there, by the level up's pixels, would be under 85 % of the
  display frame; a step down within 5 s of a step up holds the lower level, and a route's visit has at most two
  steps up (`perf::STAGE_UPS`, `StageScale::held`), in place of the waits of 5, 10 and 20 s and three returns a
  level.

## Morphs from a stage

- A morph from what a stage draws (`El::stage`; posed glyphs stepped every frame) starts its tiles where the frame
  before it drew them, also when the morph is asked for by a command with a new view that poses them elsewhere or
  hides them; it took them as the new view put them.

## Marks in the ink

- `Fx::marks_accent` (1 by default, the accent as before): how much of the accent the hover and focus marks are
  drawn in; 0 draws them in the ink.

## Frame rate

- Scrolling under the automatic level is not softened on a frame that can move the last one (`dirty::Move`):
  that frame shades only the strip scrolled in, sharp.
- From a renderer's second frame, one a frame, the material passes for the page's materials (hidden elements', one
  waiting for a morph, included: `Scene::mats`) are made ahead and kept by material; a morph's frames count the
  materials of its tiles. With one built-in global material and no override, the material pass (and its upsample) is a
  pipeline of its own for that material (the WGSL override constant `MAT`): `Renderer::specialise`,
  `Renderer::specialised`, `render::SPECIALISE_AFTER`.
- Marks changed in place (drifting motes, a turning body's glyphs) are each their own box, not one box round them all;
  many boxes are gathered by where they are, and the final frame is shaded in up to 32 boxes grown from what changed
  (`dirty::MAX_SHADE_BOXES`; the G-buffer keeps 8).
- A fast scroll with its trace showing moves the last frame too, and shades again only the pixels the trace can reach
  (`fs_trace`, `Cost::trace`); under dottr and charr the frame moves by the lattice's steps (`dirty::Move::frame`,
  `trace`, `way`, `latch`). Not under glitchr or a material from source.
- `El::local_messages`: the messages a subtree's handlers send change only it and the `El::steps` elements, so it
  is restaged after them (`Ui::stage_view`) instead of viewing and laying out the page; for a stage dragged by hand.
- Inside `El::cached`, a pixel face asked for under its native size is set and drawn at the size asked, by its
  outline's coverage (`text::FINE`, `Text::fine`).
- A renderer made after the page was painted (`Offscreen::new` in a render tool) draws it: it sends the whole glyph
  atlas on its first frame, and a cached subtree its plates do not hold is drawn again (`Ui::plates_lost`). A
  scene drawn twice with no paint between inks its plates once (`Scene::paint`).
- An element moved by its offsets in a restage stays where it was moved through a view of the page that lays
  nothing out; it had gone back to where the last layout put it.
- `Perf::path` (`perf::FramePath`): how the last frame was drawn, for a readout: whether it moved the last frame and
  the first reason it did not or was drawn whole, whether the material pass was specialised and why not, the share
  shaded at full resolution, and the soft, stage and reduced scales; `FramePath::line`, shown by the full overlay.
  `dirty::parts_why`, `Parts::refused`.
- A scroll with its trace showing over stages shaded at their own scale moves the last frame (it was drawn whole
  every frame); the stages in the viewport are shaded again where they are, slow or fast. `dirty::meet`, `Move::bx`.
- `Renderer::spec_force`: force the material pass made for the global material on (from the next frame, even over
  overrides, for measuring) or off. Off, a pass already made is not used either. The trace pass is made for the material too (`fs_trace` with `MAT`).
- A page of several built-in materials (up to `render::GATED_MAX`) is drawn by a material pass for each, each drawing
  only its own pixels (the WGSL override `ONLY`), once the same materials have held for `specialise` frames; an
  override in the global material no longer keeps the page from the pass made for it. `FramePath::gated`.
- A step of the stages' scale shades only the stages again, not the whole frame (and not the frame after either).
  `Ui::stage_governor`, `StageScale::set_level`, for checks.

## For Kern

- `pair::Faces` resolves a role as the page's root does (its `El::fonts`), and the faces kept follow it; a Lab page
  whose theme sets display to mono measured Noto Sans. `Faces::face_of`.
- `Experiment::faces`: a cell made alone (`experiment::ui`, the contact sheet) is handed the faces.
- `GlyphAt` tells a pose each glyph's pen as laid out (`pen`, `base`, `advance`), so a pose can put a kerned glyph's
  pen exactly; `GlyphAt` is `Default`.

## Drawn once

- `El::cached()`: a subtree drawn once onto a plate and laid on from it while it draws the same, so a large page
  of small text costs no instances; drawn again when it changes and after a new window size.
- `el::cached_view(over, from, ink)`: part of a cached subtree, scaled into an element, read as coverage
  nearest (a magnifier's pixels).
- `El::clip_round(radius)`: a clip with round corners, or a circle.
- A stage's absolutely placed element that moves by its offsets alone is restaged with no layout, so a chip
  dragged about a stage costs no page view or layout.
- A pointer move asks for a frame only when something shows where the pointer is, the hover or the cursor
  changes, or a drag, touch or selection is under way; hovering a still page draws nothing.
- An experiment's formula and label are selectable.

## Fixes from the website: controls at rest

- A front pushes text to read and controls about the centre of the control they are in, and a surface that holds
  them is not that control: an element that follows the pointer (`on_point`, `on_move`) or one larger than a quarter
  of the window. A full-window stage with `on_point` or `on_key` made its whole page one piece, so every commit's
  front shoved everything on it sideways at once, when the front crossed the window's centre.
- A commit inside `El::commit_scope` is over once its front has reached the scope's farthest corner, not the window's,
  and its scope takes in what the material draws round the box (the nearness spread and a front's push). A card that
  switched to `near` kept the old outline round it for two seconds and then swapped it in one frame, and every frame
  until then was drawn whole. The commit's own front stays in the scope too, as the fronts of other presses there did.
- `near` draws no bands round text with strokes under 3 px. Only the shapes' nearness takes a label's surroundings
  into an element's own material, so round a label in a control its bands showed as stray dashes over the ascenders
  and a hook under a descender. Titles keep their bands.
- The hover and focus marks are drawn over the material, exact, in the accent, and on a segment or a tab they stay
  inside the control's border. Drawn by the material, `near` outlined each bar twice, and on a segment the bars sat
  on the control's own border, which looked broken.
- The performance readout keeps its width: every field keeps its width whatever its number (`idle` is as wide as the
  rates), and its plate does not narrow while it stays open in a window of the same size. Its corner's inset no longer
  counts on both sides, so a wide inset (a stage's edge) leaves it the room beside it. Its edge used to jump about
  70 px each time the page went idle and woke.

## A stage at its own scale

- When frames are missed, the stages' device scale steps down (2 → 1.5 → 1 → 0.75) and back up after a wait that
  doubles each time; text, panels, the chrome and whatever is drawn over a stage stay at the device's scale
  (`perf::StageScale`, `Ui::reset_stage_scale`, `Perf::stage_label`). `Look::quality` below 1 pins it. A slow
  stage is held under input only once its scale is as low as it goes.
- Automatic quality goes by the intervals between frames drawn while something moves, not their CPU time, so a
  machine slow on the GPU steps down too (`perf::AutoQuality::feed(t, cost, moving)`).
- Under the automatic level, scroll areas are shaded at it while they scroll (no lower than the material's
  floor, `render::soft_floor`); the ink's edges, overlays and the rest of the page stay sharp, and 100 ms after
  the last delta one frame is drawn sharp (`Ui::scroll_soft`, `render::SOFT_SCROLL`).

## Between letters

- `pair`: the distance between two glyph outlines and the white between them, and where to set a pair for a gap
  or an area, from the page's faces handed to the state (`Ui::faces`, `Ui::on_faces`), not the text system.

## Stages, from the site

- A slow stage holds with input coalesced (`Ui::coalesce`, as the web and desktop hosts run): the input of a frame
  waited for the frame's layout, and the tick had spent it before, so the stage drew every frame of a drag.
- `El::steps` marks an element beside a stage that changes with it (a formula, a label): `Ui::on_step` returns its key
  with the stage's and `Ui::stage_view` gives it, so an experiment's own page no longer lays out every frame. Text
  whose words changed but measures as before counts as laid out the same.

## Flexbox only

- taffy is built with flexbox alone (no grid, block layout or calc), about 170 KB less in a web module. A
  `Style` with `Display::Grid` or `Display::Block` now lays out as flexbox; amenui itself used grid only for
  `stack` and `layers`.
- `stack` keeps its meaning: as wide as its widest child, as tall as its tallest, each child centred. It is a row
  whose children lie over one another by their margins (set from the widths they took), so a margin you give a
  child of a stack still counts, and a percentage margin there becomes a length.
- `layers` places each child absolutely at the top left of its box unless the child is placed already; the box
  does not grow with its children, as documented (a box with no height of its own grew with the grid before; give
  it `fill_height` or a height).

## Fixes from the website, the way back

- A ramp stroke (`Drawing::ramp`) fades as a whole: its far and middle stops follow the element's opacity as its start
  does. Coming back to an index, a window waiting for the picture to land no longer drew its rays at full size.
- A picture in a layer (after something that stirs, or a modal) is a picture for what is drawn over it, so a drawing's
  own colours (`Drawing::palette`) stay its own while its picture fades in, not the accent; a picture under 0.01 of
  its opacity no longer marks its area as a picture.

## A stage that steps alone is drawn alone

- `Ui::on_step(f)` and `Ui::stage_view(f)`: a stage (`El::stage`) whose piece changes by itself is viewed and drawn
  again alone, in the boxes the last layout gave it, while the rest of the page keeps its last frame; the frame is
  the one a whole view of the page gives. A step that lays the stage out otherwise views the whole page
  (DESIGN.md §1.35). The website's Lab index with one cell running takes 2.9 ms a frame on the CPU, from 6.9 ms.
- `perf().counters.restages` counts those frames.
- The experiment cell's stage (`experiment::frame`) is an `El::stage`.
- Text to read over other ink finds that ink by a grid over the window, faster.

## The state hears where the focus is

- `Ui::on_focus(f)`: the state is told which element has the keyboard focus whenever it moves (Tab, a press,
  `focus_key`, `clear_focus`, a modal layer opening or closing), as `on_hover` tells it where the pointer is; `f`
  returns whether the view changed. A Lab cell can run while one of its controls has the focus.

## A stage that holds while the hand works

- `El::stage()`: the live piece of a page. When its draws take more than 0.6 of a frame it holds its last image on
  frames that answer input, so the controls beside it answer in that frame, and draws again on the first frame
  without input, after a release, and during a long drag every `max(100 ms, 6 draws)` up to 500 ms (DESIGN.md
  §1.35). A stage that draws in time is drawn every frame as before.
- `Ui::slow_stage(ms)` and `Ui::stage_state(key)`, for checks; `stage` holds the numbers.
- `Props` has a new public field, `stage`.

## Fixes from the website

- Pixel type set at its face's native size because a fitted box zoomed it under that size (the Lab's formula on a
  docs stage) has its lines as much further apart: the lines no longer overlap and the text's box holds them.
- `ViewProblem::OwnColourOffPicture` and `ViewProblem::RampHue` look at what is drawn of a mark, cut by its clip: a
  ray that runs out of a band holding its picture is no longer reported.
- Posed text (`El::pose`) whose box is outside its clip draws the glyphs its pose puts inside the clip.

## One GPU for every off-screen renderer

- Every `Offscreen` in a process shares one wgpu instance, adapter, device and queue, made on first use; each keeps its
  own renderer, target and read-back buffer. Tests that ran in parallel made and dropped Vulkan instances on several
  threads at once, which crashed lavapipe (a segmentation fault in a different GPU test each run on CI). A failure to
  start is kept and returned again to every later `Offscreen`.

## Paint that does not grow with the square of the page

- Drawing only what changed matches instances in order rather than by place, so a drawing whose strokes change in
  number redraws its own box and not everything after it, and redraws up to 8 boxes a stage instead of their
  union (`dirty::parts`, `dirty::MAX_BOXES`, `Scene::cuts`; `dirty::Inputs` has a new public field, `cuts`).
- A fitted box (`El::fit`) whose subtree changes only what it draws is one view and no layout a frame (it was three
  views and two layouts): the scales found last time stand until something layout reads changes.
- What text to read holds (`holds`) finds the ink drawn before it through a 64 px grid instead of testing every
  earlier instance, and the islands of loosened controls (`islands`) walk up each node's parents once instead of
  searching every earlier node. The scene is the same; on the website's Lab index (about 7,000 instances and 420
  nodes, 1440 × 900 at 2×) a frame's paint goes from about 16 ms to about 4 ms natively.

## Widgets built once per message type

- `widgets::number`, `caption` and `experiment::frame` build their controls once, with their own messages, and map
  them to the application's; `segmented`, `choice`, `slider` and `input` build their body once per message type
  rather than once for every closure passed to them. Nothing changes in what they show or send. The website's full
  module is 11,819 bytes smaller gzipped (2,832,825 to 2,821,006).

## How selected text looks

- `Look::select` (`style::Select { halo, alpha, ink }`, 3 px, 0.34, on by default): the selection halo is opaque, the
  accent mixed into the ground as one colour (#5f2d18 on the dark theme), so halos that meet across glyphs and lines
  show no box and no darker band; selected glyphs are drawn in the ink. The halo comes and goes in one frame.
- `Look` has a new public field, `select`: code that builds one with every field named needs it.

## Number controls that keep their steps inside

- `widgets::number`: never narrower than its steps and field; a width that leaves the label no room beside them puts
  the steps on a line under it, and a label wider than the control ends in an ellipsis, read whole. A caption's and
  the Lab's numbers no longer run into each other.
- `experiment::frame`: the formula wraps to the cell instead of being cut.
- `El::clamp`: a word wider than the line is cut too.

## A drawing's own colours

- `Drawing::palette([...])`: a drawing's own colours, which its marks name as `Paint::hue(3 + k)`, so it looks the same
  under any theme; exact over a picture (a ramp between two in OKLab), hue 0 and reported off one
  (`ViewProblem::OwnColourOffPicture`, DESIGN.md §1.26). A frame's drawings have at most 32 between them.
- `Drawing` has a new public field, `palette`: code that builds one with every field named needs it. Outside a drawing,
  a `Paint` hue of 3 or more is hue 0.
- `El::map` maps an element's children in a plain loop rather than collecting them in place, which is built once for
  every pair of message types: with grain in a region, labels shown while clear, keyed elements, drawings that ride
  and their own colours, the website's full module is 4,605 bytes larger gzipped than before them (2,826,385 to
  2,830,990).

## Labels shown while clear and elements keyed across a growing picture

- `El::show_clear(inset, rank)`: a label shown only while it is clear, checked every frame against its view
  (`El::view_scale`, or the window) and the labels of lower rank, coming back with 4 px more to spare and fading
  linearly over 150 ms; hidden, it takes no pointer and is still read (DESIGN.md §1.2). A view of more than 64 is
  reported (`ViewProblem::ShowClearMany`).
- `Cmd::Flow`: an element keyed the same on both pages of a picture that grows goes from its old place to its new one
  with the picture instead of fading out and in, its size on the picture's cubic (DESIGN.md §1.10). Own-size type at
  another size on the new page is reported (`ViewProblem::FlowOwnSize`) and cross-fades once the picture lands. A
  picture with the same flow id whose box changed now grows.
- `Cmd::Flow`: a drawing's marks that ride on a growing or shrinking picture go with the picture's transform, their pen
  kept, instead of at their own size by parts, so a drawing shrinks into the window with its picture; under a view
  scale everything that rides does, own-size labels apart.
- `ViewProblem` has two new variants, and `Props` a new field, `show_clear`.

## Grain in a region

- `El::grain()`: once an element is marked, the look's grain is drawn only over what marked elements draw (their
  box, following a flowing picture), the same grain as over the whole frame; with none marked it covers the frame as
  before (DESIGN.md §1.21). `Post` has a new field, `region`, which the runtime sets.

## Smaller web module

- `El`'s fields that do not depend on its message type are in `El::props` (`Props`), which an `El` reads and writes
  as its own fields (`el.style`, `el.key`, `e.children` are as before). `El::map`, `El::zoom` and `El::new` move them
  as one, and each leaf is measured outside the generic code, so an application that embeds others' views builds less
  per message type: the website's full module is 13,937 bytes smaller gzipped (2,840,450 to 2,826,513). Code that
  builds or takes apart an `El` by naming all its fields names `props` instead.
- Web: a canvas resized by its stage is drawn again at its new size in the same refresh, so a route that changes the
  stage's size shows no empty refresh (`web/amenui.js`).

## A view scale

- `El::view_scale(s)`: what a container holds is scaled by `s` except the pen: line, border and trace widths and a
  trace's dashes stay as at 1, own-size labels keep their size, and a dot is never less than one whole device pixel
  (DESIGN.md §1.2). `El::zoom` is unchanged.
- `El` has two new public fields, `pen` and `view_scale`: code that builds one with every field named needs them.

## Labels at their own size

- `El::own_size(ax, ay)`: an element that keeps its own size when it is zoomed (`El::zoom`, `El::fit`) or rides a
  growing picture (`Cmd::Flow`): its anchor point goes where the zoomed box would put it, on whole device pixels;
  in a row or column it takes its own room, and one that then overflows its parent is reported
  (`ViewProblem::OwnSizeOver`, P1 in the audit). `El::fit` measures subtrees with one in two passes (DESIGN.md §1.2).
- Pixel faces are never drawn smaller at a higher device pixel ratio than at 1: at 2, 7 px of a 12 px face is two
  device px a face pixel, as at 1, not one (DESIGN.md §1.6.3).
- `El` has a new public field, `own_size`: code that builds one with every field named needs it.

## Strokes with a ramp

- `Drawing::ramp(a, b, w, paint, mid, far)`: a stroke whose colour and opacity go from `paint` at `a` to `far` at `b`,
  through an optional middle stop, still one instance. Over a picture the colours mix in OKLab; off one it keeps its
  first hue, and a ramp across hues there is reported (`ViewProblem::RampHue`, P1 in the audit) (DESIGN.md §1.26).
  `Mark::Line` has two new fields, `far` and `mid` (`None` for a plain line): code that builds one with every field
  named needs them.

## Islands under labels

- Flow: what rides on a growing picture moves by whole parts (a word; strokes that touch, as a box of lines),
  so a box drawn as four lines stays one box at one size while the picture grows or shrinks (DESIGN.md §1.10).
- `Ui::ambient_fps(30)` on a 60 Hz display draws every second refresh: a paced frame is due within half a
  refresh of its time, and `web/amenui.js` wakes a refresh early for it (in Chrome it was 20 a second).
- Web: when the WebGPU device fails, the full build starts on WebGL2 before it reports `device`; the canvas is
  given a WebGPU context only once a WebGPU device is there.
- Flow: cells that read a picture (`el::cells`) are not taken for the subtree's picture, so they are not drawn a
  second time as one that grows. What rides on a growing picture stays whole until it lands, and the new
  subtree's notes are there at once then: each note is drawn once every frame (DESIGN.md §1.10).
- `Blade::lean_from` and `Blade::lean_full`: the blade's colour (and its head's) goes to `lean_paint` from the one
  lean to the other on a smoothstep; both 0 (the default) keep the mix linear over `max_lean`, byte for byte
  (`Blade::mix`, DESIGN.md §1.26). Code that builds a `Blade` with every field named needs the two new ones.

- The ink under text to read is held at looseness 0, not at `legible_cap`: a label reads on its ground's colour at
  rest (DESIGN.md §1.15).
- A filled control whose own fill is loosened keeps one island: its content's ink box grown by `max(5, 0.36 × size)`,
  rounded, within its outline at rest, with a hard edge. Text on others' ink keeps its glyph island, with the gaps
  inside a word held too. `Scene` has `islands`. Under scribblr such a control's fill becomes a thread round its
  outline, with the island whole at rest fill.

## Blades, gusts and pacing

- The `webgl` feature (on by default) carries the WebGL2 backend on the web. Without it a web build has
  only WebGPU, and a browser without WebGPU fails to start: `Host::start` returns `app::web::StartError`
  (`Adapter` or `Device`; it was a `String`), the module's `start_failure()` export says which, and
  `web/amenui.js` calls `opts.onFail(kind, message)`.

- `Drawing::blade(Blade)` (`Mark::Blade`): a rooted stroke the field map bends from the flow at its root, worked
  out on the GPU every frame; its lean is its tip's tangent, its root leans 0.35 of that and it bends above its
  stiff part (`Blade::tangent`, `Blade::shape`), drawn in 10, 6 or 2 segments by its length; it keeps its length,
  tapers to half its width and can carry a head glyph on its tip, turned with it and going to `lean_paint` with
  the stroke; its own lean and the map's add, within ±`max_lean` (DESIGN.md §1.26). `Mark` has `Blade`, so exhaustive matches
  need the new arm.
- `Cmd::Gust { gust, scope }` (`fieldmap::Gust`): a band that crosses a scope with a wake behind it, worked out from
  the clock and laid over the map, never put into it: once it has passed, the map is as it was and nothing is
  asked for (DESIGN.md §1.29). `Cmd` has `Gust`.
- `El::map_life(energy_life, flow_life)`: what is put in the map within an element's box is back at rest after
  these lives, however much and however wide: nothing spreads there, and each node goes down the look's curve
  slowed to end at the life. A stir narrower than a cell is put in a cell wide.
- The warm-up goes on in idle time (`Renderer::warm_idle`; the web module's `warm_idle`, which `web/amenui.js`
  calls from `requestIdleCallback`): a page that draws nothing after its first frame is still warm soon, and its
  pieces no longer land in the frames of the next input. A warm-up step no longer asks for a frame of its own.
  A page shell of its own calls `warm_idle` the same way, asking for the next idle callback whatever a call did
  (a pipeline compile that blocked, a call that threw or found the module busy) until `warm()` is empty.
- The field map's ink for `creep` is read again only for instances that changed since the last scene, and not for
  blades or what lies inside a box with lives of its own, where nothing creeps.
- `Cmd::Flow` grows a subtree with a picture on both pages: the picture is one, in a box and a visible part
  that go from the old to the new on whole pixels; the rest of the old subtree rides with it at its own size
  and fades, the rest of the new one and what is new outside the flows comes in once it has landed, and the
  old page's ground stays around it until then (DESIGN.md §1.10). The growing picture and its riders are drawn
  in the page pass over what dissolves, so the notes keep their colour and size and no other window is drawn
  over it; a frame along the window's edge fades under it; a picture that shrinks back takes the old page with
  it, the new page's ground around it from the first frame. A flier's clip goes from the old to the new one
  instead of covering both.
- `Ui::ambient_fps(n)`: frames for what moves by itself at most `n` a second; input and what it starts are not
  paced until a second after the last input.

## Ink over pictures and character marks

- Text and shapes drawn over a picture are their own colour there, blended with the picture by their coverage and
  opacity, in the page and in a layer (DESIGN.md §1.6.1). Before, their edges (or the whole text, in the page's
  own layer) were read as the picture's colours and showed red; a site no longer needs a layer of its own for ink
  over a picture. Text to read over a picture stays as it is while the picture loosens.
- `el::cells(Cells, cols, rows)` draws the picture under it as character marks at a pitch of its own (7 × 14 px by
  default) from a ramp the caller gives, each cell in its own colour mixed toward an ink: one instance and one draw,
  nothing laid out when it moves, nothing drawn while it is still (DESIGN.md §1.6.1). `Content` has `Cells`, so
  exhaustive matches need the new arm.

## Fitting, clamping, faces that come later and plates

- Breaking: `FlowSpec` has `fade_ms` and `Look` has `overlay_keep` and `off`, so struct literals of them need the new field or
  `..Default::default()`; `Cmd` has `ClearFocus` and `ClearPlate`, `ViewProblem` has `FitShort`, `ClampAround`,
  `PlateOver` and `OnPlate`, `Off` has `ground`, and `FontError` has `Bad`, so exhaustive matches need the new arms. Every element is now the box its
  absolutely placed children are pinned in (its padding box), not the window: a page that placed an `absolute` child
  against the window from inside a padded parent moves it.
- Lengths are laid out on a 1/64 px step, so a wrapping row in a box as wide as its content stays on one line.
- Text to read and controls hold the looseness under and around them to the legibility cap (DESIGN.md §1.15): a label
  on a loose shape reads as at rest. Sites no longer need a ground-coloured patch or band under labels. Each material
  meets the island in its own way: whole cells and an island that follows the glyphs in charr, dots that join it in
  dottr, separations that close up by whole dots in halftonr, no copies of the label in glitchr, a crisp fill in scribblr.
- An element that clips, clips inside its border.
- `El::fit(Fit)` zooms a subtree to fit its box in the same layout (pixel type is set smaller, the scale is kept; at
  `min` what overflows starts at the box's start, and only what is cut is reported); `FitMode::Reflow` lays it out
  at the box's width first, with its type at its own size, and zooms it down only if it still does not fit; `El::clamp(lines)` ends text in
  the face's own ellipsis (three full stops where the face's floats off the baseline), copies what it shows without
  it, and brings the rest in from the next frame when the clamp is taken away, keeping the ellipsis until the text after it has come, with `Ui::clamped`; `El::preview(true)` draws sheets, menus and dialogs where they are written, without a
  layer (a skip link over what is beneath it and still where it is written in Tab order, a docked sheet inside the border); `Ui::line_inks` reads each line's box and ink; `Ui::clear_focus` and `Cmd::ClearFocus` take the focus off.
- Faces that come later (§1.6.5): `Ui::declare_face`, `text::with_declared`, `Ui::add_face`, `Ui::faces_wanted`;
  resources by name with `Ui::add_asset`, `Ui::asset`, `Ui::assets_wanted`. Text stands in until the face comes, then
  each line sweeps over in 240 ms; with a material each glyph comes apart a little as it changes (a heading up to 0.6). The web module carries only the three main faces; `web/amenui.js` takes
  `opts.faces`, `opts.assets` and `opts.preload` and fetches what is wanted.
- Plates (§1.33): `El::plate(Plate { fade_s })` keeps the ink a subtree draws, ageing it out over `fade_s`;
  `Cmd::ClearPlate { key }` empties it. Ageing ink fades as its coverage, with no pattern; a new stroke makes new
  only the old ink under its own outline; an empty plate is not laid on; text to read inside a plate is reported.
- A decode held at a step draws once and then the page rests (it asked for frames for good before).
- `Look::overlay_keep` keeps the readout in its corner; the readout wraps to fit a window 390 px wide.
- With less motion a flow fades in place over `FlowSpec::fade_ms` (150 ms by default). With or without it, what is
  new on the page fades in over `fade_ms`, so a route change is never the new page drawn whole over the old one.
- `Look::off` turns each effect off (close, fronts and commits, flows, morphs, ripples, loosening, recompose,
  decode, material motion, bloom, the field's dots), each going straight to its end state and asking for no frames (§1.34).
- A box whose last child was absolute no longer keeps the child's layout after it goes (it could panic in taffy).

## Coalesced input, cached word pieces, and a web page that survives a panic

- Breaking: `el::Point` and `el::Drag` have `t`, when the event happened on the runtime's clock, so struct
  literals of them need it. `experiment::Pointer::hear` measures speed between the events' own times.
- `Ui::coalesce` (on in both hosts): pointer moves, wheels and ticks lay out and paint once a frame, before it
  is drawn (`Ui::settle`, called by `Renderer::draw`); every other input lays out what waits first. Eight
  moves in a frame cost one layout instead of eight.
- The words and lines of a text drawn by `El::reveal_by` are found once per text and width, not on every
  paint (`Counters::reveals`); each glyph's word is looked up in its own line.
- The renderer reuses the lists it compares frames with instead of copying the scene into new ones.
- Web: a panic in a call no longer leaves every later call failing with "already borrowed": the application
  goes on as the panic left it (`app::slot`). GPU errors are logged, not panics, and mesh vertices go in
  buffers of at most 256 KB (`render::MESH_CHUNK`).

## Controls that hold under effects, the warm-up in the hosts, opacity and scoped commits

- Breaking: `Cmd::Commit` has a `scope: Option<String>` (`None` is the old behaviour); `MorphSpec` is
  `Clone`, not `Copy`, and has `from_last`; `Look` has `overlay_at` and `overlay_inset`, and `Gaze` has
  `echo` and `echo_ms`, so struct literals of them need the new fields or `..Default::default()`.
  A drive file's `commit` without `scope` reads as before.
- Both hosts warm the renderer up a piece a frame within 6 ms (`Renderer::warm_frame`, `WARM_MS`):
  the first frame is the plain page. `Ui::warm` and `Ui::on_warm` say how far it is; the web export
  `warm()` gives `[done, total]`, and `web/amenui.js` sets `aria-busy` and calls `opts.onWarm`.
- A frame drawn while something moves owes one more, so a motion that ends between frames ends on the
  settled page; a host no longer needs to redraw by hand after one.
- An element that leaves with its page takes its field map and decode with it; one taken away alone
  decodes out with its own spec inside its clip. `audit::off_palette` counts colours no theme makes.
- Controls inside a subtree marked `legible(false)` keep their fill and label as at rest; a tab list's
  heads and rule are the control.
- `El::pass` lets the pointer through to what is under it; `El::opacity` fades a subtree as one (at
  0 it takes no pointer); `El::commit_scope` keeps a commit inside one element's box.
- `Ui::morph_from_last` and `MorphSpec::from_last` fly an element's glyphs, each whole, from where they
  were drawn last frame to the new element's, across a route change, in every material.
- The readout goes in `Look::overlay_at`'s corner and moves on clockwise rather than cover text to read.
- The INK leader is dashed and routed round text to read; `Gaze::echo` keeps fading earlier INK boxes.
- With reduced motion, what comes and goes fades over 150 ms.
- Looks: the thread's clearances move its control points, so the curve stays smooth; a thin accent rule
  under `charr` is one unbroken row; the Crack's pieces start to leave at 0.28; a press's flash in
  `plain` keeps a filled control's label at 4.5:1.

## Materials named for what they do, and as source

- Breaking: the six printing materials are named for what they do: `dot` → `dottr`, `halftone` →
  `halftonr`, `ascii` → `charr`, `print` → `smudgr`, `lino` → `carvr`, `glitch` → `glitchr` (the
  `Material` variants and their strings); `plain`, `near` and `scribblr` keep theirs. `PicFx` is
  `Markr` (`El::markr`), the finishing pass's `Dither` is `Dithr` (`post.dithr`, `DithrKind`),
  `Cmd::Front` / `Cmd::FrontAt` are `Cmd::Ripplr` / `Cmd::RipplrAt`, and `El::ripple`,
  `El::ripple_scope` and `El::wipe` are `El::ripplr`, `El::ripplr_scope` and `El::wipr`. No aliases: an
  old material name is refused and the error names the new one, and `LookError` gives the closest name
  for any other. Look recipes start with `look:v2:`; a `look:v1:` recipe is refused with the renames
  listed. `Material` has a variant for added materials, so cast with `Material::id()`, not `as`.
- Materials as source (§1.6.4): each material is `src/materials/<name>.wgsl` with its name and numbers
  in a header; `amenui add material <name>` copies one into a project, and
  `Look::material_source(name, wgsl, params)` adds one back under a name of its own (ids 9–15,
  `Material::Custom`). `Look::numbers(m)` reads the numbers a material is drawn with.
- `El::effect(Effect { material, response, scope })` puts a material, a response and a scope on a
  subtree in one line, and `el::wrap_text` turns plain text or Markdown into a page to read.
- The thread (§1.30): one large lopsided knot and smaller ones (`knot_fall`), a few chords escaping
  (`escape`), long flat tails with a few gentle bends (`tail_turns`) at half the line's tone
  (`tail_tone`), all in `strand::ThreadShape`; it no longer lingers between a half-drawn title and a
  thread when it draws back (`strand::shown`), and it passes behind pictures without changing colour.
- Sample sheets that check themselves (`amenui::sample`, `sheet --material <name> --loose … --arrival`).
- The audit finds text a material wears away at rest (`Eroded`, P1 under 85 % of plain's ink); a
  clipped finding names the side, the distance and what cuts it. Annotation tags and `INK` readouts
  keep off text to read and off each other.
- `near` keeps a label in the ground's colour on a solid button whole. Text on a form fades its rim
  from `edge` 0.6 (range 0–0.8), and in `form3d` the pointer's light falls off smoothly.
- The warm-up (§1.23): the material arrives on the page at rest, then over 120 ms (`ui::RAMP`) the page
  loosens to its caps and condenses, so the arrival no longer jumps.
- Every public item of `strand`, `fieldmap`, `field`, `hit`, `audit`, `form` and `materials` has a doc
  comment.

## Text on a form, the warm-up and the audit

- Glyph poses have a depth and a facing (`Pose::z`, `Pose::facing`; the defaults draw as before), and a
  text element's glyphs are drawn far to near. `El::form(FormFx { back, edge, overlap, fog })` keeps text
  on a form legible: what faces away is left out, what is edge on fades, a glyph mostly covered by nearer
  ones is not drawn (measured on the true ink), the far side is dimmer (§1.26).
- `amenui::form`: `Form` wraps a block of text onto a sphere, a surface function or a mesh seen by a
  `mesh::Camera` (`Form::pose_of`), and `Form::hit` finds the pointer on the surface. The recipe `form3d`.
- `El::reveal_by(Reveal::Word | Line)`: clips, poses and the window show words and lines whole.
- Hits follow what is drawn: `Ui::glyph_at`, `Point::glyph` and `Point::z`; `El::turn` turns an element
  and its subtree, and it is pressed where it is drawn; `MeshHit` has `point`, `normal`, `t` and `uv`, and
  `Mesh::ray` casts a ray at a mesh.
- A button or link inside an element that drags (a scene with `on_point`) takes its own press.
- The warm-up (§1.23): `Renderer::begin` draws plain at once, `Warmup::step` / `Renderer::warm_step` make
  the rest a piece at a time, and the page condenses into its look when it is ready;
  `Offscreen::try_warming`. `Ui::prepare`, `Ui::ready` and `Ui::readiness` run an application's own work
  in the frames' spare time; the annotation layer reads `READY`.
- The audit (§1.31): `amenui::audit`, `drive::audit`, `render --audit` (with `--strict`, a P0 or P1 fails):
  text cut off, contrast, glyphs over each other, text too small, a page that never settles. The
  annotation layer's `INK` kind draws the same measured ink box.
- What the audit found: `Paint::DIM` is 0.66 (it was 0.5, 3.2:1 on light); the light theme's accent is
  darker (5.6:1) and the selection tint 12 %; `near` keeps text's ink whole; the loading recipe no longer
  asks for a frame every tick.

## Running applications

- `amenui::app`: the framework runs applications itself. `app::native::run` opens a winit window
  (feature `window`, on by default) with the system clipboard, the input method, touch, frames on
  demand, and `--shot` / `--exit-after` for checks under Xvfb. `app::web::Host` and the `web_app!`
  macro generate a wasm module's exports from an application's state and message types. The `App`
  trait is how a shell routes, reads the route, takes clipboard requests, and sets the look, the
  language and reduced motion; every method has a default.
- Native screen readers: `app::access` builds an AccessKit tree from the accessibility mirror for the
  window (UI Automation, NSAccessibility, AT-SPI) and turns Focus, Click, Increment and Decrement into the
  runtime's focus, activation and arrow keys.
- The mirror says more: `El::expanded`, `El::description`, `El::live` (`Role::Status` is polite by
  default), `El::invalid`, `El::required`. The accordion, select and popover report `expanded` instead of
  `checked`.
- Fonts beyond the bundled ones: `Face::fallback` for faces tried after the bundled ones, and
  `Face::system()` for the system's installed fonts (feature `system-fonts`, on by default; the native
  window adds it). `Text::missing_glyphs`. The `complex-scripts` feature for Thai, Lao, Khmer and Burmese
  line breaking.
- Any language: `Locale::Other(Tag)` for any BCP 47 tag, falling back to English, with CLDR plural rules,
  right-to-left direction by language or script, and its own `locale.name` and `locale.date`. Arabic ships
  (`Locale::AR`, `assets/i18n/ar.txt`). `Locale::ALL` lists the three shipped locales;
  `i18n::installed()` lists the application's. `Locale::name` returns a `String`.
- Focus: `El::tab_order` (positive first, 0 in document order, negative out of Tab), `El::skip_to`
  and `El::focus_shown`, and the `skip_link` component.
- `App::set_appearance` (`Appearance { dark, high_contrast }`), `App::files_hovering` and
  `App::files_dropped` (`Dropped` with a path natively, name and bytes on the web), from both shells.
- The native window draws with the bundled faces compiled in (`text::embedded`), so a shipped binary
  finds its fonts anywhere. `WindowError::Font` is gone.
- `amenui add` writes from the sources compiled into it (`sources::file`, `add::write` replaces
  `add::copy`), so an installed command works without the repository.
- Crate metadata for crates.io (description, keywords, categories, `rust-version`); the package
  leaves out CI (8.7 MiB compressed, under the 10 MiB limit).
- The native window recovers from a lost GPU device: new device and renderer, `setup` again, the `Ui`
  untouched. `native::run`'s `setup` is now `Fn`.
- `web/amenui.js`, the `amenui` npm package: the page side of the web entry (`mount`, `stage`, `css`).
- `offscreen::read_back`: one frame drawn into a texture of a given format and read back.
- `error::WindowError`.
- CI: clippy for native and web, rustdoc, the tests on lavapipe, and the website's tests against each
  framework commit.

## The framework and the website in two repositories

- The site (the gallery shell and its pages, the skeleton demo, Body and the Lab, the site documentation,
  the web shell, the native window, and the sample, acceptance and timing tools) moved to
  [amenui-dev/website](https://github.com/amenui-dev/website), with its history.
- `i18n::install`: an application installs its own message catalogs, looked up before the framework's.
  The framework's catalogs keep only the components' own strings (`widget.*`).
- `amenui::pictures`: sample pictures computed in code (formerly in `demo.rs`). `amenui::sources`:
  `AGENTS.md`, `llms.txt`, `registry.json`, `CHANGELOG.md` and every component's source, compiled into
  the library. `text::ASSETS`: where the fonts are on disk.
- `drive::render_with` takes an application's own pages; `drive::open` opens recipes only, and so does
  the `render` tool.
- The library is no longer a `cdylib`. The web entry point (`wasm-bindgen`), `winit` and `arboard` moved
  to the website repository with the native window.

## The field map and trails, the thread material, small type, Loose Thread

- Body is English only, whatever the interface language. Its language marking reads
  `LANGUAGE  EN / ZH`, and pressing it changes the language of the rest of the gallery.
- The field map (`src/fieldmap.rs`): a coarse grid over the window whose nodes hold energy and a flow
  direction. It steps at a fixed 1/60 s, spreads, decays by lifetime, meanders (`meander`) and creeps
  along ink (`creep`), returns to exactly zero within a bounded time, and needs no frames at rest.
- Trails: an element with `El::stirr` stirs the map along the path it moved. `Cmd::Stir` stirs a
  point, an element or a stretch of a path. The floating box, the bottom sheet and the card stack trail
  by default; the preset sets how strongly (Normcore has none).
- Reading the map: in the G-buffer stage it is one more source of looseness (still bounded by the
  legibility cap and each element's response), and instances that can be pushed drift along its flow
  (pictures and 3D elements loosen in place). The `dot` material's ground dots and `glitch`'s ground
  read it; `Ui::looseness` uses the same value. An element that trails and has a ground fill is drawn
  on a layer above, so loose decoration it passes over does not show through it.
- The `#stirr` page, with twelve numbers as number controls; one more line in the F2 readout; `check`
  item 19 (the former item 19, effects, is now item 17), `bench … map` and `craft <dir> stirr`.
- The `scribblr` thread material (`src/strand.rs`). At rest it is byte-identical to `plain`. Loosened,
  the fill fades and the ink becomes one continuous thin line: it first follows the glyphs' true
  outlines, bridged into one stroke (clean outlined letters); looser, it drops vertices step by step and
  merges into long polylines across several glyphs that wander off. The one to three places with the
  most energy knot into large stars, one long tail enters from the scope's edge and another leaves
  through the opposite edge, and it writes itself back in reverse. Text to read and controls never turn
  into thread, fade or move. Eight numbers are in the material parameters (`reach` 40, `knot_r` 48,
  `tail` 480, `angular` 0.9), and `Look::hold_ms` holds the thread's pose for a beat.
- Long paths with `El::trace`: a polyline drawn by progress, solid or dashed (`dash`, `gap`, by arc
  length from the start), bent by the flow, stirring the map at its head. "Draw a line" on `#stirr`.
- The thread starts from the outline as the G-buffer stage drew it, plus the offset by which the map
  pushed the instance. Vertices are ranked by one Ramer–Douglas–Peucker pass, slide onto their chord
  and are then removed, so the line follows its chords and nothing jumps on the way back. Knots are
  long chords through a center. Tails are cut at the scope's edge and stop before text to read.
  `strand::figure` draws a tangle from a fixed seed as page content.
- The `#material-scribblr` page; `check` item 20, `bench … strand`, `craft <dir> scribblr` and
  `tests/strand.rs`.
- Letter spacing with `El::tracking`. Quiet type with `El::whisper()`: lowercase, 0.6 em tracking, in a
  serif (Source Serif 4 at its 8 pt optical size, Latin only, 82 KB, OFL).
- Three new readouts in the annotation layer: edge notes `EDGE` (a hairline, ticks and a pointer mark at
  the scope's right edge, with the scope size, the pointer position, the material and theme, the
  loosest element and the number of non-zero map nodes set vertically), a waveform `OSC` (the energy of
  one row of nodes as a one-device-pixel polyline) and a level bar `LEVEL` (the mean energy). The
  annotation layer honours instance transforms, used only to turn the edge notes by 90°. The exported
  tree gains `field: { level, nonzero }`.
- The tenth preset, Loose Thread: white thread on black (or paper white), one accent, `scribblr`, only
  edge notes and corner marks on hover; the map meanders, creeps along ink and lives two seconds (a title
  the pointer sweeps loosens past 0.8 and comes back within three seconds); the thread holds every other
  beat; a 700 ms transition loosens into thread and then draws the outline. Its style page is English
  only: a lowercase serif title, one picture, one sentence, three controls as one line of small type in
  the footer, and a drawn tangle at rest.
- The finishing pass gains black and white only (`binary`: a black-and-white palette, no dither, line
  widths rounded; text, thread and annotations are thresholded by coverage rather than brightness, and
  pictures get a two-level ordered dither) and invert (`invert`: `1 − colour` at the end).
- Annotation corner marks sit 6 px outside the element's rect, off its own text.
- Fixed theme ids and style-page descriptions that had drifted after themes and styles were removed.
- `check` items 7.2 and 7.3 (edge notes, waveform, level bar, edge notes at 1080p) and 20.6 and 20.7
  (black and white only, invert), three tests in `tests/gaze.rs`, the Loose Thread page in
  `tests/i18n.rs`, `craft <dir> loose-thread` and `craft <dir> thread-ref`.

## Foundation leftovers, the Lab as a wall, Body

- Sampling follows transforms: `morph::cover_at` maps points back through each instance's own
  transform, and `cells` and `ink_bounds` use the turned bounding box, so a posed or turned glyph
  loosens from where it is drawn, and masks and decode work there too.
- Hover and focus corner marks land on their target in the frame the pointer arrives (they used to chase
  it every frame and trail behind), with only a 70 ms close-in and fade.
- The lens touches only pictures and 3D elements that may loosen and that the pointer is not operating,
  added per instance in the G-buffer stage as fronts are. Looseness belongs to the instance's own ink
  (its ink wins over another's distance band) and no longer spills onto neighbouring decoration.
- Dragging, throwing and settling the floating box joined `check`'s partial-redraw comparison (every
  material, two at 2× scale).
- Decode counts coverage of the outline, not colour, so a dialog filled with the ground colour is
  covered too at the finest level.
- DESIGN.md no longer mentions the removed chrome, liquid and collage materials.
- The Lab: each stage takes over 70% of its cell and the piece fills it, the formula is one line and the
  numbers fold into `[ params ]`. Cells that can be seen keep running and stop when scrolled away (they
  rest with reduced motion). Every cell was redone to fill its stage, with the blue under the pointer;
  decode's exploded view became planes seen at an angle, each with a leader and a label.
- Removed the film (the prologue and chapters, the slate, the timeline, before-and-after, each chapter's
  caption), the xray material and its preset (Transparent Electronics), and the tests, checks, docs and
  strings that existed only for them. The exploded scene no longer uses xray.
- Body (`#body`) is the new full-bleed landing page. A page of readable text appears, then gathers into
  a slowly turning, luminous body made of its own glyphs (the skin, glyph count conserved) and more than
  ten thousand volume marks, with a key light, rim light, fog, bloom, a reflection and halo on the
  ground, three layers of dust, parallax and a breathing lens. Small markings in the picture (form,
  seed, gather / release, language, Lab, components); an opening at the pointer shows the original text.
- Bloom samples at a per-pixel noise angle, so text no longer shows ghost copies. `Pose::hue` gives one
  glyph an accent colour.

## Fields, the Lab, decode, a narrower scope, the frame budget

- Per-instance transforms (`Instance::xf`: turn, scale and shear about the instance's own center) and
  glyph poses (`El::pose`), so text can follow a path, wrap a sphere and be lit while its outlines stay
  true.
- Drawings (`el::canvas(Drawing)`: dots and strokes are shapes with true outlines) and `El::on_point`
  (move, press-drag and leave in one handler, in the canvas's coordinates).
- Fields (`amenui::field`): noise, bumps, metaballs, curl, grids, marching-squares contours, vector
  flows, advected swarms and masked Gray–Scott.
- `El::on_visible` (run only while seen) and `El::on_outline` (an element's true outline handed to the
  state as a mask).
- The frame budget (`amenui::budget`): morph tiles, shed marks, flow fields, swarms and drawing marks all
  draw from one budget, which shrinks when a frame runs over and recovers when there is room; the readout
  shows it.
- The Lab (`#lab`): ten experiments (lit text, contours, silk, grown shapes, mercury, text on a path,
  text on a sphere, orbits, a spring, the card stack), one example file each, each with its own page.
- The decode transition (`El::decode`): an element resolves from coarse to fine over its true outline
  and disappears by the same backwards; its documentation page has an exploded 3D view and a timeline.
- Removed the chrome, liquid and collage materials, the Neo-Y2K and Mixed media presets, and film
  chapters 11 and 12 (with the finale).

## Interaction craft

- Gesture physics (`amenui::gesture`): an analytically stepped spring with velocity hand-off on
  release, a rubber band, throw projection and velocity tracking.
- A bottom sheet (detents, dismissal by projection, rubber banding, a scrim, hand-off from scrolling)
  and a floating box (the corner picked by projection).
- The formula caption: a line under a demo that prints its real parameters, each editable.
- `el::layers`: children stacked in one box.
- English text around obstacles: Liang hyphenation (en-US) and up to 0.08 em of letter spacing; pieces
  that are not justified fall below 15%.
- A dot chart: a dot's size and brightness are the area under the curve, a sweeping line, draggable
  control points.
- The zine chapter's toner halved and kept off the text (`Shed.clear`), with a one-line caption.
- A fanning card stack: angle0 × falloff^n, delay_ms × n, three looks that follow the material.
- Skeletons: `El::skeleton()` draws the real tree as a skeleton (bars are measured line widths), the two
  `Loading` numbers, zero layout shift, and a comparison of four ways to load.
- Nested menus: a safe-zone triangle, two delays, the keyboard, and a triangle that can be drawn
  (`El::on_move`, `El::guide`).

## The prologue, display not slogans, guidance, wrap, the readout, look recipes

- The film's prologue: one small interface, as it is on the left and in a material on the right, with
  a draggable divider and a row of material names.
- No slogans: a chapter's title is one element that recomposes glyph by glyph across cuts; chapters 9,
  10 and 11 redone.
- Guidance in the bottom bar: previous and next chapter, autoplay progress, hints shown only until the
  first input, bracket hints, invisible edges.
- Text around obstacles follows the true distance, leaves no slivers and does not hop while dragged;
  body text never condenses glyph by glyph; nothing is cut off at 330 px wide.
- The F2 readout: one line or every number, with a sparkline, and no frames at rest.
- `look:v1:` recipes (in a URL or a PNG text chunk); eight new mark sets grouped by family; five Geist
  Pixel faces; the `<amenui-view>` custom element.

## Scrolling, the sign and the mechanism, languages, components and docs

- Slow scrolling moves a picture by whole pixels; fast scrolling leaves each material's own trace
  (§1.8.3, `check` item 18).
- The film's third actor is a flat sign; chapter 13 is a real 3D mechanism (gears, a cam, a follower).
- Message catalogs, `t!`, plurals, a fallback chain and right-to-left rows (§1.20).
- Eighteen new components, `amenui add`, and a documentation site laid out after shadcn/ui.

## Fixes and the film

Bounded recomposition, particles emitted from mesh surfaces, local clocks, and the twelve-chapter film.

## 3D, GPU particles, dirty boxes, presets and transitions

## Ripple rules, recomposition, the model-facing surface

## Interaction and performance

Drawing only when something changed, the frame-time readout, text input, incremental layout and long
lists, touch, and the accessibility mirror.

## Annotation, every material, twelve presets, scenes

## Transforms: particles and flows

## Components and the gallery

## Marks, pixel type, themes and materials as data

## The skeleton

The element tree, taffy, parley, scrolling, clicks, focus and the number control.

## The vertical slice

Two-stage drawing, waves that follow outlines, and one codebase for native and web.
