<div align="center">

# amenui website

The site and demos of [amenui](https://github.com/amenui-dev/amenui), drawn entirely by amenui itself.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

</div>

The landing page `#body` is a paragraph that gathers into a slowly turning, glowing body made of its own
glyphs and comes back to readable text. Small markings in the picture lead to the Lab (`#lab`) and the
component gallery (`#gallery`). There are also seven showcase scenes (`#scenes`), a page for every
component, mark set, material and style, and the documentation (`docs/site/`, in English and Simplified
Chinese).

## Layout

| Path | What it holds |
| --- | --- |
| `src/gallery/` | The whole site: top bar, sidebar and every page. `src/demo.rs` is the skeleton demo page. |
| `examples/body.rs`, `examples/lab/` | The landing body and each Lab experiment, written only against amenui's public API (`website::showcase`). |
| `assets/i18n/` | The site's own strings. amenui's component strings live in the framework; `i18n::install` merges both. |
| `docs/site/` | The documentation pages shown on the site, one folder per language. |
| `web/` | The web shell: one small HTML page per address (`shell.html`), the glue between it and the module (`app.js`), `embed.js`, and the packaging script (`build.py`). |
| `src/lib.rs` | The site as an application: its `amenui::app::App` hooks, `make` and `setup`, and the web exports (`amenui::web_app!`). |
| `src/bin/` | The native window (`amenui::app::native::run`), sample sheets, GPU acceptance checks and timing tools. |
| `tests/` | Tests for the site's pages. |

## Quick start

```bash
cargo run --release --bin window            # the native window on the landing page
cargo run --release --bin window -- dialog  # or open a page by name
cargo test                                  # no GPU needed
```

This repository depends on amenui's `main` branch (`Cargo.toml`). To change both at once, put an
amenui checkout next to this one and uncomment the `[patch]` section at the end of `Cargo.toml`.

## Web version

Needs the `wasm32-unknown-unknown` target, a `wasm-bindgen-cli` of the same version as the lock file,
and Python with fonttools.

```bash
cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen --target no-modules --no-typescript --remove-name-section --out-dir dist/pkg \
  target/wasm32-unknown-unknown/release/website.wasm
cargo build --release --lib --target wasm32-unknown-unknown --no-default-features
wasm-bindgen --target no-modules --no-typescript --remove-name-section --out-dir dist/pkg-gpu \
  target/wasm32-unknown-unknown/release/website.wasm
python3 web/build.py
```

The first build is the full module (WebGPU or WebGL2); the second carries only WebGPU and is what a
browser with `navigator.gpu` loads, falling back to the full one if it cannot start.

`BASE=/ python3 web/build.py` writes the site to `dist/site/` (BASE is where it is served; the
default is `/website/`, for GitHub Pages). Serve it with `python3 -m http.server -d dist/site` and open
a page by its path (`/components/dialog/`). The shell forwards pointer, wheel, keyboard, resize and the
address to the framework.

## Tools

The drawing tools need a Vulkan driver (lavapipe works without a GPU; numbers from it are only good for
comparing with each other). Run any of them with `cargo run --release --bin <name> -- <args>`.

| Tool | What it does |
| --- | --- |
| `window [page] [--shot path.png@ms] [--exit-after ms]` | The native window (winit; Metal on macOS, Vulkan elsewhere). Reports the adapter and frame counts on stderr. |
| `render --page <page> --size 1280x800 --at 1000 --out out.png --tree out.json` | One frame and its element tree, for models and scripts. Pages are site hashes or amenui recipes (`recipe-form`). |
| `check [components]` | Reads pixels back to verify the acceptance checks in amenui's DESIGN.md. Exits non-zero on failure. |
| `pages <dir> <w> <h> <scale> [materials \| components \| perf]` | Every page × theme × material as PNGs, plus dialog, theme switch, page change, typing and narrow window states. |
| `scenes <dir> [0–6 \| perf]` | Scripted runs of the showcase scenes, contact sheets per style, and a narrow window. |
| `film <dir> [ripple \| perf]` | Morph and page change samples, and the ripple and recomposition samples. |
| `samples <dir> mesh \| particles \| transitions` | 3D under every material, particles, and the transitions. |
| `sheet <dir>` | Every mark sheet, palette and picture effect, plus the mark atlas. |
| `craft <dir> [study …]` | Filmstrips of the interaction previews (sheet, floating, stirr, scribblr…). |
| `labsheet <dir> [cells \| page \| decode \| perf]` | Contact sheets of the Lab and the decode transition. |
| `scroll <dir> [name]` | Filmstrips of scrolling under six materials, and a stillness measure. |
| `shot <dir> <w> <h> <scale>` | Frames of the skeleton demo. |
| `bench` | CPU timings (a press, a 10k-row list, rewrapping) and GPU timings (particles, dirty boxes). |
| `md <dir>` / `md --llms <amenui>/llms.txt` | Every page's Markdown form, and the page list in amenui's `llms.txt`. |

### The native window on Linux without a display

Run it in Xvfb with Mesa's lavapipe (no `WGPU_BACKEND` needed); winit also needs `libxkbcommon-x11-0`.
Real X input from `xdotool` drives it, and `--shot` frames can be compared with
`import -window root` from ImageMagick.

```bash
xvfb-run -a -s "-screen 0 1400x1000x24" cargo run --release --bin window -- --shot out/a.png@3000 --exit-after 5000
```

On macOS the window has only been type-checked (`cargo check --target aarch64-apple-darwin`).

## Deployment

Every push to `main` runs `.github/workflows/deploy.yml`, which builds the web version and puts
`dist/site/` (every page, the docs, the module and the fonts) on the `gh-pages` branch. GitHub Pages serves that branch
directly ("Deploy from a branch → gh-pages / (root)").

## License

[MIT](LICENSE)
