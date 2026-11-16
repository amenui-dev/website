# Body

A passage of text gathers itself into a slowly turning, luminous body made of its own glyphs, and goes back to being readable text. Open it at `#body` (the first scene); the code is `examples/body.rs`, written only against the public API.

## Two rest states

The page, every glyph exactly where the layout put it (the text is not posed at all), and the body, turning slowly. There is no half state at rest: `gather` is driven by a spring between 0 and 1; a click, Space or Enter toggles, and typing a value under ½ into `gather` sends it to the page, otherwise to the body.

## How it is made

- Glyph count is conserved: glyph i of n travels between its laid-out place and point i of a spiral on the body's surface. Consecutive points are neighbours on the spiral, so the passage winds round the body in reading order. Each glyph is turned to the spiral's tangent on screen, squeezed by its slope and scaled by depth (`El::pose`: `dx`, `dy`, `angle`, `sx`, `sy`).
- The body is a signed-distance form sampled to surface points: `rock` is a sphere pushed in and out by two octaves of noise, `cloud` a smooth union of six balls; both from `seed`. A ray from far out along each spiral direction is marched in to the surface; the normal is the field's gradient.
- Light: dark ground, one hue. Brightness by facing and depth: the front nearly white with the hue, the rim and the back the hue itself, the back dim. The finishing pass adds a soft bloom. While it turns, rim glyphs shed a few motes from the frame budget (never while it is the page).
- Drag turns it with inertia; let go and it coasts back to the idle turn. Near the pointer the glyphs go back to their own places on the page, so the original words can be read through the opening; leaving closes it. With reduced motion there is no idle turn.

## Parameters

| name | default | range | what it is |
|---|---|---|---|
| `gather` | 0 | 0–1 | page (0) or body (1) |
| `form` | 0 | 0–1 | rock (0) or cloud (1) |
| `seed` | 7 | 0–99 | the form's noise or balls |
| `ω` | 0.18 rad/s | 0–1.5 | idle turn |
| `open` | 140 px | 20–260 | the opening's radius |
| `size` | 0.72 ·h | 0.3–1.2 | the body's largest radius, a share of the text's height |
