# 1.8.2 指针、触摸和滚轮

**One pointer model** (`Ui::pointer_down / pointer_moved / pointer_up / pointer_cancel(id, PointerKind, x,
y)`, where `PointerKind` is Mouse, Touch or Pen). Mouse and pen behave as before: a press fires on press, a
drag is captured until release, and there is hover. A finger:

- **Has no hover**: finger movement sets no hover, gives no burst and the lens doesn't follow (`touching`);
  no hover remains after the finger lifts.
- **Press**: when what is under the finger is something dragged (a slider, a divider, a compare line) or a
  text field, or when nothing there can scroll, it presses at once the mouse way (captured, fires on press).
  Otherwise the press is only shown first: in the same frame a front is sent, a burst given and the control
  held loose (as with a mouse held down), but no message is sent.
- **Swipe**: moving more than `slop` (8 px) from the press is a scroll: the nearest scrolling ancestor of the
  press whose direction matches the finger's main direction is taken (or the nearest scrolling ancestor);
  the held control is let go and condenses again; then the content follows the finger (`drag_scroll`).
  Lifting before that is a tap, and only then are press and release sent.
- **Fling**: on lift, the velocity from the last 0.1 s of samples (the host's event times, `Ui::event_time`;
  on the web the event's `timeStamp`) is clamped to `max_velocity` (5000 px/s), and above 50 px/s scrolling
  continues: `v(t) = v₀ e^(−friction·t)` (`friction` 3.5 /s), each `tick` moving by the exact integral
  `v (1 − e^(−f dt)) / f` (`fling_step`), stopping below 20 px/s. A new press stops it. A virtual list keeps
  building new rows during a fling (`scrolled`).
- **At the end**: content never goes past either end. As the finger keeps pulling, the overscroll `over`
  grows by `rubber_band` (each time `rubber` (0.55) × the finger's movement, less and less, at most
  `max_over` 140 px); a fling that reaches the end turns its remaining velocity into a little overscroll.
  Overscroll does not move the content out (there is no bouncing box); **what is at that edge loosens**:
  every element in the scrolling node has a looseness of at least
  `0.85 × √(|over| / max_over) × (1 − distance from that edge / 160)`, so under `dottr` the text at the bottom
  breaks into dots and under `plain` it fades. Pushing back first pays off the overscroll; on release it
  returns to 0 exponentially by `spring` (9 /s) and is removed below 0.5 px.
- **A second finger** (a pinch): whatever the first finger was doing is abandoned (a held control is let
  go, a drag ends as a release), and nothing is sent until all fingers lift.
- `pointer_cancel`: the system took the pointer; nothing is sent.
- `needs_frame` is true while a fling or overscroll is in progress (§1.11). The parameters are numbers in
  `Ui::scrolling` (`Scrolling`).

**Wheel**: pixel deltas are used as they are; line deltas (web `deltaMode = 1`, native `LineDelta`) are 40 px
a line, page deltas the viewport height. A trackpad's own momentum is a stream of wheel events, so no
momentum is added. On the web a wheel with Ctrl is a trackpad pinch: no scrolling and no page zoom
(`preventDefault`).

**Wheel gestures**: a step less than `WHEEL_LATCH` = 0.5 s after the one before, with the pointer where it was,
belongs to the same gesture and goes to the node that took the one before (a scroll node, or an element with
`El::on_wheel`), as browsers keep a wheel transaction with its first target; a scroll at its end holds the
gesture. So a page scrolled by a trackpad's momentum keeps scrolling when a 3D view comes under the pointer,
instead of the view taking the rest of the momentum as dolly steps, each a message and a layout of the page
(on the M1 /components/ index, an orbit preview did that from the first change of direction: 11 ms of CPU a
frame and a page that stopped). A new gesture (after a pause or a pointer move) goes to what is under the
pointer. An `on_wheel` handler that returns `None` leaves the step to the scroll nodes.

**Minimum hit size**: when press, hover and the cursor shape look for a target (`hit_target`), the topmost
element under the pointer (or its ancestor) is it if it can be operated; otherwise it is the control, among
those narrower or shorter than `MIN_HIT = 28` px whose box grown about its center to 28 × 28 contains the
point and whose clip contains the point, with the nearest center. Only hit testing grows; drawing does not.

**Cursor shape** (`Ui::cursor()`, `el::Cursor`): text while dragging a text selection; "grabbing" during a
drag (an element that sets its own shape uses it; a divider is always `col-resize`); otherwise, from the
element under the pointer upward: its own `El::cursor`, text for a text field, "grab" for something
draggable, a pointing hand for something pressable, text for selectable text. The web sets
`canvas.style.cursor`, native `Window::set_cursor`. The divider and compare lines in the printing scene are
`col-resize`.

**Web**: `pointerdown / move / up / cancel` with `pointerType`; a press captures the pointer
(`setPointerCapture`) and calls `preventDefault` (the page decides focus); for the mouse, only the primary
button presses; `pointermove` uses `getCoalescedEvents()`; the canvas has `touch-action: none`,
`user-select: none`, `-webkit-touch-callout: none` and a transparent tap highlight, and `contextmenu` is
prevented, so double-tap zoom, the long-press menu and the browser's own panning never happen. **Native**:
winit's `Touch` (started, moved, ended, cancelled) goes into the same model; `PinchGesture` and the like are
not handled.

**Acceptance**: `tests/pointer.rs`: a fling travels `v₀ / friction` and stops after `ln(v₀ / 20) / friction`,
and two steps compose into one; overscroll gets harder to pull and has a limit; one finger swiping 300 px on
ten thousand rows moves the content exactly 300 with no row selected, then flings, is still moving after
0.3 s and finally stops (≈ 1800 / 3.5 ≈ 514 px, ±80), after which `needs_frame` is false and the rows in
the tree stay bounded; a tap: at the moment of the press the field has one more front, that row is held
loose, no message yet, and it is selected on lift; fingers don't hover, the mouse does; pulling down at the
top: the offset stays 0, overscroll < −40, the top rows loosen (> 0.3) and distant rows don't, no fling
after release, back to 0 within 1.5 s and no more frames wanted; two fingers neither press nor scroll; a
finger on a slider captures at once and still changes its value dragged far outside it; a 10 px control
can be pressed 8 px outside its edge but not more than 14 px from its center, is still drawn at 10 px, and
shows the pointing hand when hovered in the grown area; cursor shapes: text for the input field and source,
the hand for the side rail, `col-resize` for the divider (during a drag too), "grab" for the tree cut-out,
"grabbing" during a drag. On the web (SwiftShader, CDP `Input.dispatchTouchEvent` with time stamps 16 ms
apart): one swipe scrolls ten thousand rows by 290 px, the fling starts on lift and stops at 716 px, a tap
doesn't scroll, and the console has no errors.

**Events a frame brings together** (`Ui::coalesce`, on in both hosts). A host hands over every pointer move and
wheel event as it comes, several in one frame, and ticks before each. With coalescing on, a pointer move, a wheel
and a tick change the state at once (every message is delivered, in order) but lay out and paint once, before the
frame is drawn (`Ui::settle`, which `Renderer::draw` calls first): eight moves in a frame are one layout, not
eight. A move is hit-tested against the page as last laid out. Every other input (a press, a release, a key, text,
the clipboard, assistive technology, a command) lays out what was left waiting first, so it acts on the page the
state describes. Off (the default, for tests and tools that drive a `Ui` by hand), each lays out at once. Each
pointer event carries when it happened (`Point::t`, `Drag::t`: the host's event time on the runtime's clock), so a
speed is measured between the moves themselves, not between frames (`experiment::Pointer::hear`).
