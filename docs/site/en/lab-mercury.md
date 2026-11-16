# Mercury

Metaballs: the surface where Σ rᵢ²/dᵢ² reaches 1, drawn as its contour with the inside filled in dots. One ball follows the pointer on a spring; near the others they merge, pulled away they split.

## Formula

```
Σ rᵢ²/dᵢ² = 1;  m″ = k(p − m) − c·m′
```

## Parameters

Every number is a number control under the cell (type, arrow keys, drag sideways); none is a slider.

| name | default | range | what it is |
|---|---|---|---|
| `r₀` | 17 px | 4–40 | the pointer's ball |
| `r` | 12 px | 4–40 | the drifting balls |
| `k` | 120 | 10–600 | spring stiffness |
| `c` | 14 | 1–60 | spring damping |

## How it is made

`field::Balls` is a scalar field; `Drawing::dots` and `Drawing::contours` draw it. The code is `examples/lab/mercury.rs`.
