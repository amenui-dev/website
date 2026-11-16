# Text

Text is set by parley and rasterised by swash; every atlas pixel holds coverage and the signed distance to the stroke, so every material draws a glyph by its true outline. A paragraph can run around any shape (`.around`). While scrolling, text moves by whole device pixels and is never resampled.

A piece of a line beside an obstacle is justified, so the column's margin and the gap to the outline are both straight: the slack goes between glyph clusters (after each CJK character, or at spaces in Latin text), at most `Around::justify_max` ems per gap (default 0.25). Between Latin letters only when a piece has no space and `Around::justify_latin_letters` is on (default off). A piece that would need more is broken with one cluster more and tightened slightly if that fits, else it stays start-aligned; the last line of a paragraph is never justified.

```rust
use lab::wrap::{Around, Silhouette};
let mut around = Around::new(vec![Silhouette::circle(120.0, 60.0, 40.0)], 8.0);
around.justify_max = 0.3;            // ems of extra advance per gap, at most
around.justify_latin_letters = true; // a piece with no space may be spread between letters
```
