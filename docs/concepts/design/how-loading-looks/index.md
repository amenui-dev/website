# 1.23 How loading looks

`El::skeleton()` (inherited by the subtree): the subtree is laid out as usual, and when drawn each line of text is
a bar as wide as that line measures (`Text::line_boxes`) and `SKELETON_BAR` (0.56) of the line height tall, with
round ends; pictures and 3D elements are blocks; fills and borders fade; it takes no input, and doesn't fade as a
whole the way disabled does. Bars and blocks have identities of their own, so when the content arrives the text
condenses by the framework's rules and the bars disappear; the layout is the same tree with the same text, so not
one box moves by a pixel.

The rules are numbers (`widgets::Loading { skeleton_after_ms 300, swap_stagger_ms 40 }`): what doesn't depend on
the data is drawn at once; a skeleton appears only when the data is later than `skeleton_after_ms` (a quick answer
never flashes a skeleton; before that the rows are `hidden_until_morphed`, holding their place and drawing
nothing); once the data arrives, row i becomes content at `arrival + i × swap_stagger_ms`. `Loading::show(i, now)`
says what a row is, `next_change` gives the next moment of change, and the application's `on_wake` wants frames
only at those moments.

The website's skeleton page compares four approaches: a spinner (eight dots lighting in turn, a step every
100 ms), a skeleton, a progress bar (moving to 95% over the expected time) and an optimistic update (rows are
there at once, the newly added one faint until confirmed). Acceptance: unit tests in `skeleton.rs`;
`tests/loading.rs`: before 300 ms the rows have no instances while the button is drawn; during the skeleton each
heading has as many bars as lines, each bar as wide as its line (±1); when the data arrives row 0 is text and row
4 still skeleton; every row's box is equal in skeleton and content; no frames wanted afterwards. Samples
`craft <dir> loading`.

**The warm-up and the application's own preparation.** The first frame is a readable page you can use: what takes
long to make comes in over the next frames. `Renderer::begin(device, queue, format, w, h) -> Warmup` makes at once
only what plain needs to draw (the main shader, the ink, nearness, material, annotation and copy pipelines).
`Warmup::step(budget_ms, text)` (or `Renderer::warm_step` on the renderer it hands over) makes the rest in a fixed
order, at least one piece a call and more while the budget lasts: the mark atlas one set at a time
(`marks::Atlas::begin`, `AtlasBuild::step`, `finish`), the finishing pass, then the reduced-resolution upsample,
a scroll move's trace pass (`fs_trace`: compiled on the first fast scroll instead, it stalled that frame for
seconds), the morph tiles' and the mesh pass's pipelines. Each material pipeline is drawn with once as it is made, into one
pixel of a scratch target (`Renderer::prime`): a driver that finishes a pipeline on its first draw does it then,
not on the frame that first needs it (on the CPU rasteriser the first frame of the site's dark scheme, the first
with its bloom, took 0.9 s against 0.4 s for each later one); `Progress { done, total, stage }` says how far it is
(`Stage::Plain`, `Pipelines`, `Ready`). Until the atlas and the finishing pass are made, frames are drawn in the
plain look (`render::plain`: material plain, no finishing pass, no material overrides). Rule 3 of §0 says a page at
rest is plain, so this is the same page arriving early, not a lesser one; on the frame the look is ready, the page
arrives at rest and then comes apart a little and condenses (`Ui::condense_all`): over `ui::RAMP` (120 ms) each
element loosens from 0 to its cap, then condenses from there with the preset's own timing. Text to read and
controls (`legible`) do not move: their cap here is 0, so they are born condensed, and under every material their
ink in the frames after arrival is the same as at rest; only headings, decoration and pictures come apart and
condense. The frame the material arrives in is the
page at rest, so it differs from the plain frame before it only where the material has a look of its own at rest
(`tests/warmup.rs`: under 2 % of the pixels for `dottr`, whose ground has dots; for seven materials, dark and light,
over 99 % of the pixels in each legible element's ink box, grown by 2 px, are the same at 0, 60, 120 and 300 ms
after arrival as in the plain frame, or as at rest where the material has a look of its own at rest), and no element's looseness changes
by more than 0.2 from one frame to the next at 60 Hz. `Renderer::new` is the same thing
done in one go, for offline rendering and tests, with the same bytes; `Offscreen::try_warming` is an offscreen
target that warms up. The hosts call `Renderer::warm_frame(ui, WARM_MS)` once a frame drawn, and `Renderer::warm_idle`
in idle time (§1.9): it makes pieces until
6 ms are spent, and stops early rather than start a piece whose kind has before taken more than 1.5 × the budget left
(a kind not yet timed counts as too long, so after the first piece of a frame it waits for the next). It tells the
runtime how far it is (`Ui::warm`; `Ui::on_warm(fn(&mut S, Option<Progress>) -> bool)` hands it to the state, `None`
once ready); the last step reports `done = total` with `Stage::Ready`.

An application prepares its own one-off work the same way (`src/warmup.rs`): `Ui::prepare(key, job)` takes a job
that does one small piece on the state per call and returns how much is done (0..1). The runtime calls each job at
least once a frame while it runs, then again while the frame's spare time lasts (`Ui::set_prepare_budget`, 6 ms by
default); a page with jobs running wants frames. `Ui::ready(key)` reads a job's progress (1 when finished or never
asked for) and `Ui::readiness()` the least ready of the jobs and the renderer. The element being prepared is drawn
as its own skeleton (`El::skeleton()`), measured to the size it will have, so it arrives without moving.

The rules of how loading looks: never a spinner or a progress bar as the main thing on the page; the first frame is
the plain page, readable and usable; progress is shown only as a readout of the annotation layer, `READY 0.62`
beside `LEVEL` (§4.3), and only while something is not ready.

Acceptance of the warm-up: `tests/warmup.rs` (before any step the frame equals the plain look's frame byte for
byte; after the last step and the condensing it equals a frame of `Renderer::new`; at most 32 steps, none over a
fixed 3 s on lavapipe; a prepared job runs a piece a frame, its progress only goes forward, and the page rests
afterwards).
