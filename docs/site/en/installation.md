# Installation

You need the latest stable Rust. The native window needs a GPU or Mesa's lavapipe (Vulkan on the CPU); the web build needs WebGPU or WebGL2.

```sh
cargo run --release --bin window -- film
cargo test
cargo run --release --bin check
```

## Copy components into your project

Components are distributed as source, as shadcn/ui does it: `lab add` copies a component and the files it depends on, as `registry.json` lists them, into your project at the same paths; files already there are left alone.

```sh
cargo run --release --bin lab -- add dialog --to ../my-app
```

## The web build

```sh
cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen --target no-modules --no-typescript --out-dir dist/pkg target/wasm32-unknown-unknown/release/lab.wasm
python3 web/build.py
```
