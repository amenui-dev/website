# 5. The gallery

Written with the framework itself, it is an application and the web version's main page. It now lives in the
[website repository](https://github.com/amenui-dev/website) (`src/gallery/`):

- Top bar: the name `amenui` on the left; on the right the theme, material and style menus (`menu`: the button shows
  the current item, and pressing it lists them all in a layer below; `El::anchor` places the list under the button) and
  Reset. Menu items are commits (§1.5), spreading from the item pressed.
- A 220 logical px side rail in eight groups: components, marks (one page per mark set), materials (one page each: a
  fixed sample of a heading, a paragraph, a picture, a row of controls, a card and a few shapes, with looseness that can be
  fixed by number, every material parameter, and a Ripple button that sends a front), annotation (mode, kinds, density
  and draw time acting on an example), styles (one page per preset), transforms, type, and scenes (the list and seven
  scenes, §6). Below 720 px wide, the side rail becomes a horizontally scrolling row of tabs under the top bar.
- One page per component, top to bottom: the title (display size) and the function to call; one sentence on what it
  does; a preview (the live component on the page background with generous space around it); variants (one cell per
  variant and state, wrapping, with a faint caption under each); parameters (the component's own numbers, changed with
  number controls); and the source (the component's file via `include_str!`, in the mono face with faint line numbers, in
  a scroll area, long lines wrapping under their own line number).
- Mark pages: a picture in this mark set with a compare line, every number of `Markr` editable, and a choice of picture.
- One page per style preset, each a small composition of its own rather than the same sample in other colours: Gaze is a
  stream of boxed, measured pictures (filterable); Dotmatrix a departure board ticking every second; Lo-Fi Print a zine
  page; Glitch a title card that slips as you pass and a row of controls; Linocut a carved headline, a sun and a picture
  printed in strokes; Tech-Manual an exploded view and a parts list, where the pointer on a row marks the matching part in
  the drawing and the other way round (the runtime tells the state about hover with `Ui::on_hover`, and about keyboard focus with `Ui::on_focus`); Normcore a settings
  form; Cyberminimalism a command line and its filtered results; Bauhaus a geometric poster and a set of controls that
  change it. Every page has real interaction; a press sends a front, showing how the style loosens.

### 5.2 Formula captions

A line of small mono type under a live demo (`widgets::caption`, `El::caption_params(key, formula, params, msg)`) that
prints the real parameters driving the demo: the line is formatted from the same fields the demo reads, never written by
hand, and each parameter below it is a small number control (`number` scaled to 0.8) that changes that field. The
website's interaction pages (bottom sheet, floating box, nested menu, loading, card stack, dot chart) have one each, and
every cell of the Lab works this way (§5.3).

### 5.3 The Lab

`#lab`: small experiments that each do one thing, each a file in the website's `examples/lab/`, written only against the
public interface (`experiment::Experiment`: view, update, step, whether the next frame is wanted). The look is one theme
(`style::LAB`): a near-black ground, grey-white hairlines and dots, and a single blue that appears only where the pointer
touches or has an effect; all type is the mono pixel face. The page should read as a wall of living pieces, not a form
with small pictures: each cell has `[ name ]` at the top left and a one-line hint at the top right; in the middle the
**stage** (`experiment::STAGE`, height : width = 3 : 4, clipped), which the piece fills, taking over 70% of the cell's
height (drawings use 240 × 180 coordinates; text pieces are placed by the stage's width and height); below it one line of
formula (mono, unwrapped, cut off when too long) whose numbers are the values driving it now; and at the bottom a small
label on the left and `[ params ]` on the right, which opens the cell's number controls only when pressed (scaled to 0.72,
`Params::open`, toggled by the message at index `experiment::TOGGLE`), with no sliders. Cells are laid out in columns of
at least 340 px, one column in narrow windows. Ten cells: lit text (a block of small mono type filling the stage, under a
soft light), contours, silk (each mark a short line segment along the flow, long when fast and short when slow, so it
looks like flowing silk), grown (f 0.029, k 0.057, maze stripes on 120 × 90, dots sized by concentration; every `T`
seconds it changes shape by itself: a letter → a switch → a rounded square, the old pattern decoding away as the new one
decodes in), mercury, text on a path (four lines along waves of their own phase), text on a sphere (the sphere fills the
stage, the back showing through, dark and reversed), orbits, a spring and the card stack (lab-style cards: faces close
to the ground with thin outlines, the one under the pointer in the accent); each experiment also has a page
(`#lab-<name>`, the website's `docs/site/<lang>/lab-<name>.md`: the formula and a table of parameters). Wherever the
pointer touches, that one blue appears: lit text, bent contours, stirred silk, sown seeds, a held ball, a stretch of text
pulled into a curve, the text under the pointer on the sphere, attracted points, the held spring, the card under the
pointer.

**Living cells (an explicit exception to "every page comes to rest", §0 rule 3)**: Lab cells step every frame while they
can be seen (`El::on_visible`) and stop when scrolled out of view, so the Lab page never rests; their marks all come
from the one frame budget (§1.27) and shrink in proportion when there are too many; when frames exceed the budget
(`budget::usage().scale` < 1), only that proportion of the visible cells step each frame (`Lab::turn`), so the page's
cost falls with it. When the viewer asks for reduced motion (the web's `prefers-reduced-motion`, `set_reduced_motion`;
`Lab::reduced`) the original rule returns: cells step for `experiment::IDLE_S` (1.2 s) after arriving on the page or
after the last input, then only those still finishing (springs, coasting, points following the pointer) continue, and
then the whole page rests and wants no frames. Acceptance:
`the_labs_pieces_run_while_seen_and_stop_when_scrolled_out` in `tests/frames.rs` (frames wanted while seen, none when
scrolled out, at rest with reduced motion) and `every_page_comes_to_rest` (the Lab included, with reduced motion).

### 5.1 The documentation site

The gallery is organized as a documentation site after shadcn/ui (the website's `src/gallery/docs.rs`). The side rail
has sections: Getting started (introduction, installation, command line, your first view, project structure, forms),
Concepts (the tree and messages, layout, text, materials and looseness, ripple rules, transforms, presets, annotation,
3D, particles, languages, accessibility, performance, testing with render and the tree), Components (one page each, in
six groups by `registry.json`'s `group`: inputs, display, feedback, navigation, layout, data, as shadcn does), Styles /
presets, For AI (AGENTS.md, llms.txt, the component registry and recipes) and the Changelog (`CHANGELOG.md`), followed by
the original experimental pages (marks, materials, annotation, transforms, type, scenes). Long prose is in the website's
`docs/site/<lang>/*.md` (two languages, original text), drawn by a small Markdown subset renderer (`src/markdown.rs`:
headings, paragraphs, lists, code blocks, tables, links, inline code); inline code and links are drawn within the
paragraph's own text (one text per paragraph, so line breaking stays with the layout system), and link targets are listed
under the paragraph and can be pressed.

**The order of a component page** is the same on every page: the title and one sentence; a preview (Preview / Code tabs,
the code being the component's example file `examples/components/<name>.rs`, selectable with a copy button);
installation (`amenui add <name>` and the files it copies); usage (the first example's code); examples (each `ex_*`
function in the example file: how it looks and its own source); API (generated from `registry.json` and the component
file's header: functions, parameters, files, dependencies, the response to fronts, recomposition); keyboard and
accessibility (the header's states and messages, plus the role and name the web accessibility mirror gives it); right to
left (the first example in an `El::rtl(true)` subtree, and what isn't mirrored yet); and under each material (the first
example drawn once in each of twelve materials). For the thirteen components with live previews of their own, the
gallery's preview, variants, parameters and source sit below the preview. The example files are compiled into the
library, so every piece of code on the pages has been compiled; when the same example appears more than once on a page,
`El::rekey` puts its keys under different prefixes.

**Within a page**: "On this page" on the right follows the scroll (`Ui::on_scrolled` tells the state where it has
scrolled to and where each section is), and pressing an item scrolls to its section (`Cmd::ScrollTo`); at the bottom are
Previous and Next; ⌘K / Ctrl+K (`Ui::on_shortcut`) or `/` opens search (a command palette in a dialog filtering every
page name, ↑ ↓ Enter); each page's address is `#name` (`#docs-layout`, `#checkbox`, `#ai-registry`). Copy buttons hand
text to the host (`Gallery::clipboard`; on the web `take_clipboard()` writes it to `navigator.clipboard`, natively
arboard).

**`amenui add`** (`src/bin/amenui.rs`, logic in `src/add.rs`): `amenui add dialog --to ../my-app` resolves dependencies
from the registry and copies each file to the same path under the target, never overwriting existing files (`--force`
overwrites); a misspelt name gets the nearest match.

**Acceptance**: `tests/docs.rs` (every component has a page in both languages, none of its sections empty, the
right-to-left example starts from the right, it belongs to one of the six groups, the side rail has the six group names,
there are no view problems and no missing keys; every example is compiled into the library and matches its preview one
to one; every internal link in the docs points to a page); `tests/add.rs`; `tests/widgets.rs` (roles and keyboard of the
new components).
