# 1.13 Incremental updates and long lists

**The view is unchanged**: after every message, `view(state)` rebuilds the whole `El` tree (Elm style:
messages are values and the state is the only source of truth). What is saved is everything after that.

**Comparing by key, keeping taffy nodes** (`Ui::layout`). `flatten` turns the tree into flat nodes (each
node's style and children kept separately); the runtime keeps the last `TaffyTree`, each key's taffy node
(`tids`) and a hash of the content each leaf is measured by (`mkeys`: text, size, line height, face,
obstacles, `nowrap`; a picture's aspect ratio). From the back (children get taffy nodes first): for a key
that existed last time, the style (`Style: PartialEq`), the child list and the leaf's content hash are
compared, and only what differs gets `set_style` / `set_children` / `mark_dirty` (taffy marks the ancestors
dirty itself); new keys are created; missing ones are removed. A key that appears twice gets a new node the
second time. `compute_layout_with_measure` runs only if something changed (`Counters::layouts`), and taffy
recomputes only the dirty paths, using its cache for clean subtrees; if nothing changed (only colours, a
selection background or anything else that doesn't affect size) it doesn't run (`layouts_skipped`) and last
time's boxes are used.

**Flexbox only: stacks and layers** (`el::stack`, `el::layers`). taffy is built with flexbox and nothing
else (no grid, block or calc), which keeps about 170 KB out of a web module. `layers` places each child
absolutely at the top left of its box (unless the child is placed already), so the box is the parent's or
its own size and does not grow with them. `stack` is a centred row whose children lie over one another by
their margins: after a layout the width each child took (its margins included, without what the stack
added) is kept by the stack's key (`stack_w`); the first child gets margins that centre it in the widest
one's room and each after it is pulled back over that room, centred too, taking none of its own
(`stack_extra`). The row's cross size is already its tallest child, so the stack is as wide as its widest
child and as tall as its tallest, as a one-cell grid made it. The margins go into the styles before they are
compared, so a stack whose children keep their widths costs nothing more; when a width changes the margins
are set in the tree and the page is laid out again, up to three more times (a child that wrapped while the
children sat side by side takes its full width once they lie over one another). A stack in a stage
(`Ui::stage_view`) gets the same margins, so it is compared as laid out. `tests/overlay.rs`.

**The pieces of text by words or lines** (`El::reveal_by`, `Text::reveal_pieces`) are found once for the same
text, face, size, line height, width and setting and kept with the layouts, in two generations; every paint asks
again and gets them from there (`Counters::reveals` counts the misses). A glyph's piece is found in its line's
boxes (the boxes sorted by their top), the last glyph's piece and the next one looked at first, rather than among
all the text's pieces.

**The text layout cache** (`Text::layout`): parley's layouts are cached by (text, face, size, line height,
width, obstacles) in two generations: when the current one reaches 2048 entries it becomes the previous one,
entries used are moved back from the previous one, and those used in neither are dropped. Measuring, drawing
glyphs, the caret, hit testing, the selection and line moves all use the same layout. A line of text that
doesn't need breaking is the same layout at different widths: a width-limited layout that comes out as one
line is also recorded as the unlimited one, and when a width is asked for and the unlimited layout fits, it is
used. So what taffy measures and what is drawn are the same layout, and unchanged text next to a button is
never laid out again (`Counters::shaped` = parley layouts).

**Scrolling only moves**: a node keeps the relative box layout gave it (`local`). The wheel, keyboard
scrolling and scrolling the focus or caret into view only change the offset, then `place()` (absolute rects,
clips and scroll lengths from relative boxes and offsets) and `paint()` (building instances), without calling
`view`, running taffy or laying out text.

**Culling what can't be seen**: `paint` skips nodes whose rect misses their clip; within a text node, lines
whose line box (plus size and SDF margin) lies wholly above or below the clip produce no instances
(`Text::emit_in`). So long source blocks and rows out of view in a scroll area are never uploaded.

**Virtual lists** (`El::virtual_rows(count, row_h, overscan, selected, |i| El)`, the component
`widgets::list_virtual`): this scrolling node's children are built by the runtime in `flatten` from its scroll
offset and last viewport height (the window height the first time): only the visible rows
`floor(offset / row_h) .. ceil((offset + viewport) / row_h)`, plus `overscan` rows above and below. They sit
in a column `count × row_h` tall whose top padding is the height of the rows before them, so the scroll bar's
length is the whole list's. A row's key is `list:i` (in the component), or `list/row{i}` without a key, so a
row keeps its identity (birth time, taffy node) while scrolling. When the visible rows stay within the range
already built, scrolling only moves; leaving it lays out once (one `view`, and taffy computes only the
list's new rows). Keyboard: the list is one focus stop; Up, Down, Page Up / Page Down (the viewport's rows
minus one), Home and End change the selected row; after a key changes the state, the runtime scrolls the
selected row into view by its index (it may not have been built yet).

**Counters** (`Ui::counters()`, also in the readout): `layouts`, `layouts_skipped`, `shaped`, `views`,
`scenes`.

**Acceptance**: `tests/incremental.rs`: on a page of ten thousand rows, the rows in the tree ≤ the visible
rows + 2 × overscan and the scroll length is 10000 × 40 (plus borders); scrolling within the built range
leaves `layouts`, `shaped` and `views` unchanged; after scrolling far, the rows there are built, the row count
is still bounded and a switch in a row can be pressed; pressing a row, End to the last row (built and in
view), Page Up, Home back to 0; six wheel turns on the button, input, marks, type, dialog and particle pages
leave the three counters unchanged (the scene count changes); pressing twice on the button page lays out with
parley only the text that changed (the count line; taffy measures it at a few widths); after a message that
only changes a colour, `view` runs once more, `layouts` and `shaped` stay the same and `layouts_skipped`
grows by one; on the input page there are no glyph instances far below the source's clip. The numbers from
`bench` (no GPU) are in §9.
