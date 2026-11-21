# 1.36 Drawn once: cached subtrees and round clips

A page of thousands of small glyphs (the Lab's Loupe: a source file at 2.75 px, about 20,000 glyphs) costs as many
instances every frame it is drawn. `El::cached()` draws such a subtree once and keeps it.

- A cached element is a plate (§1.33, `fade_s` 0, with its limits and its rules: a picture, not content; it
  clips its children). When the page is painted, its subtree's instances are hashed. Equal to what the plate
  holds, and the plate not emptied this frame, they are dropped from the scene: the plate's span is empty and
  the plate lays on what it kept, so the frame costs no instances for it. Different, they replace what the
  plate holds where the frame draws (`PlateSpan::replace`), once: a frame drawn whole empties the plate and
  draws them onto it; a partial frame (§1.19) empties and inks the plate only within its stage-1 boxes, before
  stage 1, so a `cached_view` of it reads it inked. A restage that copies the subtree from the last paint
  with nothing in it leaves it as it was. (With `fade_s` above 0 the plate is emptied and drawn onto.)
- To the dirty region a plate laid on from what it holds counts as the instances it holds (the renderer keeps
  them as they were last inked), so a frame that inks it again is compared with what it held, as any other
  instances are. A scroll that moves the last frame (`dirty::Move`) moves the plate's ink and near images the
  same way when what it holds, in this frame and the last, is the content's (clipped by the viewport): the
  plate then has only the band scrolled into view, the edges and the boxes of what changed in it inked. When
  part of it is the content's and part lies over the viewport (a plate across a scroll area's edge), the
  frame is drawn whole ("plate across a viewport"). A `cached_view` of a plate inked this frame is drawn
  again where it is (`tests/plate_scroll.rs`: a cached sheet in a scroll area, scrolled down and up while a
  line in it changes, is the frame drawn whole byte for byte, and stage 1 draws under half the window).
- A paint in which nothing the subtree draws from changed passes it by: it is neither drawn nor hashed
  (`Counters::kept`), the plate's span is empty, and what it drew besides its instances (its identities' and
  texts' births, its texts' recomposition records and laid glyphs) is what it last drew. What it draws from is
  its nodes (key, box, clip, text and its type, fill, border, looseness, opacity, material) and what its
  ancestors give it (scale, a press held, the modal layer, the look's timing). Something it reads that its
  nodes cannot tell the same is never passed by and is compared as above: a morph or flow on the page, faces
  changing, and in it a drawing, a picture, a 3D view, a stage, posed, selected or composing text, a caret, a
  see-through group, something hidden, turned, at its own size, under a view scale, that stirs, sheds,
  traces or wipes, or that is pressed, focused, recomposing or decoding (`tests/cached.rs`).
- So it is drawn again whenever what it draws changes: it scrolls or moves (into the plate where the frame
  draws: a scroll inks its band), it loosens (a front passing, its birth), its words change, the window
  changes size (which empties every plate). A stage, a 3D view or a
  see-through group inside it is drawn with the rest, each time the rest is.
- Text in a pixel face asked for under its native size inside a cached element (Departure Mono at 2.75 px) is
  set and drawn at the size asked, as its outline's coverage at that size (a box filter of the native raster),
  not at its native size as anywhere else (§1.6), so the small page is laid out as asked and does not overlap
  itself: its faces carry `text::FINE`. At its native size and above it is a pixel face as anywhere.
- A plate is the renderer's. A renderer that meets a cached element drawing nothing into a plate it has not
  inked (one made after the page was painted, offscreen) has it drawn again (`Ui::plates_lost`), and sends the
  whole glyph atlas on its first frame. Each paint's ink goes into a plate once (`Scene::paint`), so the scene
  drawn again (no paint between) lays on what the plate holds.
- `el::cached_view(over, from, ink)` shows the part `from` (logical px on the page: x, y, w, h) of what the
  cached element `over` drew, scaled to its own box. Each pixel of the plate's ink is read as its coverage
  (alpha), nearest, and drawn as a glyph's coverage in `ink`: at a whole zoom each source pixel is a square of
  whole pixels, and the material draws it as text. It is one instance (`Instance::cached_slot`: its
  coordinates are packed below −1 with the slot; its batch, `scene::CACHED_BATCH` + slot, binds the plate's
  ink where a glyph's atlas would be). It draws nothing before the cached element has drawn once in this
  window, or inside the cached element itself.
- A cached element over the plates' limit (a fifth, or past twice the window's area) is drawn as a plain one
  and has no kept image, so a `cached_view` of it draws nothing. The state can know: `Ui::on_kept(f)` tells
  it a cached element's key and `false` when a paint finds it so, and `true` when it has a kept image again or
  goes while it had none (`Ui::kept_image(key)` reads it). The paint that finds the change asks for the next
  frame; the state is told there (or straight after a paint run by a message or a relayout) and the page is
  viewed again, so what the view does instead (the Loupe's glass shows only its rim, ×1) is a cut on that
  frame, both ways.
- `El::clip_round(radius)` clips the subtree to the box inside the border with round corners (half the side:
  a circle). The radius rides in each instance's `look[3]` above its in-place flag (twice the radius in
  quarter device px); the ink stage discards past half a pixel outside and antialiases the edge. A square
  clip inside it that cuts it is square; a clip around it that cuts it keeps its own straight edge. Presses
  are still taken anywhere in the box.

Acceptance: `tests/cached.rs` (a cached page drawn the first frame as without caching, then laid on with no
instances of its own while something else changes and the same as uncached; a frame with nothing new
identical; drawn again when its words change, with no old ink; after a new window size as uncached; a
cached view at ×4 inside a round clip of radius 40: every source pixel a 4 × 4 square of one colour, with
ink, and the box's corners outside the circle as the page without it; a page pushed over the limit, by area
and by count, told it has no kept image and viewed again, then told it has one when back under or gone).
