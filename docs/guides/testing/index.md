# Testing with render and the tree


Every recipe can be driven from outside without a window: `render` opens it offscreen, plays a list of input events on a fixed 60 Hz clock from 0 with fixed seeds, and writes the last frame as a PNG and the laid-out tree as JSON. The same arguments give the same bytes, so a picture or a tree can be compared in a test or in review.

```sh
cargo run --release --bin render -- --page button --size 1280x800 --at 1000 --out out.png --tree out.json --locale en
```

## Options

| Option | What it does |
|---|---|
| `--at <ms>` | the moment to draw, on the page's clock |
| `--events <file.json>` | input to play first: `[{"at_ms": 200, "kind": "press", "target": "button/live"}]`; kinds are press, release, move, key, wheel and text |
| `--frames <from>..<to> [--fps 60]` | a run of frames instead of one, written to `dir/%05d.png` |
| `--material`, `--theme`, `--locale`, `--scale` | the look and language to draw in |
| `--audit` | measures the frame: text cut off, contrast, glyphs over each other, text too small, a page that never settles; findings are P0 (breaks reading) to P3 (a note) |
| `--strict` | problems in the view (duplicate keys…) and P0 or P1 findings fail the run |

The tree lists every element's key, role, name, state and rectangle; tests read it to check layout and the accessibility mirror without pixels. In Rust the same calls are `amenui::drive::render` and `amenui::drive::audit`.

`check` in the website repository reads pixels back on the GPU for acceptance (20 checks, including that a slow scroll is a picture moved by whole pixels).
