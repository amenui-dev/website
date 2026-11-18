# Installation

Pick the host your interface lives in. All three draw the same pixels: the framework draws everything itself, and the host only gives it a surface, input and time.

## Native desktop (winit)

You need the latest stable Rust and a GPU, or Mesa's lavapipe (Vulkan on the CPU).

```sh
cargo run --release --bin window -- body
cargo test --release
cargo run --release --bin check
```

The window opens on the page named after `--`: `body` (the landing scene), a component such as `button`, or a `look:v3:` recipe.

## The web (wasm bundle)

WebGPU when the browser has it, else WebGL2. The WebGL2 path is the `webgl` feature, on by default; with `default-features = false` the module carries only WebGPU and is about a quarter smaller, and a browser without WebGPU reports `opts.onFail('adapter', message)` instead of starting. The build writes the site to `dist/site/`: one small HTML page per address, the glue script, the module and the font subsets as separate files with content hashes in their names.

```sh
cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen --target no-modules --no-typescript --remove-name-section --out-dir dist/pkg target/wasm32-unknown-unknown/release/website.wasm
BASE=/ python3 web/build.py
python3 -m http.server -d dist/site
```

`BASE` is where the site is served (`/` on its own domain). The page follows its address: `/components/button/` opens a component, `/zh-Hans/…` the same page in Chinese, and `#look:v3:…` applies a recipe.

## Embedding: `<amenui-view>`

The same build defines a custom element, so any page or framework that renders HTML can hold the interface. `dist/embed-example.html` is a plain page that uses it.

One `<amenui-view>` per document: the module holds one runtime and one renderer. A second one (or one on the full page) is not drawn; it says so in its box and carries `data-amenui-limit="one-per-document"`, and the first keeps working.

| Attribute | What it does |
|---|---|
| `page` | The page to show: a component's name (`button`), `body`, any page's hash. |
| `recipe` | A `look:v3:` recipe: the look it describes. |
| `theme`, `material` | By name, as the render tool takes them (`dark`, `dottr`). |
| `locale` | `zh-Hans` or `en`. |

It fires `amenui-ready` once it draws and `amenui-route` (with `detail.page`) whenever the page changes inside it, and it follows its own box as it is resized. One per document.

Plain HTML:

```html
<amenui-view page="button" theme="light" locale="en" style="height: 480px"></amenui-view>
```

React (custom elements pass attributes as they are; listen with a ref):

```tsx
const ref = useRef<HTMLElement>(null);
useEffect(() => { const v = ref.current; const f = (e: any) => setPage(e.detail.page); v?.addEventListener('amenui-route', f); return () => v?.removeEventListener('amenui-route', f); }, []);
return <amenui-view ref={ref} page={page} theme="dark" style={{ height: 480 }} />;
```

Vue (tell the compiler the tag is a custom element: `compilerOptions.isCustomElement = t => t === 'amenui-view'`):

```html
<amenui-view :page="page" theme="dark" @amenui-route="page = $event.detail.page" style="height: 480px" />
```

Svelte:

```html
<amenui-view page={page} theme="dark" on:amenui-route={(e) => (page = e.detail.page)} style="height: 480px"></amenui-view>
```

Angular (add `CUSTOM_ELEMENTS_SCHEMA` to the component):

```html
<amenui-view [attr.page]="page" theme="dark" (amenui-route)="page = $event.detail.page" style="height: 480px"></amenui-view>
```

## Manual: copy components into your project

Components are distributed as source, as shadcn/ui does it: `amenui add` copies a component and the files it depends on, as `registry.json` lists them, into your project at the same paths; files already there are left alone.

```sh
cargo run --release --bin amenui -- add dialog --to ../my-app
```

Or copy by hand: the component's page lists its files under Installation.
