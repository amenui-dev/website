# ground truth


W-03 in Works, at dusk (18:40). Five ridges stand one behind another under a low sun, and a survey is laid over them: a dashed level line at the sun's height, spot heights on three real summits, and boxes around whatever moves. Every 40 seconds a flight of seven birds crosses from right to left in 16 seconds, each bird boxed and trailing three fading echoes. A small pale cloud wears a fainter box labelled `BIRD 0.03`: the one wrong call. The pointer gets surveyed too. A scan window 300 × 190 px follows it, and inside the window the same hills are drawn as characters, the way the system sees them. The pointer's own box is labelled `HUMAN?`, and the score rises while the pointer moves the way a hand does.

## How it is made

| Part | amenui |
|---|---|
| The hills, sky and sun | one picture computed in code at start-up (960 × 600, value noise per ridge, mixed toward the sky with distance), shown with `El::picture` at pitch 1 |
| The survey, birds and boxes | one `Drawing` per frame: dashes, leaders and four-stroke rectangles on the pixel grid, with labels as small `text` elements placed absolutely |
| The scan window | 14 rows of cells, each row one line of Departure Mono with its characters set a cell apart by tracking; `El::pose` gives every glyph its own cell's colour, mixed 0.62 toward the survey ink |
| The pointer's box | a second `Drawing` and a label with `recompose(false)`, so the number changes in place without a layout |

Each cell takes a character from the ramp `. : - = + * #` by the luminance of the picture under it. There are no blank cells: the lightest two eighths are both `.`. The characters are worked out again only when the window moves, the pointer's box moves, or the ground changes, and between those times the rows keep their layout (`recompose(false)`). The cells under the pointer's box and its label are left empty. The title and the number are drawn at full strength in every state; only the small line, the strip and the status line fade. A test holds every character's darkest pixel to at least 3 : 1 against the median of its cell. Everything drawn over the picture sits in a layer of its own. Its ink replaces the picture's, so the survey is never mixed into the colours of the hills.

## What it costs

Between flights the piece asks for no frame at all, and the flight itself is paced at 30 frames a second. The scan window follows the pointer on a 120 ms spring, and only while the pointer moves. After 60 seconds without input, the flight under way ends and the piece rests until the next input. With reduced motion it is still: the birds stay halfway across, boxed, without echoes.

Measured at 1440 × 900 and 2× in headless Chrome on SwiftShader, a software renderer (not a GPU), so a real machine does better:

| A frame during a flight | Mean | 95th percentile | Worst |
|---|---|---|---|
| The scan window still | 2.17 ms | 2.9 ms | 3.3 ms |
| The pointer moving | 2.31 ms | 4.1 ms | 8.9 ms |

Layout takes 0.6 to 0.9 ms of a frame during a flight (measured natively).

At rest the piece draws 82 frames a minute, all of them during flights and none between. A frame holds 760 instances at 1440 × 900 and 412 at 390 × 844. To a screen reader the scan window's characters are one picture, named by its label.

The code is `src/gallery/works.rs` in the website repository. The annotation layer that the site's other pages use for frames and callouts is described in [Annotation](#docs-annotation).
