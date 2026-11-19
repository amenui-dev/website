# Layout

Layout is taffy's flexbox, in logical pixels; a `stack()` lays its children over one another and keeps its size; elements do not shrink unless asked. `row()`, `col()`, `stack()`, `space(w, h)`; `.gap`, `.pad`, `.grow`, `.fill_width`, `.scroll()`. `El::rtl(true)` makes a subtree run right to left. `El::zoom(k)` multiplies every length in a subtree by `k`.
