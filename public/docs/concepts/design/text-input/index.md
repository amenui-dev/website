# 1.8.1 Text input

**Data.** The application keeps a `TextEdit` in its state: `text`, `caret` and `anchor` (character
indices; when `anchor ≠ caret` the text between them is selected), `preedit` (what the input method is
composing) and undo and redo stacks (at most `UNDO = 100` steps; a run of typing is one step, a run of
deleting is one step, anything else one step each). The field shows `TextEdit::display()`: while composing,
the string is `text` with `preedit` inserted at the caret, and the selection, the composition range and the
caret are byte ranges in that string, set on the text leaf (`El::select(a, b)`, `El::mark(a, b)`, `caret`).

**Events** (`el::TextInput`, `El::on_text(fn(TextInput) -> Option<M>)`): `Key { key, mods }` (keys that are
not text: arrows, Home / End, Backspace, Delete, Enter; A / Z / Y with Cmd), `Insert(text)` (typing, an input
method commit, a drop), `Compose { text, cursor }` (the composition so far; an empty string cancels),
`Paste(text)`, `Cut` (the runtime has already put the selection on the clipboard), `Pointer { at, phase,
clicks, shift }` (press, drag, release; `at` is the byte found from the layout, on a cluster boundary;
`clicks` 2 selects a word, 3 everything) and `Goto { at, extend }` (the runtime moves the caret by the layout
to the previous or next line, the line start or end). `TextEdit::handle(ev, max, multiline)` computes the new
value, or `None` when nothing changed (the key is then passed on).

**Editing rules** (`src/widgets/input.rs` and the boundary functions in `src/text.rs`, no GPU): Left and Right
move by grapheme cluster (`unicode-segmentation`; a letter with combining marks is one), Cmd or Alt with
Left and Right by word (UAX #29 word boundaries; each CJK ideograph is a word), Shift extends the selection,
and without Shift an existing selection collapses to its end; Home / End go to the ends; Backspace / Delete
remove the selection, or one grapheme cluster, or with Cmd / Alt a word; typing replaces the selection;
Cmd+A selects all, Cmd+Z undoes, Shift+Cmd+Z or Cmd+Y redoes; `max` limits the length; in a single-line
field newlines and returns become spaces, in a multi-line field Enter inserts a newline. While composing,
keys and the pointer belong to the input method (`handle` returns `None`); starting a composition deletes
the selection first; a commit is one `Insert` and one undo step. Pointer: a click places the caret (Shift
extends), a drag selects, a double click selects a word and drags by words, a triple click selects all.

**The runtime** (`src/ui.rs`): when focus is on an element with `on_text`, `key_mods(k, Mods { shift, cmd,
alt })` goes to it first: characters as `Insert`, Up / Down and Home / End in a multi-line field as `Goto`
(`Text::nav`: parley's `Selection::previous_line / next_line / line_start / line_end`), anything else as
`Key`; only what it doesn't use goes on (Tab always cycles focus; Esc always goes on). Keys with Cmd go only
to text fields. A press in a field focuses it first, then finds the byte from the text leaf's layout
(`Text::hit`: parley `Cursor::from_point`), sends `Pointer` and captures the moves and release that follow;
repeated presses (within 0.45 s and 5 px) are counted. Also `text_input`, `ime_preedit`, `ime_commit`,
`paste`, `copy` (the field's selection, or a read-only selection), `cut`, `ime_rect` (the caret's rect in
logical px, for the host to place its hidden input element and the input method's candidate window) and
`text_focused`. After every event that is used, the caret is scrolled into view: the field and the
scrolling nodes outside it scroll by the caret's rect (a single-line field scrolls with `scroll_x` and its
text leaf is `nowrap`; a multi-line field scrolls vertically).

**How the selection is drawn** (rule 1, `Look::select`): each selected glyph is drawn once more beneath
itself with its outline moved out by `Select::halo` px (3 by default; the same mechanism as `El::weight`,
from the signed distance), hard-edged and opaque, in one colour: the accent mixed into the ground at
`Select::alpha` (0.34; #5f2d18 on the dark theme). Because the halo is opaque, the halos of neighbouring
glyphs and lines join into one shape that follows the glyphs' outlines, with no box and no darker band where
two overlap. With `Select::ink` (on by default) the selected glyphs are drawn in the theme's ink whatever
their paint, so dim text selected reads at 7 : 1 or more on the halo. The halo comes with the selection and
goes with it, in the same frame, with no motion (it does not condense). A glyph belongs to the selection if
its center falls in one of the line boxes `Selection::geometry` gives (`Text::boxes`). The composing part
has a 1.5 px underline. The selection is drawn only while the field has focus (like the caret).

**Read-only selection** (`El::selectable()`, inherited by the subtree): a press on selectable text (where
nothing can be pressed) starts a selection; dragging finds the selectable text node under or nearest the
pointer (by line distance first, then horizontal), so a selection can span lines and elements; a double
click selects a word, a triple click the paragraph; a press elsewhere clears it. The range follows node
order, and `copy()` joins the pieces with newlines between them. Each line of source on the website and the
body text of the reading scene are selectable.

**Web** (the website's `web/page.html`, the way egui does it): an invisible `<textarea>` (transparent,
2 px wide, ignoring the pointer) moves to the caret (`ime_rect`) and takes DOM focus while a field has focus,
so the keyboard, the input method and its candidate window all happen there; without a field, focus returns
to the canvas. `keydown`: keys during composition are ignored; with Ctrl / ⌘ only A, Z and Y are forwarded
(C, X and V let the browser send clipboard events); printable characters are not forwarded while a field
has focus (they arrive as `input` events); the rest go to `key_mods`. `input` (`insertText`) →
`text_input`, `insertLineBreak` → Enter; `compositionupdate` → `ime_preedit`, `compositionend` →
`ime_commit`; `copy` / `cut` / `paste` are heard on the document and use `clipboardData`. Presses call
`preventDefault`; the page decides focus.

**Native** (the website's `src/bin/window.rs`): while a field has focus, `set_ime_allowed(true)` and
`set_ime_cursor_area` (updated every frame drawn), `Ime::Preedit` → `ime_preedit`, `Ime::Commit` →
`ime_commit`; keys are not forwarded during composition; while a field has focus, a keyboard event's `text`
goes to `text_input`. The clipboard is `arboard` (through x11rb on X11, no C libraries; default features off,
no images): Ctrl+C / X / V (⌘ on macOS).

**Acceptance**: `tests/text.rs`: grapheme clusters and combining marks, word jumps in Chinese and English,
Shift selection, Cmd+A, pointer click / drag / double click and word drag / triple click / Shift click, cut,
paste, undo and redo (≥ 50 steps, a run of typing as one), composition ("n" → "ni" → commit "你"; keys do
nothing while composing; cancelling changes nothing; composing replaces the selection and the commit is one
undo step), and how multi-line and single-line fields treat newlines. Through the runtime: on the website's
input page a press in the middle of the text puts the caret there, a drag selects, a double click selects a
word, the selection halo is exactly one per selected glyph (`tests/select_look.rs`: across two lines it is one
colour within 2/255 of #5f2d18, with no box and no darker band, the dim text in it at 7 : 1 or more, there in
the first frame and gone in the frame after a press elsewhere), typing / Shift+arrows / copy / cut / paste /
undo / select all, the composition shows in the field underlined, commit and cancel, the input method's rect
is in the field, a long line scrolls sideways in the field with the caret inside (Home returns to 0), a
multi-line field moves up and down by line and to the line end, and Enter inserts a newline; selecting three
lines of source copies those three lines and a press elsewhere clears it; in the article, a double click
selects a word and a drag selects several lines. On the web (SwiftShader, real clipboard events): after
pressing the field the hidden textarea has focus; typing; composing with CDP `Input.imeSetComposition` (the
composition is not text yet) and committing "你" with `Input.insertText`; cancelling a composition; the
hidden element is over the field; Shift+arrows selects two characters, Ctrl+C puts them on the system
clipboard (read back with `navigator.clipboard.readText()`), Ctrl+X cuts, Ctrl+V pastes into a multi-line
field, Ctrl+Z undoes, and the console has no errors.
