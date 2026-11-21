# 1.33 Plates

A plate keeps the ink drawn on it. `El::plate(Plate { fade_s })` on an element: what its subtree draws is not wiped
the next frame; new ink goes over the old, and a pixel drawn again is new again. It is a picture, not content: it is
not in the accessibility mirror, its text cannot be selected, and a new window size empties it. `Cmd::ClearPlate {
key }` empties it in one frame, hard (a cut for the beat); `fade_s` (0–60 s in steps of 0.5; 0 keeps the ink for
good) is the soft way.

- The plate draws nothing of its own: no edge, no ground, no dimming. An empty plate is byte for byte the page without
  it, and costs the same: a plate that has held no ink since it was made or cleared is not laid on (no extra pass).
- What is on a plate is ink, not something to read or press. Text to read and controls go beside the plate or over
  it, as its siblings, not its children; one inside a plate's subtree is reported (`OnPlate` in `Ui::problems`, P1 in
  the audit). A label meant to stay readable must not rely on ink that fades: with `fade_s` set, what a plate holds
  goes, so a label drawn onto it (or strokes that make a label) is gone too; it belongs over the plate, as a sibling.
- Cost, as regression lines on lavapipe at 1440 × 900 (a whole-window plate, the fifth set's measurement: keeping ink
  12.3 → 15.0 ms, ageing 49 → 57.5 ms): a frame keeping ink at most 125 % of the same page without a plate, an ageing
  frame at most 120 %. On a real GPU a whole-window plate may add at most 0.6 ms a frame; that measurement is still to
  be made before a release, and if it is over, plates are for music videos and the lab only. It reaches as far as its clip box and ink is cut at the box's edge, with no fade there (that would draw the box).
- The ink on a plate is ink: the near image is max-blended, so strokes laid one over another keep a true outline, and
  every material draws it as usual. There is no material code for plates.
- At most 4 plates in a window, their images together at most twice the window's area; more are drawn as ordinary
  elements (keeping nothing) and `Ui::problems` reports `PlateOver`. A slot newly taken starts empty.

The renderer keeps four images for each plate: ink, near, and when each pixel was drawn, in 1/120 s ticks as 24 bits,
twice (where the ink was drawn, and where its nearness reached). A new stroke's spread keeps its own outline alive
without making the old ink under it new, so the line between old and new ink is the new stroke's outline, to within a
pixel. In the G-buffer stage the plate's instances of this frame are drawn into its ink and near images and its ages, and the
kept images are laid on in their place (premultiplied over the ink, max over the near image). The plate clock stops
while a `Hold` lasts and never runs back: `Rewind` does not take ink back. A transition's mask acts on the plate's ink
as on anything else, and the plate of a page that went is emptied when the page goes.

**Ageing** (`fade_s` above 0): a pixel is as drawn until four fifths of `fade_s` have passed since it was last drawn
(`a` = 0.8), then its ink's coverage is multiplied by `1 − smoothstep(0.8, 1, a)`, so it fades as plain's condensing
fades in, with no pattern of its own, and is gone at `fade_s`. The material draws what is left as it draws any ink with
that coverage. The near image is a distance, which a fade would move, so it stays whole until `a` is 1. While
only ageing moves, the runtime asks for at most 30 frames a second; when the last stroke has gone the page is the
clean page and asks for none. With less motion it is the same. A plate with `fade_s` 0 keeps its ink without asking
for frames; it belongs in music videos, the lab and pages that say they are a drawing board, not on ordinary pages.

Frames with plates redraw only what changed, as any page does (§1.19); a frame in which ink is ageing redraws every
plate whole. Text to read on a plate's ink holds what is under it by §1.15's rule, as on any other ink.

Acceptance: `tests/plate.rs` (a stroke stays when the view no longer has it; the cleared frame is as never drawn; an
empty plate is as no plate in nine materials and both themes; with `fade_s` 4, `a` = 0.25 as just drawn, the ink
unchanged before 0.5, never growing, empty at 1, at most 30 frames a second and none after; `fade_s` 2 empty 2 s after
the last stroke with no frames after; ink on a plate loosening in dottr; clearing and inverting in one frame under 5 %
of pixels changing; at `a` 0.9 in plain every pixel inside a stroke within 2/255 of the others and halfway between
ink and paper to within 3 %, the ink never growing from 0.8 to 1 in steps of 0.02 and none just past 1; an old stroke at
0.9 with at most 55 % of the ink of a new one at 0.5, which is 99 % as just drawn; old ink 2 px from a new stroke's
outline as old as the rest of it; a `Hold(2)` byte for byte; five plates keeping four; a new window size emptying them; the same
timeline three times the same 300 frames; halfway through a crack the ink only in the pieces not yet turned and gone
at the end; nothing on a plate in the accessibility mirror, and text to read in one reported; an empty plate not laid
on, one with ink laid on once).
