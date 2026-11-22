# 1.6.2 The materials

A material is a function in the screen-space shading stage,
`(coverage, SDF, field, position, time, parameters) → colour`, dispatched by the material number in a
uniform (or the override in this point's `near.a`). Each material has 8 parameters, all in one uniform
array (`style::MAT_PARAMS` has their names, ranges, steps and defaults; on the material page each is a
number control). Looseness is `k = (field + near.b) × the style's looseness scale`; with `k = 0` every
material outputs what `plain` does unless noted (rule 3). Shared helpers: `sdist(p)` (`near.r` as a signed
distance in device pixels) and its gradient; `ink_inward(p)` (walk back along the gradient to the nearest
outline and take the ink colour there, to colour what is drawn outside the outline; pictures are never
sampled); value noise.

| Material | At rest | Loose | Parameters |
|---|---|---|---|
| `plain` | As it is | Fades | — |
| `dottr` | Small dots on the ground that rise and fall with the field | Text breaks into dots by cell, dot size = the ink in the cell | (`Fx::pitch`) |
| `halftonr` | As it is | Rotated screen: each cell averages 4 points at its center, dot radius ∝ √(coverage × gain); cells switch one by one where `k(center) > 0.12 + 0.5 × hash`; with `split > 0`, three screens (15°/75°/0° plus `angle − 45°`) each take one colour channel, each offset by `split` | pitch, angle, separation offset, gain |
| `charr` | As it is | Each cell picks a character from the character mark set (the per-pitch set of §1.6.1) by the ink in the cell; at `k < 0.6` half the pitch, coarser as it loosens | pitch, levels, colouring (0 ink / 1 mono) |
| `smudgr` | Paper grain (two layers of noise, amplitude `grain × 2.75%`); accent ink is sampled separately at `(misreg, 0.6 misreg)` and overprinted with multiply on light grounds | Ink spreads outward along the outline by `spread × k × noise`, thresholded by noise into photocopy edges; specks on the paper, dropouts in the ink | grain, misregistration, specks, bleed |
| `carvr` | As it is | Ink and ground only: the outline moves by two layers of noise (the slow sway and chips of a knife); in a band of `band × k` on both sides of the outline, white wedge-shaped cuts of varying length tapering inward are carved inside, and broken ink lines along the outline are left outside | cuts, band, cut length, cut spacing, jitter |
| `glitchr` | As it is | Rows of height `slice`, hashed per beat of `hold` ms: some rows shift sideways, a few blocks repeat the block to their left, the RGB channels separate by `split × k`, a few rows invert; a `burst` (a rect the runtime puts in the field while a control is hovered or pressed, `Field::burst`) loosens it there too | row height, displacement, channel separation, hold, blocks |
| `near` | The G-buffer itself, in the ink and its greys: iso-distance lines of the SDF (hairline inside ink, the ink at 0.5, 0.3, 0.15 outside), the outline, faint ink | The dim grey over it where loose | spacing, fill |
| `scribblr` | As it is (the shading stage is exact) | The fill fades and the ink becomes one thin line of constant width: along the outline → bridged into one stroke → wandering, knotting, throwing out tails; the thread is shape instances drawn in the G-buffer stage (§1.30) | line width, sampling, wander, knot, knot radius, tail, follow the map, angularity |

**Text to read keeps the resting look.** `smudgr`, `carvr` and `charr` take their effect from `kt_loose`: the
looseness under the field with the element's own looseness, gated by `smoothstep(0.18, 0.26, …)`, so text held at
the legibility cap (0.15) reads as the material at rest while a front or a loosen effect passes (`tests/wrap.rs`: in
seven materials loosened to 0.6, at least 99 % of the legible ink's pixels equal the material's own rest frame).
**Past half loose** four materials go on changing with the element's own looseness `kl` (no new numbers): `dottr`'s
lattice opens to 1.6 × the pitch and its units stray twice as far; `charr` switches its cells at 0.3 and 0.7 (half,
then 1.5 × the pitch; one cell size per element at a time) and halves its levels toward 1; `halftonr` opens its
screen to 1.5 × the pitch and doubles the split, accent ink keeps one screen in its own colour, and each separation
is half the ink's own colour so no pixel shows a single channel (on a light ground: cyan and magenta a third of the
way to the ink and the ink itself, no yellow; ink that is partly the accent's is left to the accent's screen); `glitchr` shifts and repeats blocks up to twice as
far, only where something is drawn, and flashes rows only there. `sheet --sweep <n>` (`sample::sweep`) measures the
share of the title's pixels that changed from rest and moved from the cell before at each looseness
(`tests/sample.rs`). Under `charr` a thin accent element (a rule, thinner than a cell) is one unbroken row of accent
characters under any cell size: one cell's height centred on the rule's centreline is its row, every cell along it
shows at least a middle character, and nothing of the rule shows above or below it. The rule's cells take one size
for each stretch of three pitches, by the looseness on the rule there, so where a stir makes the size change along
it the characters change size between whole characters and stay at the same height. A cell over a picture's edge is not made into characters.

**The press flash** in `plain`: a pressed control is held at the legibility cap, which in `plain` fades its ink
toward the ground (`1 − 0.7 √cap`). The fade stops where a filled control's label (the ground on the accent, or dim
text, text or ink on the ground) would fall under 4.5 : 1 (it keeps 4.8 : 1, for antialiased strokes;
`Theme::press_keep(cap)`); where that leaves less than 6 % of a flash to see, the flash goes the other way, the ink
standing further from the ground, which only adds contrast. A pair already under 4.5 : 1 at rest is the theme's own.
`tests/audit.rs` checks every recipe during the flash, dark and light, for P1 contrast findings.

**Pictures past half loose** differ by material: `dottr` keeps the marks (§1.6.1); `halftonr` draws a screen at 1.5 ×
its pitch whose dots follow the cell's brightness (bright cells big on a dark ground, dark ones on a light ground,
from 0.25 to 0.95 of the pitch, over the tones pictures mostly have); `charr` draws square cells of the title's size,
each a character by the cell's brightness in the cell's average colour over a faint tile of it (`tests/sample.rs`:
fully loose, any two of the three differ in over a quarter of the picture; halftonr's largest dot is at least twice as
wide as its smallest).

Acceptance (`check` items 5 and 6): except `near`, every material at looseness 0 matches `plain` pixel for
pixel in body text (`smudgr` within its grain amplitude, ≤ 4/255 by default; `dottr` compares only the glyphs'
own pixels, since its ground has dots anyway), with a control showing the comparison sees a difference once
loosened; in the light theme, no fully loosened picture under any material (including while a front passes)
contains a page-background pixel.

**Sample sheets that check themselves** (`amenui::sample`, `sheet --material <name> --loose 1,0.7,0.4,0.2,0
[--theme <id>] [--arrival]`): one small page (a label, a large title, a rule and a picture loosened by `k`, a
paragraph and a solid button) drawn at each looseness side by side, and with `--arrival` how the material arrives
after the warm-up (plain, then 120 ms and 300 ms after it is ready). The sheet asserts what it should show and the
tool fails otherwise: neighbouring cells differ, the fully loose cell of `scribblr` has a thread, the cell at rest
is plain's frame byte for byte for a material whose rest is plain, and 120 ms after it is ready the material is on
the page. `tests/sample.rs` draws every material's sheet, dark and light.

**Weight** (`El::weight(px)`): a text instance carries an offset in device pixels in `extra.x`, and the
G-buffer stage recomputes coverage from the signed distance in the glyph atlas (moving the outline out by
`px`), so headings can be made heavier without a bold font file; particle sampling uses the same
computation. Lino, Bauhaus, Zine and Glitch headings use +1.5 to +3.

`near` (the eighth, id 7) draws the G-buffer in the ink and its greys only (the accents are for what is held,
not a material's rest): the signed distance as iso-lines every `spacing` px, in the hairline grey inside ink
and, outside it, the three nearest in the ink at 0.5, 0.3 and 0.15 (none further out); the outline in ink, the
ink itself faintly by `fill`, and the dim grey at 0.35 where loose. It is a view for seeing the mechanism and
has a look of its own at rest. Parameters: spacing 3 px, fill 0.25. The exploded scene uses it.
