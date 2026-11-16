# Stack

The card stack (`widgets::card_stack`) in the lab's look: hover or press a card and the ones beneath fan out.

## Formula

```
angle(n) = angle0 × falloff^n,  delay(n) = delay × n
```

## Parameters

Every number is a number control under the cell (type, arrow keys, drag sideways); none is a slider.

| name | default | range | what it is |
|---|---|---|---|
| `angle0` | 42° | 0–90 | first card's turn |
| `falloff` | 0.70 | 0.1–1 | each next one's share |
| `delay` | 28 ms | 0–200 | stagger |

## How it is made

Each card has its own spring with velocity hand-off (DESIGN.md §1.24). The code is `examples/lab/stack.rs`.
