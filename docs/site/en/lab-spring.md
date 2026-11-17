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

`gesture::Animated::hold` makes the mass follow the finger while it is pressed, and a velocity tracker measures how fast it was moving. On release, `to(x₀, v, now)` hands that speed to the spring, so a throw keeps its momentum instead of stopping dead. The spring is solved exactly at each frame, not stepped, so it is the same at any frame rate.

The formula line shows the damping ratio ζ as it changes: below 1 the mass overshoots and rings, at 1 it returns as fast as it can without overshooting, above 1 it creeps back. The trace under the track is the last two seconds of position. The code is `examples/lab/spring.rs`.
