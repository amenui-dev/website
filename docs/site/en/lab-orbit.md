# Orbit

Dots on circular orbits at their Kepler speeds, pulled off their paths by the pointer and sprung back when it goes.

## Formula

```
ω(r) = ω₁·r^−1.5;  δ″ = G(m − p)/(|m − p|² + 64) − kδ − cδ′
```

## Parameters

Every number is a number control under the cell (type, arrow keys, drag sideways); none is a slider.

| name | default | range | what it is |
|---|---|---|---|
| `ω₁` | 1.20 rad/s | 0–4 | speed of the innermost ring |
| `G` | 9000 | 0–40000 | pull |
| `k` | 26 /s² | 1–200 | spring back |
| `c` | 5 /s | 0–30 | damping |

## How it is made

A drawing of dots and ring strokes (`Drawing::dot`, `Drawing::polyline`). The code is `examples/lab/orbit.rs`.
