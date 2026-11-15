# Registry schema

`registry.json` lists every component as source to copy, the way shadcn/ui's registry does. `lab add` reads it; the documentation's component pages are made from it.

| Field | Type | Meaning |
|---|---|---|
| `name` | string | The component's name, as `lab add` takes it (`dialog`, `scroll_area`). |
| `title` | string | Its name in Chinese. |
| `group` | string | Where the docs list it: `inputs`, `display`, `feedback`, `navigation`, `layout` or `data`. |
| `description`, `description_en` | string | One sentence in each language. |
| `functions` | string[] | The functions it exports (`dialog`, `dialog_sized`). |
| `files` | string[] | Source files, relative to the repository root, copied to the same paths. |
| `dependencies` | string[] | Other registry items it needs (copied with it). |
| `response` | string | How it answers fronts (`nudge`, `loosen`, `ignore`). |
| `recompose` | string | What recomposes when its text changes. |
| `recipes` | string[] | Recipes that use it. |

An item named `base` holds the files every component needs (the element tree, the runtime).

```sh
cargo run --release --bin lab -- add dialog --to ../my-app
```
