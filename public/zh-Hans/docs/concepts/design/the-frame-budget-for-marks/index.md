# 1.27 记号的帧预算

Many small marks (morph cells, shed marks, flow field marks, swarms in experiments, the marks of a drawing) used to
have limits of their own (4000, `count_max`…) and stuttered when there were many. Now they all draw from one frame
budget (`src/budget.rs`): `budget::grant(want)` gives how many this frame may have (`want × scale`, and no more
than what remains of `MAX_MARKS` = 24000 this frame); `budget::cap()` is the most one drawing may draw (beyond it
marks are dropped evenly by stride); the runtime starts a frame at each `tick` (`begin_frame`) and, after each
frame, gives its time to `feed`: 8 consecutive frames over `TARGET_MS` (14 ms) lower the scale by a fifth (to no
less than 0.05), and 60 consecutive frames under half raise it by a tenth (up to 1). Only what can be seen moves:
flow field marks move only while their box is in view, and experiments only when `on_visible` says they are seen.
There is one way to draw: they are all shape instances in the same instance buffer (morph cells have their own
pass and scale their cell count). The readout (F2) shows how many this frame used, how many were asked for, the
limit and the scale. Acceptance: unit tests in `budget.rs`; `tests/budget.rs` (asking for a million marks keeps
the instance count and the frame time bounded).
