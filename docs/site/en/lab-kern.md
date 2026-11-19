# Kern

The distance field between two letters: one word whose letters are bodies on springs, and each spring's rest length is measured between the letters' outlines, not their boxes. Under each pair is the smallest distance between the two outlines; above it, the white area between them. The tightest pair is in the accent. Drag a letter along the baseline: its neighbours keep the rule and are pushed along, touching but never overlapping, and no letter leaves the stage; let go and they spring back.

## Formula

```
metric:  x_i = the shaper's pen                       (the face's advances, kerning included)
gap:     x_i = x_(i−1) + min dx : d(A, B + dx) ≥ target,   d(A, B) = min |p − q|, p ∈ A, q ∈ B
area:    x_i = x_(i−1) + dx : white(A, B + dx) = W     (W the pairs' mean in metric)
```

## Parameters

| name | default | range | what it is |
|---|---|---|---|
| `target` | 8 px | 0–40 | in `gap`, how far apart every pair is at its closest; the slider stops at the largest value that keeps the word 16 px inside each edge of the stage, and the panel prints it |
| `size` | 176 px | 44–220 | the most the word is drawn at; the drawn size is the largest whole px under it at which the word, in `metric`, is at most 80 % of the stage |
| `stiffness` | 600 | 100–900 | the springs |
| `damping` | 44 | 10–60 | the springs |

`[ metric ]`, `[ gap ]` and `[ area ]` choose the rule, and the letters move to it on their springs. `[ dims ]` shows and hides the measures. The field under the word changes it: up to 10 characters, as many as fit the stage at 44 px.

## How it is made

The word is set in the site's sans, Noto Sans SC, where the three rules give three different settings: `metric` is where the shaper puts each letter, kerning included, as the layout tells the word's pose (`GlyphAt::pen`); `gap` and `area` then respace those pens by the outlines. A pixel monospace would not show this, since every letter reaches the same columns of its cell and every pair's gap comes out equal. Each letter's ink comes from the page's faces (`Ui::on_faces`, `pair::Faces`); the distance between two outlines and the white between them are `pair::gap` and `pair::area`, measured every frame a letter moves, and the spacing a rule asks for is found once by `pair::spaced` and `pair::spaced_area`. The word is one text, so it selects and copies as a word: each glyph is moved from where the layout set it to its letter's place (`El::pose`). The code is `examples/lab/kern.rs`.
