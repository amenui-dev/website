# 1.21 收尾处理：后期效果、抖动和调色板

**Data** (`src/post.rs`, serde, in a preset's `post`, and therefore in `look:v3:` recipes; old presets without
the field have everything off): `PostSpec { bloom, chroma, scanlines, grain, vignette, curvature, dither,
palette }`, each a number with a default, all off by default: `Bloom { strength 0, threshold 0.7, radius 8 }`,
`Chroma { offset 0 }` (logical px; red and blue each offset that much outward from the center),
`Scanlines { strength 0, period 3 }`, `Grain { amount 0, size 1 }` (fixed grain that doesn't crawl over time: a
still page is an unchanging picture), `Vignette { strength 0, radius 0.55 }` (a fraction of the half diagonal),
`Curvature { amount 0 }` (a point at distance d from the center shows what is at `d × (1 + amount × d²)`, black
outside the frame), `Dithr { kind, levels 4, strength 1 }` and `Palette { kind, colors }`. `Look::post` is its
fixed-size form (palettes of at most 16 colours).

**The order is fixed**: bloom → chromatic offset → scanlines → grain → vignette → curvature, then quantization:
dither and palette. Curvature is the last thing done to the image, so the shader computes its mapping first and
every earlier step reads at the mapped position; dithering uses the screen's own pixels. Quantization happens in
sRGB (with an sRGB swap chain the colour is converted there and back).

**Grain in a region** (`El::grain()`): once any element on the page is marked, the grain is drawn only inside the
box round everything marked elements draw this frame (each instance within its clip, in device px), so a picture that
flows carries it where it is on each frame; the grain itself is the same (its hash, amount and cell), so a window
shows the page's grain pixel for pixel. With nothing marked it covers the whole frame. The runtime sets the region in
the look it draws with (`Post::region`). `tests/grain.rs`.

**Bloom in a region** (`El::bloom()`): the same for the bloom. Once any element is marked, the bloom glows only
inside the box round what marked elements draw this frame, and only from what is drawn inside it (a tap outside the
box adds nothing), so a bright control beside a glowing picture stays its own colour and lights nothing. With
nothing marked it covers the whole frame, as before, and the pass is the same pixel for pixel. The region is set
each frame from where the marked elements are (`Post::bloom_region`); the finishing pass runs on the kept frame
whenever it is shown, so a scroll's or a stage's moved frame takes it like any other. `tests/bloom.rs`.

**Dithering** (`DithrKind`): `bayer2`, `bayer4`, `bayer8` (recursive Bayer matrices); `blue_noise`: not a real
blue-noise mask (that would need a texture or void-and-cluster at start-up) but the R2 low-discrepancy sequence
`fract(0.7549 x + 0.5698 y)`, as even and textureless; `atkinson_like`: real error diffusion runs pixel after
pixel and doesn't fit one fragment pass, so this is an ordered stand-in: 4×4 Bayer thresholds squeezed into
1/8–7/8, so that, as with Atkinson (which passes on only 3/4 of the error), the darkest and brightest quarters are
solid and the pattern sits in the midtones. Without a palette each channel is quantized to `levels` steps (the
threshold minus 0.5, times `strength`, added in); with a palette it dithers by about one palette step first and
then takes the nearest colour (squared distance weighted 0.30, 0.59, 0.11).

**Palettes** (`PaletteKind`, data, colours defined by this project): `one_bit` (near black and paper),
`moss4` (four moss greens), `amber` (four steps of amber), `cyan_magenta` (ink, cyan, magenta, white), `cga4`
(black, green, red, yellow), `originals8` (eight named colours: ink, paper, brick, ochre, moss, slate, plum, sky)
and `user` (`colors`, at most 16).

**Implementation**: the renderer already copies the kept frame whole to the target each frame (§1.19). With
effects on, that step becomes `fs_post` (the same full-screen triangle reading the kept frame), so there is no
extra pass; with everything off it is the original copy and costs nothing. The kept frame, the dirty boxes and
the old picture a commit takes are all from before finishing, so partial redraw and transitions are unaffected;
bloom (two rings of twelve samples) and curvature are full-screen, but this pass was a full-screen copy anyway.
The legibility rules are unchanged: finishing loosens nothing, and still text stays whole (dither and palette
change the colours at glyph edges).

**Black and white only** (`PostSpec::binary`): the palette becomes `one_bit` and dithering is off; without
dithering the shader takes the nearest palette colour, which is a threshold at mid luminance, so no new dither
kind is needed. Line widths are rounded to device pixels at the same time (`scribblr` and trail lines, at least
1 px). **Invert** (`PostSpec::invert`): the output is `1 − colour`, in sRGB, after everything (after
quantization), so the ink on the page, the annotation layer and pictures in transition all invert together;
with `binary` there are still only two colours. Both are switches, off by default.

**Where they are used**: Neo-Dotmatrix uses the amber palette with 4×4 dithering (strength 0.8) and a little
bloom; Glitch uses chromatic offset 1.5, scanlines 0.22, grain, vignette and curvature 0.03; Lo-Fi Print uses
grain and vignette; the Loose Thread style page has black-and-white-only and invert switches (§4.4).

**Acceptance**: unit tests in `post.rs` (Bayer thresholds are a permutation, 2×2 is the classic 0 2 / 3 1,
Atkinson-style thresholds lie in 1/8–7/8, palettes, JSON round trips, all off by default); `preset.rs` (recipes
round-trip `post.dithr.levels`; old presets without the field read as all off); `check` item 17 (formerly item
19): at strength 0 byte-identical to off; each effect turned on alone changes the picture, and the same frame
drawn twice is byte-identical; after two-level dithering every channel is only 0 or 255; for every fixed palette
and one user palette under four dithers, every pixel is a palette colour (±1/255; control: most pixels are not
when off); with finishing on, partial and full redraw are byte-identical. Samples `samples <dir> looks`.
