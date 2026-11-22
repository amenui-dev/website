# 1.8 Input

Pointer position → hit test in reverse order → hover / press / focus. Tab cycles focus; Enter and Space
activate. Hover and focus are shown by four corner marks around the target (hover frames elements that can
take focus, be pressed or be dragged). The marks land on a new target in the frame the target changes
(computed on pointer movement, not at the next `tick`) and never slide between targets; the only easing is
`ui::MARKS_SETTLE` (70 ms), closing in from 3 px outside and going from half to full opacity, fading out
over the same time on leaving. A front (a press's) pushes them as it pushes the control they frame, as one piece
about its centre, so a pressed button never slips out from between its corners
(`the_focus_corners_move_with_the_control_a_press_pushes`). They are drawn over the material, exact, in the accent
(the annotation layer): drawn by a material they were its shapes, with `near`'s bands round each bar. On a part of a
compound control (a segment, a tab: a control inside another that draws a border round it) they stay inside that
border instead of sitting on it (`the_marks_on_a_segment_stay_inside_its_controls_border`). They used to chase the target each frame by `1 − e^(−16·dt)` and always
trailed the pointer. Acceptance: `hover_marks_land_in_the_pointers_frame` in `tests/pointer.rs`. The wheel
finds the nearest scrolling ancestor.

- A press fires on press (not on release; fingers may differ, §1.8.2). While held, the target's instances
  stay loose; on release they are born again and condense from loose. Hit testing uses the minimum hit
  size of §1.8.2.
- The corner marks follow the most recent input: with the pointer they frame the hovered control, or the
  focus when nothing is hovered; with the keyboard they frame the focus.
- The wheel goes to the nearest scrolling ancestor first, and to the next one out once that has reached
  its end; the steps of one gesture stay with what took the first (§1.8.2). When focus moves to a control outside a scroll area's view, the area scrolls to show it.
- A number control that loses focus while being typed in behaves as if Esc were pressed.
- **Tab order**: `El::tab_order` puts positive numbers first, lowest first, then everything at 0 (the
  default) in document order; a negative number keeps an element out of Tab while a press, a skip link or
  `Ui::focus_key` can still focus it (as HTML's `tabindex`). **Skip links**: activating an element with
  `El::skip_to(target)` (Enter, Space, a press, assistive technology's activation) moves focus to
  `target`, or the first element inside it that takes focus, and scrolls it into view.
  `El::focus_shown` draws an element and its subtree only while focus is on it or inside it; it stays in
  the accessibility mirror and takes focus and presses. `widgets::skip_link` is the two together.
  Acceptance: `tests/focus.rs`.
- **Dragging**: pressing an element with `on_drag` makes the runtime record its handler and the press
  position (capture); later moves and the release go to it wherever the pointer is. A new press ends an old
  capture that was never released. Because the handler is the one from the press, the values in it are
  those at the start of the drag: a number control dragged sideways computes "start value + offset / 6 px
  × step", so dragging back and forth accumulates no error, and less than half a step is not a drag, so a
  click only focuses.
- **Text input**: see §1.8.1.
- **Layers**: while a `modal` layer is open, only elements in it can be hit, hovered and focused, and Tab
  cycles within it. When it opens, focus moves to the first control in it that takes keys (a text field),
  or else its first control; when it closes, focus returns to where it was. Instances outside the layer
  stay loose at the looseness the layer gives (0.35 for a dialog). Esc, when the focused control does not
  use it, sends the layer's close message.
- **Disabled**: a `disabled` subtree has no handlers, cannot take focus, and all its instances' opacity is
  multiplied by 0.4.
- Tabs, segmented choices and lists each take a single focus stop and change inside with the arrow keys;
  their items can be pressed only with the pointer (`focusable(false)`). A list's selected row carries
  `reveal`, so it is scrolled into view when the keyboard changes the row.
- The view can know the window's logical size (`Ui::on_resize`) and use another layout when narrow.
- Elements do not shrink by default (`flex-shrink: 0`); use `grow` or `shrink` to let them. The root's
  size is the window.

**Clearing the focus.** `Ui::clear_focus()` and `Cmd::ClearFocus` take the focus off whatever has it, with the
focus marks fading as when it moves on; nothing is focused again until the keyboard or a press says so. A host calls it
after a route change that should not leave the old page's control focused (`tests/focus.rs`).
