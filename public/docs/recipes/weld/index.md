# Weld


The distance field: each chip is a rounded box with a signed distance, and the outline is their smooth union. Drag a chip near another and the two outlines neck together, with the gap printed between them; let go inside `join` and they are one group, one outline with the labels side by side, moved as one. Pull a member away fast and it tears off. A press without a drag turns a chip on.

## Formula

```
d_i = |max(q, 0)| + min(max(qx, qy), 0) − r,  q = |p − c_i| − (h_i − r)
d = smin(d_1, …, d_6; 2k),  smin(a, b; k) = min(a, b) − max(k − |a − b|, 0)² / 4k
```

## Parameters

| name | default | range | what it is |
|---|---|---|---|
| `k` | 18 px | 0–40 | the blend: two chips 2k apart do not touch each other's outline |
| `join` | 12 px | 4–32 | let go closer than this and the two join |
| `tear` | 56 px | 24–120 | how far a member can be pulled from its place before it parts |
| `bands` | 2 | 0–3 | lines at 6, 12 and 18 px outside the outline |
| `radius` | 6 px | 0–14 | the chips' corners |

`[ scatter ]` puts the chips in new places, each alone. Keys on a focused chip: arrows move it 8 px, Enter joins it to the nearest chip within `join`, Backspace takes it out of its group, Space turns it on.

## How it is made

The union is sampled on a 2 px grid over the chips' bounds (`field::Grid::sample`) and traced by marching squares at 0, 6 and 12 px (`Grid::contours`). The chips are elements: their fills, their labels (selectable text) and their group, which is an element of its own in the tree. Nothing moves but what the hand moves and the spring that takes a chip to its place, so the piece draws no frame at rest. The code is `examples/lab/weld.rs`.
