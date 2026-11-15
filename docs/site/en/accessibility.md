# Accessibility

The screen is drawn by the framework, so the runtime builds an accessibility mirror from the tree: every element's role (the WAI-ARIA role of the same name), name and state. The web build puts it in an invisible DOM tree; focus and activation go both ways. Every component page lists its keyboard behaviour and roles.
