# 1.17 面向模型的接口

This framework is written by models: a model can't see the window, so it needs deterministic pictures,
readable structure and errors that say what is wrong.

**`render`** (`src/bin/render.rs`, its core in `drive::render`, which the tests call directly):
`render --page <page or recipe-name> --size 1280x800 --scale 1 --material <id> --theme <id> --at <ms>
--events <file.json> --out out.png [--tree out.json] [--strict]`. In this repository the pages are the recipes
(`recipe-form`); the website's `render` also opens its own pages (`button`, `material-dottr`, `scene-morph`).
The clock is fixed: from 0 in steps of 1/60 s up to `--at`, each event put in at its time or the first step
after it (in the order given), ending exactly at `--at`; there are no random numbers (every hash has a fixed
seed). So the same arguments give a byte-identical PNG (`tests/render.rs`: the same renderer twice and a new
renderer once give three identical images; another moment gives a different one). `--events` is a JSON list of
`{at_ms, kind, ...}`, `kind` being `press` / `release` / `move` (`x`, `y` in logical px, or `target`: an
element's key, using its box's center), `key` (a DOM key name, with `shift`, `cmd`, `alt`), `wheel` (`dx`,
`dy`), `text` (`text`) or `cmd` (`cmd`: a command for the runtime, `drive::CmdSpec`: `{"ripplr": {x, y,
strength}}`, `{"ripplr_at": {key, strength}}`, `{"stir": {x, y | key | path, from, to, v, energy, radius}}`,
`{"press": {key}}`, `{"commit": {x, y}}`, `{"hold": n}`, `"retrigger"`, `{"rewind": beats}`, `{"set": {name,
value}}`). In place of `at_ms` an event may give `at`, a place on the look's beat (`"3.2.1"`, §1.32); without a
beat that is an error. `--frames <from>..<to>` writes a run of frames instead of one (`--out dir/%05d.png`) at
`--fps` 60, 30, 20, 15 or 12 (each a whole number of the clock's ticks; another rate is an error listing these):
frame `n` is the frame `--at n × 1000 / fps` gives, and with `--audit` each frame is audited
(`drive::frames`, `drive::audit_frames`; `tests/frames.rs`). `--material` is a material name or number; `--theme` is one of `drive::THEME_IDS`
(dark, light, paper…), a theme name or a number. Problems in the view are printed to standard error, and with
`--strict` the exit code is 1.

**The tree export** (`Ui::export_tree() -> tree::TreeNode`, `serde::Serialize`): each element's key (and
whether the view wrote it), kind (text / image / control / shape / box), role and name from the accessibility
mirror, rect and visible part, text, material (`global` or the override's name), response, legibility, ripple
scope, recomposition, focusable / focused / hovered / pressed / disabled, scroll offset and content length, how
loose its loosest instance is right now (by `Ui::looseness`'s rules), and its children. `tests/tree.rs` checks
rects, kinds, roles, responses, the JSON shape and focus on a card at two widths.

**Errors written for their reader** (`src/error.rs`, `thiserror`): `FontError` (which file couldn't be read,
that the fonts are in `assets/`, how to write the path), `LookError` (an unknown material / theme / page,
listing what can be written), `SurfaceError` (a zero size, installing lavapipe when there is no GPU adapter, a
size mismatch), `DriveError` (the event JSON's format, a target not found with the nearest keys, a missing
position, key names) and `ViewProblem` (`Ui::problems()`: the same key used twice, giving both paths and the
`.key(format!(..))` way to write it; the same flow id twice on a page; an anchor to an element that doesn't
exist; a ripple container key that doesn't exist). There is no bare `unwrap` on any path user code can reach;
`Offscreen::new` panics with `SurfaceError`'s message on failure, and `Offscreen::try_new` returns it. Every
`Offscreen` in a process draws with one instance, adapter and device, made on first use (each keeps its own renderer
and target), because making and dropping Vulkan instances on several threads at once crashes lavapipe. The
`expect`s that remain are internal invariants (taffy creating nodes, the renderer's blank images).

**Documentation**: `AGENTS.md` at the repository root (three sentences, the mental model, a compiling example
for each of the ten things you write, which is the doc test `amenui::agents` so `cargo test` compiles every
one, the rules, how to verify with `render` and `--tree`, commands, where things live); `llms.txt`; the recipes
in `docs/recipes/` with their code in `examples/`, compiled into the library (`amenui::recipes`,
`autoexamples = false` in `Cargo.toml`); `tests/recipes.rs` checks that each recipe page's code block equals its
file exactly, that each recipe lays out without view problems, that its own interaction does what it says, and
that it renders. Every component file starts with Purpose / Props / States / Messages / Usage; each
`registry.json` item has `response`, `recompose` and, when there is one, `recipe`; `tests/registry.rs` checks
that every file in `src/widgets/` is listed with these five.
