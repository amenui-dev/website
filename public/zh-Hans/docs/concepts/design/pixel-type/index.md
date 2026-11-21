# 1.6.3 像素字

Both routes are needed:

- **Faces that are pixel fonts.** The face is a value in the theme. Two open-source pixel fonts are bundled:
  Fusion Pixel for CJK (SIL OFL; upstream has 8, 10 and 12 px faces, and only the 12 px proportional
  Simplified Chinese one is bundled) and Departure Mono for Latin (SIL OFL, 11 px). A pixel font must be
  rendered at an integer multiple of its face size with anti-aliasing off when rasterizing (coverage
  thresholded at 0.5), or it blurs. Implementation: `Text` can register several faces and each text
  element picks one; a face can be marked as a pixel font with its face size, its font size snaps to the
  nearest integer multiple in device pixels (at least 1×), layout and drawing use that size, and glyphs
  land on whole device pixels with no subpixel offset. The multiple is never below the one at one device
  pixel per logical pixel times the ratio, rounded down: `max(round(size · dpr / native), floor(m1 · dpr))`
  with `m1 = max(1, round(size / native))`, so 7 px of a 12 px face is 12 device px at 1 and 24 at 2, not
  12 (it is never smaller at a higher ratio; `tests/own_size.rs`).
- **Any face can be printed as pixels.** The material samples the glyph through a coarse grid and fills each
  cell whose coverage is over half. The pitch is the size of a "pixel" and can be set as a number. Type from
  this route is less tidy than a real pixel font and suits transitions and large headings. Implementation:
  a text element with `.print(pitch)`; the glyph instance carries the pitch into `near.a`, and the material
  averages coverage per cell there and fills it when over half.

Acceptance: in screenshots of pixel fonts at 1× and 2×, stroke edges hold only two colours (ink and ground),
with no grey fringe.
