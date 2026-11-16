# Contours

Iso-lines of a field: drifting noise plus a bump under the pointer, sampled on a 61 × 39 grid and traced by marching squares into polylines, drawn as strokes.

## Formula

```
f(x, y) = n(x/s, y/s, t) + a · e^(−|p − m|²/r²)
```

## Parameters

Every number is a number control under the cell (type, arrow keys, drag sideways); none is a slider.

| name | default | range | what it is |
|---|---|---|---|
| `s` | 52 px | 12–120 | noise feature size |
| `a` | 0.70 | 0–2 | bump height |
| `r` | 30 px | 6–90 | bump radius |
| `n` | 9 | 2–16 | number of levels |

## How it is made

`field::Grid::sample` and `Grid::contours`; `Drawing::contours` turns them into strokes. The code is `examples/lab/contours.rs`.
