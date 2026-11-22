# 1.6.6 Glyph outlines, between letters

Spacing letters by the true gap between them, or by the white area between them, needs their outlines, not
their boxes, and it happens in the state (a piece that pulls letters on springs), which has no text system.
`Ui::faces()` hands it the page's faces and nothing else (`pair::Faces`, cheap to clone; `Ui::on_faces(f)`
gives them to the state at once and again when a face comes). `Faces::glyph(font, ch, px)` is one
character's ink at a size, made once: its advance and, for each pixel row, where its ink starts and ends from
the pen, each edge placed within its pixel by its coverage (a pixel face is cut at half, as on the page).
Over that, for a pair with the second set `dx` px after the first (pen to pen):

- `pair::gap(a, b, dx)`: the smallest distance between their ink, px, each row a segment a pixel tall
  (rows next to each other touch); less than 0 when they overlap along a row, by how much.
- `pair::area(a, b, dx)`: the white between them, px², over the rows where both have ink.
- `pair::spaced(a, b, target)` and `spaced_area(a, b, target)`: the `dx` that gives that gap or that white,
  to 1/64 px.

Where the shaper set a word (its own setting, kerning included, to measure from) and where its letters were
drawn reach the state and a test this way:

- `El::on_pens(f)` on a text: after a paint that draws it, `f` is sent each glyph's pen x as laid out
  (logical px in the content box, not put on a pixel; the numbers a pose is told as `GlyphAt::pen`, in the
  order of `GlyphAt::i`), when they differ from those last sent: once for a word at a size, again when the
  word, size or face changes. A text that goes is forgotten and told again when it comes back. A text not
  posed leaves out lines far outside its clip, as it draws them.
- `Ui::glyphs_drawn(key)`: each glyph of the text as the last paint drew it, after its pose (`GlyphDrawn`:
  its number, the box its raster covers as drawn, logical px in the window, the box round it when turned, and
  its opacity). No pixel of a glyph's coverage is outside its box, so two glyphs whose boxes do not meet
  share no pixel, and where they meet a raster test over the boxes says whether they do.

A role resolves as the page's root resolves it, by its `El::fonts` (the `Faces` a `Ui` hands out follow its root
as it changes; `Faces::face_of`); inside a subtree with fonts of its own, name the face with `Font::Face`. A cell made
alone (`experiment::ui`) is handed them by `Experiment::faces`. Nine pair distances take about 0.1 ms in a release build (176 px letters). `tests/pair.rs`:
for every pair of `Voyage` at 176 px, set 0, 8 and 24 px apart by `spaced`, the nearest ink pixels of the
two in the text system's own coverage are that far apart to within a pixel; `spaced_area` gives the white
asked for; a text's pens are sent once, equal to a pose's, kerned (`AVATAR`), again for other words; the
glyphs drawn are moved by their poses and every inked pixel of the word is inside one of their boxes.
