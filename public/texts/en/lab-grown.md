# Grown to shape

Gray–Scott reaction–diffusion confined to a real outline. The shape is an ordinary element, laid out and hidden; `El::on_outline` sends its true coverage as a mask, and the maze pattern grows inside it and stops at its edge. Two letters, a toggle control and a rounded square take turns by themselves every `T` seconds; each change decodes the old pattern away and the new shape in (`El::decode`), and every dot is sized by how much v is there.

## Formula

```
u′ = Du∇²u − uv² + f(1 − u),  v′ = Dv∇²v + uv² − (f + k)v,  Du = 1, Dv = 0.5
```

## Parameters

Every number is a number control under the cell (type, arrow keys, drag sideways); none is a slider.

| name | default | range | what it is |
|---|---|---|---|
| `f` | 0.029 | 0.01–0.08 | feed |
| `k` | 0.057 | 0.04–0.07 | kill |
| `steps` | 24 /frame | 1–40 | steps per frame |
| `T` | 7 s | 2–30 | time per shape |

## How it is made

`field::GrayScott` (masked: outside it u = 1, v = 0); dragging seeds v. The code is `examples/lab/grown.rs`.
