# 1.31 检查

One command says what is wrong with a page as drawn: `render --page <name> --audit` prints a list of findings, most
serious first, and with `--strict` a P0 or P1 is an error. `amenui::audit::audit(ui, px, ground)` takes a
deterministic frame's pixels, the same frame drawn with every glyph hidden (the ground under the text, before the
finishing pass) and the runtime that drew them, and returns `Vec<Finding>` (`level`, `key`, `kind`, `measured`,
`limit`, `fix`); `drive::audit(spec, faces, o)` renders a page and audits it, `drive::audit_with` for an
application's own pages (`Driven::audit`). The view's structural problems (`Ui::problems()`) are on the same list as P1 `Structure`.
Everything is measured; each `fix` is one sentence to act on, naming the element and the number.

| kind | what is measured | level |
|---|---|---|
| `Clipped` | text to read whose ink (`Instance` boxes after poses and turns) is cut by a clip, the window or a reveal; or a word drawn in part | a whole glyph or more, or a word in part: P0; less than a glyph: P1 |
| `Contrast` | per line of text, the strongest ink pixel against what is drawn under it (WCAG 2) | text to read below 3:1 P0, below 4.5:1 P1; a lone mark (no letter or digit, such as › or −) below 3:1 P1; text not to read below 3:1 P3 |
| `Overlap` | the share of a glyph's ink covered by another drawn glyph, in one element or between two to read | text to read over 10 %: P1; posed text over its `overlap` + 0.05: P2 |
| `Small` | the drawn height of text to read (size × scale × the pose's vertical scale) | under 9 device px: P1 |
| `Eroded` | at looseness 0, the ink pixels (differing from the ground by at least 24 and by at least half the line's contrast) inside each text-to-read ink box, against the same page drawn in `plain` | under 85 % of plain's: P1 |
| `Restless` | after the last input, the clock run on at 60 Hz: still asking for frame after frame 2 s later with nothing visible marked `El::ongoing()` (a timed wake now and then, such as a caret's blink, is not restless) | P1 |

Posed text dimmed by depth on purpose is read by its nearer half; the smallest size of posed text is its 10th
percentile among the glyphs drawn at half opacity or more. A clipped finding says which side is cut, how far
the ink reaches past it (px) and what cuts it (the clipping element's key, or the window), and suggests
`reveal_by(Reveal::Word)` when a reveal cuts a word. The eroded check guards materials with a look of their own at
rest (`near`, `charr`, and materials added from source); one whose rest is plain passes by construction.

The annotation layer shows what the audit measures: `gaze::INK` draws the true ink's bounding box of an element
(after transforms and poses) dashed, with a leader dashed 4 px on, 4 px off from a corner of that box to the
nearest point of the drawn outline (of a shape's true outline too), never outside the box: of the corners 12 px clear
of text to read, the one farthest from the outline, the line kept round text to read (`strand::clear_path`). A
leader shorter than 16 px (two dashes) is not drawn, so a box that hugs its ink, as text's does, has none. Then an
`INK w×h` label (`hit::ink_of`, `Ui::leaders`), from the same measure the clipping and overlap checks use. The
annotation colour (`Gaze::colour`) is used as given where it stands 4.5:1 from the page's ground; on a ground where
it does not (the light theme) it is the same hue darkened until it does (`Gaze::colour_on`), for the boxes, leaders,
labels and guides alike. Tags
and `INK` readouts never cover text to read or each other: each tries outside the ink box above at the left, then
below, then to the right (`gaze::place_tag`), takes the first place that meets no text-to-read ink box and no other
label, and when none is free only the box is drawn.

What the audit found on the recipes, and what changed for it: `Paint::DIM` was 0.5 of the way from ground to ink,
under 4.5:1 on the light theme (3.2:1); it is 0.66 (5.2:1 light, 6.9:1 dark). The light theme's accent was under
4.5:1 against its ground, both as text and under a solid button's label (3.4:1); it is darker, (0.64, 0.20, 0.035)
at 5.6:1, and the selection tint is 12 % instead of 14 %. A selected list row's note is `TEXT`. The `near` material
drew text as outline and 25 % fill, too faint at text sizes, and lost the outline entirely on a filled panel (the
panel's nearness is max-blended over the glyph's); text keeps its own ink whole there, and text set in the ground's
colour (a label on a solid button) takes the ink's colour, since `near` draws the panel open. The loading recipe's
tick asked for a new view every frame forever; it asks only when a row changes.

Acceptance: `tests/audit.rs` (five pages broken on purpose, each reporting its finding at its level: text past the
window `Clipped` P0, grey on grey `Contrast` P0, two lines over each other `Overlap` P1, 6 px text `Small` P1, an
element that never stops moving `Restless` P1, and not when it is marked `ongoing`; text a material wears away at
rest `Eroded` P1; a calm page has no findings;
the seventeen recipes in every material, dark and light, have no P0 or P1); `render --page recipe-form --audit
--strict` exits 0.

A component's examples are a page too: `render --page component-<name>` lays out `examples/components/<name>.rs`
one example under another (`recipes::components::ui`). In Claude Code the repository's save hook
(`.claude/settings.json`, `tools/audit-hook.sh`) renders, after an Edit or Write, the one page a view file belongs to
(a recipe, a component's examples, or its widget in `src/widgets/`) with `--audit --strict` and hands the P0 and P1
findings and the view's problems back to the model; AGENTS.md is also its `amenui` skill (`tools/skill.sh`,
`tests/skill.rs`).

`Ui::line_inks(key)` reads each line of a text element as drawn: its text range, line box and ink box
(`LineInk`), for tests and for an application that places things by the text's real ink.
