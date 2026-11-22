# 1.6.5 Faces that come later

A page need not wait for its faces. `Ui::declare_face(name, FaceDecl { stand_in, scale })` (or
`text::with_declared(faces)`, which declares the five Geist Pixel faces with the pixel face as stand-in and the
serif with the sans) gives a face its number at once, for themes and `El::fonts`; until its bytes come, text in it is
set, read and laid out in the stand-in at `scale` times its size, at rest, in its final colour, size and line height.
Nothing is a skeleton, a blank or a wait.

`Ui::faces_wanted()` lists the declared faces that have not come and that the page laid out last uses, by name; the
host reads it after each frame or page change and fetches them (the framework fetches nothing). `Ui::add_face(name,
bytes)` adds one: everything that came in one frame changes at the next tick together, in one layout. Each line then
sweeps over from its start to its end in 240 ms (`ui::SWEEP`): a glyph changes when the sweep passes it, from the
stand-in's glyph for the same character, moved to where the new one stands, to the new face's; within 60 ms of its
change (`SWEEP_LOOSE`) it is loosened by the material on a sine up to the legibility cap. Only a heading
(`El::heading(level)`, the heading role) goes up to 0.6: a large title that is not marked as one stays within the
cap (`Look::legible_cap`, 0.15 by default) and barely comes apart. So at every moment the title has one set of
glyphs, changed ones left of unchanged ones, and its ink never under 80 % of the lesser face's. 300 ms after the bytes
came the page is byte for byte the page that had the face from the start, and it rests. scribblr draws one thread per
element, so a line whose face is changing becomes a thread as loose as its glyphs are on average, and nothing keeps
out of it meanwhile; it is text again when the sweep has passed. The thread's looseness starts and ends at 0, rises to at most 0.6 for
a heading and the cap for other text, and moves by at most 0.15 a frame (`tests/faces.rs`,
`under_scribblr_a_changing_line_is_a_thread_within_its_limits`; measured peaks 0.144 for the heading, 0.034 for body
text, steps of at most 0.075). Measured: with dottr and scribblr each frame of the
sweep differs from plain's in the text's box by 0.30–2.6 of the title's ink beyond what the material draws at rest,
and by 0 from 300 ms (`tests/faces.rs`, `with_a_material_each_glyph_comes_apart_as_it_changes_face`). With less motion both glyphs
are drawn where the new one is, the new fading in over 150 ms. A face added before the first frame is never seen
standing in. Bytes that are not a font are refused (`FontError::Bad`): the face keeps its stand-in for good, is no
longer wanted, and nothing on screen changes.

Resources work the same way: `Ui::asset(name)` returns what has come, and asking for what has not puts it in
`Ui::assets_wanted()`; `Ui::add_asset(name, bytes)` adds it. On the web, the module carries only the three main faces
(sans, pixel, mono); `web/amenui.js` takes `opts.faces` and `opts.assets` (a map of name → URL, or a function), fetches
what is wanted after each frame, and `opts.preload` names faces to fetch with the module, added before the first
frame. The host exports are `faces_wanted`, `add_face`, `assets_wanted` and `add_asset`.

Acceptance: `tests/faces.rs` (declared faces drawn as their stand-ins, byte for byte, and wanted only when used; a face
added before the first frame never seen standing in; an arrival in one layout, the page after it as built in, and no
frames 2 s later; two faces in one frame in one layout; bytes that are not a font changing nothing and not asked for
again; the same arrival three times the same frames; resources kept by name and asked for until they come).
