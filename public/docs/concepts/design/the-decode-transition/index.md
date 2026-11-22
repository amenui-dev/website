# 1.28 The decode transition

`El::decode(Decode { tiers [32, 16, 8, 4], stagger_ms 90, duration_ms 720, seed 7, hold })` (`src/decode.rs` is the
part that needs no GPU): when an element appears it is covered by blocks that resolve from coarse to fine over a
few tiers, finally revealing the real content; when the element leaves the view, the runtime plays the same thing
backwards with its last frame's instances.

- **It uses the loosening machinery**: blocks are not a grid in a shader; the element's true outline is sampled
  the way morphs sample (`morph::outline_cells`, how particle cells are cut) for coverage on the finest tier's
  grid, and inked cells are gathered into each tier's coarser cells, so every tier covers all of the ink and none
  of the box's empty space. Coverage follows the outline, not colour (`morph::cover_at`): a panel filled with the
  ground colour is an outline too, so a dialog's inside is covered even at the finest tier; only the cells a morph
  flies use `morph::ink_at` (visible ink, since ground-coloured cells would fly invisibly). Blocks are ordinary
  shape instances (true outlines, drawn by every material), and the content is the element's own instances,
  shown at the right moment.
- **Timing**: with `T` tiers, tier i's blocks each appear at a hashed moment in `[i·S, i·S + W]` (`S` the stagger,
  `W = (D − (T−1)·S) / 2`); a block disappears when its last child appears; at `C = (T−1)·S + W` every tier has
  appeared, the content is drawn beneath the blocks, and the finest blocks each disappear within `[C, C + W]`. So
  at every moment in between each inked cell is under some block, and the content is not drawn before `C`; at
  progress 0 and 1 there are no blocks at all (nothing half covered at rest). Colours are the current look's first
  two accents, chosen by hash.
- `hold: Some(u)` stops it at progress u (the documentation page's timeline). Played backwards, the content drawn
  before `C` is the instances from before it disappeared, with the element's own `Decode` spec and inside the clip it
  was last drawn in (a decoding element taken away alone, its page staying, decodes out within its scroll area or
  dialog, not over the page).
- The documentation page (`#decode`): a card held at the timeline's progress; the same plan pulled apart in depth
  by tier (one plane per tier seen at an angle, squashed to 0.42 height and sheared 0.55, each tier 74 px further
  down and right, the content as the bottom layer; each plane shows that tier's blocks lit at this moment, with a
  leader on the right to a label beside it: cell size, when it appears and how many are lit now); a draggable
  timeline (each tier's span, the content's span, the current position); every number a number control; and
  "Open dialog" opens a dialog that appears by decoding and plays backwards when closed.

Acceptance: unit tests in `decode.rs` (no blocks at 0 and 1; from the first tier being fully present until the
content appears, every finest cell is covered; mean cell size shrinks; deterministic); `tests/decode.rs` (a dialog
opening shows only blocks first, the content appears after `C`, and there are no blocks at the end; closing plays
backwards and leaves nothing at the end).

A decode held at a step (`DecodeSpec::hold`) draws that step once and then the page rests: it asks for no more
frames until something changes (`tests/frames.rs`).
