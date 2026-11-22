# 1.7 标注层

A third pass, drawn after the material, pixel exact, never loosened, in integer multiples of Departure
Mono (edge notes use the serif, §4.2). Its data comes from the real layout (`src/gaze.rs` is the part that
needs no GPU; `Ui::paint_gaze` turns its records into instances, and the renderer draws them over the final
frame with a blend-only pipeline that honours each instance's own `xf`, used only to turn edge notes by
90°).

- **Settings** (`Gaze`, in `Look`): `mode` (off / hover: the deepest element under the pointer and its
  ancestors / all / focus path: the focused element and its ancestors), `kinds` (a bit set), `density` (at
  most how many, larger areas first and those with `El::annotate` before all), `draw_ms`, `colour` and
  `only_marked` (annotate only elements with a note). `El::gaze_scope()` limits annotation to a subtree (the
  whole window without one); `El::annotate(text)` is an explicit note.
- **Candidates**: drawn, more than half visible, at least 6 × 6, not the root, and not a wrapper the same
  size as its parent. Once chosen they are numbered in tree order (01, 02…).
- **Kinds**: `FRAME` corner marks 6 px outside the element's rect (so they don't cover its own ink); `BOX`
  a 1 px box exactly on the rect's edges; `INDEX` the number; `KIND` the element type (text / image /
  control / shape / box) or its note; `SIZE` "w×h" at the element's bottom-right inside it; `LEADER` an
  elbow leader from the element's near side to a column of labels at the scope's left or right edge, the
  labels packed greedily downward at their wanted heights without overlapping and pushed back up from the
  end when they pass the bottom (`gaze::pack`); `RULER` rulers along the scope's top and left edges (10 /
  50 / 100) marking the pointer; `CROSS` a crosshair and coordinates at the pointer; `GRID` a baseline
  every 8 px and 12 columns; `EDGE` edge notes (§4.2); `OSC` a waveform and `LEVEL` a level bar (§4.3), with a
  `READY 0.62` readout beside it while the page is still preparing (§1.23); `INK` the box round the element's true
  ink, dashed, with a dashed leader from a corner of it to its drawn outline and an `INK w×h` label (§1.31). `Gaze::echo` (0 by
  default) keeps up to that many earlier INK boxes of a moving element as thin outlines (no label, no fill), one
  taken every `echo_ms` (200 ms, snapped to the beat grid, §1.32), the newest at most one interval old; the k-th (from
  1) of n is drawn at `0.6 × (1 − (k − 1) ÷ n)` of the live box's opacity (0.6, 0.4, 0.2 for three), under the live element: nothing of an echo is drawn
  inside the live INK box. An echo goes once it is n intervals old. A hold takes none and fades none (`Ui::echo_times`).
  Two labels on the same corner step aside to the right. The last three are readouts of the whole scope,
  not tied to elements: always present in "all" mode, and in hover and focus-path modes while the pointer
  is in the scope; what they drew is recorded in `Ui::readout()` (`gaze::Readout`).
- **Appearing**: the first time an element is annotated its time is recorded; its frame draws itself in
  over `draw_ms` (the box travels around its four sides, corner marks grow, leaders reach out and then
  turn), and numbers and sizes count up from 0. Everything is recomputed from the current layout every
  frame, so the numbers follow page changes, flows and window changes.

Acceptance: `cargo test` (`tests/gaze.rs`) checks the records of a known tree (rects are node rects,
numbers, sizes, kinds, notes), density and notes only, the ancestor chains for hover and focus,
non-overlapping labels, drawing in and counting up, and the edge notes, waveform and level bar (§4.2,
§4.3); `check` item 7 reads pixels back and checks that each corner mark's four corners sit exactly on the
layout rect grown by 6 px, and that the pixel beyond is not annotation colour; items 7.2 and 7.3 are in
§4.2 and §4.3.
