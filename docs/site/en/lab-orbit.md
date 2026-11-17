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

Ninety dots sit on four elliptical rings. Each one's angle advances at its Kepler speed, so the inner rings lap the outer ones; on top of that every dot carries an offset with its own velocity, pushed by the pointer's pull and held by a damped spring (`k` and `c`). The `+ 64` in the pull keeps it finite when the pointer is right on a dot.

When the pointer leaves, the pull fades and the springs bring every dot back onto its ring. Dots near the pointer take the accent. Everything is one drawing of dots and ring strokes (`Drawing::dot`, `Drawing::polyline`). The code is `examples/lab/orbit.rs`.
