# 1.24 A stack of cards

`widgets::card_stack` (`Stack { count 5, angle0 42, falloff 0.7, delay_ms 28, spring (k 260, c 24), look, card,
paints, outline, outlines }`). By default every card is raised (`paints [RAISED]`), outlined 1 px (`outline`), the
top card in the ink and the ones under it in the hairline grey (`outlines [INK, HAIRLINE]`: card n takes entry n,
the last one repeating; empty, the ink); categories are not told apart by the accents. Flat, the top card's
rectangle is the scene's `Mesh3d.snap`: its edges are on whole device pixels however large the box is drawn (a
page that zooms it to fit a narrow window too), so at rest each side's outline is one whole pixel at its grey, not
two at half strength.
Hovering or pressing a card (a 3D element's `Pick`) turns the n-th card beneath it (n from 0) to
`angle0 × falloff^n` degrees, starting after `delay_ms × n` ms, each with its own spring with velocity hand-off
(§1.8.4), about its own bottom-left corner; the cards above stay. A press keeps it fanned, Esc closes it, and the
arrow keys change card. The cards are one `scene3d` (a cube each), so their look follows the material
(`StackLook::of`): under `plain` they are flat cards seen face on (an orthographic camera); under `smudgr` and the
like they have thickness, seen from above at an angle, outlined, in two tones, with visible edges; under `dottr`,
`charr` and `halftonr` they are flat cards handed to the mark material; at rest they are drawn as in `plain`, and they
loosen to 1 only while the cards fan and settle (toward 1 within `LOOSEN_S` = 120 ms while any card's spring moves,
back to 0 within `TIGHTEN_S` = 240 ms after the last one stops, then no frames; a fan held open and still is at 0). The formula caption under the page prints this moment's `angle(n)` and `delay(n)`. Acceptance: unit
tests in `card_stack.rs` (the formula, the order of delays, holding, Esc, the look by material); `tests/stack.rs`
(the pointer picks the top card, at 40 ms cards 1 and 2 have started and card 3 hasn't, after stopping each card
equals the formula, no frames at rest, closing on leave); `tests/card_stack.rs` (marked cards' pixels are plain's
at rest and from 240 ms after the springs stop; from 120 ms after the fan starts a quarter of their inside
differs, under `halftonr` 15%, whose dots at full ink meet, with some in every window of 3 × 3 cells; at rest in a
520 × 340 box the top card's outline is one pixel at the ink within 2/255 on every side, with no pixel part way to it
beside it, at 1x and 2x, dark and light). Samples `craft <dir> stack`: one row per look.
