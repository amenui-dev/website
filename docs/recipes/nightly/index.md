# nightly


W-04 in Works, at night (23:10). Flat dark water under a starry sky. Across the sky runs one line of type on a slow curve: the build's log, `nightly  ok  ok  ok  build 0412  ok  passing …`. An aurora hangs from the line, 540 rays rising from it, tall in the folds and short in the gaps. On the horizon a small orange light carries a callout, `KP 5   STEP 3/7`. The curve drifts and the rays' lengths travel slowly along it. Moving the pointer near a fold leans it toward the pointer. A press runs the next step of the build: a pulse travels both ways along the line from the nearest point, and the rays it reaches brighten and lengthen for half a second. After step 7 the whole curtain flares, the callout says `PASSING`, the build number rises by one, and a new curtain comes in. Under the horizon the curtain's broken reflection shifts once with every press.

## How it is made

| Part | amenui |
|---|---|
| The sky, stars and water | one picture computed in code (the sky from `#070b16` to `#12233a` at the horizon, 80 % down, a hairline, the water, 130 stars of 1 or 2 px, a faint grain), shown with `El::picture` |
| The rays | one `Drawing` of `Drawing::ramp` strokes, 1 px wide, all in one draw. Each ramps from `#baffd8` at its root through `#2fae8c` at 30 % of its length to transparent at its tip, mixed in OKLab over the picture under it. The tallest turn toward the accent over their top third, as a second ramp from two thirds up |
| The rays' lengths | 40 to 360 px, from a noise that travels along the line at 12 px a second; the build number is the seed, so each build has its own curtain |
| The line of type | one `text` laid out once, 150 cells of Departure Mono (100 glyphs, spaces not counted). `El::pose` moves each glyph to its place along the curve and turns it with the curve, 7 px under the rays' roots, and `recompose(false)` keeps the layout while the curve drifts |
| The fold | the tallest ray within 150 px of the pointer, leaning up to 18 px toward it on a spring that settles in about 0.4 s, with its neighbours falling off over 60 px |
| The callout and the reflection | two labels and a leader to the light; 20 rows of short dashes under the horizon, where the resting rays stand tallest |

On a phone the line runs from 6 % to 94 % of the width with as many cells as fit at the desktop's pitch, cut after a whole word, and the rays keep the desktop's spacing of 2.35 px: 384 rays at 1024 wide and 146 at 390.

## What it costs

The drift is paced at 30 frames a second (`Ui::ambient_fps`) while the piece is awake; a pulse, a flare and the fold's spring ask for frames of their own only while they run. After 60 seconds without input the piece rests. With reduced motion the curtain is still: a press counts the step and shifts the reflection, with no pulse and no flare, and the fold takes its lean at once.

Measured at 1440 × 900 with motion paced at 30 frames a second. A frame holds 1,228 instances.

| A frame | Mean | 95th percentile | Worst |
|---|---|---|---|
| At rest, natively (CPU) | 0.08 ms | 0.08 ms | 0.88 ms |
| A pulse, natively (CPU) | 0.78 ms | 0.91 ms | 1.02 ms |
| At rest, the first minute, headless Chrome at 2× | 1.96 ms | 2.60 ms | 14.2 ms |
| A pulse, headless Chrome at 2× | 1.57 ms | 2.60 ms | 2.70 ms |

The browser figures use WebGPU on SwiftShader, a software renderer (not a GPU), so a real machine does better. The first frame, which lays out the page, takes 62 ms natively and 92.8 ms in the browser; the browser's first-minute figures leave it out.

The palette is temporary: `#1f5f86` at the rays' tips comes back when a drawing can have colours of its own.

The code is `src/gallery/nightly.rs` in the website repository. Text on a curve is described in [Flow: text on a path](lab-flow.md).
