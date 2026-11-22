# 1.14 The accessibility mirror

The picture is drawn into a canvas, invisible to screen readers. So the runtime builds a mirror from the
interface tree (`Ui::a11y() -> Vec<A11yNode>`, no GPU, no platform): every element's key, its nearest ancestor
in the mirror, role, name, state (value and range, text, checked, selected, heading level, expanded,
description, live, invalid, required, modal, disabled, focusable) and where it is drawn.

**Roles** (`el::Role`, the WAI-ARIA roles of the same names) are written by components (`El::role`,
`El::label`, `El::checked`, `El::selected`, `El::value_range`, `El::text_value`, `El::heading`,
`El::expanded`, `El::description`, `El::live`, `El::invalid`, `El::required`): a button →
button; a switch → switch (`aria-checked`); a slider → slider (`aria-valuenow / min / max`); tabs → tablist /
tab (`aria-selected`); text fields and multi-line fields → textbox (the name is the placeholder, the value
the text); a dialog's card → dialog (`aria-modal`, named by its title); lists and the ten thousand rows →
list / listitem (`aria-selected`); a picture → img (named by its caption, "Picture" without one); the page
title → heading 1, section headings → heading 2; an accordion header, a select and a popover's button →
`aria-expanded`; a status line or toast → status, announced politely (`aria-live`) unless it says otherwise.
Without a role: something pressable and focusable is a
button; text not inside a control is text; a picture is img. Without a name, the text inside the element is
used (the names of buttons, tabs, list items and switches are their own text), and that text does not
appear again on its own. While a dialog is open, only what is inside it is in the mirror (the page beneath is
inert to screen readers).

**Syncing**: `Ui::a11y_diff()` compares by key the way layout does: against what was last handed over,
changed or new nodes give `Set`, missing ones `Remove`, and a focus change `Focus`. So one press of a button
sends only the count line and the focus. While a scroll goes on (`SOFT_STOP` after its last step) and nothing was laid
out, only places change: nothing is sent but focus, and the first call after the scroll rests sends the new places
(`Ui::next_wake` asks for that frame); a scroll frame neither builds the mirror nor serialises it
(`tests/access.rs`).

**Web** (`web/amenui.js`, and the website's `web/page.html`): a tree of `<div>`s in `#a11y` (nested as in the mirror), each with
`role` and `aria-*` attributes, fixed at the position where it is drawn; the whole tree has `opacity: 0` and
ignores the pointer (present for screen readers, absent for eyes and the pointer). It is updated after every
event and every frame by `a11y_update()` (a JSON diff). **Focus in both directions**: when a mirror element
gets focus (the screen reader moves focus there), `a11y_focus(key)` gives that control the runtime's focus;
when the runtime's focus changes, DOM focus follows only if it was already in the mirror (otherwise the canvas
or the hidden input element keeps the keyboard). **Activation**: a click on a mirror element (a screen
reader's "press") calls `a11y_activate(key)` → `Ui::activate`: the same message, front, commit and
condensing as Enter. Keyboard events come from the window as usual, so with focus on the mirror's slider the
arrow keys change it.

**Native** (`app::access`, AccessKit through `accesskit_winit`, so UI Automation on Windows,
NSAccessibility on macOS and AT-SPI on Linux): the window is one `Window` node and every `A11yNode` an
`accesskit::Node` under its mirrored parent (role, name, bounds in window px, value and range, text,
toggled, selected, expanded, description, live, invalid, required, level, modal, disabled), with ids hashed
from keys so they stay put across rebuilds. The tree is built only while assistive technology listens, after
each drawn frame. Actions: Focus → `focus_key`, Click → `activate`, Increment / Decrement → focus and → / ←.
The window is created hidden and shown once the adapter is attached, as AccessKit requires.

**Acceptance**: `tests/a11y.rs`: #button has a focusable button, a disabled button, the page title as a
level 1 heading and the count as text, and the button's text doesn't appear on its own; the second
`a11y_diff` is empty; `activate` sends the same message as a press and focuses it, and the diff after it
holds only what changed; with #dialog open there is a modal dialog with everything else inside it, a close
button and a text field, and the page beneath is absent; the slider's value is within its range, the switch
has a checked state, each tab list has exactly one selected tab, the list has a selected item, the text
field's value is its text, and the picture has a name; after a screen reader gives the switch focus, Space
toggles it and `activate` toggles it back. `app::access`'s unit test: the AccessKit tree has the window
and every mirrored node, a status is polite, bounds are in window px, and Click reaches the runtime. Web (Playwright's `ariaSnapshot()`): #button's buttons, heading,
count and disabled state; pressing the button through the accessibility tree makes the count 1; with #dialog
open there is a dialog with `aria-modal="true"`, the page title is absent, and the text field and buttons are
present; #slider has a named slider with `aria-valuenow / min / max`; with focus on the mirror's slider, → takes
the value from 40 to 41. No console errors.
