# 1.5 The wave field

Now: up to 8 fronts (position, start time, strength), the pointer, and one half-plane source (the compare
wipe: the x of a vertical line, a strength and the rect it acts on), computed analytically and passed as
uniforms. Later: a low-resolution field texture stepped on the GPU each frame (energy, flow), with scroll
velocity, drags and an audio spectrum as inputs.

How events map to the field:

| Event | What goes into the field |
|---|---|
| Press | One front, reaching only its scope (by default the control pressed, §1.15); nothing for a press on empty space |
| Pointer move | The lens (loosens only pictures and the background, never text) |
| Press and hold | The lens grows stronger; release sends a front |
| Drag | A trail along the path |
| Scroll | A shear over the whole screen by velocity |
| Typing a character | That glyph condenses from loose |
| Commit (page change, theme or material switch) | A front from where it was triggered, with the old and new state on either side of it |

How a commit works: pressing a control with `commit` (or with an ancestor that has it) makes the runtime
keep a copy of the current scene and field (`Split`). The first time the renderer sees it, it draws that
old picture in full into a `before` image with the previous frame's look; after that, the material's last
step chooses by distance from the origin: outside the front (radius = elapsed time × wave speed) is the old
picture, inside is the new. The front is dropped once it leaves the screen. Theme and material switches
go this way. A page change is not a commit: there is only the front at the press, the new page's
instances have new identities and condense from loose, the top bar and side rail keep their identities
and do not condense again, and the parts both pages share flow across (§1.10).
