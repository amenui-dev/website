# touch grass

W-01 in Works, at noon (12:00). A straw field seen from standing height, side on, flat to a level horizon under a pale sky with one thin high cloud. On the horizon a mast carries an orange windsock and a note: the wind, and how long until the next gust. The sock hangs at rest and lifts while a gust passes the mast, level in a strong one. The field is 1,570 stalks in 15 rows at 1440 × 900 (392 on a phone); the far rows are ticks, the near ones are taller than the title, and the stalks in the nearer half carry a head that is a glyph from `' " ; : ! i l ,`. Each stalk has a length of its own and leans a little downwind at rest. No stalk stands within 24 px of the title, its number or the small line, and no head within 8 px of the links at the foot, the status line or the note at the mast. On a phone the 24 px hold for the title and its number only: the small line's two lines keep heads 8 px away, and a stalk whose head would come nearer than that is planted without it. Wind comes as gusts with stillness between them. A gust is a band that crosses the field from left to right, the near rows behind the far ones, so the band runs diagonally; inside it the stalks lean and show their pale side, and behind it they sway back through upright once. Every fourth gust is strong. The pointer walks through the field: the stalks near where it has been lie down away from it and take 6 seconds to stand up again, the far end of the path first.

## How it is made

| Part | amenui |
|---|---|
| The sky, the cloud and the ground | one picture computed in code at start-up (960 × 600, the horizon at 64 % of its height, 62 % on a phone), shown with `El::picture` at pitch 1. The ground has long streaks: value noise of four octaves, ± 18 of 255, its period (30 + 300 d) px wide and (3 + 22 d) px high at depth d |
| The stalks | one `Drawing` of `Mark::Blade`s: each rooted on its row, 5 + 24 · t^1.4 px apart at depth t with one in six left out, its row's length × (0.80 + 0.35 h) long and leaning 0.06 ± 0.06 rad at rest (`lean`, which the map's lean adds to), `width` 0.75 at the far row to 1.5 at the near one, `max_lean` 72°. Upright it is `#7b6a2c`, mixed 55 % toward the ground in the far rows; leaning it turns toward `#fff6c8` at full strength at every depth, the pale side: not at all up to 8° of lean, wholly from 20°, a smoothstep between (`lean_from`, `lean_full`), so a stalk at its own lean or swaying back is its upright colour. A head is the blade's `Head`, a glyph of Departure Mono on its tip in `#5d4f1e` |
| The gusts | `Cmd::Gust`: a band 330 px wide at 240 px a second, its far side 500 px ahead of its near side along the depth's 0.53th power (on a phone 40 % and 45 % of the window's width at most: 156 and 175 px at 390), `strength` 0.79 (57°), 1.0 for a strong one (72°), with its wake behind it |
| The path | `Cmd::Stir` at the pointer, each side's flow away from it, over 10 + 60 · t px at depth t, each with three times the look's flow; `El::map_life(8, 8)` on the field, so 3 seconds after the pointer the stalks on its path still lean about 20° and show their pale side, by 6 seconds they are under 8° and upright in colour, and at 8 they are at rest. A pointer that moves more than 320 px in one event starts the path again where it lands, with nothing stirred between |
| The windsock | its own small drawing, 22 px long, 8 px wide at the mast and 4 at its tip; 55° below the horizontal at rest, lifted to 55° · (1 − f) by the gust's value f at the mast's foot, on the frames the gust draws anyway |
| The windsock's note | two labels and a leader; the countdown changes in place without a layout (`recompose(false)`) |

The blades are bent on the GPU from the field map at their roots: while only the wind moves, nothing is laid out, painted or built on the CPU. A gust is never put into the map; it is worked out from the map's clock and laid over it, so once it has passed, the map is exactly as it was and no frame is asked for. What the pointer stirs goes into the map with a layout, at most 30 times a second. The map reads the scene's ink for `creep` only where an instance changed, and never for the blades or anything inside the field's own life, where nothing creeps.

## What it costs

What moves by itself is paced at 30 frames a second (`Ui::ambient_fps`), input is not. Between gusts the only frames are the countdown's, one a second. After 60 seconds without input the gust under way ends and the piece rests until the next input. With reduced motion it is still: one gust band held with its centre 72 % across the far row, from the mast down to the foot, and the sock lifted as at the band's peak, 55° · (1 − 0.79), about 11.5° below the horizontal.

Measured at 1440 × 900 and 2× in headless Chrome on SwiftShader, a software renderer (not a GPU), so a real machine does better:

| A frame | Mean | 95th percentile | Worst |
|---|---|---|---|
| The field at rest and in gusts | 3.3 ms | 7.7 ms | 15.3 ms |
| The pointer walking | 4.6 ms | 9.4 ms | 13.6 ms |

Counting only the frames a gust draws, the mean is 2.35 ms and the 95th percentile 5.1 ms. The first frame, which lays out and paints the page, takes 89.5 ms. A pointer that jumps more than 320 px in one event stirs nothing between, so a jump costs no more than a step. Natively a frame during a gust takes 1.1 ms of CPU, a pointer event 2.4 ms (a layout every other event), and a jump of 800 px 2.1 ms.

The piece opens with its first gust already 72 % across, so its first minute at rest draws about 1,100 frames: about 160 in the first 5.5 seconds as that gust finishes, then 30 a second during each crossing of about 10 seconds and one a second between. A frame holds 6,348 instances at 1440 × 900 and 1,482 at 390 × 844, every blade and head counted; when frames are slow the drawing is thinned evenly.

The code is `src/gallery/touch.rs` in the website repository. The field map is described in [Ripples](#docs-ripples).
