# Testing with render and the tree

`render` plays events on a fixed 60 Hz clock and draws the last frame and its tree; the same arguments give the same bytes.

```sh
cargo run --release --bin render -- --page button --size 1280x800 --at 1000 --out out.png --tree out.json --locale en
```

`check` reads pixels back on the GPU for acceptance (18 checks, including that a slow scroll is a picture moved by whole pixels).
