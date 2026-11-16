# Spring

A mass on a spring, the same `gesture::Spring` the sheets use, solved exactly. Drag it along its track; let go and it springs back with the speed it was thrown at, its last two seconds traced beneath.

## Formula

```
m·x″ = −k(x − x₀) − c·x′,  ζ = c / (2√(km))
```

## Parameters

Every number is a number control under the cell (type, arrow keys, drag sideways); none is a slider.

| name | default | range | what it is |
|---|---|---|---|
| `k` | 180 | 10–1200 | stiffness |
| `c` | 6 | 0–80 | damping |
| `m` | 1 kg | 0.2–5 | mass |

## How it is made

`gesture::Animated::hold` follows the finger; `to(x₀, v, now)` hands the throw to the spring. The code is `examples/lab/spring.rs`.
