# Silk

Many marks carried by the curl of drifting noise (a flow without sources, so they never bunch), plus a flow grid the pointer's velocity is stirred into and that fades back. Each mark is drawn as the stroke it travelled. The count is a request to the frame budget.

## Formula

```
ṗ = v · s · curl n(p/s) + u(p);  u += v_ptr · e^(−d²/24²);  u ×= e^(−dt/τ)
```

## Parameters

Every number is a number control under the cell (type, arrow keys, drag sideways); none is a slider.

| name | default | range | what it is |
|---|---|---|---|
| `N` | 1600 | 100–20000 | marks asked for |
| `v` | 16 px/s | 0–60 | drift speed |
| `s` | 60 px | 10–200 | noise feature size |
| `τ` | 0.6 s | 0.05–3 | how long a stir lasts |

## How it is made

`field::Swarm::advect`, `field::curl`, `field::Flow::stir`; `budget::grant(N)` says how many this frame may move. The code is `examples/lab/silk.rs`.
