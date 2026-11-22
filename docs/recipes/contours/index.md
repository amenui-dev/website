# Contours


Iso-lines of a field: drifting noise plus a bump under the pointer, sampled on a 61 × 46 grid and traced by marching squares into polylines, drawn as strokes.

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

The field is the sum of two parts: `field::Noise`, which drifts slowly with time, and `field::Bump`, a Gaussian under the pointer whose height fades in and out as the pointer arrives and leaves. `field::Grid::sample` samples it at every node; `Grid::contours` runs marching squares at each of the `n` levels and joins the segments into polylines, and `Drawing::contours` turns them into strokes.

Where the bump bends a line, the line takes the accent, so the pointer's effect stays visible against the drifting noise. A small `s` gives tight, busy lines; a large one gives a few broad ones. The code is `examples/lab/contours.rs`.
