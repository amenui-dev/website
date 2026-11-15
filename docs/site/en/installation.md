# Installation

Pick the host your interface lives in. All three draw the same pixels: the framework draws everything itself, and the host only gives it a surface, input and time.

## Native desktop (winit)

You need the latest stable Rust and a GPU, or Mesa's lavapipe (Vulkan on the CPU).

```sh
cargo run --release --bin window -- film
cargo test --release
cargo run --release --bin check
```

The window opens on the page named after `--`: `film`, a component such as `button`, or a `look:v1:` recipe.

## The web (wasm bundle)

WebGPU when the browser has it, else WebGL2. The build writes one self-contained page, `dist/index.html`, with the module, the font subsets and the glue inlined.

```sh
cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen --target no-modules --no-typescript --out-dir dist/pkg target/wasm32-unknown-unknown/release/lab.wasm
python3 web/build.py
```

The page follows its address: `#button` opens a component, `#look:v1:…` applies a recipe.

## Embedding: `<lab-view>`

The same build defines a custom element, so any page or framework that renders HTML can hold the interface. `dist/embed-example.html` is a plain page that uses it.

One `<lab-view>` per document: the module holds one runtime and one renderer. A second one (or one on the full page) is not drawn; it says so in its box and carries `data-lab-limit="one-per-document"`, and the first keeps working.

| Attribute | What it does |
|---|---|
| `page` | The page to show: a component's name (`button`), `film`, any page's hash. |
| `recipe` | A `look:v1:` recipe: the look it describes. |
| `theme`, `material` | By name, as the render tool takes them (`dark`, `dot`). |
| `locale` | `zh-Hans` or `en`. |

It fires `lab-ready` once it draws and `lab-route` (with `detail.page`) whenever the page changes inside it, and it follows its own box as it is resized. One per document.

Plain HTML:

```html
<lab-view page="button" theme="light" locale="en" style="height: 480px"></lab-view>
```

React (custom elements pass attributes as they are; listen with a ref):

```tsx
const ref = useRef<HTMLElement>(null);
useEffect(() => { const v = ref.current; const f = (e: any) => setPage(e.detail.page); v?.addEventListener('lab-route', f); return () => v?.removeEventListener('lab-route', f); }, []);
return <lab-view ref={ref} page={page} theme="dark" style={{ height: 480 }} />;
```

Vue (tell the compiler the tag is a custom element: `compilerOptions.isCustomElement = t => t === 'lab-view'`):

```html
<lab-view :page="page" theme="dark" @lab-route="page = $event.detail.page" style="height: 480px" />
```

Svelte:

```html
<lab-view page={page} theme="dark" on:lab-route={(e) => (page = e.detail.page)} style="height: 480px"></lab-view>
```

Angular (add `CUSTOM_ELEMENTS_SCHEMA` to the component):

```html
<lab-view [attr.page]="page" theme="dark" (lab-route)="page = $event.detail.page" style="height: 480px"></lab-view>
```

## Manual: copy components into your project

Components are distributed as source, as shadcn/ui does it: `lab add` copies a component and the files it depends on, as `registry.json` lists them, into your project at the same paths; files already there are left alone.

```sh
cargo run --release --bin lab -- add dialog --to ../my-app
```

Or copy by hand: the component's page lists its files under Installation.
