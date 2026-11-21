# 1.34 Effects that can be turned off

Each effect the framework draws can be turned off from the look, without the application building its view again:
`Look::off: style::Off`, every field `false` (on) by default. An effect that is off goes straight to its end state:
nothing is lost but the motion, every control still works, it asks for no frames and draws no pass of its own. A
switch takes effect on the next frame; a morph, a flow, a recomposition or a decode in flight when its switch goes off
ends where it was going, and fronts in flight stop.

| field | off means |
|---|---|
| `close` | new things are there at once, without condensing (`Fx::enter`) |
| `fronts` | `Cmd::Ripplr` and a commit's front send nothing; `Cmd::Commit` shows the new look at once, with no transition |
| `flows` | `Cmd::Flow` does nothing: the page changes at once |
| `morphs` | `Cmd::Morph`, `Ui::morph` and `Ui::morph_from_last` do nothing: the new picture is there at once |
| `ripples` | a press sends no front: it changes the control and nothing round it |
| `loosen` | a press does not hold what is pressed loose or condense it again after, and no burst runs on press or hover |
| `recompose` | changed text is replaced at once (§1.16) |
| `decode` | `El::decode` text is there at once (§1.28) |
| `material_still` | the materials' clocks stand (their rest pose: no drift, no slips, no wandering thread) and the beat stops |
| `bloom` | no bloom pass (`post.bloom`) |
| `ground` | the field's dots, the ground's own marks (dottr's ground units, smudgr's paper grain), are not drawn: the ground is the theme's plain ground, the same under fronts, the pointer and the field map; a material copied with `amenui add material` reads the switch as `ground_marks()` |

Acceptance: `tests/off.rs` (for each switch, the frame after what would set the effect off is byte for byte the frame
the effect settles on, and the page asks for no frames in the 2 s after; a flow switched off mid-flight ends at once;
everything is on by default; with `ground` off the empty ground is the theme's ground under a front and the pointer,
and no frames are asked for once the front is over).
