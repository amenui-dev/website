# 1.8.3 How scrolling looks

Text "aliasing and shimmering" while scrolling had five sources (the `scroll` tool's sample
`scroll-before.png`: an article, ten thousand rows and the button page at 120 / 800 / 3000 px/s in six
materials). Rows scrolling into view were treated as new instances and condensed from loose (in fast
scrolling the whole screen was condensing: grey in `plain`, dots in `dottr`, three-colour halftone in
`halftonr`, characters in `charr`); this was the largest. The scroll offset was fractional, so a shape's two
edges rounded separately and its height jumped between n and n + 1, and glyph baselines rounded separately,
putting rows a pixel out of line. The materials' lattices and noise (the dot ground, the paper grain, the
halftone screen) were fixed to the screen, and text slid over them. Shapes' and glyphs' local and atlas
coordinates in the G-buffer stage were interpolated, so the same box moved by a pixel rounded differently
and anti-aliased edges differed by 1/255. The theme's background gradient was computed from screen y even
without a gradient, again 1/255 off.

Now (`style::Scroll`, in `Look::scroll` and a preset's `interaction.scroll`; the "Scrolling" section of the
list page has number controls):

- **At rest and slow** (below `calm_speed`, 600 px/s by default): scrolling is a picture moved by whole
  device pixels. When placing, the runtime rounds the offset to device pixels (the stored offset stays
  fractional, so the fling integral loses nothing); instances that appear because of scrolling (`paint` in
  `Ui::scrolled`) don't condense, since they were there all along, also when coalesced input leaves that paint for
  the frame (`Ui::settle` paints as a scroll does when only scrolls asked for it, and by key when a scroll and something else
  did: what was in the layout before does not condense, a new key does; it had painted them as new, and a
  fast scroll on the web condensed most of the screen, about 80% of /components/ in stage 1 every frame); local and atlas coordinates in the
  G-buffer stage are computed in the fragment from the pixel position minus the flat box origin (`exact_i`),
  not interpolated; a background without a gradient is not computed from y. Inside a scrolling node, the
  material's cells, hashes and noise use content coordinates (`anc`: the sum of the offsets of the scrolling
  nodes this point is in, in whole device pixels), so lattices, paper grain and screens move with the
  content. `check` item 18: in every material, scrolling the button page slowly by one device pixel per
  frame, the scroll area (not counting 24 px from its edges) of each frame equals the previous frame moved
  up one pixel, byte for byte; control: without movement the two frames differ. In the `scroll` tool's
  report the "difference after moving" at slow speed is 0 for all three pages in six materials.
- **Fast**: each scrolling node's velocity is found at every `tick` from the change in its offset (an
  exponential average with `SPEED_TAU` = 60 ms, so one wheel notch isn't taken for a fling); the trace's
  strength is `smoothstep(calm_speed, full_speed, |v|)` (600 to 2600 by default), settling to 0 by smoothstep
  within `settle_ms` (140) after it stops (`Ui::scroll_motion`). The runtime puts up to eight visible
  scrolling nodes (viewport, content offset, strength, direction of content movement) into the field
  (`Field::scrolls` / `scrollo`), and the shader takes the strongest at this point in `scroll_here`. Traces
  are drawn only outside the ink, behind the direction of motion, a whole row at a time; the text itself
  stays as it is, so body text never passes the legibility cap:
  - `plain`: a soft trail (within half of `trail`, at most 0.14 opacity);
  - `dottr`: content lands on the lattice row by row (the content offset is rounded to the pitch before
    drawing, `latch`), leaving a few rows of phosphor dots behind (dot size by ink coverage, tinted toward
    the accent);
  - `halftonr`: the second colour (the accent) offset behind on a fine screen, by `misreg × strength`;
    `smudgr`: the accent offset behind with multiply, like misregistered two-colour printing;
  - `charr`: lands on the character cells (`latch` with the cell pitch), leaving a few faint characters
    behind;
  - `glitchr`: an occasional whole band offset: band height `band`, at most once every `slip_ms`, one band
    only, offset by `slip × strength`, with a chance equal to the strength;
  - `carvr`: a carved echo: a few broken ink-and-ground-only lines behind the direction of motion;
- The numbers in presets: Normcore has no trail, Cyberminimalism a very short one, Dotmatrix and Linocut
  longer, Lo-Fi Print offsets by 4 px, and Glitch slips bands further and more often.
- Drawing only what changed (§1.19): while a scrolling node's offset changes or its trace shows, its
  viewport (grown by the material's read range, trail and band slip) goes into the shading stage's boxes.

At speed, "landing on the lattice row by row" is mostly invisible at 60 Hz (3000 px/s moves 50 px a frame at
pitch 5); what shows is the phosphor dots. That is as specified, with nothing added. Wheel notches are not
smoothed (the browser's `wheel` events jump one notch at a time), so a 100 px notch is one jump at 60 Hz;
trackpads and flings are continuous.
