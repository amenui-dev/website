# ground truth

W-03 in Works, at dusk (18:40). Five ridges stand one behind another under a low sun, and a survey is laid over them: a dashed level line at the sun's height, spot heights on three real summits, and boxes around whatever moves. Every 40 seconds a flight of seven birds crosses from right to left in 16 seconds, each bird boxed and trailing three fading echoes. A small pale cloud wears a fainter box labelled `BIRD 0.03`: the one wrong call. The pointer gets surveyed too. A scan window 300 × 190 px follows it, and inside the window the same hills are drawn as characters, the way the system sees them. The pointer's own box is labelled `HUMAN?`, and the score rises while the pointer moves the way a hand does.

## How it is made

| Part | amenui |
|---|---|
| The hills, sky and sun | one picture computed in code at start-up (960 × 600, value noise per ridge, mixed toward the sky with distance), shown with `El::picture` at pitch 1 |
| The survey, birds and boxes | one `Drawing` per frame: dashes, leaders and four-stroke rectangles on the pixel grid, with labels as small `text` elements placed absolutely |
| The scan window | the same picture again, shown through a `Markr` slot made from the character set (7 × 14 cells, 10 levels), clipped to the window's rectangle |
| The pointer's box | a second `Drawing` and a label with `recompose(false)`, so the number changes in place without a layout |

The scan window is cut into up to four rectangles around the pointer's box, so the box's ink is never read as characters. Everything drawn over the picture sits in a layer of its own. Its ink replaces the picture's, so the survey is never mixed into the colours of the hills.

## What it costs

Between flights the piece asks for no frame at all, and the flight itself is paced at 30 frames a second. The scan window follows the pointer on a 120 ms spring, and only while the pointer moves. After 60 seconds without input, the flight under way ends and the piece rests until the next input. With reduced motion it is still: the birds stay halfway across, boxed, without echoes.

The code is `src/gallery/works.rs` in the website repository. The annotation layer that the site's other pages use for frames and callouts is described in [Annotation](#docs-annotation).
