# 1.3 场景（`scene.rs`）

Three kinds of instance share one record (112 bytes): `rect`, `uv`, `paint` (tone, accent, opacity,
kind; for a picture the accent bits hold its `Markr` slot), `clip`, `extra` (border width, picture index
or a glyph's print pitch; looseness at rest; birth time), `look` (material override code; the accent's hue
0/1/2; the box a height material measures height in, the top and height of the text line or the shape
itself, in device pixels) and `resp` (how far the field may loosen it and how it pushes it, §1.15).

- Glyphs: one instance per glyph. Each atlas pixel stores coverage and the signed distance to the
  nearest stroke (a distance transform, within 12 px).
- Shapes: rectangles with an analytic distance, filled or border only.
- Pictures: one instance per texture. Where a texture has alpha (a cut-out such as `pictures::CUTOUT`),
  pixels under half alpha count as nothing: no ink written, not counted as picture, so the picture has
  a true outline.

An instance's identity is `key + index`. Its birth time is recorded the first time it appears, and the
material condenses it from that. Text content that changes counts as a new instance. The scene records
which instance a layer starts at (`layer`); no batch crosses that position.
