# 1.24 卡片堆

`widgets::card_stack` (`Stack { count 5, angle0 42, falloff 0.7, delay_ms 28, spring (k 260, c 24), look, card,
paints, outline, outlines }`). By default every card is raised (`paints [RAISED]`), outlined 1 px (`outline`), the
top card in the ink and the ones under it in the hairline grey (`outlines [INK, HAIRLINE]`: card n takes entry n,
the last one repeating; empty, the ink); categories are not told apart by the accents.
Hovering or pressing a card (a 3D element's `Pick`) turns the n-th card beneath it (n from 0) to
`angle0 × falloff^n` degrees, starting after `delay_ms × n` ms, each with its own spring with velocity hand-off
(§1.8.4), about its own bottom-left corner; the cards above stay. A press keeps it fanned, Esc closes it, and the
arrow keys change card. The cards are one `scene3d` (a cube each), so their look follows the material
(`StackLook::of`): under `plain` they are flat cards seen face on (an orthographic camera); under `smudgr` and the
like they have thickness, seen from above at an angle, outlined, in two tones, with visible edges; under `dottr`,
`charr` and `halftonr` they are flat cards handed to the mark material and held at looseness 1, so they are marks
even at rest. The formula caption under the page prints this moment's `angle(n)` and `delay(n)`. Acceptance: unit
tests in `card_stack.rs` (the formula, the order of delays, holding, Esc, the look by material); `tests/stack.rs`
(the pointer picks the top card, at 40 ms cards 1 and 2 have started and card 3 hasn't, after stopping each card
equals the formula, no frames at rest, closing on leave). Samples `craft <dir> stack`: one row per look.
