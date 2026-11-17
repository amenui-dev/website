# Accessibility

The screen is drawn by the framework, not by the platform's controls, so the runtime keeps a second, invisible copy of the interface for assistive technology: the accessibility mirror. It is built from the same tree as the picture on every rebuild, so it can't drift out of step.

## The mirror

Each element in the mirror has a role (named after the WAI-ARIA role of the same name: `Role::Button`, `Role::Slider`, `Role::Dialog`, `Role::Link` and 30 more), a name (`El::label`) and its state: checked, selected, value and range, level, disabled, expanded, required, invalid, a longer `description`, and `live` for regions that announce changes (`Live::Polite` or at once).

| Where | What reads the mirror |
|---|---|
| Native (macOS, Windows, Linux) | AccessKit, under one window node. Element ids come from their keys, so a screen reader keeps its place across rebuilds. |
| Web | An invisible DOM tree with the matching `role` and `aria-*` attributes. |

Focus and activation go both ways: a screen reader moving focus or pressing a button arrives as the runtime's own focus change and click.

## Keyboard

Tab and Shift+Tab move through focusable elements in tree order. `El::tab_order(n)` puts an element earlier (a positive `n`) or takes it out of the cycle (a negative `n`). `widgets::skip_link` gives a page a "Skip to content" link that appears when focused and moves focus into its target. `El::focus_shown` keeps an element hidden until it has keyboard focus.

## Appearance

The window's dark or light setting and the web's `prefers-contrast` reach the application as `App::set_appearance`; `prefers-reduced-motion` reaches the runtime and the application, which shorten or drop motion.

Every component page lists its keyboard behaviour and roles.
