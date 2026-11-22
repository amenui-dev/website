# 1.32 The beat

`Look::beat: beat::Beat { bpm, steps, swing }` (a preset's `interaction.beat`): `bpm` 40–300, 0 is off (the
default); `steps` a beat 1–8 (4); `swing` 0–0.5 (0), how much later every second step starts, in steps. A bar is
four beats. With a beat, what moves on its own falls on its steps: a front's radius, glitchr's held rows, scribblr's
held pose, a transition's steps and a morph's flight are drawn at the start of the step the beat clock is in
(`Ui::beat_grid`); line widths, opacity and condensing are not rounded (the shader's condensing reads the clock off
the grid). Off, every clock is the runtime's own, byte for byte as before the beat.

The beat clock runs with the runtime's (`Ui::beat_time`; `Ui::beat()` is `BeatNow { bar, beat, step, phase }`,
each from 1, for the application's own state) and only the beat's commands move it against it: `Cmd::Hold(n)` stops
it for `n` steps (the frame stays byte for byte, and frames are still wanted: the scene presented is the one the hold
began with, whatever the application changes meanwhile, and after it the page is drawn from the state as it is then), `Cmd::Retrigger` plays the current
step again from its start, `Cmd::Rewind(beats)` takes it back (never before 0) and with it what it carries (fronts,
transitions, held rows); the application's state, the field map, condensing and the materials' own clocks (the
ground's drift, scribblr's pose) are not taken back. With a beat, text to read is not pushed by fronts (a push a step
at a time reads as the page shaking), only loosened up to `legible_cap`; without one it is pushed as before. `Cmd::Set {
name, value }` sets a number of the look over the application's (`Look::set`: `loose_scale`, `legible_cap`,
`loose_max`, `press_ms`, `fx.*`, `beat.*`, a material's number as `<material>.<name>` in `MAT_PARAMS`, clamped to its
range); a switch (`post.invert`, `post.binary`) given 1 turns over and given 0 goes back to the look's own.
`Ui::command(cmd)` sends any command from outside the application; a script schedules them (§1.17, `kind: "cmd"`,
`at: "bar.beat.step"`). There is no audio, no beat detection and no video: the beat comes from the script, and the
result is a run of PNGs.

`Look::hold_ms` is gone: a held pose of `h` ms is a beat of one step `h` long, at the fewest steps a beat that keep
`bpm` in range (`Beat::of_hold`; 83 ms is 240.96 bpm in 3 steps). Recipes are `look:v3:`; a `look:v2:` recipe is
refused, with the beat its `interaction.hold_ms` becomes. Acceptance (`tests/beat.rs`): at 120 bpm and four steps a
front's radius moves a whole step or not at all between frames; frames during `Hold(2)` are identical; the frame
after `Rewind(1)` is the frame drawn a beat before; `Set` of `post.invert` once turns every pixel over (within
1/255), twice gives the frame back; an event at `"3.2.1"` and one at 4500 ms give the same frame; `bpm` 0 with any
steps and swing, and the beat's commands, change nothing.
