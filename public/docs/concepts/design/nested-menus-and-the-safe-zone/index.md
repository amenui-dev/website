# 1.8.5 Nested menus and the safe zone

`widgets::nested_menu` (`Menu { items, submenu_delay_ms 120, close_delay_ms 200, row_h 30, width 200, pad 4,
overlap 4, show_safe_zone }`). The open path is state, and so is the geometry (each row `row_h`, each menu
`width`, a submenu beside its row), so the triangle can be tested without layout. Pointer movement over a
menu is a message (`El::on_move`: local coordinates while a mouse or pen moves over the element, `None` on
leaving; fingers have no hover).

- The safe zone: the triangle from the pointer's **previous** position to the two near corners of the
  deepest submenu. While the pointer is in the triangle, sibling rows it passes don't take over: switching
  waits `close_delay_ms`, restarted on every move inside; outside the triangle, resting on a row for
  `submenu_delay_ms` makes it take over (opening its submenu if it has one). Rows in the deepest menu become
  current at once, and their submenus open after `submenu_delay_ms`.
- While waiting, `Menu::due()` gives the time and the application's `on_wake` returns it, so a frame is
  wanted only at that moment, not every frame.
- Keyboard: Up and Down move within the deepest menu, Right or Enter opens a submenu (with its first row
  current), Left or Esc closes one level, and Enter on a leaf chooses it; `update` returns the chosen path.
- Debug layer: with `show_safe_zone`, the triangle goes to the annotation layer through `El::guide` (a closed
  polyline in local coordinates), drawn dotted in the annotation colour whether the annotation layer is on
  or not.
- Acceptance: unit tests in `nested_menu.rs`; `tests/menu.rs`: a diagonal over sibling rows doesn't change
  the submenu, the triangle is drawn in the annotation layer, and no frames are wanted once in the submenu;
  straight down outside the triangle, the row changes after `submenu_delay_ms` with a timed wake while
  waiting. Samples: `craft <dir> menu`.
