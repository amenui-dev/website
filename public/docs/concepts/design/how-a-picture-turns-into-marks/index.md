# 1.6.1 How a picture turns into marks

When a picture loosens, **the picture itself turns into marks**; no grid is laid over it. The rules:

1. **What shows between marks is always the picture's own dark or black, never the page's background.**
   The same holds in the light theme. A picture has its own ground.
2. **Pixelate first, then cut marks.** The first step is seamless blocks of colour: each cell takes the
   average colour beneath it, with no seams between cells. As the pitch approaches 1 px it is the
   original picture. Only the second step draws a mark in each cell.
3. **Tone is expressed by changing the mark, not by shrinking a shape.** Luminance is quantized into a
   number of levels (a parameter), each level mapped to one mark of the mark set.
4. **Mark sets are swappable**; each set is one row of small tiles in the atlas:
   - Ordered dither blocks: solid, checkerboard, sparse dots, dense dots (the low-resolution poster look).
   - Characters: a run of characters ordered by density.
   - Strokes: short lines whose direction follows the picture's luminance gradient (like waving grass).
   - Light points: dots with a halo of glow; dark areas have none.
5. **Colouring** is a parameter: original, monochrome (the theme's ink), two-tone, or a given colour.
6. Optional: pushing marks outward by luminance (a character portrait with depth); glow.

Parameters: the picture's pitch, the number of levels, the mark set, the colouring, the ground's
darkness, how strongly orientation follows the gradient, depth, and glow.

Text and shapes have no ground of their own; when they loosen, the page shows through, which is right.

**Character marks over a part of a picture** (`el::cells`). An element over a picture can draw the
picture beneath it as characters at a pitch of its own, rectangular or square (7 × 14 px by default):
its box is cut into whole cells from its top left corner, each cell takes one character of a ramp the
caller gives (least ink first; a character given twice takes two bands of brightness; a ramp without a
blank is a grid full everywhere), by the brightness of the picture under the cell in equal bands, dark
to light or light to dark (`light_dense`), and the character is drawn in the cell's own colour mixed
toward an ink (`mix`, 0.62 by default). Between the characters the picture shows as it is. The GPU reads
the picture and picks the characters: the element is one instance and one draw whatever its number of
cells, the characters are cut once per face, ramp, size and cell into the glyph atlas, and moving the
element lays nothing out. Still, it costs nothing.

**Ink over a picture.** A pixel where a picture is drawn is shown from the picture's colours, so text and
shapes drawn over a picture are written there as their own colour, the colour the material shows them in
at rest. Their antialiased edges then blend with the picture by their coverage, and a see-through group
(`El::opacity`) by its opacity, whatever their siblings are doing; nothing of the ink's codes is read as the
picture's red, green or blue. Text to read over a picture is shown as it is within its island, however loose
the picture is beneath (§1.15). Ink over a picture that is not text to read loosens with the picture. In the
modal layer (and any layer: what is drawn after something that stirs, parts in flight) the layer's own pictures
are pictures like the page's, and what is drawn over one of them, or a faint shape (opacity under 0.5, a line
riding a flight) over any picture, is ink over it; a panel of the layer over the page's picture covers it, and
its ink is the layer's, as before. A picture marks where it is only while it shows (opacity 0.01 or more), so a
window waiting to fade in leaves the colours under it as they are. A cut-out counts as its whole box: over its transparent parts the ink's edges are drawn
whole, not blended with the page.

#### Implementation

**The mark atlas** (`marks`, RG8: r is the mark's coverage, g its height for relief; generated on the CPU at
start-up with 6 mip levels and uploaded once):

- A row is a mark set and a column a luminance level, always 16 columns. Each tile is 32×32 px and tiles
  seamlessly on all four sides. With fewer than 16 `levels`, level n uses column
  `round(n × 15 / (levels − 1))`.
- Mips go down to one pixel per tile: when cells are smaller than 32 px the mip `log2(32 / P)` is read,
  which gives the mark's average rather than a few aliased pixels. Sample coordinates are clamped half a
  pixel inside the tile, so a neighbouring tile is never read.
- Ordered dither: level k is the black-and-white pattern from comparing the 8×8 Bayer threshold matrix
  with k/15, scaled up to the tile (4×4 px per element). Level 0 is empty and the last level solid.
- Characters: every printable ASCII character is drawn once with the text system and **its actual ink is
  measured**; then 16 evenly spaced targets are taken between 0 and the maximum ink, each target taking
  the character whose ink is closest and greater than the previous level's. Density is never ordered by
  eye. The face is Departure Mono, drawn at 2× and then stretched to twice the width so the 7×14 character
  cell fills the square tile (one character per cell, as in ASCII art).
- **Characters are cut again for the pitch they are shown at** (`char_tiles`). A 32 px tile shrunk below
  16 px blurs, and the picture reads as grey noise. So for each device pitch from 4 to 40, one row of P×P
  tiles is cut using an integer multiple of Departure Mono's size (capital height 8 px, multiple
  `P / 9`); the glyphs already sit on whole pixels and are never resampled. Characters that don't fit the
  tile at that pitch (descenders, for example) are not used there. Bold forms (the glyph stacked one face
  pixel to the right), denser than every regular character, join the selection so the brightest levels
  are brighter. Each pitch picks its own 16 by measured ink. Below 8 px, the 8 px tiles are averaged
  down. In that range the shader reads the character set pixel by pixel (`textureLoad`); outside it, it
  reads the 32 px atlas.
- Strokes: a line almost the length of the tile, with round ends, thicker at higher levels. Direction is
  not stored in the atlas; the shader rotates the sample coordinates, and a turned line wraps into the
  neighbouring tile at the edge, so strokes join into hatching.
- Light points: a disc with radial falloff, radius and brightness growing with the level, soft-edged.

**Parameters** (`Markr`, one uniform struct; up to 8 sets, an instance picks one by the slot in `extra`):

| Name | Meaning | Range |
|---|---|---|
| `pitch` | Pitch, in logical px | 1–64 |
| `levels` | Number of luminance levels | 2–16 |
| `set` | Row of the mark set | integer |
| `colour` | 0 original, 1 the theme's ink, 2 two-tone, 3 a given colour | integer |
| `ground` | The ground's brightness: 0 black, 1 the cell's own colour | 0–1 |
| `gamma` | Curve from luminance to level | 0.3–3 |
| `spread` | Width of the dithered transition between neighbouring levels; 0 is hard steps | 0–1 |
| `follow` | How strongly marks turn to follow the luminance gradient | 0–1 |
| `depth` | How far samples are pushed apart by luminance, in logical px | 0–24 |
| `glow` | Glow strength | 0–2 |
| `palette`, `colours` | Palette row; how many of its first colours to use, 0 for unlimited | integer; 0–16 |
| `relief` | Relief strength, 0 for none | 0–3 |
| `regions`, `region_by`, `region_count`, `region_sets` | Cells per region; by hash (0) or by the region's luminance band (1); how many sets; four set indices | integer |
| `tint_a`, `tint_b` | The two colours for two-tone and a given colour | 0–1 |

`pitch` is in logical pixels; the device pitch is `round(pitch × scale)`, so on a 1× display
`pitch = 1` is one device pixel.

**In the shader, one pixel of a picture is computed like this** (`k` is the looseness here, 0 to 1):

1. `k = 0`: output the colour in the coverage buffer. Done.
2. The current pitch is `P = round(mix(1, pitch, smoothstep(0, 0.5, k)))` device pixels and
   `cell = floor(p / P)`. One grid is shared by the whole screen, so neighbouring pictures and text line
   up. `k` is taken at the center of the cell the point is in (the cell is found once from this point's
   `k`), so a cell has only one pitch.
3. The cell's colour `c` = the average of 4 points in the cell (9 when `P > 16`) read from the coverage
   buffer pixel by pixel (no filtering). Outputting `c` here gives seamless pixelation; with `P = 1` it
   equals the original.
4. Luminance `L = dot(c, 0.299, 0.587, 0.114) ^ gamma` (with `c` before palette limiting). The level
   `n = floor(L * levels + (bayer(cell) - 0.5) * spread)`, clamped to 0 … `levels - 1`.
5. Coordinates in the cell `uv = fract(p / P)`. If `follow > 0`: the gradient angle comes from the
   differences of the mean luminance of the left/right and up/down neighbouring cells, θ is that turned
   by 90° (along the isophote, so marks follow the form), and `uv` is rotated about the cell center by
   θ (times `follow`). Flat parts of the picture (a small gradient) are not turned.
6. The mark `m = marks[set][n](uv)`.
7. The ink colour by `colour`: `c`, the theme's ink, an interpolation between two colours by `L`, or a
   given colour. The ground `g = c * ground`. For the character set in original colour, the ink is `c`
   raised to full brightness (same hue, divided by its largest component): characters express tone by
   changing glyph, and darkening the ink again by the cell's own darkness would count darkness twice and
   make dark pictures vanish.
8. The cell's result `r = mix(g, ink * relief_factor, m) + glow * halo`. The halo is the light this cell
   and its eight neighbours each emit by luminance, falling off with distance from their centers, so
   bright marks spill light between cells (with only this cell's softened `m`, light could not leave the
   cell and would look blocky).
9. Output `mix(c, r, smoothstep(0.5, 1, k))`: in the first half of the time cells only coarsen; marks
   emerge in the second half.

With `depth > 0` and a pointer, before step 3 `p` is pushed by `L(p) * depth` in the direction toward the
pointer, giving parallax that follows the pointer (`L` is read at this point, not from the last frame).

A one-pixel cell has no room for a mark: marks, palette limiting, parallax and glow all come in through
`w = clamp(P − 1, 0, 1)`, and step 9 is really `mix(c, r, smoothstep(0.5, 1, k) * w)`. So with
`pitch = 1` any `k` outputs the original, which is exactly what acceptance requires.

**The page background appears nowhere in these nine steps.** That is rule 1 in code: a picture's pixels are
computed only from the picture's own colours, its marks and black.

**Acceptance**: in the light theme, fully loosen a picture; no pixel inside the picture's rect in the
screenshot has the page background colour (compared by sampling). With `pitch = 1` and any `k`, the output
equals the original pixel for pixel (within 1/255). `check` reads pixels back to test both (every sample
picture × every mark set × several pitches, ground brightnesses and colourings, plus palettes, relief,
glow and regions), each with a control showing that the comparison itself can see the background and the
marks.

#### Mark sets and three additions (after ASCII Magic's categories)

| Mark set | What a tile draws | Extra capability needed |
|---|---|---|
| Characters | Characters ordered by ink | none |
| Dither | Bayer patterns | palette |
| Pixel art | Solid blocks with a very faint grid line | palette |
| Mosaic | Solid blocks with slightly darker edges, like tiles | relief |
| Bricks | A solid block with a round stud | relief |
| Dots / crosses / diagonals / diamonds / dashes | The matching geometric mark, size or weight by level | none |
| Light points | Soft-edged dots | glow |
| Strokes | Short lines following the gradient | orientation |
| Mixed | A different mark set per region | regions |

Three additional capabilities, all parameters in `Markr`:

1. **Palette limiting** (`palette`, `colours`). The cell's colour becomes the nearest colour of a palette.
   A palette is a 16×1 row; all palettes stack into one small texture and `palette` is the row; `colours`
   is how many of its first colours are used, 0 for no limit. Two-colour dithering is `colours = 2`. It
   happens after step 3: `c = nearest(palette, c + (bayer(cell) - 0.5) * spread)`. There are six: black
   and white (2 colours), risograph three-colour (3), handheld green (4), CGA (4), warm five-step (5) and
   PICO-8 (16). The low-resolution limited-palette poster is the pixel-art set plus a palette plus
   `spread`.
2. **Relief** (`relief`). The mark atlas has one more channel for height. In step 8, the height gradient
   gives a shading factor multiplied into the ink, producing the bevels of mosaic tiles and the studs of
   bricks. Nothing is computed with `relief = 0`.
3. **Regions** (`regions`). The screen is divided again into large cells of `P * regions`, each using its
   own mark set: `set = region_sets[hash(region) or the region's mean luminance band]`, from the first
   `region_count` sets (at most 4). Region boundaries are stepped, which gives the mixed look.

**The compare wipe** is an interaction, not a material: a draggable vertical line, `k = 0` on one side and
`k = 1` on the other. It is a still, straight front expressed by the same field (the field gains a
half-plane source, max-combined with fronts and limited to the picture's visible rect). A picture element
gets it with `.wipr(position)`; dragging the picture (`on_drag`) moves it, and the position can be set
exactly with a number control. The line itself is drawn one pixel wide in the accent colour.
