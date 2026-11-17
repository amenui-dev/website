# Command line

Components are distributed as source, the way shadcn/ui does it: you copy a component's file into your project and change it there. `amenui add` does the copying.

```sh
cargo run --release --bin amenui -- add dialog --to ../my-app
cargo run --release --bin amenui -- add button number input --to ../my-app --force
```

- `add <name>…` takes one or more names as `registry.json` lists them (`button`, `scroll-area`, `context_menu`…).
- It resolves what each one depends on (every component needs `base`, the shared sizes in `src/widgets/mod.rs`; a dialog needs the button) and copies every file to the same path under the target.
- A file that is already there is left alone; `--force` overwrites it.
- `--to <dir>` is the project (the current directory when left out).
- A misspelt name is an error that names the closest one.

The logic is `amenui::add` (`src/add.rs`); `tests/add.rs` runs it into a scratch directory. Every component page shows its own `add` line and the files it copies.
