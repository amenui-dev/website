# Right to left

A row lays out from the right when its direction is right to left: `El::rtl(true)` on the row (or any ancestor), or a locale that reads right to left. Taffy's `Direction::Rtl` does the layout; text runs are shaped by parley in their own direction, so mixed Latin and right-to-left text in one paragraph stays in logical order.

## What follows the direction

| Thing | Right to left |
|---|---|
| A row's children | From the right edge. |
| Padding and margins named start and end | Mirrored. |
| A text's alignment | Starts at the right. |
| Icons that point (arrows, chevrons) | Mirrored by the component, not by the framework. |

## Testing

`tests/i18n.rs` checks that a right-to-left row starts at the right; a component page's RTL section shows its first example in a right-to-left row.
