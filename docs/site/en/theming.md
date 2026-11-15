# Theming and dark mode

A theme is a set of colours and the faces for display, body and monospace (`style::THEMES`). Paints are names, not colours: `Paint::INK`, `TEXT`, `DIM`, `FAINT`, `GROUND`, `ACCENT` and the hues `Paint::hue(i)`, so the same view is right in every theme. Dark and light are two themes of the same view: switch with the top bar's Theme menu, `Msg::Theme`, the render tool's `theme`, or `<lab-view theme="dark">`.

## What a theme holds

| Field | Meaning |
|---|---|
| `ground`, `ink`, `accent` | The page's ground, the strongest ink, the one accent. |
| `hues` | The secondary colours for planes and charts. |
| `display`, `body`, `mono` | The faces for each role (`text::SANS`, `PIXEL`, `MONO`, `GEIST_PIXEL`). |
| `sizes` | The type scale: display, title, body, small. |

## Restraint

One accent per page. Dark themes do best with a near-black ground and a single saturated accent; the readout and the film's slate show the style.
