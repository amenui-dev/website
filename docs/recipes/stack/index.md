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

Each card has its own spring with velocity hand-off (DESIGN.md §1.24): when the pointer moves to another card before the fan has settled, the springs keep the speed they had and turn toward the new target instead of starting again from rest. Card *n* starts *delay × n* after the first, so the fan opens from the top card down and closes the same way.

The cards are drawn in the lab's look, near the ground with a hairline outline; the card under the pointer takes the accent. Turning `falloff` to 1 gives every card the same angle, and a `delay` of 0 moves them all at once. The code is `examples/lab/stack.rs`.
