# 1.8.4 手势物理（`src/gesture.rs`）

Dragged things don't use animations; they use a few numbers in the state (no GPU):

- `Spring { stiffness 380, damping 36, mass 1, rest_distance 0.5, rest_speed 8 }`: a damped oscillator
  stepped by its analytic solution (underdamped, critically damped and overdamped), so ten small steps land
  where one large step does; damping ratio 0.92. `Animated` is a number it drives: `hold` follows the
  finger, `to(target, v, now)` hands the release velocity to the spring, and `step(now)` moves only while
  `moving()`, landing exactly on the target at rest and wanting no more frames (the application asks for
  frames with `Ui::on_wake` while `moving()`).
- `RubberBand { coeff 0.55 }`: d px past the limit shows as `(1 − 1/(d·coeff/dim + 1))·dim`; it starts at
  0.55 of the finger's movement and never reaches `dim`.
- `project(v, decel)`: a throw that decays by `decel` (0.998) per millisecond stops `v/1000·decel/(1−decel)`
  further on. `Velocity`: the velocity from the last 0.1 s of samples (as for finger flings, §1.8.2).
- **Bottom sheet** (`widgets::bottom_sheet`, `Sheet { detents [0.45, 0.9], dismiss_velocity 900,
  projection_decel 0.998, backdrop 0.5, spring, rubber, height }`): the height is state and the sheet moves
  with `El::shift` (dragging doesn't lay out); on release it stops at the detent nearest the projected end,
  and dismisses when the end is below half the lowest detent, or below the lowest detent moving down faster
  than `dismiss_velocity`; above the highest detent it rubber-bands; the backdrop is the page background at
  opacity `backdrop × progress to the lowest detent`; Esc or a press on the backdrop dismisses it; its role
  is dialog, and when presented focus goes to the handle (`Cmd::Focus`). It goes after the page in
  `el::layers` (each child starts at the same box's top-left and the box does not grow with its children;
  taffy 0.15's absolute positioning resolves insets against the root rather than the parent, so `absolute`
  is not used).
- **Hand-off** (runtime, in addition to §1.8.2): a finger pressing a vertical scroll area inside a dragged
  element (one with `on_drag`) scrolls first; when the scroll area is at the top (offset 0, no overscroll)
  and the finger pulls down, from that step the gesture passes to the element: its drag starts there (Start
  and this step's Move), and the finger belongs to it afterwards.
- **Floating box** (`widgets::floating`, `Floating { size, area, corner_inset 16, projection_decel 0.998,
  spring, rubber }`): drag it anywhere, rubber-banding past the area's edges; on release the corner nearest
  the projected end (not the release position) is chosen, and x and y springs carry the release velocity
  there; the arrow keys change corners.
- Acceptance: unit tests in `gesture.rs`, `bottom_sheet.rs` and `floating.rs` (analytic steps compose, rest,
  the band is bounded, projection); `tests/gesture.rs` simulates through the runtime at 60 Hz: a slow drag
  returns to its detent, a fast throw dismisses and hands its velocity to the spring, pulling up gets harder
  and never passes the limit, a list pulled down at its top becomes a sheet drag, the corner is chosen by
  projection, and no frames are wanted at rest. Samples: `craft <dir> sheet floating`.
