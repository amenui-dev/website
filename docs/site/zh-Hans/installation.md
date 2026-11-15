# 安装

需要最新的稳定版 Rust。原生窗口要一块显卡，或者 Mesa 的 lavapipe（CPU 上的 Vulkan）；网页版要 WebGPU 或 WebGL2。

```sh
cargo run --release --bin window -- film
cargo test
cargo run --release --bin check
```

## 把组件拷进自己的项目

组件以源码分发，像 shadcn/ui：`lab add` 按 `registry.json` 把一个组件和它依赖的文件拷到你的项目里，路径不变，已经有的文件不覆盖。

```sh
cargo run --release --bin lab -- add dialog --to ../my-app
```

## 网页版

```sh
cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen --target no-modules --no-typescript --out-dir dist/pkg target/wasm32-unknown-unknown/release/lab.wasm
python3 web/build.py
```
