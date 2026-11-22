# Flow: text on a path


Four lines of text on a travelling wave, with a bump the pointer pulls into them. Every glyph sits on the curve turned to its tangent.

## Formula

```
y(x) = A·sin(2πx/λ − ωt) + (m_y − y₀)·e^(−(x − m_x)²/σ²);  angle = atan y′(x)
```

## Parameters

Every number is a number control under the cell (type, arrow keys, drag sideways); none is a slider.

| name | default | range | what it is |
|---|---|---|---|
| `A` | 9 px | 0–40 | wave height |
| `λ` | 150 px | 20–400 | wavelength |
| `ω` | 1.6 rad/s | 0–8 | wave speed |
| `σ` | 40 px | 6–160 | width of the pull |

## How it is made

`El::pose` gives each glyph a vertical offset `dy` and an `angle`: the glyph's instance carries its own transform (`Instance::xf`), so it is drawn turned with its true outline, not as a rotated picture. The angle is the arctangent of the curve's slope at the glyph's centre.

Four lines spread down the stage, each riding the wave a little behind the one above. Near the pointer, the nearest line's baseline is pulled toward it with a Gaussian of width `σ`, and the pulled glyphs take the accent. An `A` of 0 flattens the wave and leaves only the pull. The code is `examples/lab/flow.rs`.
