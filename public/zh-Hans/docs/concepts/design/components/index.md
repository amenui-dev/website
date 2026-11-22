# 3. 组件

Button, number control, text field, switch, slider, tabs, choices (`choice` and the segmented `segmented`), list,
card, dialog, image, divider, scroll area, and the rest listed in `registry.json`.

Components are plain functions returning `El<M>`. Like shadcn, they are distributed as source: one file per
component (`src/widgets/`), and `registry.json` at the repository root lists each component's name, title,
description, files and the other components it depends on (all depend on `base`, the shared sizes in
`src/widgets/mod.rs`; the slider depends on the number control's `Spec`, the dialog on the button). Users copy the
source and change it. The tree a component returns can be resized further with `El` methods (`.height`, `.pad`,
`.radius`, `.border`, `.disabled`).

Buttons come in three variants: default (outlined), quiet (no border) and accent (accent fill with text in the page
background colour); unavailable is `.disabled(true)`.
