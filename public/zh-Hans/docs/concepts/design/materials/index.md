# 1.6 材质

The screen-space shading stage is one full-screen shader. Each material is a set of functions:
`loosened(p)` (what to draw here when loose), `ground(p)` (the background) and `resolve(...)` (the
composite). A material is chosen as global plus per-instance override. Every parameter is a number in
`Fx`, each with a name, a range and a step.

There are nine (§1.6.2; `scribblr` was added later, §1.30). Under any material a picture turns into marks
by its own `Markr` slot, the slot being its instance override (`glitchr` is the exception: it displaces the
whole frame, but sampling inside a picture never leaves it). In `plain`, looseness `b` fades text and
shapes to `mix(1, 0.3, √b)`: the square root makes the 0.35 looseness under a dialog visible.

`El::material(id)` gives a subtree another material (the inner one wins when nested); it works under any
global material. `El::effect(Effect { material, response, scope })` puts a material, a response (§1.15) and a
scope on a subtree in one line, leaving what is `None` as it is; a scope of `Scope::Container(None)` also makes the
element the container, so the effect's fronts stay inside it. `el::wrap_text(src)` turns plain text or Markdown
(the subset `markdown.rs` reads; a link shows as its text) into a page to read, 600 px wide and keyed `wrap/…`, so
existing text takes an effect without being rewritten. Text to read under an effect stays legible (§1.15): a
looseness given to the subtree (`El::loosen`) reaches its paragraphs, list items and code at most at
`Look::legible_cap`, while headings take the full value; a showcase marks text `legible(false)`. A control inside
such a subtree (an element with its own handler, other than a picture, canvas or 3D scene) is legible again unless it
says `legible` itself, so its fill and label stay at the cap as at rest: a showcase loosens its decoration, never its
buttons. A tab list's heads and rule are the control (`tabs` marks them).

**Names.** A material that prints the page is named for what it does, the verb with an `r`: `dottr`, `halftonr`,
`charr`, `smudgr` (ink that spreads, specks and falls out of register), `carvr`, `glitchr`, `scribblr`. `plain`
(nobody's hand) and `near` (the mechanism laid open) are not printings and keep their names. The same rule names
`Markr` (the picture marks, §1.6.1), `Dithr` (the finishing pass's dither, `post.dithr`), `Cmd::Ripplr` and
`El::ripplr` (a front) and `El::wipr` (the compare wipe). There are no aliases: an old material name is refused,
and the error names the new one (`style::RENAMED`); for any other unknown name the error gives the closest. Look recipes start with
`look:v3:`; a `look:v1:` recipe, from before the names changed, is refused with the renames listed, and a `look:v2:` one, from before the beat, with its beat (§1.32).
