# 1.15 Ripple rules

A press (even on empty space) used to send a front from the press that crossed the whole window, scattering
every glyph it passed into dots; with repeated presses the text stayed scattered. Now a front has a
**scope**, each element has its own **response**, text and controls have a **legibility cap**, and fronts from
the same element **don't stack**.

**Scope** (`el::Scope`, `El::ripplr(Scope)`, inherited by the subtree):

| Scope | Reaches | When |
|---|---|---|
| `Own` | The pressed element's own subtree | The default for a pressed control |
| `Container(None)` / `Container(Some(key))` | The nearest ancestor marked `El::ripplr_scope()` / the element with that key | An area whose contents should respond together (the styles page, the ripple section of the button page) |
| `Global` | The whole window | Theme and material switches (commits), page changes (side rail items), `Cmd::Ripplr` |
| `Off` | No front | |

A press on the page's empty space (no control beneath) sends nothing by default; only when an ancestor of
the empty space sets a scope. In the runtime a scope becomes **one box per front** (`Field::scopes`, device
pixels, the element's rect intersected with its clip): in the G-buffer stage an instance is pushed and
loosened only by fronts whose box contains its center (`in_scope`); grounds without instances (dottr's ground
dots, glitchr's row slips) test per pixel whether they are in the box. Boxes are used rather than a scope index
per instance because an instance is in several scopes at once (the button's subtree, its area, the window) and
one index can't express that; testing by center is exact for "own" and "an area" (children's centers are
inside their ancestors' boxes), at the cost of 8 more vec4s in the uniforms.

**Response** (`el::Response`, `El::respond`, inherited by the subtree): `Ignore` (not pushed, not loosened:
the caret, the selection halo, focus marks), `Nudge` (pushed a little, `burst` slips as usual, looseness
stays 0) and `Loosen` (the original behaviour). By default, text, controls (focusable, pressable, draggable,
text fields) and everything inside controls are `Nudge`; decorative shapes, pictures and boxes are `Loosen`.
Large headings in scenes set `Loosen` themselves (the sound scene's title); the material page's samples set
`Loosen` and `legible(false)` throughout, because they exist to show how a material loosens text.

**Legibility cap** (`Look::legible_cap`, 0.15 by default): what is marked as text to read or a control
(`El::legible(bool)`, inherited; text and controls by default) is loosened at most to this value by the
field's effects (fronts, holding, condensing again after a press, overscroll). Morphs, flows, recomposition and
new instances condensing are intended and not capped. Holding and overscroll ignore whether the response is
`Nudge` (they are the element's own interaction) and look only at the cap: 0 for `Ignore`, `legible_cap` for
legible, 1 otherwise. Condensing again after a press and release starts from the cap: the birth time is
recorded as `now − enter × (1 − cap)`.

**How instances carry it**: the instance record gained a vec4 `resp` (112 bytes): x is how much looseness
fronts may add at most (0 for `Ignore` and `Nudge`, `legible_cap` for legible `Loosen`, 1 otherwise), y is
whether it is pushed (0 / 1). The G-buffer stage's `own_loose`: `k = max(static looseness, 1 − birth)`, plus
`clamp(wave, 0, max(resp.x − k, 0))`, so a wave only fills looseness up to the cap, and what is already
looser than the cap (a held picture, the page under a dialog) gets nothing more. This value goes into
`near.b`, so a front's looseness is now computed per instance in the G-buffer stage, and the shading stage's
`kt` and `pic_k` add only the half-plane (the compare wipe).

**The lens** (decided from first principles): the lens is the pointer's share of the field. Like fronts, it
acts through each instance's response, and it does one thing only: let a person see which marks a picture or a
3D object becomes in this material. So it touches only pictures and 3D elements whose response is `Loosen`,
that are not a single legible block (`resp.y` < 1.5), and that neither they nor their ancestors are operated by
the pointer (no press, drag, `on_pick` or `on_move` handler): a pointer there is there to use it, not to look at
its material. Eligible instances get `resp.w = 1`, and the G-buffer stage's `own_loose` adds the lens to their
looseness up to their cap; text, shapes, drawings (`canvas`) and controls are never touched; the page's ground
(dottr's ground cells) follows the lens as before. Previously, pictures added the lens in the shading stage no
matter what, and 3D elements in the G-buffer stage, so decoration such as the card stack's cards and charts
darkened under the pointer. Acceptance: `check` item 15 (partial redraw stays byte-identical with the lens on
pictures and meshes). Dot's row slips act only where `near.b > 0`, so text that is only nudged doesn't slip.

**No stacking**: each front records the element that sent it (a hash of its key, `Field::sources`). While the
same element's previous front is still alive, a new press restarts it (new origin and time) rather than adding
another (`Field::front_in`). Each pixel's looseness, after all sources are added, is clamped to
`Look::loose_max` (1 by default; the shading stage's `kt`, `pic_k`, `loose1`).

**The CPU model**: `Ui::looseness(key, now)` uses the same rules (for each instance at its center: its own
looseness plus the fronts that reach it, up to its cap, clamped to `loose_max`, without the style's looseness
scale) to give how loose the loosest instance in a subtree is; tests and the tree export use it.
`Field::wave_at` is the CPU version of the shader's `wave_in`.

**Holding what is under text** (the renderer's hold image): one rule, the ink under text to read is held at
looseness 0, so the label reads on the colour its ground has at rest (`GATHER →` on its fill's (255, 107, 43)).
`legible_cap` bounds only how loose legible ink itself may get, not what is under it. Two shapes:

- A filled control whose own fill is loosened (`El::loose` on the control itself): one island a control, the
  control's own fill not coming apart there. It is the ink box of all its content (text, arrows, icons) grown by
  `g = max(5, 0.36 × its text's size)` logical px with corners of radius `g`, cut by the control's outline at rest;
  a hard edge, anti-aliased over one pixel, with no transition band: outside it the material is as loose as the
  fill is. Marks that meet its edge join it (they are not cut). A control inside a loosened subtree keeps its fill
  whole (the cap above), and has no island.
- Text over another element's ink: an island that follows the glyphs, as below. Where two neighbouring glyphs of a
  word in a line are less than `2 × r1` apart (and less than a fifth of the text's size, a space being wider), the
  gap between their ink boxes is held as a glyph is, so a word is one island and no glyph is an island alone.

Each instance of text to read that has ink of something not legible drawn before it within `r1` holds the looseness
there: to 0 within `r0`, then from the legibility cap fading out by `r1`, with `r0 = max(3, size / 4)` and `r1 = r0 + max(6, size / 2)` logical px by the type size (of the label, for a
control; a glyph's `r1` is at most the 12 px the near image's nearness reaches). A heading is a title and holds
nothing. The renderer draws the holds into a two-channel image before the G-buffer stage: the hold, and the core
(within `r0`, or the control's island, with a 1 px anti-aliased edge). Every source of looseness (a shape's own, fronts, the field map) is
capped by the hold, so a label on a loose shape reads as at rest while the shape around it loosens.

How each material meets the island: within the core every material draws the frame at rest, exactly; from `r0` to
`r1` it draws its own transition, never a fade by transparency. A material's lattice (charr's cells, dottr's dots,
halftonr's screens) is sized by the ink's own looseness, not the held one, so its pitch does not change near a
label and no cell is cut; the hold only decides what each whole cell draws. charr: cells are drawn whole or not at
all, denser towards the island, and the island follows the glyphs (not their box); dottr: dots grow towards the
island and join it, with no seam of paper; halftonr: its separations print over the ink, so closed up they are
the ink itself; across the band each cell drops its dot whole at a hold of its own, and no dot is left that would
reach the island or lie within 3 px of it, so nothing round the island is smaller than a dot, off its screen or a line
of colour; glitchr: a slipped row never samples within a hold, so no copy of the label appears; scribblr:
the fill holds to `r0` with a crisp edge, and the thread goes round the island. `tests/legible.rs`
(`the_edge_of_the_island_has_no_box_seam_rim_or_copy`, and the held area of charr at most 92 % of the ink box grown
by `r1`: measured 0.24–0.35). Measured on dottr at looseness 1: the label's own ink 1.000 of its ink at rest, within
`r0` 0.993, beyond `r1` 0.988 of the shape's loose ink unchanged; it costs about 7 ms a frame on lavapipe for the 528
holds of a full gallery page (60.9 → 68.8 ms, 64.6 → 71.7 ms). `tests/legible.rs`. A control's island: for all nine materials at looseness 0.3, 0.6 and 1, both themes and labels of 11, 14 and 20
px, every pixel of the island a pixel in from its edge (but the label's ink) is the material's frame at rest to
1/255, beyond 12 px of it the control is as it is without its label (glitchr's rows aside), and every material but
plain loosens at least 50 px of the control away from the island (scribblr into its thread); text on others' ink:
within `r0` the frame at rest to 1/255, and every glyph of a word in one island
(`a_control_whose_own_fill_loosens_keeps_one_box_round_its_content_at_rest`,
`text_on_others_ink_keeps_an_island_at_rest_and_one_a_word`).

**Acceptance**: `tests/ripple.rs`: default responses and legibility (the count button's text is `Nudge` and
legible; decorative shapes `Loosen`; an `Ignore` neighbour; keyed text recomposes); with "own", the front's box
is the button's rect and neighbours outside stay at looseness 0; with "this area", the box is the area,
`Loosen` neighbours loosen (> 0.5), `Nudge` and `Ignore` ones stay 0, and legible text stays within the cap; a
press on empty space sends no front; under three scopes, ten presses of the count button in 0.3 s loosen the
button's text no more than one press does, no more than 2 fronts are alive, and `needs_frame` is false within
the normal settling time after the last press (the longest of front lifetime, condensing and recomposition);
with `loose_max = 0.4` a `Loosen` neighbour is at most 0.4 loose. `tests/frames.rs`: a press on empty space
sends no front; a press on a button does.
