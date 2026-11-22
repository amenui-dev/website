# 6. 场景

The original film (a prologue and eleven chapters, the slate and timeline, the before-and-after divider, each chapter's
caption) was removed entirely by the owner's decision: it was only a showcase. The xray material and its preset
(Transparent Electronics) were removed with it. The general foundations (commits, flows, recomposition, shedding, flow
fields, text around obstacles, 3D mechanisms) stay in the library, with their own pages and tests. The landing page is
now Body (§6.2). The scenes live in the website repository.

### 6.1 Seven scenes (in the gallery)

Seven scenes, each a whole page (the whole window under the top bar, without the side rail), at `#scene-<name>`;
`#scenes` is the list page, seven cards each with a small thumbnail composition built from the same parts, and pressing
one flows the card's title (`El::flow`) to the scene's title position. The top bar's left side becomes "‹ Scenes", and the
style, theme and material menus work as usual; the top bar also has a Scenes entry. Code: the website's
`src/gallery/scenes.rs`.

Shared rules: quiet at rest, with one obvious way to start; what moves moves only by loosening, flying and closing (fronts
in the field, morphs, flows, condensing); no boxes between text and what is behind it. A collapsible parameter bar at the
bottom, the scene's numbers as number controls, plus Reset. Esc (when no control uses it, `Ui::on_unused_key`) returns to
the list; Tab reaches every control (segmented choices and tabs take one stop each, checked by `tests/scenes.rs`). Below
720 px the parameter bar becomes one horizontally scrolling row. Each scene applies its own default style on opening
(table below), after which the style menu switches as usual; the `scenes` tool gives samples of each scene in its default
style and two others, and in a 420 × 800 narrow window.

| # | Address | Default style | What it is | Interaction | Parameters |
|---|---|---|---|---|---|
| 1 | `scene-opening` Opening | Neo-Dotmatrix | A screen full of dots (a passage made of ●) and a Start button. Pressed, the dots scatter into particles and gather into the title "The same mechanism", pause, then scatter again and assemble a page (title, search field, three picture cards, a switch, a number, a button, tabs). Then a "machine" operates the page on a beat: its mark is a pointer-like thing of about 34 px (a crosshair, four corner brackets, a center dot), framed by the annotation layer with its coordinates written beside it (`{xy}`), which also frames the control it is on and writes what it is doing. Each step the mark flies from the last control to the next (a §1.10 morph with a large stagger, so it is a short string of particles), acting only after landing, with a front where it acts; it types into the text field (each glyph condensing), flips the switch, adds 4 to the number, changes tab and presses Publish. A line at the bottom says what it is doing ("02 / 09  typing"). Any press, scroll or key hands control to you: the machine stops, the mark disappears, and the bottom line becomes "You took over" and "Continue the demo". When the demo ends the page is an ordinary page. | Start (button, Enter); any action takes over; continue; every control on the page | beat, flight, arc, pitch |
| 2 | `scene-reading` Reading an article | Lo-Fi Print | An article of about 900 words, the body set with `.around` around obstacles: the lone tree cut out of its ground (a picture with alpha), a circle, or both. Where an obstacle splits a line into two pieces, each piece takes a line. | Drag the tree and the circle (the arrow keys move them too, Shift × 4), the body breaks again at every step; a segmented choice changes obstacles | gap, picture width, font size, line height |
| 3 | `scene-printing` How a picture prints | Cyberminimalism | A picture filling the whole stage (cropped to fit, centered), cut by four dividers into five windows, each the same picture in the same place with its own `Markr` slot (characters, palette-limited pixel art, strokes following form, light points, brick relief); above each window its number, mark set and pitch. The character window reads clearly even on a dark ground: pitch at least 9, 16 characters, original colour brightened. Dividers sit in the gaps between windows and labels above the picture, so every pixel on the picture is the picture's own. At rest it is the original; the pointer's lens is hard-edged (`Fx::lens_hard`): fully loose within its radius, which is this window's real marks, and untouched outside, with no half-loosened blocks. A second mode, "One mark set": the whole picture is one window, with mark set, pitch, levels and palette set by number in the parameter bar, and a compare line (`El::wipr`) with the original on the left and the marks on the right, moved by dragging the picture or the dot under it. | Move the pointer; drag the dividers or move them with the arrow keys; five / one; change picture; drag the compare line | five: pitch, levels, lens, looseness; one: pitch, levels, mark set, palette |
| 4 | `scene-morph` Morph | Neo-Dotmatrix | Four forms in the same place, the stage filling almost the whole window: a very large heavy word (`El::weight`), a picture, the same picture's marks (held at looseness 1) and a complete small interface (a card with a title, a picture, a sentence, a Favourite switch, a Copies number and a Go button). Every change of form is a §1.10 morph: the source loosens, particles fly, the target closes. Once the small interface lands it can be used; pressing Go turns it back into the word. A row of steps under the stage (01 word — 02 picture — 03 marks — 04 interface) jumps to the step pressed; a Loop switch makes it hold each form for 2.6 s and go on by itself. | Press the stage, Next, → / Space / Enter, ← to go back; jump by step; the Loop switch; the small interface's controls | flight, arc, stagger, pitch |
| 5 | `scene-xray` Taking it apart | Tech-Manual | A small application (a title, a picture, a list, a button). While Space (or the "Hold to take apart" button) is held, the top layer flows away to the right, and in its place its outline layer (borders only) and the G-buffer layer (the `near` material, panels unfilled, since a filled panel would cover the SDF of everything above it) condense, each labelled with its name; the annotation layer frames the title, picture, list and button with numbers, names and sizes. On release the top layer flies back and the other two loosen and fade. In narrow windows the three layers stack vertically. | Hold Space or the button; a "Keep apart" switch | layer spacing, annotation count, iso-line spacing |
| 6 | `scene-sound` Sound | Glitch | A generated signal (no audio input): a kick on every beat (a low band decaying exponentially), a snare on even beats, hats on the offbeat (a high band) and a chord in the mids, with a little grain by seed. The heights of 32 spectrum bars are the bands' levels, and loud bars loosen by level (scattering into dots under `dottr`, slipping under `glitchr`). Every beat sends a front from the bottom center of the page with strength = intensity × low band (no front at intensity 0, so the page doesn't move), and the field's push grows with it, so the whole page swells with the low end; fronts land only on the title, lyrics, spectrum and ground: the parameter bar, the play button and the top bar are `El::calm`, so fronts pass beside them without pushing or tearing. The title and four lines of lyrics (the current one larger) stay readable. | Play / pause (button, Space) | tempo BPM, intensity, low band, seed |
| 7 | `scene-styles` Ten styles | Normcore | One interface (a weather card: a title, a picture, a paragraph, a button, a switch, a four-day forecast) and twelve style buttons. The buttons are commits: the new style spreads from where it was pressed. | Press a style button; ← → switch in turn | wave speed, wave width, trail length |

Runtime interfaces added for the scenes: `Ui::on_unused_key` (keys nothing used go to the state: Esc, Space, arrows),
`Ui::on_key_up` and `Ui::key_up` (the web page and the native window both forward key releases; the take-apart scene uses
them), `Cmd::Ripplr` (the state sends a front; the sound scene's beat), `Cmd::RipplrAt` (a front from an element's center,
where the opening's machine acts), `Ui::on_input` (tells the state before every press, scroll and key from a person; the
opening uses it to hand over), `El::calm` (up to four boxes that fronts pass beside without pushing or loosening; the
scenes' parameter bars and the top bar), `Fx::lens_hard` (the lens's edge: 0 is the original bell, 1 fully loose within
the radius), and `{xy}` in annotation notes (replaced by the element center's coordinates in the scope).

Acceptance: `cargo test` (`tests/scenes.rs`: addresses, Esc, default styles, every pressable thing can take focus, a list
card's title flows when it opens its scene, the opening's steps, the opening's mark is in the window and 28–36 in size, any
press or key hands over, continuing goes on, reflow after a drag, dividers, the single-set compare line, hold to take apart
and release, the morph, jumping by step and looping, the small interface works once landed, beats put fronts in the field,
no front at intensity 0, the sound scene's parameter bar and controls are calm); `check` item 9: in every scene at rest,
outside the declared active area (the printing scene's picture) no instance is loose, condensing or under a front (with a
control); `scenes` gives every scene's samples plus `scenes.png` and `scenes-list.png`; the web version under WebGL2 opens
the list and every scene and operates each once with no console errors.

### 6.2 Body (the landing page and first scene)

`#body` is the first page the web version and `window` open, and the first item of the side rail's scenes group; its code
is the website's `examples/body.rs`, written only against the public interface, and the gallery only fills the window with
it (`src/gallery/body.rs`): no top bar, no documentation shell, no explanatory text, no sliders or comparisons. It should
look like a frame from a game or a work of art, not a demo panel. An original descriptive passage (a day on a tidal flat,
`body.p1`–`body.p5`) first appears as a real, readable page of text, and after `DWELL` (2.4 s) gathers by itself into a
slowly turning, luminous body made of the passage's own glyphs; another press returns it to text.

- **Two rest states, nothing in between**: text (with no `pose` at all, identical to ordinary layout glyph by glyph) and
  body (only the idle turn and the lens's breathing). `gather` is carried between the two by a spring (k 30, c 11); a press
  (without a drag), Space, Enter or the gather / release marking in the picture switches it.
- **Skin and volume**: the passage's glyphs are the body's glowing skin, their count conserved: glyph i travels between its
  laid-out position and point i of a spiral on the surface (earlier in reading order goes first; `progress` is exactly 0 and
  1 at 0 and 1), turned to the spiral's screen-space tangent, squashed by slope and scaled by perspective, up to about twice
  body size facing the viewer and small on the back. The volume is a second set of marks: about eleven thousand small dots
  (`VOLUME`, asked of the frame budget), two thirds on the shell (seen through the skin from behind) and one third inside,
  each appearing at its own moment as it gathers and disappearing as it releases; they are not text and not conserved.
- **The form is a signed distance field**: `rock` is a stone of large low-frequency swells plus a layer of mid frequencies,
  cut by five planes into a few facets; `cloud` is a smooth union of seven spheres (lobes); both are set by `seed`. Along
  the spiral's direction it marches inward from radius 2 to the surface, with the normal from the gradient. Sampling is
  cached by (form, seed, glyph count) and computed once.
- **Light**: one key light from the upper left front. The lit side is near white hot (ink with a touch of hue), the middle
  this form's hue, the shadow side its dark; a rim light around the silhouette (on the shadow side a lifted dark, so the
  silhouette always reads); fog by depth. Each form has its colours: rock is bone white, ember orange and deep violet;
  cloud is pale cyan, cyan and indigo; the third colour is only for where the pointer touches. The ground is a dark
  gradient with a horizon, and the finishing pass has bloom (strength 1.4, threshold 0.35, radius 10; bloom samples at a
  per-pixel noise angle, so text no longer shows ghost copies, a fix to `post_bloom`) and a vignette.
- **Space**: on the ground under the body, a halo and a pool of reflection (half of the volume marks, squashed and
  inverted, fainter with distance); three depths of dust drifting slowly with parallax by depth; the lens breathes slowly
  (zoom ±1.8%, pitch ±0.03); the pointer's position in the window brings parallax to the whole picture (the body turns a
  little and the whole shifts a little).
- **Interaction**: dragging turns it with inertia (`gesture::Velocity` hands over to coasting, returning by e^(−1.3t) to the
  idle speed of 0.16 rad/s); as the pointer nears, an opening (a very stiff spring, k 520) opens: glyphs whose original
  position is in the opening return exactly to their place on the page (sharp, readable), and glyphs and dots of the body
  that would fall in the opening aren't drawn, so its edge is clean; it closes when the pointer leaves. Fingers work the
  same. ← → nudge the turn. With reduced motion there is no idle turn, no breathing and no falling dots.
- **English only** (the user's decision): the text and the small markings on this page are English whatever the interface
  language (`Locale::En.tr`, with `body.*` only in the English catalog, except `body.label`, which is also the caption on
  the scene list card); the language marking reads `LANGUAGE  EN` or `LANGUAGE  ZH`, and pressing it changes the language
  of the rest of the gallery.
- **Controls in the picture**: a few markings in small mono type, like engraving on an instrument, joined by thin elbow
  leaders to the body's outline (each leader starting from the farthest point of the projected skin): form (rock / cloud,
  press to switch), seed (press to add one, arrow keys up and down), gather / release; language at the top right (press to
  change the rest of the gallery's language; this page stays English), and links to the Lab and the components at the
  bottom left. Below 700 px only the corner markings remain.
- **Rest**: the text state wants no frames (apart from the 2.4 s after arriving); the body wants frames only for the idle
  turn and breathing (at `ω` 0 or with reduced motion it rests too).

Acceptance: `tests/body.rs` (the glyph count is equal at four moments: text, halfway, body and back; the text state has no
poses and each glyph has the text's own colour, and after returning each glyph's rect, transform and colour equal the start
one by one; it gathers by itself after arriving; the text state draws no frames for two seconds, the body's idle turn draws
and its angle grows, and at `ω = 0` and with reduced motion the body draws no frames either; dragging turns it, it keeps
turning after release, a drag doesn't switch, and it coasts back to the idle turn; glyphs near the pointer return to their
flat positions and the opening closes after it leaves); `tests/i18n.rs` (no Han characters in either language, and the
language marking names the current language); `labsheet <dir> body` gives `body.png` (eight frames), `body-large.png` and
`body-cloud.png` (1600 × 1000) and a frame's time (with one and three copies of the text).
