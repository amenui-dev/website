# Loupe

A magnifier over a real page. This piece's own source is set in columns at 2.75 px, too small to read: a grey texture. Under the glass the same text is set again at reading size from its outlines, hard-edged. On the rim, in the accent, the zoom and the line and column under the glass's centre. Move to read. `[ coverage ]` shows how the renderer sees the page, each pixel's coverage as a grey, with the pixel grid at ×8 and above; `[ distance ]` draws bands of the distance field from the glyphs' ink every 2 px. A press pins the glass and the text in it selects and copies; a second press frees it. Drag the rim to resize; `[` and `]` change the zoom.

## Formula

```
shown:  the page within radius / zoom px of the glass's centre
size:   zoom × 2.75 px        (11, 22 or 33 px: whole multiples of the face)
rim:    ×zoom · LINE l · COL c,  (l, c) the character under the centre
```

## Parameters

| name | default | range | what it is |
|---|---|---|---|
| `zoom` | 8 × | 4, 8, 12 | how many times larger the glass sets the text |
| `radius` | the lesser of 132 px and 30 % of the stage's width | 64–240 px on a stage under 480 px wide, else 80–240 | the glass's radius |
| `columns` | 8 | 4–10 | columns of the page |

`[ ink ]`, `[ coverage ]` and `[ distance ]` choose what the glass shows; `[ grid ]` shows and hides the pixel grid in `coverage`.

## How it is made

The page is laid out and drawn once and kept (`El::cached`); `coverage` shows part of what was kept, enlarged (`el::cached_view`). The glass holds a fixed set of line slots, as many as its height and width need at this zoom and radius: each holds one row of the page, padded with spaces to the column's width, and the glyphs outside the circle are left out by their poses (`El::pose`). A moving glass moves its slots and changes a slot's text only when the row under it changes, so nothing is laid out again; the glass and its rim are made again by themselves, and the kept page is not drawn again (`El::local_messages`). Pinned, the glass holds exactly the text it shows, so a selection copies it with no padding. The distance bands come from the glyphs' outlines (`pair::Faces`): their ink on a 4 px grid, its distance transform, and the transform's contours. The code is `examples/lab/loupe.rs`.
