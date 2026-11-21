# 1.22 自行运动的记号：脱落和流场

`src/motes.rs` (no GPU). Both are data on elements; the runtime moves them and draws them as shape instances
(small squares in ink, `Response::Ignore`, above the page and below the annotation layer):

- **Shedding** (`El::shed(Shed { rate, life_ms, gravity, drift, size })`, by default `rate 0`, 900 ms,
  240 px/s², 30 px/s, 2.5 px): when the element's rect center moves d px between two `tick`s, `rate × d / 100`
  marks are released from its edges (between the old and new positions; fractions accumulate), each with its own
  sideways drift, falling by `gravity` and fading out within `life_ms`. Nothing is released while the element is
  still, and with no marks there is no cost (`Ui::motes_moving` is false, no frames wanted). With `Shed.clear`
  above 0 (default 0, off) marks avoid text: none are released within `clear` of any line of text
  (`line_boxes`), and a mark falling toward a line fades by distance within `clear` and is gone a pixel before
  touching it; in `tests/motes.rs` the dragged sign uses `clear` 10 (measuring every mark's distance to every
  line, frame by frame).
- **Flow fields** (`El::flow_field(FlowField { scale, speed, density, size, alpha })`, by default 140 px,
  0 px/s, 6 per ten thousand square px, 2 px, 0.4): the element's box holds `density × area / 10000` marks
  (positioned by hash, so the same frames give the same bytes), moving at `speed` along the direction of a fixed
  continuous noise field (value noise turned into an angle, two turns), entering from the opposite edge when
  they leave the box. At `speed` 0 they don't move and want no frames; the page sets it, so "no frames at rest"
  holds whenever the page stops it.

Acceptance: unit tests in `motes.rs` (the field is continuous, nothing moves at speed 0, counts);
`tests/motes.rs` (a page with its own text and a draggable sign): a still sign sheds nothing, dragging 185 px
sheds at least 4, they fall, all are gone at the end of their life, no frames are wanted afterwards, and none
comes closer than `clear` to a line of text; a flow field with speed has moving marks, and at speed 0
`motes_moving` is false with 0 frames in two seconds.
