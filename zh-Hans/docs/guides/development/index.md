# 开发



The [architecture reference](../reference/architecture.md) maps what each module owns. This guide covers
building, testing and the tools in this repository.

## Build and test

Use a recent stable Rust toolchain with rustfmt and Clippy.

```sh
cargo build --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

The tests need no GPU; the few that draw use one when it is there. They cover layout, input, focus and
text editing, the exported tree, ripples and recomposition, transforms, i18n, the registry and the
recipes. `AGENTS.md` and `README.md` are doc tests, so every code block in them compiles and runs.

`tests/recipes.rs` fails when the code block in a page of `docs/recipes/` differs from its file in
`examples/`. After you change a recipe, copy the file into its page.

## Tools

One deterministic frame of a recipe and its laid-out tree, after scripted input:

```sh
cargo run --release --bin render -- --page recipe-counter --size 1280x800 --at 1000 --out out.png --tree out.json
cargo run --release --bin render -- --help
```

The clock is fixed at 60 Hz from zero, so the same arguments give the same bytes. `--events` takes a
JSON list of inputs; [AGENTS.md](../../AGENTS.md#how-to-verify) describes the format.

Contact sheets of every picture mark set, palette and option, plus the mark atlas and the raw pictures:

```sh
cargo run --release --bin sheet -- sheets
```

Copy components and what they depend on into a project:

```sh
cargo run --release --bin amenui -- add button dialog --to ../my-app/src/widgets
```

The `quick` profile is optimized without link-time optimization, for iterating on the tools:
`cargo run --profile quick --bin render -- …`.

## Working with the website

The [website](https://github.com/amenui-dev/website) depends on this repository's `main` branch and
holds the native window, the web build, the GPU acceptance check (`check`), the benchmarks and many
tests that drive its pages. To try a framework change there, check out both repositories side by side
and uncomment the `[patch]` section at the end of the website's `Cargo.toml`.

## Conventions

- Every numeric parameter is a named field with a default, and the pages show it as a number control.
- Text a person reads comes from the message catalogs in `assets/i18n/`. Add a key to every catalog.
- Demo and example text on screen is English.
- Commit messages are `<scope>: <what changed>`, for example `text: fall back to system fonts`.
- Design docs use standard graphics terms (coverage buffer, signed distance field, G-buffer).
