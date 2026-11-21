# 4. 风格 = 主题 × 材质 × 材质参数 × 标注 × 字体

- **A theme** is data: colours, type scale, spacing, line widths, corner radii. The current `Theme`: the background
  (possibly a top-to-bottom gradient from `ground` to `ground2`), ink, three accents (`Paint::hue(0/1/2)`), which
  face each of the display, body and mono roles uses, the type scale (heading, subheading, body, small) and thin and
  thick line widths. There are fifteen: dark, light, pixel, and each style's own (Loose Thread has a black and a
  paper set: `THREAD`, `THREAD_PAPER`). `style::THEME_IDS` are their ASCII names, matching `THEMES` one to one. The
  root sets `.fonts(theme.roles())`, and text elements use roles (`Font::Display / Body / Mono`) or a face directly.
- **The greys** (`Theme::greys`) are opaque colours of the theme's own between its ground and its ink: `raised`
  (surfaces off the ground: a stage, a preview box), `hairline` (card frames, rules, dividers, control outlines at
  rest), `dim` (labels, numbers, hints, descriptions) and `text` (prose, idle control labels). `Paint::RAISED`,
  `HAIRLINE` (`FAINT`), `DIM` and `TEXT` stand at fixed tones (20, 51, 168 and 219 of 255, so the ink image holds
  them exactly) and are drawn as those colours exactly; a tone between two is drawn straight between their colours
  (`Theme::grey`, the shaders' `grey`), so a grey is never the ink at an alpha. A theme without greys of its own
  takes `Greys::mixed` (the plain mixes of ground and ink at those tones), or `Greys::mixed_dim(ground, ink, 4.5)`,
  the same with `dim` taken toward the ink as little as it takes to stand 4.5:1 from the ground; a theme made from another with its own
  ground and ink sets them (`greys: Greys::mixed(ground, ink)`), or it keeps the other's.
- **The extreme** (`Paint::EXTREME`, `Theme::extreme`) is white on a dark scheme and black on a light one: past the
  ink, for what the pointer is over. In the ink image it is a fourth hue code (0.45; the code the glyph heights once
  took, which nothing wrote).
- **The default theme** (`DARK`, `LIGHT`) takes the AA.9 table: ground `#1a1b1d` / `#f1f0eb`, raised `#232528` /
  `#e6e5df`, hairline `#383b40` / `#cbc9c1`, dim `#989da4` / `#5f6266`, text `#d6d8d4` / `#2b2c2e`, ink `#f2f2ee` /
  `#141517`, accent `#f47a3c` / `#b23f0d`, second `#7db0f7` / `#1a5cc7`, third (the warning colour) `#f2c94c` /
  `#7d5c00`. Its controls rest in ink on the ground: a primary button (`Variant::Primary`, one in each group of
  actions) is an ink plate with a ground label that turns to the extreme under the pointer; a switch that is on is an
  ink track with a ground knob (off: a hairline track, a dim knob); a slider's run and thumb are ink, the rest of the
  run hairline; a radio's dot, a checkbox's tick and outline, a pressed toggle and the current option of a choice are
  ink (a toggle, an option and the current page of a pagination as an ink plate with a ground label), as are a
  current tab's rule, a progress bar's fill and the label of a selected list row or menu option; a selected or
  keyed row's plate (`widgets::SELECTED`) is the raised grey, not a tint of the accent, and the hairline grey on a
  surface that is itself raised (`widgets::selected_on`); a hot menu row (command, context and nested menus) is
  that plate with an ink label, and a skip link an ink plate with a ground label. Menus, selects, context and
  nested menus and the sheets (sheet, bottom sheet) are raised: the raised grey in a hairline frame
  (`widgets::MENU_SURFACE`), so their selected and hot rows are the hairline grey; a popover stays on the ground. The hover and focus marks are
  ink. Outlines of interactive controls (a default button, an input, a number, a select or menu button, a
  checkbox, a radio, a choice, a toggle, a page of a pagination) are dim at rest and ink under the pointer or with
  focus (`El::border_hover`), and ink while a select or menu is open or a number is typed into; the hairline grey
  is for card frames, rules, dividers and the outer frame of a group (panels, dialogs, sheets, popovers, toasts,
  tooltips, menus). The accent is for what is held or dragged (a slider's thumb, a switch's knob while pressed),
  a number's caret, a dot chart's dot under the pointer and its control points while dragged, and the explicit
  `Variant::Accent`. Categories (an avatar's people, a chart's series) are told apart by ink, dim and text
  (`widgets::SHADES`), and by shape where three are not enough; `second` and `warn` are not category colours. The
  framework's examples follow the same rule: a main action is `Variant::Primary`, and the accent is only in a
  named Accent or colour specimen or on what is held or dragged.
- **Paints that answer the pointer** (`El::fill_hover`, `El::fill_held`, `El::border_hover`): an element's fill while
  the pointer is over it or the control it is part of (the nearest pressable element at or above it), and while
  that is pressed or dragged; its border while the pointer is over it or it, or the control it is part of, has
  focus. Each falls back to the paint at rest (held to the hover fill under the pointer). The page is painted again
  when what they answer changes; a cached subtree with one is drawn every time.
- **A material** is how the shading stage draws.
- **Annotation** is whether the third pass is on and what it draws.

A style preset (`style::Style`, plain data; `style::styles()` gives ten) is: a theme, a material, that material's
parameters, annotation settings, a looseness scale (Normcore's is 0.3), the text lattice pitch, and the `Markr` for
pictures on the style page. The top bar's Style menu sets all of them at once, after which the theme and material
menus work as usual; switching is a commit that spreads from the press. Opening a style page applies it too (its
side rail item is a commit). The table is the design intent; the actual presets are in `styles()`:

| Preset | Theme | Material | Annotation | How |
|---|---|---|---|---|
| Neo-Dotmatrix | Black, monochrome | `dottr` | off | A global dot lattice; loosened, each dot's size is the ink beneath it |
| Lo-Fi Print | Paper, two inks | `smudgr` | off | Halftone dots, misregistration, paper grain, ink bleed (thresholding the SDF) |
| Glitch | Black | `glitchr` | off | Band slips, channel separation, block displacement, holding the last frame |
| Metamorphosis Linocut | Paper, monochrome | `carvr` | off | Ink = a threshold of `SDF + noise`, edges like knife cuts; white cut along iso-lines; interpolating two states' SDFs is the morph |
| Algorithmic / Digital Gaze | Black, monospace | any | on: boxes, numbers, confidence-style labels, tracking lines | The annotation layer |
| Tech-Manual | Grid paper, black + one signal colour | `plain` + hatched fills | on: leaders, numbers, dimension lines | Another way of drawing the annotation layer |
| Normcore | System grey, default sizes | `plain` | off | Only a theme |
| Cyberminimalism | Near black, hairlines, monospace | `plain` | off | Only a theme |
| Bauhaus | Primary colours, geometric shapes | `plain` | off | A theme plus geometric shape components |
| Loose Thread | White thread on pure black (or black on paper), one accent | `scribblr` | hover: `EDGE`, `FRAME` | At rest: small type, a picture and a drawn tangle; headings that are touched come apart into thread (§4.4) |

The `plain` material also goes through two-stage drawing; it just doesn't loosen. Waves, condensing and corner marks
are still there.

### 4.1 Presets are data, with their own transitions

`preset::Preset` (serde, `to_json` / `from_json`): a name, the theme (by name) and its three faces, the material and
its 8 numbers, annotation, the looseness scale, the lattice pitch, the pictures' `Markr`, an **interaction
character** (`Interaction`: press feedback duration, front speed / width / lifetime, the default responses of text
and controls and of everything else, recomposition parameters) and a **transition**
(`Transition { duration_ms, steps: [Step; 4] }`). `Preset::apply(&Look)` gives the look (`Look::press_ms`,
`Look::roles` and `Look::transition` are new); the runtime uses `roles` to decide how elements without a response
respond and `press_ms` for the press burst. `Preset::numbers()` lists every number by its JSON path
(`transition.steps.0.a`) and `set_number(path, value)` changes it: the website's number controls are generated from
the data.

**A transition is a small program**: up to four steps, each with a kind, a start and end within the transition time
(0–1) and five numbers; the material's last step composites by it between the old picture (what was on screen at the
moment of the commit) and the new: how much of the new shows here, where each side samples from, what colour is laid
over, how far the old fades toward the ground, how far channels separate. Kinds: reveal (circle, line, row by row,
shuffled cells, a torn line; row by row, the rows just swept show only lattice dots), slipping bands (each band
shifts, channels separate, a few hard cuts, then lock), fade (with a little displacement), light bar (a photocopier:
the new page behind the bar, toner noise settling with distance), colour blocks (a few rows of flat theme colour slide
from left to right, revealing the new page), loosen (the whole page loosens by its material, computed on the CPU into
`rip.y`), layers (separating nearer the brighter, swapping, closing), detection boxes (the largest few elements are
boxed in reading order, the box drawn along its edges, and the inside swapped for the new), carve (the old fades to the
ground, knife cuts grow on both sides of the new outline, then ink floods in), outline first (outlines and boxes are
drawn first in reading order, the content filled in after), torn reveal (torn along a noisy line, the old layer
sliding away offset) and crack (`StepKind::Crack`, id 11; numbers `preset::CRACK`: cell size 120 px, angle 70°, bias
0.6, waver 2 px, hard edge 1): the page breaks along the edges of a Voronoi tiling turned by the angle and squashed
across it by the bias. The cracks run out from where the commit was pressed (the page's centre without one): a pixel's
crack is drawn once the run (0 to the page's diagonal over the first 0.4 of the step) has passed it, and the cracks
stay thin, from 0.6 px to `0.06 a` (7 px) at 0.4, measured in page px whichever way they run. Their edges are straight
but for a slow waver (`d` px every `a / 5` px). Then each piece leaves whole at its own time `t = 0.28 + 0.6 × (0.6 ×
distance from the press ÷ the farthest corner's + 0.4 × the piece's hash)`: between `t` and `t + 0.15` its own crack
widens (slowly, then fast: the fourth power) until it covers the piece, so at any moment most pieces are whole or
gone. (Against the farthest corner rather than the diagonal, so the last pieces leave at the step's end wherever the
press was.) Inside a commit's scope (`El::commit_scope`) the Crack is measured against the scope's box: pieces of
`clamp(short side ÷ 3.5, 16, a)` px, the cracks' width (`0.06 ×` that, at least 1 px) and waver (every fifth of it)
with them, the run over the box's diagonal and the distance against the box's farthest corner from the press; and the
step takes `clamp(box diagonal ÷ page diagonal, 0.5, 1)` of its time (`Split::span`). Without a scope the bytes are
as before. The `preset` recipe's Crack button commits the tech-manual look through an 800 ms crack
(`tests/transition.rs`: the last frame is the new page drawn directly, a tenth of the way in under a tenth of it
shows, what shows only grows, the same numbers give the same bytes; at 0.2 nothing of the new page shows farther than
0.6 diagonals from the press; at 0.3 under 12 % shows; at 0.6 at most a quarter of the pieces are between 15 % and
85 % gone). During a masked transition every pixel is one page's whole colour (ground, ink and edge together); no
element blends its own colours between the two themes, and `render --frames <the transition> --audit` finds no P0 in
any frame.

The ten presets' transitions (`presets()`, all ≤ 900 ms): Gaze detection boxes; Dotmatrix row-by-row lattice; Lo-Fi
Print light bar; Glitch slipping bands with three hard cuts and then lock; Linocut carve; Tech-Manual outline first
with boxes; Normcore none (0 ms); Cyberminimalism a 120 ms fade with 6 px displacement; Bauhaus colour blocks; Loose
Thread 700 ms: loosen first (the whole page into thread, 0–0.55), then outline first (0.45–1). `scribblr`'s thread is
computed on the CPU, so the loosen step is computed once more on the CPU (`Transition::loosen`, the same function as
the material pass), and text to read still goes only to `legible_cap`.

**Where they are used**: switching to a style (the top bar menu, a style page's side rail item, a scene) and entering a
style's page are commits (§1.5) using the target style's transition (the look is already the new one after the commit).
**Interruptible**: the commit's `before` is the picture on screen at that moment (the frame the renderer keeps, §1.19),
so switching again mid-transition continues from how it looked then toward the new target; input stays available
throughout. The frame where the transition ends (time ≥ 1) is the new picture directly, without the program. A look
without a transition (`duration_ms` 0) uses the original front spreading from the press.

**The page**: `#styles` (style switching): from / to choices (choosing "from" applies it at once), Play (a commit),
every number of the target style's interaction and transition (number controls generated from the data), and JSON
export / import (a multi-line field that can be copied and pasted). **Acceptance**: unit tests in `preset.rs` (JSON
round trip, number paths, ≤ 900 ms); `check` item 16: for every preset, the frame after the transition ends is
byte-identical to the same page drawn directly with the target preset (the material clock aligned to the transition's
side), and mid-transition and both ends differ (except Normcore); samples `samples <dir> transitions` (6 frames per
transition, one row each).

### 4.2 Small type and edge notes

**Tracking**: `El::tracking(em)`, −0.05–1.5, 0 by default, laid out by parley (`text::Setting { around, track }`), so
measured widths, selection, recomposition and `scribblr`'s outlines follow it.

**Quiet type**: `Font::Whisper` is a fixed face, not a theme role: Source Serif 4 at its 8 pt optical size and weight
400, Latin only (82 KB, OFL, `assets/licenses/source-serif`; `text::SERIF`). `El::whisper()` makes the text lowercase,
in this face, with 0.6 em tracking; the size is set by the caller, 11–13 px. There is no Chinese serif: in demos small
type is Latin only, and Han characters fall back to the fallback face.

**Edge notes** (`gaze::EDGE`): a one-device-pixel hairline at the scope's right edge, exactly on the scope rect's
rightmost column; ticks every 10 / 50 / 100 px (3 / 6 / 9 px, inward), and a 14 px mark at the pointer's height. To the
line's left, a few lines of small type set vertically, read bottom to top (turned 90° counter-clockwise), in the serif
at 12 px with 0.12 em tracking and 18 px between lines, starting a new column to the left when they don't fit and never
leaving the scope. They all state current values: the scope's width × height, the pointer's coordinates in the scope
(`x — y —` when it isn't there), the material and theme (`THEME_IDS` names), how loose the loosest instance is right now
(by `Ui::looseness`'s rule) and the number of non-zero nodes in the field map. Turned text: each glyph instance turns
about its own center (`xf = [−1, 1, −1, −1]`), the center moved to its place on the turned line; widths and heights
are integers, so after turning glyphs still land on whole pixels and are not resampled.

Acceptance: `edge_notes_say_what_is_true_now` in `tests/gaze.rs`: every line equals the runtime's current value
(scope, pointer, material and theme, `looseness("root")`, `fieldmap().nonzero()`), the hairline is at the scope's right
edge, and every line is inside the scope without overlapping. `check` item 7.2 (`dottr` and `plain` at 1×, `plain` at
2×): the hairline is every row of the scope's last column, not the column outside, one device pixel wide between ticks;
every pixel of the type is a mix between ground and annotation colour (within 2/255 under `plain`), each line has
full-colour pixels, and over 30% of its ink is solid (coverage > 0.6). Item 7.3: at 1920 × 1080 and scale 1, over 30%
of each edge note's ink is solid (currently at least 42%).

### 4.3 Reading the field map: a waveform and a level bar

The field map (§1.29) is how disturbed the page is right now, so it is read like an instrument; no audio is involved.

- **`OSC`**: a line across the scope (stopping 40 px from the right edge when edge notes are on, so it doesn't cover
  them). One point per column of map nodes: `x` is that column's position, and the height is that column's energy in
  some row (the quantized byte, as the shader reads it) times `osc_amp`, upward from the baseline
  `scope top + height × osc_y`. The row is the pointer's row, or the row with the most energy when the pointer isn't in
  the scope (`FieldMap::strongest_row`). It is drawn as a one-device-pixel polyline without anti-aliasing: each device
  column gets one vertical run of whole pixels covering the heights where the line enters and leaves that column, so
  the colour is exactly the annotation colour, never mixed with the material beneath.
- **`LEVEL`**: a 6 px vertical bar at the scope's left edge, rising from the bottom to
  `scope height × min(1, mean energy / level_full)` (`gaze::level_height`, the mean energy being `FieldMap::level`).
- Numbers (in `Gaze`, with number controls on the website's annotation page): `osc_amp` 4–120 px, default 40; `osc_y`
  0–1, default 0.5; `level_full` 0.01–0.5, default 0.08.
- With the map at 0 the waveform is flat and the level bar 0 high, and no frames are wanted (no frames while the map is
  still; the readouts ask for none of their own). The exported tree's root gains `field: { level, nonzero }`.

Acceptance: `the_waveform_and_the_level_read_the_field_map` in `tests/gaze.rs`: with the map at 0 every point is on the
baseline, the level is 0 and two seconds draw 0 frames; after a stir at x, the highest point is within a cell of x and
rises above the baseline by the energy there times `osc_amp` (within 1 px), reading the row with the most energy; the
level bar equals the formula, and `export_tree().field.level` equals what was drawn; once the map settles both are flat
again. `check` item 7.2: in every device column away from the baseline the annotation colour is one continuous run,
exactly one device pixel where flat, with no annotation colour mixed into the pixels next to it; the level bar's height
equals the readout (±1 px); control: after a stir the waveform is not flat.

### 4.4 Loose Thread

The pieces above, combined into the tenth preset.

| Item | Value |
|---|---|
| Theme | `THREAD` (pure black ground, white ink, one accent `[1.0, 0.33, 0.20]`, the other two accents being the ink), or `THREAD_PAPER` (paper white ground, near-black ink). The accent is the annotation colour and appears only where the pointer touches |
| Material | `scribblr` |
| Type | Headings and body as usual; small type with `El::whisper()` |
| Annotation | Hover, `EDGE` and `FRAME` |
| Interaction | Text loosens by default too (`roles` gives text `Loosen`); text to read doesn't become thread or move under `scribblr` (§1.30) |
| Field map | `stirr` 1.0, `meander` 0.5, `creep` 0.15, `energy_life` 2.0, `bleed` 0.06, `breath` 0 (when the pointer sweeps a heading not meant for reading, its looseness passes 0.8, is still over 0.3 a second later, and returns to 0 within three seconds) |
| Hold | `beat` 240.96 bpm, 3 steps (an 83 ms step) |
| Transition | 700 ms: `Loosen` (0–0.55) then `Outline` (0.45–1) |
| Finishing | Off by default; the style page can turn on black and white only (`binary`) and invert (`invert`) |

The style page (`#style-loose-thread`) is a real piece, English only within its scope whatever the interface language
(`Locale::En.tr`, with `thread.*` only in the English catalog): a line of quiet type at the top, `amenui — no. 10`; the
title `loose thread` in `Font::Whisper` at 30 px, lowercase, 0.3 em tracking; a picture with the caption `fig. 1`; a
sentence of 14 px body text on the right; and a footer line of quiet type that is three controls (a two-option radio
`black · paper` and two switches, `1-bit` and `invert`), the selected one in the accent, drawn without switches or
borders but reachable by keyboard and in the accessibility mirror as a radio group, radios and switches. 72 px on the
right are left for edge notes. The page has a drawn tangle as content: a knot of radius 70 at the picture's bottom right
(one corner of the knot behind the picture) and a line passing through it from the top of the page to above the footer,
1 px in ink, never covering text to read (`El::trace`, with points from `strand::figure` on a fixed seed); it draws in
over 700 ms on entering the page, then rests and wants no frames; it draws under every material, so rule 3 is
unaffected, and the map's flow pushes it a little and lets it return as the pointer passes (`pull` 2.5). As the pointer
moves over the composition, a 1 px point following the pointer stirs the map (`El::stirr`, radius 72), so thread
appears only where touched; the title is set as text not for reading (`legible(false)`) and comes apart into a single
thread with knots and two tails; body text and controls stay. Narrow windows have no tangle. Samples
`craft <dir> loose-thread` and `craft <dir> thread-ref`.

Acceptance: the unit tests in `preset.rs` and `check` item 16 (every preset's end frame equals drawing it directly)
include it automatically. `the_loose_thread_page_loosens_its_title_all_the_way_and_text_to_read_holds` in
`tests/strand.rs`: at rest there is no thread and two seconds draw 0 frames, the tangle is at least 80% of the scope's
height and enters no box of text to read; after the pointer sweeps the title its looseness is at least 0.8, with two
tails (the end tail's ends at least `300 × k` apart, the start tail at least half as long), every point of the thread
inside the scope, and the tails entering no box of text to read; a second after the pointer leaves it is at least 0.3,
and at three seconds 0, with no thread and no frames wanted; throughout, no thread belongs to an element for reading,
and the glyphs to read keep their rest positions and opacity one by one. `loose_thread_shows_no_chinese_in_either_language`
in `tests/i18n.rs`: under both interface languages the text and names in the scope are the same with no Han character,
the title is lowercase in `text::SERIF`, and the three controls have their roles and are reachable by keyboard.
`corner_marks_never_lie_on_ink` in `tests/gaze.rs`: with the pointer resting on the small type, the title, the picture,
the body text and the two switches, no pixel of a corner mark falls in any glyph's box (corner marks are 6 px outside the
element rect). `check` item 20.6: with `binary` the read-back frame has only two colours; text and thread are
thresholded by coverage (coverage of at least half is ink, however bright its colour; a very faint fill is still
ground), pictures get a two-level 4 × 4 ordered dither (by the square root along the ground-to-ink range, so dark
pictures keep their midtones on a dark ground), and the annotation layer uses the same coverage rule; for every glyph of
body text, small type and control labels and every edge note, the inked pixel counts with and without it (without, a
pixel counts when at least halfway from the ground to its fullest colour) differ by at most a fifth, and ink takes 20% to
80% of the picture. Item 20.7: with `invert` every pixel is `255 − value` of the original (within 1, currently 0), and
turned off it is byte-identical to the original; together with `binary` there are still only two colours.
