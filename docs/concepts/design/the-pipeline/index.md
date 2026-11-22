# 1. The pipeline

```
view(state) → element tree → layout (taffy + parley) → scene (instances) ─┐
event → hit test → message → update(state)                                │
event → wave field (fronts, pointer, scroll velocity…)                    │
                                                                          ▼
          G-buffer stage: instances drawn as they are into the coverage and SDF buffers
          Screen-space shading stage: the material reads both buffers and the field → frame
          Annotation layer (optional)
```

### 1.1 Describing (`el.rs`)

`El<M>`: `key`, a taffy `Style`, content (none, text or picture), fill, border, `on_press: M`, `on_key`,
`on_drag` (a press and the drag that follows: phase, pointer position in the element, the element's
size, offset from the press), `on_blur`, `focusable`, `scroll` (vertical, or `scroll_x` horizontal; `bar`
shows a thin position bar), `disabled` (the subtree takes no input and is drawn faint), `modal` (a
dialog-style layer, §1.8), `commit` (pressing it is a commit, §1.5), `reveal` (after a key changes the
state, scroll it into view, such as the selected row), `fonts` (which face each of the three type roles,
display, body and mono, uses in this subtree), `loose` (looseness at rest), `wipe` (the compare wipe
line), and children. Text content carries its face (a role or a face directly), the pitch it is printed
as pixels at, and the caret (drawn only while its control has focus); picture content carries the
`Markr` slot it uses. `El::map` turns a tree's messages into another type, so one application can be
embedded in another's view. A view is `fn(&S) -> El<M>` and an update is `fn(&mut S, M)` (Elm style;
messages are plain values). A node without a key uses its path as its key. Also: `on_text`, `multiline`,
`selectable` (§1.8.1), `cursor` (§1.8.2), `virtual_rows` (§1.13), `role`, `label` and the rest (§1.14).

Three more change how a subtree is drawn and hit. `El::pass(true)` lets the pointer through the element to what is
under it (its children still take the pointer); it is still drawn and the field still reaches it, for a drawing laid
over controls. `El::opacity(k)` draws the subtree as one and then makes it see-through as one, so where two of its
children overlap only the top one shows: the renderer draws the group's instances into a texture of its own (made the
first time a group is drawn) and lays it over the ink at `k`; a group inside a group fades its instances by its own
`k` instead. At 0 the subtree is not drawn and takes no pointer; the accessibility mirror still has it. A frame with
a group is drawn whole (§1.19). `El::commit_scope()` keeps a commit inside the element's box: the commit's front
(§1.5) travels inside its clip, every pixel outside the box grown by the material's reach (the nearness spread and a
front's push, what a material draws round an element) stays as it was, and the fronts of presses within it stay
inside it too (it is their container). The commit is over once its front has reached the box's farthest corner, not
the window's: the marks round the box arrive with it rather than a second later. `Cmd::Commit { x, y, scope }` names such an element by key.
