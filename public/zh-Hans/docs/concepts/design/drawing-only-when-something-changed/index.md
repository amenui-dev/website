# 1.11 只在有变化时绘制

Drawing a frame is not something done on every display refresh. The runtime answers "should a frame be
drawn now", and the host (the web page, the native window) draws only then, sleeping the rest of the time
until input or a timed wake.

**Interface** (`src/ui.rs`, no GPU): `Ui::needs_frame(now) -> bool` (after `tick(now)`),
`Ui::next_wake(now) -> Option<f32>` (`now` means draw right away; a later time is a timed wake; `None`
means only input can change anything), `Ui::frame_drawn()` (called by the renderer after every frame, already
done in `Renderer::draw`) and `Ui::invalidate()` (the host lost the picture, for example when the native
window read back a frame for saving without presenting it).

`needs_frame(now)` is true if and only if any of these holds:

| Condition | How it is decided |
|---|---|
| Input or a scene change since the last frame | `dirty`: set by every input entry point (pointer leave, press, release, wheel, key down, key up, resize) and every `paint`, cleared by `frame_drawn`. A pointer move sets it only when something shows where the pointer is (a picture or 3D view under a lens that is on, `Fx::lens` 0.5 or more; a morph's particles, the annotation layer on), when it changes the hover or the cursor's shape, or during a touch, a drag, a selection or a field's pointer capture; a handler it reaches asks for its own frame |
| A front is still alive | Some front's strength > 0 and `now − start < Fx::wave_life` (in the shader a front's strength reaches 0 at `wave_life`) |
| A commit's front is still on screen | `(now − t) × wave_speed × scale < diagonal` |
| A brief `burst` (hover, press) | `Field::burst_at(now) > 0` |
| An instance is condensing | `now < latest birth time + Fx::enter` (`paint` records the latest birth time among the instances drawn) |
| A morph, a flow, an old page dissolving | `run`, `flows` or `ghosts` non-empty, or the one more pass due after the last animation ended (`animating`) |
| Corner marks sliding or fading | Their position or opacity changed in the last `tick`; within 0.1 px of the target counts as arrived, and opacity above 0.995 counts as 1 and below 0.02 as 0, so slides end |
| Annotations drawing in | `now < latest appearance time + draw_ms` |
| Something loose under a time-varying material | `Material::time_varying()` (glitchr) and some drawn instance has static looseness (by its own override material or the global one) |
| A timed wake is due | The caret's next blink phase (every `BLINK = 0.53` s from the caret's last move), or a time the state asks for through `Ui::on_wake(fn(&S, now) -> Option<f32>)` |

Timing comes from the state: `Ui::on_look(fn(&S) -> Look)` lets the runtime read `wave_life`,
`wave_speed`, `enter` and the material (`Timing`), once per `tick` and before each layout. The website's
`on_wake`: the departure board page wants the next whole second; the opening scene's machine wants every
frame while it runs and the sound plays; the morph scene in a loop wants the moment its current shape has
been held long enough; otherwise `None`.

**The material clock.** Glitchr's row slips on a beat of `hold` ms and the drifting dots on dottr's ground used
to read real time, so a still page kept changing. Now they read the material clock in `fa.y`,
`Ui::material_clock()`, which advances at a `tick` only if something was moving the moment before
(`moving_at`). So a still page is an unchanging picture; when a front passes, dottr's ground drifts as before
and then stops where it is. Real time is still used for births, fronts and bursts, which end by themselves.

**Web** (the website's `web/page.html`): `frame(t)` first `tick`s, draws only if needed, and returns when the
next frame is due: 0 to continue with `requestAnimationFrame`, a positive number of seconds to start again
after a `setTimeout`, −1 to stop. Every input event, `hashchange` and resize first brings the clock to the
event's time (every input entry point takes `t`, because the last `tick` may have been long ago, and a
hover's burst and a press's front must start now), then `kick()`s (requests a frame if none is running).
`frames()` returns how many frames were drawn and how many wakes drew nothing (for tests and the readout).

**Native** (the website's `src/bin/window.rs`): in `about_to_wait`, `request_redraw` when
`next_wake(now) ≤ now`, otherwise `ControlFlow::WaitUntil` (the earliest of the next wake, the two-second
frame report, a capture and closing), or `ControlFlow::Wait` if none. `RedrawRequested` doesn't draw when
`needs_frame` is false (counted as skipped). Standard error reports frames drawn and skipped every two
seconds.

**Acceptance**: `tests/frames.rs`: on every website page and every scene (the sound scene paused), after
input (the pointer enters and moves, one scroll, Tab) and the longest animation that page can start (front
lifetime, condensing, a commit's front crossing the window, annotations drawing in, flows), `needs_frame` is
false and the next wake is not "now"; #button draws 0 frames idle for 3 s, at least 25 frames within half a
front's lifetime after one press, and stops when the front ends; with the input field focused the caret
blinks by timed wakes (the next wake within one phase, 3–5 frames for four phases); the departure board
wakes every second, not every frame. Web (WebGL2, SwiftShader, real time, a 480 × 360 window so a frame is
quick enough): #button draws 0 frames idle for 3 s, 21 frames in the 1.5 s after a press on its ground (the
front lives 1.4 s), then 0 again idle for 3 s. DevTools' virtual clock is not suited to counting frames:
SwiftShader takes seconds of real time per frame and the virtual clock doesn't wait for it. Native (Xvfb and
lavapipe): idle on #button, the two-second reports show no new frames; after one press with xdotool, about 33
frames in two seconds, then none again.
