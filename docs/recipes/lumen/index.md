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
| `L0` | 0.16 | 0.05–1 | tone at rest |
| `r` | 54 px | 8–160 | the light's radius |
| `ω` | 0.35 rad/s | 0–2 | speed of the idle path |

## How it is made

The text is one block of small monospaced type. `El::pose` asks each glyph's pose from its centre, and the pose returns only a tone: the glyph's colour is multiplied by `L(d)`, while its outline and position are untouched. That is why the dim text is still readable letters, not dissolved ones.

The light follows the pointer on a spring. When nobody is there, it wanders over the text in a slow figure of eight at speed `ω`. Where the pointer holds the light, the brightest part of the beam takes the accent. `L0` is the tone everything falls back to; at 1 the light has no effect. The code is `examples/lab/lumen.rs`.
