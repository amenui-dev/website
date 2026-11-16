# Loupe: text on a sphere

A block of text wrapped round a sphere. A glyph's place in the block is a longitude and latitude; the sphere turns, the glyph goes where its point projects, squeezed by the slope, dimmed with depth and left out on the far side. Drag to spin; let go and it coasts back to its idle turn.

## Formula

```
λ = 2π(x − ½) + θ,  φ = 2.6(y − ½);  screen = c + R·(cos φ sin λ, sin φ);  ω → ω₀ at e^(−μt)
```

## Parameters

Every number is a number control under the cell (type, arrow keys, drag sideways); none is a slider.

| name | default | range | what it is |
|---|---|---|---|
| `R` | 66 px | 30–120 | sphere radius |
| `ω₀` | 0.30 rad/s | 0–2 | idle turn |
| `μ` | 1.6 /s | 0.1–8 | how fast a spin settles |

## How it is made

`El::pose` with `dx`, `dy`, `sx`, `sy`, `tone` and `alpha`; `GlyphAt::span` gives the block. The code is `examples/lab/loupe.rs`.
