# 1.2 Layout

The taffy tree is kept between layouts, and only what differs from the last view is changed (§1.13).
Text leaves are measured by parley: given a width, they return a height. Units are logical pixels; any
font size, no monospace requirement. Output: every node's absolute rect and clip rect.

Scroll containers: a `scroll` node clips its children; the offset lives in the runtime, keyed by `key`;
horizontal scrolling offsets x. `El::clip()` clips without scrolling (windows in scenes, tickers).

**Text around true outlines** (`src/wrap.rs`, no GPU). A text element with `.around(Around)` sets its body
around the true outline of obstacles, not their boxes. An obstacle is a `Silhouette`: from some row on,
one row per logical pixel, the horizontal span the ink covers in that row (`circle`, `rounded`,
`from_alpha`, which takes the first and last pixel above half alpha in each row of a picture);
coordinates are in the text's own content box. `Around { obstacles, gap, min }`: line breaking runs top to
bottom, taking one line-height band at a time and finding the pieces of the column width the band leaves
free (each obstacle's widest span over the band's full height, widened by `gap` on each side; where the
band misses the ink but comes within `gap` of it above or below, a circle of radius `gap` swept along the
outline blocks it, so there is a gap above and below too). The band is snapped to whole pixels like a line
box, and pieces narrower than `min` are skipped. Each piece takes one line (two pieces, left and right,
take two lines, left first), set from the piece's start. A piece that touches an obstacle on one side
(and is not the paragraph's last line) is justified: the remaining width goes between clusters (after
every character next to CJK; at spaces in Latin text; between letters only for a Latin piece without
spaces and with `Around::justify_latin_letters` on, off by default). So a left piece runs from the
column's left edge to the outline minus `gap`, a right piece from the outline plus `gap` to the column's
right edge, both edges flush, and the text is `gap` from the outline on both sides. Each gap grows by at
most `Around::justify_max` (in ems, default `wrap::JUSTIFY_MAX_EM` = 0.25). Where more would be needed,
the line is broken again with one more cluster (one more Latin word); if every gap then shrinks by at most
`wrap::SHRINK_EM` (0.05, or `SHRINK_SPACE_EM` = 1/9 when all gaps are spaces) that is used, otherwise the
piece is set from its start and is ragged on the obstacle side.

Implementation: lines are first broken as usual, recording each line's start, band, text range and how
much each gap grows. If any line is justified, the same text is laid out again with parley's
`LetterSpacing` on those clusters and broken at the recorded lengths (`break_next_with_length`), so the
break points (including the rule that closing punctuation never starts a line) are the same as the first
time, and drawing, selection, the caret and line boxes all follow the justified result. Layout happens
only when an obstacle moves to a different grid position; pointer movement never lays out. A word wider
than its piece leaves the piece empty; when a band's pieces are used up, the next line height is tried;
with no piece at all, the band moves down 4 px. Line breaking is still parley's; each line just gets its
own start and width through `BreakerState::set_line_x / set_line_y / set_line_max_advance`, so mixed CJK
and Latin text, punctuation, font sizes and line heights work as usual. Measuring
(`Text::measure_around`) and drawing go through the same function, cached by the obstacles' hash. The
height reaches the bottom of the last line.

Acceptance: `tests/wrap.rs` reads the glyph atlas and checks that every inked pixel is at least
`gap − 1` from the circle's ink (beside the circle, the horizontal distance in that line's band; above
and below, the straight-line distance; with a control that ignores the obstacle). `tests/wrap_around.rs`
measures, at twenty positions of the tree cut-out and in two languages, each line box (`Ui::line_boxes`) against
the outline on both sides: in bands with text on both sides the two distances differ by at most 2 px, and
a right piece is `gap` from the outline (plus or minus one grid step), except left pieces that stopped at
the justification limit. The same file checks, at twenty positions and in two languages, that every piece
beside the circle except the last line has its outer edge within 1 px of the column edge and its inner
edge `gap` from the outline plus or minus one grid step, or is a piece set from its start because of the
limit, at most 15% in either language. English first stretches spaces to the limit, then adds up to
`wrap::TRACK_MAX_EM` (0.08 em) between letters, and if that is still short, breaks part of the next word
with Liang's en-US patterns (the `hyphenation` crate, MIT/Apache, a dictionary of about 90 KB compiled in)
and puts it in this piece with a hyphen. The hyphen is one more glyph drawn after the line's last glyph
after layout, and `line_boxes` counts it in the line width. `Around::hyphenate` is on by default and
`track_max` can be set to 0 to turn tracking off; `tests/wrap_around.rs` also checks lines with
hyphenation among the twenty positions. Glyphs never overlap each other (both languages). `check` item 8
reads pixels back in the light theme to check the same (a circle and the tree cut-out's outline, several gaps).

Layers: a `modal` element, wherever it is written in the view, is lifted out by the runtime and made the
root's last child, filling the window, so it is drawn on top. A face missing a glyph falls back to the
first registered face (Noto Sans SC), so even the monospace face can show Chinese.

**Lengths on a grid.** Every length the layout produces is on a 1/64 px step (the parley and taffy sizes are
rounded to it before they are compared), so a wrapping row in a box exactly as wide as its content stays on one
line: of 3000 random rows of text and shapes measured and then set in a box of the measured width, 597 wrapped
before, none after (`tests/render.rs`).

**Pinned children.** `El::absolute(left, top, right, bottom)` places an element against its parent's padding box,
not the window: every element is a positioned box, as `position: relative` is in CSS. Any of the four insets work,
two opposite ones stretch it, and it follows its parent at any depth, in a scroll and under zoom
(`tests/render.rs`, 6 pairs × 3 depths × scroll × 3 zooms). A box whose children were all absolute and have gone
lays out as an empty box (taffy kept such a box's list of absolute children when it became a leaf; the runtime now
gives every leaf a positioning area so the list is emptied).

**Fitting a subtree** (`El::fit(Fit { mode, min, max, align })`): the container's one child is laid out at 1 to find its
natural size, then zoomed (`El::zoom`) by the scale that fits the container's content box (`mode`, `FitMode`; by default it only shrinks, to `min` 0.25 at least), placed at `align` (fractions; 0.5, 0.5 centres it), in the
same layout: two taffy passes, no hook after layout. The scale depends only on the box and the natural size, so a
larger box never gets a smaller scale. Pixel type does not change the scale: each pixel text in the subtree is set
at the largest whole multiple of its face's size that is no larger than the scale makes it, so it is only ever smaller
than in proportion and never overflows; under its face's native size it is set at that size (never smaller), and its
lines are then that much further apart, so wrapped lines do not overlap and its box holds them (`tests/param_boxes.rs`,
`a_formula_scaled_with_its_cell_is_as_tall_as_its_wrapped_lines`). At `min` and still too large it is drawn at `min` and clipped, and
`Ui::problems` reports `FitShort`; in a direction it overflows it is placed from the start (left, top), so its
beginning is seen and its end cut, and in the other it follows `align`. Exactly at `min` and within the content box
(inside any border) it is not cut and nothing is reported. `tests/render.rs`
(`a_fit_keeps_its_scale_with_pixel_type_starts_what_overflows_and_reports_only_what_is_cut`). A content-sized child is
measured in a holder 10⁵ px wide, so it is measured at its own width, not the container's.

`FitMode::Reflow` is for a specimen that should read as on its page (a docs stage): the child is first laid out in
place, at the box's width, its type at its own size and its rows wrapping there; only if it still reaches out of the
content box (measured from the box's centre, by every part under it except what scrolls inside a scroller) is it
zoomed down, and laid out again at that scale (up to four passes, each only smaller, so it settles where a smaller
scale wraps it into fewer lines), to `min` at least. Pixel type is zoomed with the rest (a reflow scales by little,
and a pixel face's next whole size down would halve it). `max` is not used. `tests/render.rs`
(`a_reflow_fit_wraps_at_the_box_width_and_shrinks_only_what_still_overflows`). Against the website's `fit_stage`, at
the docs stills' two sizes (100 stills), 80 are within 1 % of their pixels once a shift of up to 3 px is allowed and
65 within 0.2 %; the rest are Lab canvases whose fields depend on their offset, the skeleton stage (which `fit_stage`
draws a third of its size, counting a part laid out off the page) and the narrow dot chart, settled at a slightly
different scale.

Within any fit, pixel text is not set smaller when the scale is 1.

The scales found are kept. On the next layout the view is zoomed by them first, and if nothing layout reads has
changed since the last layout (no style, no text and no box size: a piece that only draws differently), that is the
layout, with no probe, one view and no taffy pass; anything else probes again as above. `tests/fit_once.rs`.

**Own-size elements** (`El::own_size(ax, ay)`): a label on a drawing that is zoomed (`El::zoom`, `El::fit`) or carried
by a growing picture (`Cmd::Flow`) keeps its own size: its type, box, border and pose, and everything it holds, are
not zoomed; where it is (insets, margins, `shift`) is. Placed absolutely, it goes where its point (`ax`, `ay`: 0–1
across and down its box) would be in the zoomed box, then onto whole device pixels, so its type never sits on half a
pixel and is drawn pixel for pixel as at 1. In a row or column it takes its own size's room where the layout puts
it (its siblings and the gaps zoom; it does not), and one that then reaches out of its parent's content box is
reported (`ViewProblem::OwnSizeOver`, P1 in the audit). `El::fit` measures a subtree that holds one at 1 and again
at the scale found, and solves the scale from the two sizes (a fixed part and a part that scales), still two
passes. On a growing picture it rides at its own point (§1.10). `tests/own_size.rs`.

**A view scale** (`El::view_scale(s)`): a view of a drawing at scale `s` (a map growing from a thumbnail to a window).
Everything the container holds is scaled as by `El::zoom` (positions, lengths, fills, corner radii, a drawing's dots)
except the pen: the widths of a drawing's strokes and blades, of borders (their room in the layout too) and of a trace,
and a trace's dashes and gaps, stay as at 1; own-size elements keep their size. A dot scaled under 1.5 device px across
is drawn as the one whole device pixel its centre is in, never gone and never fainter. The container keeps its own
box; it applies to its children as the view is laid out, so it may be set before or after they are added. `El::zoom`
is unchanged and scales the pen too. `tests/render.rs` (`a_view_scale_scales_geometry_and_not_the_pen`: at 0.8, 0.64
and 0.4, line and border widths pixel for pixel as at 1, geometry `s` times within half a pixel, a 1.5 px dot one
device pixel at 0.4, the label's type as at 1).

**Labels shown while clear** (`El::show_clear(inset, rank)`): a label on a drawing whose scale changes (an own-size
label on a view scale) is shown only while it is clear, checked every frame. Each view (the nearest `El::view_scale`
container, or the window) takes its labels by rank, the smaller first and then in the order of the view, and shows
each whose whole box is `inset` px inside the view and at least 2 px from every label already shown. One that is
hidden comes back only with 4 px more to spare (`inset` + 4 inside, 6 apart), so a label at the edge of fitting does
not flicker. It goes and comes by opacity, linearly over 150 ms from where it is; a label whose target has just
changed starts from where it is, not from when it last moved. Hidden, it is at opacity 0 and takes no pointer, keeps
its place in the layout and is still in the accessibility mirror. A label on its way keeps frames coming (§1.11). A
view checks at most 64 labels; past that the rest are shown as they are and the view is reported
(`ViewProblem::ShowClearMany`). `tests/clear.rs` (at 0.64 two that would overlap keep the lower rank; from 1 to 0.64
and back each moves at most a ninth a frame and turns at most twice).

**Clamping text** (`El::clamp(lines)`): a text element shows at most `lines` lines; text that does not fit ends in
the face's own ellipsis on the last line, in the text's colour and size, on the baseline, drawn by the material as
the rest of the line is (no fade, no mask). It breaks after a word, or between characters when that would leave more
than a quarter of the last line empty, and leaves no space or punctuation before the ellipsis. The ellipsis is the
face's U+2026 as shaping sets it; a face without one, or whose one has its ink's bottom more than a device px above
the baseline (the pixel face's floats at the middle of the x height), ends in three full stops at the face's own
spacing. `Ui::clamped(key)` says whether it was cut; the accessibility mirror reads the whole text; selecting and
copying it gives what is shown, without the ellipsis (it is not part of the text). Taking the clamp away brings the
rest in from the next frame: with nothing to fly from, its glyphs come in whole, cell by cell in reading order, over
the recomposition's flight (§1.16), and the ellipsis stays where it is, whole, until the first glyph after it has
more than half its ink, so the end of the line always shows one or the other. With `around` it does not clamp yet
(`ClampAround` in `Ui::problems`). `tests/text.rs` (`a_clamp_taken_away_always_ends_the_line_in_its_ellipsis_or_what_comes_after`,
`a_clamp_taken_away_starts_to_expand_on_the_next_frame`,
`a_clamped_text_copies_what_it_shows_without_the_ellipsis_and_a_pixel_face_ends_in_full_stops`).

**Previews** (`El::preview(true)`): a modal element inside (a sheet, a context menu, a dialog) is drawn where it is
written, laid out and clipped by the preview, without a layer: nothing beneath it is held loose, focus is not kept
inside, a press outside or Esc sends no dismiss, input elsewhere is not blocked and the mirror does not call it
modal. What is drawn only with focus (`focus_shown`, a skip link) is always drawn, over its siblings as when it has
focus, so its fill covers the ink beneath it; Tab still reaches it in the order it is written, so a skip link written first
is the first stop, in a preview and outside one. The preview clips a pixel inside its border, so a sheet docked to its
edges draws no second line along the border. (Any element that clips, clips inside its border.) For galleries and
documents. `tests/menu.rs`
(`in_a_preview_a_skip_link_covers_what_is_under_it_and_a_sheet_keeps_inside_the_border`), `tests/focus.rs`
(`a_skip_link_written_first_is_the_first_tab_stop`).
