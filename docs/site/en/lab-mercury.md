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
| `r₀` | 22 px | 4–40 | the pointer's ball |
| `r` | 16 px | 4–40 | the drifting balls |
| `k` | 120 | 10–600 | spring stiffness |
| `c` | 14 | 1–60 | spring damping |

## How it is made

`field::Balls` is a scalar field: the sum of rᵢ²/dᵢ² over three drifting balls and the pointer's ball. `field::Grid::sample` samples it, `Drawing::contours` draws the surface at 1 and a faint outer line at 0.6, and `Drawing::dots` fills the inside with dots that grow as the field rises.

The pointer's ball rides a spring (`k`, `c`); when the pointer leaves, it springs back to the middle with the speed it had. Near the pointer, lines and dots take the accent. The formula line also shows the field's value at the pointer, which passes 1 as you enter a ball. The code is `examples/lab/mercury.rs`.
