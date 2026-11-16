# Body

The landing page (`#body`). A passage of text is first a page you can read; then it gathers itself into a slowly turning, luminous body made of its own glyphs, and goes back to being readable text. The code is `examples/body.rs`, written only against the public API; the page is the scene alone, full-bleed, with no chrome.

## Two rest states

The page, every glyph exactly where the layout put it (the text is not posed at all), and the body, turning slowly while the camera breathes. A spring carries `gather` between them; a click, Space or Enter, or the gather / release marking toggles.

## How it is made

- The passage's glyphs are the body's bright skin and their count is conserved: glyph i of n travels between its place on the page and point i of a spiral on the surface, turned to the spiral's tangent, squeezed by its slope, larger in front. The volume is a second population of some eleven thousand small marks from the frame budget, on the far shell and inside, born while it gathers and gone when it releases.
- The form is a signed-distance field: `rock` is a sphere with large relief and a few flat facets, `cloud` a smooth union of lobes; the seed places them.
- One key light from the upper left: the lit side white-hot, the middle in the form's hue, the far side its deep shade, a rim on the silhouette, depth fog, the finishing pass's bloom and a vignette. Rock is bone-white, ember and deep violet; cloud pale cyan, cyan and indigo.
- Beneath it, a pool of reflected marks and a faint spill of light; dust drifts at three depths with parallax; the whole picture follows the pointer a little.
- Drag to turn it (it coasts back to its idle turn). Near the pointer the glyphs snap back to their places on the page, so the words can be read through a clean opening.
- The controls are small markings in the picture with leader lines to the silhouette: form, seed, gather / release, the language, and the way to the Lab and the components.
