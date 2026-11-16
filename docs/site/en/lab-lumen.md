# Lumen: light on text

A light over a paragraph. Every glyph is drawn whole; only its tone follows the light's falloff, so text the light has left is dim but complete letters.

## Formula

```
L(d) = L0 + (1 − L0) · e^(−d²/r²)
```

## Parameters

Every number is a number control under the cell (type, arrow keys, drag sideways); none is a slider.

| name | default | range | what it is |
|---|---|---|---|
| `L0` | 0.22 | 0.05–1 | tone at rest |
| `r` | 46 px | 8–160 | the light's radius |
| `ω` | 0.35 rad/s | 0–2 | speed of the idle path |

## How it is made

`El::pose` asks each glyph's pose from its centre; the tone is multiplied, the outline untouched. The code is `examples/lab/lumen.rs`.
