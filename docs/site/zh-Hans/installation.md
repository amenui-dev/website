# 安装

先选界面放在哪里。三种宿主画出来的像素一样：所有东西都由框架自己画，宿主只给它一块画面、输入和时间。

## 桌面原生（winit）

需要最新的稳定版 Rust，以及一块 GPU，或者 Mesa 的 lavapipe（在 CPU 上跑的 Vulkan）。

```sh
cargo run --release --bin window -- body
cargo test --release
cargo run --release --bin check
```

窗口打开 `--` 后面写的页面：`body`（落地的场景）、一个组件（比如 `button`），或者一个 `look:v3:` 配方。

## 网页（wasm 包）

浏览器有 WebGPU 时用 WebGPU，否则用 WebGL2。WebGL2 这条路是 `webgl` 功能，默认打开；设成 `default-features = false` 后模块只带 WebGPU，小大约四分之一，没有 WebGPU 的浏览器不会启动，而是调用 `opts.onFail('adapter', message)`。构建把网站写到 `dist/site/`：每个地址一个小 HTML 页面，加上胶水脚本、模块和字体子集，各自是单独的文件，文件名里带内容哈希。

```sh
cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen --target no-modules --no-typescript --remove-name-section --out-dir dist/pkg target/wasm32-unknown-unknown/release/website.wasm
BASE=/ python3 web/build.py
python3 -m http.server -d dist/site
```

`BASE` 是网站所在的路径（在自己的域名上是 `/`）。页面跟着地址走：`/components/button/` 打开一个组件，`/zh-Hans/…` 是同一页的中文版，`#look:v3:…` 套用一个配方。

## 嵌进别的页面：`<amenui-view>`

同一个构建定义了一个自定义元素，所以任何渲染 HTML 的页面或框架都能放进这个界面。`dist/embed-example.html` 是一个用它的普通页面。

一个文档里只有一个 `<amenui-view>`：模块里只有一个运行时和一个渲染器。第二个（或者完整页面上再放一个）不画，在自己的框里说明这一点并带上 `data-amenui-limit="one-per-document"`，第一个照常工作。

| 属性 | 作用 |
|---|---|
| `page` | 显示哪一页：组件名（`button`）、`body`，或任何页面的 hash。 |
| `recipe` | 一个 `look:v3:` 配方：套用它描述的外观。 |
| `theme`、`material` | 按名字，和渲染工具接受的一样（`dark`、`dottr`）。 |
| `locale` | `zh-Hans` 或 `en`。 |

它开始画时发出 `amenui-ready`，里面换页时发出 `amenui-route`（`detail.page` 是页面）；它跟着自己的盒子改变大小。一个文档里放一个。

普通 HTML：

```html
<amenui-view page="button" theme="light" locale="en" style="height: 480px"></amenui-view>
```

React（自定义元素的属性原样传过去；事件用 ref 监听）：

```tsx
const ref = useRef<HTMLElement>(null);
useEffect(() => { const v = ref.current; const f = (e: any) => setPage(e.detail.page); v?.addEventListener('amenui-route', f); return () => v?.removeEventListener('amenui-route', f); }, []);
return <amenui-view ref={ref} page={page} theme="dark" style={{ height: 480 }} />;
```

Vue（告诉编译器这是自定义元素：`compilerOptions.isCustomElement = t => t === 'amenui-view'`）：

```html
<amenui-view :page="page" theme="dark" @amenui-route="page = $event.detail.page" style="height: 480px" />
```

Svelte：

```html
<amenui-view page={page} theme="dark" on:amenui-route={(e) => (page = e.detail.page)} style="height: 480px"></amenui-view>
```

Angular（在组件里加 `CUSTOM_ELEMENTS_SCHEMA`）：

```html
<amenui-view [attr.page]="page" theme="dark" (amenui-route)="page = $event.detail.page" style="height: 480px"></amenui-view>
```

## 手动：把组件拷进你的项目

组件以源码分发，和 shadcn/ui 的做法一样：`amenui add` 按 `registry.json` 把一个组件和它依赖的文件拷进你的项目，路径不变；已经有的文件不动。

```sh
cargo run --release --bin amenui -- add dialog --to ../my-app
```

也可以手动拷：组件页的“安装”一节列出了它的文件。
