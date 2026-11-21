# 1.16 重组

When keyed text changes its string (a count, a number control's value, a label, a list row's text), the pixels
of **the clusters that changed**, and only those, recompose into the new text: the old glyphs' cells fly onto
the new glyphs' shapes; the unchanged prefix and suffix stay where they are, and those whose position changed
slide to their new x.

**Parameters** (`morph::Recompose`, in `Look::recompose`; number controls in the ripple section of the button
page):

| Name | Default | Meaning |
|---|---|---|
| `duration_ms` | 160 | From the first cell leaving to the last landing |
| `stagger` | 0.2 | The share of the flight departures are spread over |
| `count_max` | 600 | The most cells each changed glyph flies (beyond it cells grow by whole device pixels) |
| `slide_ms` | 120 | How long unchanged glyphs that moved take to slide (ease in-out) |
| `min_interval_ms` | 80 | A change sooner than this after the last one is replaced directly, with no cells, slides or condensing (a readout that changes every frame never recomposes) |
| `max_tiles` | 24000 | The most recomposition cells alive across the window at once; changes beyond it are replaced directly |

**Switch**: `El::recompose(bool)` (inherited). By default, text with an explicit key recomposes; text in text
fields (subtrees of `on_text`) never does, since typing must appear at once or it fights the caret. A number
control's value is `{key}/shown`, off while typing a draft. A page change (a layout with `Cmd::Flow`) doesn't
recompose; strings are replaced as usual (the title is flowing). Neither do the two elements of a morph nor
anything under a dialog.

**Finding what changed**: for each text node that recomposes, the runtime records how it was last drawn (the
string, layout parameters, each glyph's identity and instance, `TextRec`). At the next `paint`, if the string
differs, the common prefix and suffix by grapheme cluster give the old and new "changed" byte ranges; which
part a glyph belongs to is decided by whether its center falls in that range's selection boxes (`Text::boxes`,
the same method as the selection halo). Prefix and suffix glyphs correspond one to one in order: new glyphs
take the old glyphs' birth time (they don't condense again), and those whose rect differs slide (`Slide`).

**How it flies**: the changed old and new glyphs are each cut into cells on the screen grid (`morph::tiles`,
starting at 1 device pixel, `curl = 0`, so straight lines), and pairing, stagger and easing are the same as
§1.10 (`pair`, `progress`, `position`, `alpha`). The difference is how cells are drawn: each cell is the piece
of a glyph instance that this cell cuts out (`piece`: the same glyph with atlas coordinates cut in
proportion), so cells are ordinary glyph instances drawn into the coverage and SDF buffers as usual, every
material applies, and the renderer needs no change. Before its own switch point (`misc[3]`, 0.3–0.7) a cell
shows the old pieces and after it the new pieces, fully opaque, at the cell's current position; SPLIT / MERGE
and dust fade by the `alpha` rule. Pixels move and swap rather than cross-fade, so text in flight stays solid
(cross-fading was tried: both sides were half transparent and the whole glyph turned grey halfway). At
progress 0 the cells together are exactly the old text and at progress 1 exactly the new: the cells cut the
same set of device pixels, and the side with zero opacity writes nothing (`step(0.01, alpha)` in the SDF
buffer). The new glyphs are drawn transparent during the flight (`hide`) and swapped back for the real glyphs
the moment it ends, long past their birth time, so they don't condense.

**Changing again midway**: when text that is recomposing changes its string again, it sets off again from
where the cells are: cells in flight are the new round's sources, each carrying the pieces of whichever side
it shows more of at that moment (pieces are cut from whole glyphs once and never cut again, so they don't
multiply; merging cells and dust that are fading out are dropped), plus cells cut from old glyphs that changed
this time. New glyphs still hidden from the last round count as changed if they fall in this round's prefix or
suffix, and recompose along with it rather than jumping in. There is no queue and no restart from the old
text. The new round lands when the last round would have (at least a third of a flight), so with repeated
presses the digits keep moving forward rather than flying from the start every time and staying on the old
text.

**Acceptance**: `tests/ripple.rs`: "Count 19" → "Count 20" hides only the two digits, "Count " keeps its birth
time and position, the cells are in the scene and frames are wanted; changing to "Count 21" halfway still
recomposes only those two characters; after the flight no glyph is hidden, there are no cells, nothing
condenses and `needs_frame` is false; text without a key is replaced directly; "9" → "10" recomposes both
characters. `check` item 13: under `plain` and `dottr`, for three pairs of strings ("19" → "20", a Chinese label
with "9" → "10", "3.5 px" → "3.6 px"), the first frame is compared with an ordinary rendering of the old text
and the last frame (after every cell has landed, before the end) with the new, over the whole window, pixel for
pixel, with a difference of 0; halfway and both ends differ (control). `film <dir> ripple` produces three
samples (§9).
